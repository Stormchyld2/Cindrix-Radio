//! Live internet radio: connect, strip and read ICY now-playing text, decode with
//! symphonia (no seeking needed), and optionally tee the raw audio into a recording file.
use serde::Serialize;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::core::errors::Error as SymError;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::{MediaSourceStream, ReadOnlySource};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub type NowPlaying = Arc<Mutex<Option<String>>>;
pub type Recorder = Arc<Mutex<Option<std::fs::File>>>;
pub type CurrentInfo = Arc<Mutex<Option<StreamInfo>>>;

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
pub struct StreamInfo {
    pub url: String,
    pub name: Option<String>,
    pub genre: Option<String>,
    pub bitrate: Option<u32>,
    pub content_type: Option<String>,
}

impl StreamInfo {
    /// File extension for a raw recording of this stream.
    pub fn extension(&self) -> &'static str {
        let ct = self.content_type.as_deref().unwrap_or("").to_ascii_lowercase();
        if ct.contains("mpeg") || ct.contains("mp3") { "mp3" }
        else if ct.contains("aac") { "aac" }
        else if ct.contains("ogg") { "ogg" }
        else if ct.contains("flac") { "flac" }
        else { "mp3" }
    }
}

/// Reads an Icecast/SHOUTcast stream, removing the metadata blocks that are mixed into
/// the audio every `metaint` bytes and publishing the current song title.
pub struct IcyReader<R: Read> {
    inner: R,
    metaint: usize,
    left: usize,
    now: NowPlaying,
    rec: Recorder,
}

impl<R: Read> IcyReader<R> {
    pub fn new(inner: R, metaint: usize, now: NowPlaying, rec: Recorder) -> Self {
        IcyReader { inner, metaint, left: metaint, now, rec }
    }

    fn parse_meta(&self, block: &[u8]) {
        let text = String::from_utf8_lossy(block);
        let text = text.trim_end_matches('\0');
        if let Some(start) = text.find("StreamTitle='") {
            let rest = &text[start + "StreamTitle='".len()..];
            let end = rest.find("';").or_else(|| rest.rfind('\'')).unwrap_or(rest.len());
            let title = rest[..end].trim().to_string();
            *self.now.lock().unwrap() = if title.is_empty() { None } else { Some(title) };
        }
    }

    fn record(&self, data: &[u8]) {
        if let Some(f) = self.rec.lock().unwrap().as_mut() { let _ = f.write_all(data); }
    }
}

impl<R: Read> Read for IcyReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.metaint == 0 {
            let n = self.inner.read(buf)?;
            self.record(&buf[..n]);
            return Ok(n);
        }
        if self.left == 0 {
            let mut len = [0u8; 1];
            match self.inner.read_exact(&mut len) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(0),
                Err(e) => return Err(e),
            }
            let l = len[0] as usize * 16;
            if l > 0 {
                let mut block = vec![0u8; l];
                self.inner.read_exact(&mut block)?;
                self.parse_meta(&block);
            }
            self.left = self.metaint;
        }
        let max = buf.len().min(self.left);
        let n = self.inner.read(&mut buf[..max])?;
        self.left -= n;
        self.record(&buf[..n]);
        Ok(n)
    }
}

/// symphonia wants a Sync reader; a Mutex makes any Send reader Sync.
struct SyncRead<R: Read + Send>(Mutex<R>);
impl<R: Read + Send> Read for SyncRead<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> { self.0.lock().unwrap().read(buf) }
}

pub struct StreamSource {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    rate: u32,
    channels: u16,
    sample_buf: Option<SampleBuffer<f32>>,
    frames_cap: u64,
    buf: Vec<f32>,
    pos: usize,
}

impl StreamSource {
    pub fn new<R: Read + Send + 'static>(reader: R, ext: Option<&str>) -> Result<Self, String> {
        let mss = MediaSourceStream::new(Box::new(ReadOnlySource::new(SyncRead(Mutex::new(reader)))), Default::default());
        let mut hint = Hint::new();
        if let Some(e) = ext { hint.with_extension(e); }
        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
            .map_err(|e| format!("Unsupported or unreadable stream: {e}"))?;
        let format = probed.format;
        let track = format.default_track().ok_or("The stream has no audio")?;
        let track_id = track.id;
        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            .map_err(|e| format!("Cannot decode this stream: {e}"))?;
        let mut s = StreamSource { format, decoder, track_id, rate: 44100, channels: 2, sample_buf: None, frames_cap: 0, buf: Vec::new(), pos: 0 };
        if !s.refill() { return Err("The stream ended before any audio arrived".into()); }
        Ok(s)
    }

    fn refill(&mut self) -> bool {
        loop {
            let packet = match self.format.next_packet() { Ok(p) => p, Err(_) => return false };
            if packet.track_id() != self.track_id { continue; }
            match self.decoder.decode(&packet) {
                Ok(decoded) => {
                    let spec = *decoded.spec();
                    let cap = decoded.capacity() as u64;
                    if self.sample_buf.is_none() || cap > self.frames_cap {
                        self.sample_buf = Some(SampleBuffer::<f32>::new(cap, spec));
                        self.frames_cap = cap;
                    }
                    let sb = self.sample_buf.as_mut().unwrap();
                    sb.copy_interleaved_ref(decoded);
                    self.buf.clear();
                    self.buf.extend_from_slice(sb.samples());
                    self.pos = 0;
                    if self.pos == 0 && self.rate == 44100 && self.channels == 2 && self.frames_cap == cap {
                        // first packet tells us the real format
                        self.rate = spec.rate;
                        self.channels = spec.channels.count() as u16;
                    }
                    if !self.buf.is_empty() { return true; }
                }
                Err(SymError::DecodeError(_)) => continue, // a bad frame in a live stream: skip it
                Err(_) => return false,
            }
        }
    }
}

impl Iterator for StreamSource {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.pos >= self.buf.len() && !self.refill() { return None; }
        let x = self.buf[self.pos];
        self.pos += 1;
        Some(x)
    }
}

impl rodio::Source for StreamSource {
    fn current_frame_len(&self) -> Option<usize> { None }
    fn channels(&self) -> u16 { self.channels }
    fn sample_rate(&self) -> u32 { self.rate }
    fn total_duration(&self) -> Option<Duration> { None }
}

/// Pull the first stream address out of a .pls or .m3u playlist.
pub fn first_url(text: &str) -> Option<String> {
    for line in text.lines() {
        let l = line.trim();
        if l.len() > 6 && l[..5].eq_ignore_ascii_case("file1") && l.contains('=') {
            let v = l.splitn(2, '=').nth(1)?.trim();
            if v.starts_with("http") { return Some(v.to_string()); }
        }
        if l.starts_with("http://") || l.starts_with("https://") { return Some(l.to_string()); }
    }
    None
}

/// Station lists (.m3u / .pls): (name, url) pairs.
pub fn parse_station_list(text: &str) -> Vec<(Option<String>, String)> {
    let mut out = Vec::new();
    let mut pending: Option<String> = None;
    let mut pls_titles: std::collections::HashMap<String, String> = Default::default();
    for line in text.lines() {
        let l = line.trim();
        if let Some(rest) = l.strip_prefix("#EXTINF:") {
            pending = rest.splitn(2, ',').nth(1).map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
        } else if l.starts_with("http://") || l.starts_with("https://") {
            out.push((pending.take(), l.to_string()));
        } else if let Some((k, v)) = l.split_once('=') {
            let k = k.trim().to_ascii_lowercase();
            if let Some(n) = k.strip_prefix("title") { pls_titles.insert(n.to_string(), v.trim().to_string()); }
            else if let Some(n) = k.strip_prefix("file") {
                if v.trim().starts_with("http") { out.push((pls_titles.get(n).cloned(), v.trim().to_string())); }
            }
        }
    }
    // titles can come after files in a .pls
    for (i, (name, _)) in out.iter_mut().enumerate() {
        if name.is_none() { *name = pls_titles.get(&(i + 1).to_string()).cloned(); }
    }
    out
}

fn client() -> Result<reqwest::blocking::Client, String> {
    // no total timeout: a live stream is supposed to run for hours
    reqwest::blocking::Client::builder()
        .user_agent("CindrixRadio/0.1")
        .connect_timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())
}

fn header(r: &reqwest::blocking::Response, k: &str) -> Option<String> {
    r.headers().get(k).and_then(|v| v.to_str().ok()).map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn is_playlist(url: &str, ct: &str) -> bool {
    let u = url.split('?').next().unwrap_or("").to_ascii_lowercase();
    let c = ct.to_ascii_lowercase();
    u.ends_with(".pls") || u.ends_with(".m3u") || c.contains("scpls") || c.contains("x-mpegurl") && !u.ends_with(".m3u8") || c.contains("audio/mpegurl")
}

fn is_hls(url: &str, ct: &str) -> bool {
    let u = url.split('?').next().unwrap_or("").to_ascii_lowercase();
    u.ends_with(".m3u8") || ct.to_ascii_lowercase().contains("vnd.apple.mpegurl")
}

/// Connect to a stream, following one level of .pls/.m3u playlist.
fn connect(url: &str, depth: u8) -> Result<(reqwest::blocking::Response, StreamInfo), String> {
    if !crate::podcasts::is_http(url) { return Err("A stream address must start with http:// or https://".into()); }
    let resp = client()?.get(url.trim()).header("Icy-MetaData", "1").send()
        .and_then(|r| r.error_for_status()).map_err(|e| format!("Could not connect: {e}"))?;
    let ct = header(&resp, "content-type").unwrap_or_default();
    if is_hls(url, &ct) { return Err("This is an HLS (.m3u8) stream, which is not supported yet".into()); }
    if depth == 0 && is_playlist(url, &ct) {
        let mut text = String::new();
        resp.take(65536).read_to_string(&mut text).map_err(|e| e.to_string())?;
        let next = first_url(&text).ok_or("That playlist has no stream address in it")?;
        return connect(&next, 1);
    }
    let info = StreamInfo {
        url: url.trim().to_string(),
        name: header(&resp, "icy-name"),
        genre: header(&resp, "icy-genre"),
        bitrate: header(&resp, "icy-br").and_then(|b| b.split(',').next().and_then(|n| n.trim().parse().ok())),
        content_type: Some(ct).filter(|c| !c.is_empty()),
    };
    Ok((resp, info))
}

/// Check a stream address and report what it is, without decoding it.
pub fn probe(url: &str) -> Result<StreamInfo, String> { connect(url, 0).map(|(_, i)| i) }

pub struct Opened { pub source: StreamSource, pub info: StreamInfo }

pub fn open(url: &str, now: NowPlaying, rec: Recorder) -> Result<Opened, String> {
    let (resp, info) = connect(url, 0)?;
    let metaint = header(&resp, "icy-metaint").and_then(|m| m.parse::<usize>().ok()).unwrap_or(0);
    *now.lock().unwrap() = None;
    let ext = Some(info.extension());
    let reader = IcyReader::new(resp, metaint, now, rec);
    let source = StreamSource::new(reader, ext)?;
    Ok(Opened { source, info })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn shared() -> (NowPlaying, Recorder) { (Arc::new(Mutex::new(None)), Arc::new(Mutex::new(None))) }

    fn meta_block(text: &str) -> Vec<u8> {
        let mut b = text.as_bytes().to_vec();
        let pad = (16 - b.len() % 16) % 16;
        b.extend(std::iter::repeat(0).take(pad));
        let mut out = vec![(b.len() / 16) as u8];
        out.extend(b);
        out
    }

    #[test]
    fn icy_reader_strips_metadata_and_reports_the_title() {
        let mut data = vec![];
        data.extend([1u8; 8]);
        data.extend(meta_block("StreamTitle='Artist - Song A';"));
        data.extend([2u8; 8]);
        data.push(0); // empty metadata block: title unchanged
        data.extend([3u8; 8]);
        let (now, rec) = shared();
        let mut r = IcyReader::new(Cursor::new(data), 8, now.clone(), rec);
        let mut out = Vec::new();
        r.read_to_end(&mut out).unwrap();
        assert_eq!(out, [vec![1u8; 8], vec![2u8; 8], vec![3u8; 8]].concat());
        assert_eq!(now.lock().unwrap().as_deref(), Some("Artist - Song A"));
    }

    #[test]
    fn icy_reader_without_metaint_passes_data_through() {
        let (now, rec) = shared();
        let mut r = IcyReader::new(Cursor::new(vec![9u8; 100]), 0, now.clone(), rec);
        let mut out = Vec::new();
        r.read_to_end(&mut out).unwrap();
        assert_eq!(out.len(), 100);
        assert!(now.lock().unwrap().is_none());
    }

    #[test]
    fn recording_tees_audio_bytes_only() {
        let dir = std::env::temp_dir().join(format!("cindrix-rec-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("r.mp3");
        let (now, rec) = shared();
        *rec.lock().unwrap() = Some(std::fs::File::create(&path).unwrap());
        let mut data = vec![7u8; 8];
        data.extend(meta_block("StreamTitle='X';"));
        data.extend([8u8; 8]);
        let mut r = IcyReader::new(Cursor::new(data), 8, now, rec);
        let mut sink = Vec::new();
        r.read_to_end(&mut sink).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), [vec![7u8; 8], vec![8u8; 8]].concat());
        let _ = std::fs::remove_dir_all(dir);
    }

    fn wav(secs: f32) -> Vec<u8> {
        let rate = 44100u32;
        let n = (rate as f32 * secs) as usize;
        let mut pcm = Vec::new();
        for i in 0..n {
            let v = (0.3 * (2.0 * std::f32::consts::PI * 440.0 * i as f32 / rate as f32).sin() * i16::MAX as f32) as i16;
            pcm.extend(v.to_le_bytes());
        }
        let mut b = Vec::new();
        b.extend(b"RIFF"); b.extend((36 + pcm.len() as u32).to_le_bytes()); b.extend(b"WAVEfmt ");
        b.extend(16u32.to_le_bytes()); b.extend(1u16.to_le_bytes()); b.extend(1u16.to_le_bytes());
        b.extend(rate.to_le_bytes()); b.extend((rate * 2).to_le_bytes()); b.extend(2u16.to_le_bytes()); b.extend(16u16.to_le_bytes());
        b.extend(b"data"); b.extend((pcm.len() as u32).to_le_bytes()); b.extend(pcm);
        b
    }

    #[test]
    fn stream_source_decodes_a_non_seekable_stream() {
        use rodio::Source;
        let src = StreamSource::new(Cursor::new(wav(0.5)), Some("wav")).expect("decodes");
        assert_eq!(src.channels(), 1);
        assert_eq!(src.sample_rate(), 44100);
        let samples: Vec<f32> = src.collect();
        assert!(samples.len() > 20000, "got {}", samples.len());
        let peak = samples.iter().fold(0.0f32, |m, x| m.max(x.abs()));
        assert!(peak > 0.2 && peak < 0.4, "peak {peak}");
    }

    #[test]
    fn garbage_is_rejected_not_played() {
        assert!(StreamSource::new(Cursor::new(vec![0u8; 4096]), None).is_err());
    }

    #[test]
    fn playlist_parsing() {
        assert_eq!(first_url("[playlist]\nNumberOfEntries=1\nFile1=http://a.example:8000/live\nTitle1=A"), Some("http://a.example:8000/live".into()));
        assert_eq!(first_url("#EXTM3U\n#EXTINF:-1,Name\nhttps://b.example/stream"), Some("https://b.example/stream".into()));
        assert_eq!(first_url("nothing here"), None);
        let list = parse_station_list("#EXTM3U\n#EXTINF:-1,Alpha FM\nhttp://a.example/1\nhttp://b.example/2\n");
        assert_eq!(list, vec![(Some("Alpha FM".into()), "http://a.example/1".into()), (None, "http://b.example/2".into())]);
        let pls = parse_station_list("[playlist]\nFile1=http://c.example/s\nTitle1=Gamma\n");
        assert_eq!(pls, vec![(Some("Gamma".into()), "http://c.example/s".into())]);
    }

    #[test]
    fn hls_and_playlist_detection() {
        assert!(is_hls("https://x/y.m3u8", ""));
        assert!(!is_hls("https://x/y", "audio/mpeg"));
        assert!(is_playlist("https://x/y.pls", ""));
        assert!(!is_playlist("https://x/live", "audio/mpeg"));
    }

    #[test]
    fn recording_extension_follows_content_type() {
        let mk = |c: &str| StreamInfo { content_type: Some(c.into()), ..Default::default() };
        assert_eq!(mk("audio/mpeg").extension(), "mp3");
        assert_eq!(mk("audio/aacp").extension(), "aac");
        assert_eq!(mk("application/ogg").extension(), "ogg");
    }
}

//! HLS (.m3u8) live radio. A background thread keeps the playlist fresh, downloads each
//! audio segment, unwraps MPEG-TS if needed, and hands plain AAC/MP3 bytes to the decoder
//! through a small buffer, so the audio never waits on the network.
//! Not supported: encrypted segments and fragmented-MP4 segments (reported as errors).
use reqwest::Url;
use std::io::Read;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::time::Duration;

#[derive(Debug, PartialEq)]
pub struct Segment { pub seq: u64, pub url: Url }

#[derive(Debug, PartialEq)]
pub enum Playlist {
    Master { variant: Option<Url> },
    Media { target_secs: u64, segments: Vec<Segment>, endlist: bool, encrypted: bool, fmp4: bool },
}

fn attr<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let at = line.find(&format!("{key}="))? + key.len() + 1;
    let rest = &line[at..];
    if let Some(q) = rest.strip_prefix('"') { q.split('"').next() } else { rest.split(',').next() }
}

/// Read a master or media playlist. `base` is the playlist's own address (for relative links).
pub fn parse(text: &str, base: &Url) -> Result<Playlist, String> {
    if !text.trim_start().starts_with("#EXTM3U") { return Err("That address did not return an HLS playlist".into()); }
    let join = |u: &str| base.join(u.trim()).map_err(|e| format!("Bad link in the playlist: {e}"));
    if text.contains("#EXT-X-STREAM-INF") {
        // audio stations: take the best variant up to 160 kbps, else the smallest
        let mut variants: Vec<(u64, Url)> = Vec::new();
        let mut pending: Option<u64> = None;
        for line in text.lines().map(str::trim) {
            if line.starts_with("#EXT-X-STREAM-INF") {
                pending = Some(attr(line, "BANDWIDTH").and_then(|b| b.parse().ok()).unwrap_or(u64::MAX));
            } else if !line.is_empty() && !line.starts_with('#') {
                if let Some(b) = pending.take() { variants.push((b, join(line)?)); }
            }
        }
        let best = variants.iter().filter(|(b, _)| *b <= 160_000).max_by_key(|(b, _)| *b)
            .or_else(|| variants.iter().min_by_key(|(b, _)| *b)).map(|(_, u)| u.clone());
        let variant = match best {
            Some(u) => Some(u),
            None => text.lines().map(str::trim).filter(|l| l.starts_with("#EXT-X-MEDIA") && l.contains("TYPE=AUDIO"))
                .filter_map(|l| attr(l, "URI")).next().map(join).transpose()?,
        };
        return Ok(Playlist::Master { variant });
    }
    let (mut target, mut seq, mut endlist, mut encrypted, mut fmp4) = (6u64, 0u64, false, false, false);
    let mut segments = Vec::new();
    for line in text.lines().map(str::trim) {
        if let Some(v) = line.strip_prefix("#EXT-X-TARGETDURATION:") { target = v.trim().parse().unwrap_or(6); }
        else if let Some(v) = line.strip_prefix("#EXT-X-MEDIA-SEQUENCE:") { seq = v.trim().parse().unwrap_or(0); }
        else if line.starts_with("#EXT-X-ENDLIST") { endlist = true; }
        else if line.starts_with("#EXT-X-KEY") { if attr(line, "METHOD").map_or(false, |m| m != "NONE") { encrypted = true; } }
        else if line.starts_with("#EXT-X-MAP") { fmp4 = true; }
        else if !line.is_empty() && !line.starts_with('#') {
            segments.push(Segment { seq: seq + segments.len() as u64, url: join(line)? });
        }
    }
    Ok(Playlist::Media { target_secs: target, segments, endlist, encrypted, fmp4 })
}

/// Drop a leading ID3 tag (HLS audio segments often start with one).
pub fn strip_id3(data: &[u8]) -> &[u8] {
    if data.len() >= 10 && &data[..3] == b"ID3" {
        let size = data[6..10].iter().fold(0usize, |a, b| (a << 7) | (*b as usize & 0x7F));
        let footer = if data[5] & 0x10 != 0 { 10 } else { 0 };
        return data.get(10 + size + footer..).unwrap_or(&[]);
    }
    data
}

/// Pull the audio elementary stream (ADTS AAC or MP3) out of an MPEG transport stream.
/// `audio_pid` remembers which stream is audio between segments.
pub fn ts_audio(data: &[u8], audio_pid: &mut Option<u16>) -> Vec<u8> {
    let mut out = Vec::new();
    let mut pmt_pid: Option<u16> = None;
    let start = (0..188.min(data.len())).find(|&i| data.get(i) == Some(&0x47) && data.get(i + 188).map_or(true, |b| *b == 0x47)).unwrap_or(0);
    for p in data[start..].chunks_exact(188) {
        if p[0] != 0x47 { continue; }
        let pusi = p[1] & 0x40 != 0;
        let pid = (((p[1] & 0x1F) as u16) << 8) | p[2] as u16;
        let mut off = 4usize;
        match (p[3] >> 4) & 3 { 1 => {}, 3 => off += 1 + p[4] as usize, _ => continue }
        let Some(pl) = p.get(off..).filter(|s| !s.is_empty()) else { continue };
        if pid == 0 && pusi {
            // PAT: first real program points at the PMT
            let t = pl.get(1 + pl[0] as usize..).unwrap_or(&[]);
            if t.len() >= 12 {
                let end = (3 + (((t[1] & 0x0F) as usize) << 8 | t[2] as usize)).saturating_sub(4).min(t.len());
                let mut i = 8;
                while i + 4 <= end {
                    let prog = (t[i] as u16) << 8 | t[i + 1] as u16;
                    if prog != 0 { pmt_pid = Some(((t[i + 2] & 0x1F) as u16) << 8 | t[i + 3] as u16); break; }
                    i += 4;
                }
            }
        } else if pusi && pmt_pid.is_some() && Some(pid) == pmt_pid {
            let t = pl.get(1 + pl[0] as usize..).unwrap_or(&[]);
            if t.len() >= 12 {
                let end = (3 + (((t[1] & 0x0F) as usize) << 8 | t[2] as usize)).saturating_sub(4).min(t.len());
                let mut i = 12 + (((t[10] & 0x0F) as usize) << 8 | t[11] as usize);
                while i + 5 <= end {
                    let (ty, epid) = (t[i], ((t[i + 1] & 0x1F) as u16) << 8 | t[i + 2] as u16);
                    if matches!(ty, 0x0F | 0x03 | 0x04) { *audio_pid = Some(epid); break; }
                    i += 5 + (((t[i + 3] & 0x0F) as usize) << 8 | t[i + 4] as usize);
                }
            }
        } else if Some(pid) == *audio_pid {
            if pusi {
                if pl.len() >= 9 && pl[0] == 0 && pl[1] == 0 && pl[2] == 1 {
                    if let Some(rest) = pl.get(9 + pl[8] as usize..) { out.extend_from_slice(rest); }
                }
            } else {
                out.extend_from_slice(pl);
            }
        }
    }
    out
}

/// Turn one downloaded segment into plain audio bytes.
pub fn segment_audio(data: &[u8], audio_pid: &mut Option<u16>) -> Vec<u8> {
    if data.first() == Some(&0x47) { ts_audio(data, audio_pid) } else { strip_id3(data).to_vec() }
}

type Item = Result<Vec<u8>, String>;

/// Reads the audio of a live HLS stream as one continuous byte stream.
pub struct HlsReader { rx: Receiver<Item>, cur: Vec<u8>, pos: usize, rec: crate::stream::Recorder }

impl HlsReader {
    pub fn start(url: &str, rec: crate::stream::Recorder) -> Result<HlsReader, String> {
        let url = Url::parse(url).map_err(|e| e.to_string())?;
        let (tx, rx) = sync_channel::<Item>(3);
        std::thread::spawn(move || {
            if let Err(e) = run(&tx, url) { let _ = tx.send(Err(e)); }
        });
        Ok(HlsReader { rx, cur: Vec::new(), pos: 0, rec })
    }
}

impl Read for HlsReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        while self.pos >= self.cur.len() {
            match self.rx.recv() {
                Ok(Ok(b)) => { self.cur = b; self.pos = 0; }
                Ok(Err(e)) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
                Err(_) => return Ok(0), // stream finished
            }
        }
        let n = buf.len().min(self.cur.len() - self.pos);
        buf[..n].copy_from_slice(&self.cur[self.pos..self.pos + n]);
        self.pos += n;
        if let Some(f) = self.rec.lock().unwrap().as_mut() { use std::io::Write; let _ = f.write_all(&buf[..n]); }
        Ok(n)
    }
}

fn fetch(client: &reqwest::blocking::Client, url: &Url) -> Result<Vec<u8>, String> {
    client.get(url.clone()).send().and_then(|r| r.error_for_status()).and_then(|r| r.bytes()).map(|b| b.to_vec()).map_err(|e| format!("Could not download from the stream: {e}"))
}

fn run(tx: &SyncSender<Item>, start: Url) -> Result<(), String> {
    let client = reqwest::blocking::Client::builder().user_agent("CindrixRadio/0.1")
        .connect_timeout(Duration::from_secs(10)).timeout(Duration::from_secs(30)).build().map_err(|e| e.to_string())?;
    let mut url = start;
    let mut audio_pid: Option<u16> = None;
    let mut last_seq: Option<u64> = None;
    let mut fails = 0u8;
    for _hop in 0..4 {
        // follow a master playlist down to the audio playlist
        let text = String::from_utf8_lossy(&fetch(&client, &url)?).into_owned();
        match parse(&text, &url)? {
            Playlist::Master { variant } => { url = variant.ok_or("The stream lists no audio")?; continue; }
            Playlist::Media { .. } => break,
        }
    }
    loop {
        let text = match fetch(&client, &url) {
            Ok(b) => { fails = 0; String::from_utf8_lossy(&b).into_owned() }
            Err(e) => { fails += 1; if fails > 5 { return Err(e); } std::thread::sleep(Duration::from_secs(2)); continue; }
        };
        let Playlist::Media { target_secs, segments, endlist, encrypted, fmp4 } = parse(&text, &url)? else { return Err("Unexpected playlist".into()) };
        if encrypted { return Err("This stream is encrypted, which is not supported".into()); }
        if fmp4 { return Err("This stream uses fragmented MP4 segments, which are not supported yet".into()); }
        let first = match last_seq {
            None => if endlist { 0 } else { segments.len().saturating_sub(3) },
            Some(l) => segments.iter().position(|s| s.seq > l).unwrap_or(segments.len()),
        };
        for seg in &segments[first..] {
            let bytes = match fetch(&client, &seg.url).or_else(|_| fetch(&client, &seg.url)) { Ok(b) => b, Err(_) => { last_seq = Some(seg.seq); continue; } };
            let audio = segment_audio(&bytes, &mut audio_pid);
            last_seq = Some(seg.seq);
            if !audio.is_empty() && tx.send(Ok(audio)).is_err() { return Ok(()); } // listener left
        }
        if endlist { return Ok(()); }
        std::thread::sleep(Duration::from_millis((target_secs * 1000 / 2).clamp(1000, 3000)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::{Arc, Mutex};

    fn base() -> Url { Url::parse("https://radio.example/live/index.m3u8").unwrap() }

    #[test]
    fn master_playlist_picks_best_audio_variant_up_to_160k() {
        let m = "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=64000,CODECS=\"mp4a.40.5\"\nlow/a.m3u8\n#EXT-X-STREAM-INF:BANDWIDTH=128000\nmid/a.m3u8\n#EXT-X-STREAM-INF:BANDWIDTH=320000\nhigh/a.m3u8\n";
        assert_eq!(parse(m, &base()).unwrap(), Playlist::Master { variant: Some(Url::parse("https://radio.example/live/mid/a.m3u8").unwrap()) });
        let big = "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=900000\nhd.m3u8\n#EXT-X-STREAM-INF:BANDWIDTH=500000\nsd.m3u8\n";
        let Playlist::Master { variant } = parse(big, &base()).unwrap() else { panic!() };
        assert_eq!(variant.unwrap().path(), "/live/sd.m3u8", "smallest when all are large");
    }

    #[test]
    fn media_playlist_numbers_segments_and_flags_unsupported() {
        let m = "#EXTM3U\n#EXT-X-TARGETDURATION:10\n#EXT-X-MEDIA-SEQUENCE:100\n#EXTINF:9.9,\nseg100.aac\n#EXTINF:9.9,\nhttps://cdn.example/seg101.aac\n";
        let Playlist::Media { target_secs, segments, endlist, encrypted, fmp4 } = parse(m, &base()).unwrap() else { panic!() };
        assert_eq!((target_secs, endlist, encrypted, fmp4), (10, false, false, false));
        assert_eq!(segments[0], Segment { seq: 100, url: Url::parse("https://radio.example/live/seg100.aac").unwrap() });
        assert_eq!(segments[1].seq, 101);
        assert_eq!(segments[1].url.host_str(), Some("cdn.example"));
        let enc = "#EXTM3U\n#EXT-X-KEY:METHOD=AES-128,URI=\"k\"\n#EXTINF:5,\na.ts\n#EXT-X-ENDLIST\n";
        let Playlist::Media { encrypted, endlist, .. } = parse(enc, &base()).unwrap() else { panic!() };
        assert!(encrypted && endlist);
        let map = "#EXTM3U\n#EXT-X-MAP:URI=\"init.mp4\"\n#EXTINF:5,\na.m4s\n";
        let Playlist::Media { fmp4, .. } = parse(map, &base()).unwrap() else { panic!() };
        assert!(fmp4);
        assert!(parse("<html>nope</html>", &base()).is_err());
    }

    #[test]
    fn id3_tag_is_skipped() {
        let mut d = b"ID3\x04\x00\x00\x00\x00\x00\x05".to_vec();
        d.extend([1, 2, 3, 4, 5]);
        d.extend([0xFF, 0xF1]);
        assert_eq!(strip_id3(&d), &[0xFF, 0xF1]);
        assert_eq!(strip_id3(&[0xFF, 0xF1]), &[0xFF, 0xF1]);
    }

    /// One 188-byte transport packet, padded with an adaptation field.
    fn ts_packet(pid: u16, pusi: bool, payload: &[u8]) -> Vec<u8> {
        assert!(payload.len() <= 183);
        let mut p = vec![0x47, ((pusi as u8) << 6) | (pid >> 8) as u8, pid as u8];
        let pad = 183 - payload.len();
        p.push(0x30); // adaptation field + payload
        p.push(pad as u8);
        if pad > 0 { p.push(0); p.extend(std::iter::repeat(0xFF).take(pad - 1)); }
        p.extend_from_slice(payload);
        assert_eq!(p.len(), 188);
        p
    }

    fn section(table: u8, body: &[u8]) -> Vec<u8> {
        // pointer field, table id, section length (body + 4 crc), body, fake crc
        let len = body.len() + 4;
        let mut s = vec![0, table, 0xB0 | (len >> 8) as u8, len as u8];
        s.extend_from_slice(body);
        s.extend([0, 0, 0, 0]);
        s
    }

    #[test]
    fn transport_stream_audio_is_extracted() {
        let pat = section(0, &[0, 1, 0xC1, 0, 0, /* program 1 -> PMT pid 0x100 */ 0, 1, 0xE1, 0x00]);
        // PMT: program 1, pcr pid, no program info, one AAC stream on pid 0x101, one video on 0x102
        let pmt = section(2, &[0, 1, 0xC1, 0, 0, 0xE1, 0x01, 0xF0, 0x00, 0x0F, 0xE1, 0x01, 0xF0, 0x00, 0x1B, 0xE1, 0x02, 0xF0, 0x00]);
        let mut pes = vec![0, 0, 1, 0xC0, 0, 0, 0x80, 0x00, 0x00];
        pes.extend(b"AUDIO-ONE");
        let mut ts = ts_packet(0, true, &pat);
        ts.extend(ts_packet(0x100, true, &pmt));
        ts.extend(ts_packet(0x101, true, &pes));
        ts.extend(ts_packet(0x102, true, b"video bytes that must be ignored"));
        ts.extend(ts_packet(0x101, false, b"-AUDIO-TWO"));
        let mut pid = None;
        assert_eq!(ts_audio(&ts, &mut pid), b"AUDIO-ONE-AUDIO-TWO");
        assert_eq!(pid, Some(0x101));
        // a later segment without PAT/PMT still works because the audio stream is remembered
        let mut later = ts_packet(0x101, true, &pes);
        later.extend(ts_packet(0x101, false, b"!"));
        assert_eq!(segment_audio(&later, &mut pid), b"AUDIO-ONE!");
        assert_eq!(segment_audio(b"ID3\x04\x00\x00\x00\x00\x00\x00rest", &mut pid), b"rest");
        // garbage never panics
        let _ = ts_audio(&[0x47, 0xFF, 0xFF], &mut None);
        let _ = ts_audio(&vec![0x47; 400], &mut None);
    }

    /// Tiny local web server: path -> body.
    fn serve(routes: Vec<(&'static str, Vec<u8>)>) -> String {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut s) = stream else { continue };
                let routes = routes.clone();
                std::thread::spawn(move || {
                    let mut buf = [0u8; 2048];
                    let n = s.read(&mut buf).unwrap_or(0);
                    let req = String::from_utf8_lossy(&buf[..n]).to_string();
                    let path = req.split_whitespace().nth(1).unwrap_or("/").to_string();
                    match routes.iter().find(|(p, _)| *p == path) {
                        Some((_, body)) => { let _ = write!(s, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()); let _ = s.write_all(body); }
                        None => { let _ = write!(s, "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"); }
                    }
                });
            }
        });
        format!("http://{addr}")
    }

    #[test]
    fn reader_follows_master_and_streams_every_segment_then_ends() {
        let base = serve(vec![
            ("/m.m3u8", b"#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=96000\nmedia.m3u8\n".to_vec()),
            ("/media.m3u8", b"#EXTM3U\n#EXT-X-TARGETDURATION:2\n#EXT-X-MEDIA-SEQUENCE:0\n#EXTINF:2,\ns0.aac\n#EXTINF:2,\ns1.aac\n#EXT-X-ENDLIST\n".to_vec()),
            ("/s0.aac", b"first-".to_vec()),
            ("/s1.aac", b"second".to_vec()),
        ]);
        let rec: crate::stream::Recorder = Arc::new(Mutex::new(None));
        let mut r = HlsReader::start(&format!("{base}/m.m3u8"), rec).unwrap();
        let mut out = Vec::new();
        r.read_to_end(&mut out).unwrap();
        assert_eq!(out, b"first-second");
    }

    #[test]
    fn encrypted_stream_gives_a_clear_error() {
        let base = serve(vec![("/e.m3u8", b"#EXTM3U\n#EXT-X-KEY:METHOD=AES-128,URI=\"k\"\n#EXTINF:2,\ns.ts\n".to_vec())]);
        let rec: crate::stream::Recorder = Arc::new(Mutex::new(None));
        let mut r = HlsReader::start(&format!("{base}/e.m3u8"), rec).unwrap();
        let err = r.read_to_end(&mut Vec::new()).unwrap_err();
        assert!(err.to_string().contains("encrypted"), "{err}");
    }
}

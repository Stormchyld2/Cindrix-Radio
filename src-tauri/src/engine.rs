//! Audio output. One dedicated thread owns the output device (the rodio stream is
//! not Send), and the rest of the app talks to it through a channel. Every decoded
//! sample passes through the global equalizer before it reaches the speakers.
use crate::dsp::{EqSettings, Equalizer};
use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use crate::stream::{self, CurrentInfo, NowPlaying, Recorder, StreamInfo, StreamSource};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

/// EQ settings shared between the UI side and the audio thread.
#[derive(Clone)]
pub struct SharedEq(Arc<RwLock<(u64, EqSettings)>>);

impl SharedEq {
    pub fn new(s: EqSettings) -> Self { SharedEq(Arc::new(RwLock::new((0, s.clamped())))) }
    pub fn set(&self, s: EqSettings) {
        let mut g = self.0.write().unwrap();
        g.0 += 1;
        g.1 = s.clamped();
    }
    fn snapshot(&self) -> (u64, EqSettings) { self.0.read().unwrap().clone() }
}

/// Wraps any f32 source and applies the equalizer, picking up live setting changes.
pub struct EqSource<S: Source<Item = f32>> {
    inner: S,
    eq: Equalizer,
    shared: SharedEq,
    version: u64,
    countdown: u32,
    frame: Vec<f32>,
    pos: usize,
    channels: usize,
}

impl<S: Source<Item = f32>> EqSource<S> {
    pub fn new(inner: S, shared: SharedEq) -> Self {
        let channels = inner.channels().max(1) as usize;
        let (version, settings) = shared.snapshot();
        let eq = Equalizer::new(inner.sample_rate(), channels, settings);
        EqSource { inner, eq, shared, version, countdown: 0, frame: Vec::new(), pos: 0, channels }
    }
}

impl<S: Source<Item = f32>> Iterator for EqSource<S> {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.pos >= self.frame.len() {
            // refill one small block so we process in frames, not single samples
            self.frame.clear();
            self.pos = 0;
            let want = self.channels * 256;
            for _ in 0..want {
                match self.inner.next() { Some(x) => self.frame.push(x), None => break }
            }
            if self.frame.is_empty() { return None; }
            // a trailing partial frame is passed through unfiltered by chunking rules; pad is not needed
            if self.countdown == 0 {
                let (v, s) = self.shared.snapshot();
                if v != self.version { self.eq.set(s); self.version = v; }
                self.countdown = 4;
            } else { self.countdown -= 1; }
            self.eq.process(&mut self.frame);
        }
        let x = self.frame[self.pos];
        self.pos += 1;
        Some(x)
    }
}

impl<S: Source<Item = f32>> Source for EqSource<S> {
    fn try_seek(&mut self, pos: Duration) -> Result<(), rodio::source::SeekError> {
        self.frame.clear();
        self.pos = 0;
        self.inner.try_seek(pos)
    }
    fn current_frame_len(&self) -> Option<usize> { None }
    fn channels(&self) -> u16 { self.inner.channels() }
    fn sample_rate(&self) -> u32 { self.inner.sample_rate() }
    fn total_duration(&self) -> Option<Duration> { self.inner.total_duration() }
}

static TRACE: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// A short log of what the player is doing, shown in Settings → Advanced & Diagnostics.
pub fn trace(msg: impl Into<String>) {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() % 100_000).unwrap_or(0);
    let mut t = TRACE.lock().unwrap();
    t.push(format!("[{secs}] {}", msg.into()));
    if t.len() > 60 { t.remove(0); }
}
pub fn trace_dump() -> Vec<String> { TRACE.lock().unwrap().clone() }

/// Counts decoded audio so the log can say when and why a file stopped.
struct Counted<S: Source<Item = f32>> { inner: S, n: Arc<AtomicU64>, done: bool, rate: u32, ch: u16 }

impl<S: Source<Item = f32>> Iterator for Counted<S> {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        match self.inner.next() {
            Some(x) => { self.n.fetch_add(1, Ordering::Relaxed); Some(x) }
            None => {
                if !self.done {
                    self.done = true;
                    let secs = self.n.load(Ordering::Relaxed) as f64 / (self.rate.max(1) as f64 * self.ch.max(1) as f64);
                    trace(format!("decoder ended after {secs:.1}s of audio"));
                }
                None
            }
        }
    }
}

impl<S: Source<Item = f32>> Source for Counted<S> {
    fn current_frame_len(&self) -> Option<usize> { self.inner.current_frame_len() }
    fn channels(&self) -> u16 { self.inner.channels() }
    fn sample_rate(&self) -> u32 { self.inner.sample_rate() }
    fn total_duration(&self) -> Option<Duration> { self.inner.total_duration() }
    fn try_seek(&mut self, pos: Duration) -> Result<(), rodio::source::SeekError> {
        let r = self.inner.try_seek(pos);
        if r.is_ok() { self.n.store((pos.as_secs_f64() * self.rate as f64 * self.ch as f64) as u64, Ordering::Relaxed); self.done = false; }
        r
    }
}

/// Decodes and equalizes ahead of time on its own thread, so the phone's audio callback only
/// copies ready samples. This keeps playback steady when the CPU is busy or the audio route
/// (Bluetooth) uses larger buffers; a seek discards what was decoded ahead.
struct Prefetch {
    rx: std::sync::mpsc::Receiver<(u64, Vec<f32>)>,
    ctl: std::sync::mpsc::Sender<Duration>,
    want: u64,
    cur: Vec<f32>,
    pos: usize,
    eof: bool,
    ch: u16,
    rate: u32,
    total: Option<Duration>,
}

impl Prefetch {
    const CHUNK: usize = 4096;
    fn new<S: Source<Item = f32> + Send + 'static>(mut inner: S) -> Self {
        let (ch, rate, total) = (inner.channels(), inner.sample_rate(), inner.total_duration());
        let (tx, rx) = std::sync::mpsc::sync_channel::<(u64, Vec<f32>)>(16);
        let (ctl, crx) = std::sync::mpsc::channel::<Duration>();
        std::thread::spawn(move || {
            let mut gen = 0u64;
            loop {
                while let Ok(d) = crx.try_recv() { let _ = inner.try_seek(d); gen += 1; }
                let mut chunk = Vec::with_capacity(Self::CHUNK);
                while chunk.len() < Self::CHUNK {
                    match inner.next() { Some(x) => chunk.push(x), None => break }
                }
                let ended = chunk.len() < Self::CHUNK;
                if !chunk.is_empty() && tx.send((gen, chunk)).is_err() { return; }
                if ended {
                    if tx.send((gen, Vec::new())).is_err() { return; }
                    // finished: wait in case the listener seeks back, and exit once we are dropped
                    match crx.recv() { Ok(d) => { let _ = inner.try_seek(d); gen += 1; } Err(_) => return }
                }
            }
        });
        Prefetch { rx, ctl, want: 0, cur: Vec::new(), pos: 0, eof: false, ch, rate, total }
    }
}

impl Iterator for Prefetch {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.pos >= self.cur.len() {
            loop {
                if self.eof { return None; }
                match self.rx.recv() {
                    Ok((g, v)) => {
                        if g != self.want { continue; }
                        if v.is_empty() { self.eof = true; return None; }
                        self.cur = v;
                        self.pos = 0;
                        break;
                    }
                    Err(_) => return None,
                }
            }
        }
        let x = self.cur[self.pos];
        self.pos += 1;
        Some(x)
    }
}

impl Source for Prefetch {
    fn current_frame_len(&self) -> Option<usize> { None }
    fn channels(&self) -> u16 { self.ch }
    fn sample_rate(&self) -> u32 { self.rate }
    fn total_duration(&self) -> Option<Duration> { self.total }
    fn try_seek(&mut self, pos: Duration) -> Result<(), rodio::source::SeekError> {
        self.want += 1;
        self.eof = false;
        self.cur.clear();
        self.pos = 0;
        let _ = self.ctl.send(pos);
        Ok(())
    }
}

/// Open a file and queue it on the sink (through the equalizer).
fn append_file(sink: &Sink, path: &str, sh: &SharedEq) -> Result<(), String> {
    let f = File::open(path).map_err(|e| e.to_string())?;
    let dec = Decoder::new(BufReader::new(f)).map_err(|e| e.to_string())?;
    let (rate, ch, total) = (dec.sample_rate(), dec.channels(), dec.total_duration());
    trace(format!("play {} ({rate} Hz, {ch} ch, length {:?})", Path::new(path).file_name().and_then(|n| n.to_str()).unwrap_or("?"), total.map(|d| d.as_secs())));
    let counted = Counted { inner: dec.convert_samples::<f32>(), n: Arc::new(AtomicU64::new(0)), done: false, rate, ch };
    sink.append(Prefetch::new(EqSource::new(counted, sh.clone())));
    Ok(())
}

enum Cmd { Play(String), Ready(u64, StreamSource, StreamInfo), Pause, Resume, Stop, Volume(f32), Seek(u32), Reroute }

#[derive(Clone, Default)]
pub struct Status {
    pub position_secs: u32,
    pub ended: bool,
    pub error: Option<String>,
    pub now_playing: Option<String>,
    pub recording: bool,
    pub connecting: bool,
}

/// "YYYY-MM-DD_HHMMSS" in UTC, for recording file names.
pub fn utc_stamp(epoch_secs: u64) -> String {
    let days = (epoch_secs / 86400) as i64;
    let rem = epoch_secs % 86400;
    // civil-from-days (Howard Hinnant)
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}_{:02}{:02}{:02}", rem / 3600, (rem % 3600) / 60, rem % 60)
}

pub struct Engine {
    tx: Mutex<Sender<Cmd>>,
    status: Arc<Mutex<Status>>,
    ended: Arc<AtomicBool>,
    gen: Arc<AtomicU64>,
    connecting: Arc<AtomicBool>,
    now: NowPlaying,
    rec: Recorder,
    info: CurrentInfo,
    pub eq: SharedEq,
}

impl Engine {
    pub fn start(eq: EqSettings, volume: f32) -> Engine {
        let (tx, rx) = channel::<Cmd>();
        let status = Arc::new(Mutex::new(Status::default()));
        let ended = Arc::new(AtomicBool::new(false));
        let shared = SharedEq::new(eq);
        let (st, en, sh) = (status.clone(), ended.clone(), shared.clone());
        let gen = Arc::new(AtomicU64::new(0));
        let connecting = Arc::new(AtomicBool::new(false));
        let info: CurrentInfo = Arc::new(Mutex::new(None));
        let (gen_t, conn_t, info_t) = (gen.clone(), connecting.clone(), info.clone());

        std::thread::spawn(move || {
            let mut force = false;
            let open = |vol: f32| -> Result<(OutputStream, rodio::OutputStreamHandle, Sink), String> {
                let (stream, handle) = OutputStream::try_default().map_err(|e| format!("No audio output: {e}"))?;
                let sink = Sink::try_new(&handle).map_err(|e| format!("Audio error: {e}"))?;
                sink.set_volume(vol);
                Ok((stream, handle, sink))
            };
            let (mut _stream, mut _handle, mut sink) = match open(volume) {
                Ok(t) => { trace("audio output opened"); t }
                Err(e) => {
                    trace(e.clone());
                    st.lock().unwrap().error = Some(e);
                    // keep draining commands so callers never block
                    while rx.recv().is_ok() {}
                    return;
                }
            };
            let mut cur_vol = volume;
            let mut cur_path: Option<String> = None;
            let mut paused = false;
            let mut last_pos = Duration::ZERO;
            let mut last_move = std::time::Instant::now();
            let mut playing = false;
            let mut is_stream = false;
            loop {
                match rx.recv_timeout(Duration::from_millis(250)) {
                    Ok(Cmd::Play(path)) => {
                        sink.stop();
                        match append_file(&sink, &path, &sh) {
                            Ok(()) => {
                                sink.play();
                                playing = true;
                                paused = false;
                                is_stream = false;
                                cur_path = Some(path);
                                last_pos = Duration::ZERO;
                                last_move = std::time::Instant::now();
                                en.store(false, Ordering::SeqCst);
                                st.lock().unwrap().error = None;
                            }
                            Err(e) => { trace(format!("cannot open file: {e}")); playing = false; st.lock().unwrap().error = Some(format!("Cannot play {path}: {e}")); }
                        }
                    }
                    Ok(Cmd::Ready(g, src, i)) => {
                        if g == gen_t.load(Ordering::SeqCst) {
                            sink.stop();
                            sink.append(EqSource::new(src, sh.clone()));
                            sink.play();
                            playing = true;
                            is_stream = true;
                            paused = false;
                            cur_path = None;
                            en.store(false, Ordering::SeqCst);
                            *info_t.lock().unwrap() = Some(i);
                            conn_t.store(false, Ordering::SeqCst);
                            st.lock().unwrap().error = None;
                        }
                    }
                    Ok(Cmd::Reroute) => { trace("audio device changed"); force = true }
                    Ok(Cmd::Pause) => { trace("pause"); paused = true; sink.pause() }
                    Ok(Cmd::Resume) => { trace("resume"); paused = false; last_move = std::time::Instant::now(); sink.play() }
                    Ok(Cmd::Stop) => { trace("stop"); sink.stop(); playing = false; cur_path = None; }
                    Ok(Cmd::Volume(v)) => { cur_vol = v; sink.set_volume(v) }
                    Ok(Cmd::Seek(secs)) => { let r = sink.try_seek(Duration::from_secs(secs as u64)); trace(format!("seek to {secs}s: {}", if r.is_ok() { "ok".to_string() } else { format!("failed ({:?})", r.err()) })); }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return,
                }
                // Watchdog: if the phone's audio route changed (Bluetooth, headset) the output can go
                // silent while the clock stands still. Reopen the output and carry on from there.
                let pos = sink.get_pos();
                if playing && !paused && !is_stream && cur_path.is_some() && !sink.empty() {
                    if pos != last_pos { last_pos = pos; last_move = std::time::Instant::now(); }
                    if force || (pos == last_pos && last_move.elapsed() > Duration::from_secs(3)) {
                        force = false;
                        trace(format!("output stalled at {}s; reopening the audio output", pos.as_secs()));
                        match open(cur_vol) {
                            Ok((ns, nh, nsink)) => {
                                let path = cur_path.clone().unwrap_or_default();
                                match append_file(&nsink, &path, &sh) {
                                    Ok(()) => {
                                        let r = nsink.try_seek(pos);
                                        trace(format!("reopened; seek back to {}s: {}", pos.as_secs(), if r.is_ok() { "ok".to_string() } else { format!("failed ({:?})", r.err()) }));
                                        nsink.play();
                                        sink.stop();
                                        sink = nsink;
                                        _handle = nh;
                                        _stream = ns;
                                    }
                                    Err(e) => trace(format!("reopen failed: {e}")),
                                }
                            }
                            Err(e) => trace(format!("reopen failed: {e}")),
                        }
                        last_move = std::time::Instant::now();
                    }
                } else { force = false; last_pos = pos; last_move = std::time::Instant::now(); }
                let mut s = st.lock().unwrap();
                s.position_secs = sink.get_pos().as_secs() as u32;
                if playing && sink.empty() {
                    trace(format!("output empty at position {}s", s.position_secs));
                    playing = false;
                    if is_stream { s.error = Some("The stream stopped. The station may have gone offline.".into()); }
                    else { en.store(true, Ordering::SeqCst); }
                }
                s.ended = en.load(Ordering::SeqCst);
            }
        });

        Engine { tx: Mutex::new(tx), status, ended, gen, connecting, now: Arc::new(Mutex::new(None)), rec: Arc::new(Mutex::new(None)), info, eq: shared }
    }

    fn send(&self, c: Cmd) { let _ = self.tx.lock().unwrap().send(c); }
    pub fn play_file(&self, path: &str) {
        self.gen.fetch_add(1, Ordering::SeqCst);
        self.connecting.store(false, Ordering::SeqCst);
        self.stop_recording();
        *self.now.lock().unwrap() = None;
        self.ended.store(false, Ordering::SeqCst);
        self.send(Cmd::Play(path.to_string()));
    }

    /// Connect to a live stream in the background; sound starts when audio arrives.
    pub fn play_stream(&self, url: &str) {
        let g = self.gen.fetch_add(1, Ordering::SeqCst) + 1;
        self.stop_recording();
        *self.now.lock().unwrap() = None;
        *self.info.lock().unwrap() = None;
        self.connecting.store(true, Ordering::SeqCst);
        self.status.lock().unwrap().error = None;
        self.send(Cmd::Stop);
        let (tx, url) = (self.tx.lock().unwrap().clone(), url.to_string());
        let (now, rec, gen, conn, status) = (self.now.clone(), self.rec.clone(), self.gen.clone(), self.connecting.clone(), self.status.clone());
        std::thread::spawn(move || {
            trace(format!("radio: connecting to {url}"));
            let (rtx, rrx) = std::sync::mpsc::channel();
            let u2 = url.clone();
            std::thread::spawn(move || {
                let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| stream::open(&u2, now, rec)))
                    .unwrap_or_else(|_| Err("The stream decoder crashed on this station".to_string()));
                let _ = rtx.send(r);
            });
            let res = rrx.recv_timeout(std::time::Duration::from_secs(25))
                .unwrap_or_else(|_| Err("This station did not answer in time. Try another one.".to_string()));
            match res {
                Ok(o) => { trace(format!("radio: connected ({:?})", o.info.content_type)); let _ = tx.send(Cmd::Ready(g, o.source, o.info)); }
                Err(e) => {
                    trace(format!("radio: failed: {e}"));
                    if gen.load(Ordering::SeqCst) == g {
                        conn.store(false, Ordering::SeqCst);
                        status.lock().unwrap().error = Some(e);
                    }
                }
            }
        });
    }

    /// Start saving the live stream to <dir>/<station> <date>.<ext>. Returns the file path.
    pub fn start_recording(&self, dir: &Path, station: &str) -> Result<PathBuf, String> {
        let info = self.info.lock().unwrap().clone().ok_or("Play a station first, then record it")?;
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let name = format!("{} {}.{}", crate::podcasts::sanitize_name(station), utc_stamp(secs), info.extension());
        let path = dir.join(name);
        let f = std::fs::File::create(&path).map_err(|e| e.to_string())?;
        *self.rec.lock().unwrap() = Some(f);
        Ok(path)
    }

    pub fn set_error(&self, msg: &str) { self.status.lock().unwrap().error = Some(msg.to_string()); }

    pub fn stop_recording(&self) { *self.rec.lock().unwrap() = None; }
    pub fn pause(&self) { self.send(Cmd::Pause) }
    pub fn resume(&self) { self.send(Cmd::Resume) }
    pub fn stop(&self) {
        self.gen.fetch_add(1, Ordering::SeqCst);
        self.connecting.store(false, Ordering::SeqCst);
        self.stop_recording();
        self.send(Cmd::Stop);
    }
    pub fn set_volume(&self, v: f32) { self.send(Cmd::Volume(v)) }
    pub fn seek(&self, secs: u32) { self.send(Cmd::Seek(secs)) }
    pub fn reroute(&self) { self.send(Cmd::Reroute) }
    pub fn set_eq(&self, s: EqSettings) { self.eq.set(s) }
    pub fn status(&self) -> Status {
        let mut s = self.status.lock().unwrap().clone();
        s.now_playing = self.now.lock().unwrap().clone();
        s.recording = self.rec.lock().unwrap().is_some();
        s.connecting = self.connecting.load(Ordering::SeqCst);
        s
    }
    pub fn take_ended(&self) -> bool { self.ended.swap(false, Ordering::SeqCst) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rodio::source::SineWave;

    fn collect(shared: &SharedEq, n: usize) -> Vec<f32> {
        let src = SineWave::new(1000.0).take_duration(Duration::from_millis(500)).amplify(0.1);
        EqSource::new(src, shared.clone()).take(n).collect()
    }
    fn peak(v: &[f32]) -> f32 { v[v.len() / 2..].iter().fold(0.0, |m, x| m.max(x.abs())) }

    struct Probe { seeks: Arc<Mutex<Vec<Duration>>>, n: usize }
    impl Iterator for Probe {
        type Item = f32;
        fn next(&mut self) -> Option<f32> { if self.n == 0 { None } else { self.n -= 1; Some(0.0) } }
    }
    impl Source for Probe {
        fn current_frame_len(&self) -> Option<usize> { None }
        fn channels(&self) -> u16 { 1 }
        fn sample_rate(&self) -> u32 { 44100 }
        fn total_duration(&self) -> Option<Duration> { None }
        fn try_seek(&mut self, p: Duration) -> Result<(), rodio::source::SeekError> { self.seeks.lock().unwrap().push(p); Ok(()) }
    }

    #[test]
    fn recording_stamp_is_a_readable_utc_date() {
        assert_eq!(utc_stamp(0), "1970-01-01_000000");
        assert_eq!(utc_stamp(1_762_560_000 + 3661), "2025-11-08_010101");
        assert_eq!(utc_stamp(951_782_400), "2000-02-29_000000");
    }

    #[test]
    fn eq_source_forwards_seeks() {
        let seeks = Arc::new(Mutex::new(Vec::new()));
        let mut e = EqSource::new(Probe { seeks: seeks.clone(), n: 1000 }, SharedEq::new(EqSettings::default()));
        e.try_seek(Duration::from_secs(5)).unwrap();
        assert_eq!(*seeks.lock().unwrap(), vec![Duration::from_secs(5)]);
    }

    #[test]
    fn eq_source_flat_is_transparent() {
        let shared = SharedEq::new(EqSettings::default());
        let out = collect(&shared, 20000);
        assert!((peak(&out) - 0.1).abs() < 0.01, "peak {}", peak(&out));
    }

    #[test]
    fn eq_source_applies_settings_set_before_playback() {
        let mut s = EqSettings::default();
        s.gains_db[5] = 12.0;
        let shared = SharedEq::new(s);
        let out = collect(&shared, 20000);
        assert!(peak(&out) > 0.3, "peak {}", peak(&out));
    }

    #[test]
    fn eq_source_picks_up_live_changes() {
        let shared = SharedEq::new(EqSettings::default());
        let src = SineWave::new(1000.0).take_duration(Duration::from_secs(2)).amplify(0.1);
        let mut e = EqSource::new(src, shared.clone());
        let before: Vec<f32> = (&mut e).take(10000).collect();
        let mut s = EqSettings::default();
        s.gains_db[5] = 12.0;
        shared.set(s);
        let after: Vec<f32> = (&mut e).take(40000).collect();
        assert!(peak(&after) > peak(&before) * 2.0, "{} vs {}", peak(&after), peak(&before));
    }
}

//! Audio output. One dedicated thread owns the output device (the rodio stream is
//! not Send), and the rest of the app talks to it through a channel. Every decoded
//! sample passes through the global equalizer before it reaches the speakers.
use crate::dsp::{EqSettings, Equalizer};
use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::sync::atomic::{AtomicBool, Ordering};
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

enum Cmd { Play(String), Pause, Resume, Stop, Volume(f32), Seek(u32) }

#[derive(Clone, Default)]
pub struct Status {
    pub position_secs: u32,
    pub ended: bool,
    pub error: Option<String>,
}

pub struct Engine {
    tx: Mutex<Sender<Cmd>>,
    status: Arc<Mutex<Status>>,
    ended: Arc<AtomicBool>,
    pub eq: SharedEq,
}

impl Engine {
    pub fn start(eq: EqSettings, volume: f32) -> Engine {
        let (tx, rx) = channel::<Cmd>();
        let status = Arc::new(Mutex::new(Status::default()));
        let ended = Arc::new(AtomicBool::new(false));
        let shared = SharedEq::new(eq);
        let (st, en, sh) = (status.clone(), ended.clone(), shared.clone());

        std::thread::spawn(move || {
            let device = OutputStream::try_default();
            let (_stream, handle) = match device {
                Ok(d) => d,
                Err(e) => { st.lock().unwrap().error = Some(format!("No audio output: {e}")); 
                    // keep draining commands so callers never block
                    while rx.recv().is_ok() {}
                    return; }
            };
            let sink = match Sink::try_new(&handle) {
                Ok(s) => s,
                Err(e) => { st.lock().unwrap().error = Some(format!("Audio error: {e}")); while rx.recv().is_ok() {} return; }
            };
            sink.set_volume(volume);
            let mut playing = false;
            loop {
                match rx.recv_timeout(Duration::from_millis(250)) {
                    Ok(Cmd::Play(path)) => {
                        sink.stop();
                        let opened = File::open(&path).map_err(|e| e.to_string())
                            .and_then(|f| Decoder::new(BufReader::new(f)).map_err(|e| e.to_string()));
                        match opened {
                            Ok(dec) => {
                                sink.append(EqSource::new(dec.convert_samples::<f32>(), sh.clone()));
                                sink.play();
                                playing = true;
                                en.store(false, Ordering::SeqCst);
                                st.lock().unwrap().error = None;
                            }
                            Err(e) => { playing = false; st.lock().unwrap().error = Some(format!("Cannot play {path}: {e}")); }
                        }
                    }
                    Ok(Cmd::Pause) => sink.pause(),
                    Ok(Cmd::Resume) => sink.play(),
                    Ok(Cmd::Stop) => { sink.stop(); playing = false; }
                    Ok(Cmd::Volume(v)) => sink.set_volume(v),
                    Ok(Cmd::Seek(secs)) => { let _ = sink.try_seek(Duration::from_secs(secs as u64)); }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return,
                }
                let mut s = st.lock().unwrap();
                s.position_secs = sink.get_pos().as_secs() as u32;
                if playing && sink.empty() { playing = false; en.store(true, Ordering::SeqCst); }
                s.ended = en.load(Ordering::SeqCst);
            }
        });

        Engine { tx: Mutex::new(tx), status, ended, eq: shared }
    }

    fn send(&self, c: Cmd) { let _ = self.tx.lock().unwrap().send(c); }
    pub fn play_file(&self, path: &str) { self.ended.store(false, Ordering::SeqCst); self.send(Cmd::Play(path.to_string())); }
    pub fn pause(&self) { self.send(Cmd::Pause) }
    pub fn resume(&self) { self.send(Cmd::Resume) }
    pub fn stop(&self) { self.send(Cmd::Stop) }
    pub fn set_volume(&self, v: f32) { self.send(Cmd::Volume(v)) }
    pub fn seek(&self, secs: u32) { self.send(Cmd::Seek(secs)) }
    pub fn set_eq(&self, s: EqSettings) { self.eq.set(s) }
    pub fn status(&self) -> Status { self.status.lock().unwrap().clone() }
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

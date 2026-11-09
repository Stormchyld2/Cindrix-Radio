//! Global 10-band graphic EQ (RBJ peaking biquads) with preamp and bypass.
//! Bands: 31/62/125/250/500 Hz and 1/2/4/8/16 kHz, +/-12 dB.
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

pub const BANDS_HZ: [f32; 10] = [31.0, 62.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0];
pub const MAX_DB: f32 = 12.0;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EqSettings {
    pub bypass: bool,
    pub preamp_db: f32,
    pub gains_db: [f32; 10],
    /// Karaoke vocal reduction, 0.0 (off) to 1.0 (full). Applied before the EQ, stereo only.
    #[serde(default)]
    pub vocal_reduction: f32,
}

impl Default for EqSettings {
    fn default() -> Self { EqSettings { bypass: false, preamp_db: 0.0, gains_db: [0.0; 10], vocal_reduction: 0.0 } }
}

impl EqSettings {
    pub fn clamped(mut self) -> Self {
        self.preamp_db = self.preamp_db.clamp(-MAX_DB, MAX_DB);
        for g in self.gains_db.iter_mut() { *g = g.clamp(-MAX_DB, MAX_DB); }
        self.vocal_reduction = 0.0; // the vocal remover was removed; ignore anything saved by older builds
        self
    }
}

#[derive(Clone, Copy, Default)]
struct Biquad { b0: f32, b1: f32, b2: f32, a1: f32, a2: f32, z1: f32, z2: f32 }

impl Biquad {
    fn peaking(sample_rate: f32, freq: f32, gain_db: f32, q: f32) -> Self {
        let a = 10f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * (freq.min(sample_rate * 0.45)) / sample_rate;
        let alpha = w0.sin() / (2.0 * q);
        let cos = w0.cos();
        let a0 = 1.0 + alpha / a;
        Biquad {
            b0: (1.0 + alpha * a) / a0,
            b1: (-2.0 * cos) / a0,
            b2: (1.0 - alpha * a) / a0,
            a1: (-2.0 * cos) / a0,
            a2: (1.0 - alpha / a) / a0,
            ..Default::default()
        }
    }
    #[inline]
    fn run(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        // flush tiny values: denormal numbers make low-frequency filters very slow on phone CPUs
        if self.z1.abs() < 1e-15 { self.z1 = 0.0; }
        if self.z2.abs() < 1e-15 { self.z2 = 0.0; }
        y
    }
}

/// Transparent below 0.8; rounds peaks off smoothly instead of hard-clipping them (no crackle or pumping).
fn soft_limit(x: f32) -> f32 {
    let a = x.abs();
    if a <= 0.8 { x } else { x.signum() * (0.8 + 0.2 * ((a - 0.8) / 0.2).tanh()) }
}

pub struct Equalizer {
    sample_rate: f32,
    channels: usize,
    settings: EqSettings,
    filters: Vec<[Biquad; 10]>, // one chain per channel
    preamp: f32,
    /// vocal reducer: low-pass memory for the centre (mid) signal, so bass and kick stay
    lp: f32,
    lp2: f32,
    lp_alpha: f32,
}

impl Equalizer {
    pub fn new(sample_rate: u32, channels: usize, settings: EqSettings) -> Self {
        let mut e = Equalizer { sample_rate: sample_rate as f32, channels, settings: EqSettings::default(), filters: vec![], preamp: 1.0, lp: 0.0, lp2: 0.0, lp_alpha: 1.0 - (-2.0 * PI * 150.0 / (sample_rate as f32)).exp() };
        e.set(settings);
        e
    }

    pub fn set(&mut self, settings: EqSettings) {
        let s = settings.clamped();
        let mut chain: [Biquad; 10] = std::array::from_fn(|i| Biquad::peaking(self.sample_rate, BANDS_HZ[i], s.gains_db[i], 1.414));
        if self.filters.len() == self.channels {
            // keep filter memory so dragging a slider does not click
            for (ch, old) in self.filters.iter().enumerate() {
                for i in 0..10 { chain[i].z1 = old[i].z1; chain[i].z2 = old[i].z2; }
                let _ = ch;
            }
        }
        self.filters = vec![chain; self.channels];
        self.preamp = 10f32.powf(s.preamp_db / 20.0);
        self.settings = s;
    }

    /// Process interleaved samples in place.
    pub fn process(&mut self, buf: &mut [f32]) {
        if self.settings.bypass { return; }
        for frame in buf.chunks_mut(self.channels) {
            for (ch, x) in frame.iter_mut().enumerate() {
                let mut v = *x * self.preamp;
                for f in self.filters[ch].iter_mut() { v = f.run(v); }
                *x = soft_limit(v);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(freq: f32, sr: u32, n: usize) -> Vec<f32> {
        (0..n).map(|i| 0.1 * (2.0 * PI * freq * i as f32 / sr as f32).sin()).collect()
    }
    fn peak(v: &[f32]) -> f32 { v[v.len() / 2..].iter().fold(0.0, |m, x| m.max(x.abs())) }

    fn stereo(l: &[f32], r: &[f32]) -> Vec<f32> { l.iter().zip(r).flat_map(|(a, b)| [*a, *b]).collect() }
    fn left_peak(b: &[f32]) -> f32 { b.chunks(2).skip(b.len() / 4).fold(0.0, |m, f| m.max(f[0].abs())) }

    #[test]
    fn flat_eq_is_transparent() {
        let mut e = Equalizer::new(44100, 1, EqSettings::default());
        let mut b = sine(1000.0, 44100, 8000);
        let orig = b.clone();
        e.process(&mut b);
        assert!((peak(&b) - peak(&orig)).abs() < 0.001);
    }

    #[test]
    fn plus_12db_at_1k_boosts_about_4x() {
        let mut s = EqSettings::default();
        s.gains_db[5] = 12.0;
        let mut e = Equalizer::new(44100, 1, s);
        let mut b = sine(1000.0, 44100, 16000);
        let before = peak(&b);
        e.process(&mut b);
        let ratio = peak(&b) / before;
        assert!(ratio > 3.5 && ratio < 4.5, "ratio {ratio}");
    }

    #[test]
    fn bypass_leaves_audio_untouched() {
        let mut s = EqSettings::default();
        s.gains_db = [12.0; 10];
        s.bypass = true;
        let mut e = Equalizer::new(44100, 2, s);
        let mut b = sine(500.0, 44100, 4000);
        let orig = b.clone();
        e.process(&mut b);
        assert_eq!(b, orig);
    }

    #[test]
    fn gains_are_clamped_to_12db() {
        let mut s = EqSettings::default();
        s.gains_db[0] = 99.0;
        assert_eq!(s.clamped().gains_db[0], 12.0);
    }
}

//! Player state machine shared by Windows and Android UIs.
//! The audio backend (decode, 10-band EQ, output) plugs in behind this in the
//! next milestone; the UI is already wired to these commands.
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Kind { Music, Radio, Podcast, Audiobook, Video }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Track {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub kind: Kind,
    pub duration_secs: Option<u32>,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct PlayerState {
    pub current: Option<Track>,
    pub queue: Vec<Track>,
    pub playing: bool,
    pub position_secs: u32,
    pub volume: f32,
}

pub struct Player(pub Mutex<PlayerState>);

impl Player {
    pub fn new() -> Self {
        Player(Mutex::new(PlayerState { volume: 0.8, ..Default::default() }))
    }
}

impl PlayerState {
    pub fn play(&mut self, t: Track) {
        self.current = Some(t);
        self.position_secs = 0;
        self.playing = true;
    }
    pub fn toggle(&mut self) {
        if self.current.is_some() { self.playing = !self.playing; }
    }
    pub fn set_volume(&mut self, v: f32) { self.volume = v.clamp(0.0, 1.0); }
    pub fn enqueue(&mut self, t: Track) { self.queue.push(t); }
    pub fn next(&mut self) {
        if self.queue.is_empty() { self.playing = false; return; }
        let t = self.queue.remove(0);
        self.play(t);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn t(id: &str) -> Track {
        Track { id: id.into(), title: id.into(), subtitle: String::new(), kind: Kind::Music, duration_secs: None }
    }
    #[test]
    fn next_advances_queue() {
        let mut s = PlayerState::default();
        s.play(t("a")); s.enqueue(t("b"));
        s.next();
        assert_eq!(s.current.unwrap().id, "b");
        assert!(s.playing);
    }
    #[test]
    fn toggle_without_track_stays_stopped() {
        let mut s = PlayerState::default();
        s.toggle();
        assert!(!s.playing);
    }
    #[test]
    fn volume_is_clamped() {
        let mut s = PlayerState::default();
        s.set_volume(3.0);
        assert_eq!(s.volume, 1.0);
    }
}

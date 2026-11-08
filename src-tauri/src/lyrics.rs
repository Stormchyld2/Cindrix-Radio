//! Lyrics the user pastes in themselves. Cindrix Radio ships none and looks none up online.
//! They are stored only on this device (lyrics.json in the app data folder), keyed by the
//! song's file, and are never uploaded, shared or sent anywhere.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

pub const MAX_CHARS: usize = 100_000;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Entry { key: String, path: String, title: String, text: String }

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Summary { pub path: String, pub title: String, pub chars: usize }

pub struct LyricsStore { items: Mutex<Vec<Entry>>, path: PathBuf }

/// Same song whatever the slash style or letter case (Windows paths).
fn key(path: &str) -> String { path.replace('\\', "/").to_lowercase() }

impl LyricsStore {
    pub fn load(path: PathBuf) -> LyricsStore {
        let items = std::fs::read_to_string(&path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
        LyricsStore { items: Mutex::new(items), path }
    }

    fn save(&self, items: &[Entry]) {
        if let Some(d) = self.path.parent() { let _ = std::fs::create_dir_all(d); }
        let tmp = self.path.with_extension("json.tmp");
        if let Ok(body) = serde_json::to_string(items) {
            if std::fs::write(&tmp, body).is_ok() { let _ = std::fs::rename(tmp, &self.path); }
        }
    }

    pub fn get(&self, path: &str) -> Option<String> {
        let k = key(path);
        self.items.lock().unwrap().iter().find(|e| e.key == k).map(|e| e.text.clone())
    }

    /// Save lyrics for a song; empty text removes them.
    pub fn set(&self, path: &str, title: &str, text: &str) -> Result<(), String> {
        if path.trim().is_empty() { return Err("Pick the song these lyrics belong to.".into()); }
        if text.chars().count() > MAX_CHARS { return Err("That is too long for lyrics.".into()); }
        let k = key(path);
        let mut items = self.items.lock().unwrap();
        items.retain(|e| e.key != k);
        if !text.trim().is_empty() {
            items.push(Entry { key: k, path: path.to_string(), title: title.trim().to_string(), text: text.to_string() });
        }
        self.save(&items);
        Ok(())
    }

    pub fn list(&self) -> Vec<Summary> {
        let mut v: Vec<Summary> = self.items.lock().unwrap().iter()
            .map(|e| Summary { path: e.path.clone(), title: e.title.clone(), chars: e.text.chars().count() }).collect();
        v.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("cindrix-radio-ly-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d.join("lyrics.json")
    }

    #[test]
    fn lyrics_are_keyed_by_song_and_survive_restart() {
        let p = temp("a");
        let s = LyricsStore::load(p.clone());
        s.set("C:\\Music\\Song.mp3", "Song", "my own words").unwrap();
        assert_eq!(s.get("c:/music/song.MP3").as_deref(), Some("my own words"), "slash style and case do not matter");
        assert_eq!(LyricsStore::load(p.clone()).get("C:\\Music\\Song.mp3").as_deref(), Some("my own words"));
        assert_eq!(s.list(), vec![Summary { path: "C:\\Music\\Song.mp3".into(), title: "Song".into(), chars: 12 }]);
        let _ = std::fs::remove_dir_all(p.parent().unwrap());
    }

    #[test]
    fn empty_text_removes_and_bad_input_is_refused() {
        let s = LyricsStore::load(temp("b"));
        s.set("/m/a.mp3", "A", "x").unwrap();
        s.set("/m/a.mp3", "A", "   ").unwrap();
        assert_eq!(s.get("/m/a.mp3"), None);
        assert!(s.set("", "A", "x").is_err());
        assert!(s.set("/m/a.mp3", "A", &"x".repeat(MAX_CHARS + 1)).is_err());
        s.set("/m/a.mp3", "A", "one").unwrap();
        s.set("/m/a.mp3", "A", "two").unwrap();
        assert_eq!(s.list().len(), 1, "saving again replaces");
        assert_eq!(s.get("/m/a.mp3").as_deref(), Some("two"));
    }
}

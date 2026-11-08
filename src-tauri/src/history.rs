//! Resume playback: remembers where each podcast / audiobook was, so reopening it
//! continues from that spot. Saved to the app data folder with an atomic write
//! (temp file, then rename) so a power loss cannot corrupt it.
use crate::library::Source;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Entry {
    pub secs: u32,
    pub played: bool,
}

#[derive(Serialize, Deserialize, Default)]
struct File { entries: BTreeMap<String, Entry> }

/// Stable key: "<source id>:<path relative to the source>" when the file lives in a
/// library source (so a changed drive letter keeps its progress), otherwise the full path.
pub fn key_for(path: &str, sources: &[Source]) -> String {
    let p = Path::new(path);
    for s in sources {
        if let Ok(rel) = p.strip_prefix(&s.path) {
            return format!("{}:{}", s.id, rel.to_string_lossy().replace('\\', "/"));
        }
    }
    path.replace('\\', "/")
}

#[derive(Clone)]
pub struct HistoryStore { entries: Arc<Mutex<BTreeMap<String, Entry>>>, path: PathBuf, last_save: Arc<Mutex<std::time::Instant>> }

impl HistoryStore {
    pub fn load(path: PathBuf) -> HistoryStore {
        let entries = std::fs::read_to_string(&path).ok()
            .and_then(|s| serde_json::from_str::<File>(&s).ok()).map(|f| f.entries).unwrap_or_default();
        HistoryStore { entries: Arc::new(Mutex::new(entries)), path, last_save: Arc::new(Mutex::new(std::time::Instant::now())) }
    }

    fn write(&self, e: &BTreeMap<String, Entry>) {
        if let Some(d) = self.path.parent() { let _ = std::fs::create_dir_all(d); }
        let tmp = self.path.with_extension("json.tmp");
        let body = serde_json::to_string_pretty(&File { entries: e.clone() }).unwrap();
        if std::fs::write(&tmp, body).is_ok() { let _ = std::fs::rename(tmp, &self.path); }
    }

    /// Forget every saved position and played mark.
    pub fn clear(&self) {
        let mut g = self.entries.lock().unwrap();
        g.clear();
        self.write(&g);
    }

    /// Save a position now.
    pub fn record(&self, key: &str, secs: u32) {
        let mut g = self.entries.lock().unwrap();
        g.insert(key.to_string(), Entry { secs, played: false });
        self.write(&g);
        *self.last_save.lock().unwrap() = std::time::Instant::now();
    }

    /// Save a position only if `every_secs` have passed since the last save.
    pub fn record_periodic(&self, key: &str, secs: u32, every_secs: u64) {
        if self.last_save.lock().unwrap().elapsed().as_secs() >= every_secs { self.record(key, secs); }
    }

    pub fn mark_played(&self, key: &str) {
        let mut g = self.entries.lock().unwrap();
        g.insert(key.to_string(), Entry { secs: 0, played: true });
        self.write(&g);
    }

    /// Where to start: the saved spot, or 0 for new or finished items. Skips the first
    /// few seconds so an accidental start does not count as progress.
    pub fn resume_at(&self, key: &str) -> u32 {
        match self.entries.lock().unwrap().get(key) {
            Some(e) if !e.played && e.secs > 5 => e.secs,
            _ => 0,
        }
    }

    pub fn get(&self, key: &str) -> Option<Entry> { self.entries.lock().unwrap().get(key).cloned() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::Kind;

    fn src(id: &str, path: &str) -> Source { Source { id: id.into(), path: path.into(), kinds: vec![Kind::Audiobook], builtin: false } }
    fn temp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("cindrix-hist-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn key_survives_a_drive_letter_change() {
        let a = key_for("E:\\Audiobooks\\Guide\\Chapter_02.mp3", &[src("s1", "E:\\Audiobooks")]);
        let b = key_for("F:\\Audiobooks\\Guide\\Chapter_02.mp3", &[src("s1", "F:\\Audiobooks")]);
        assert_eq!(a, b);
        assert_eq!(a, "s1:Guide/Chapter_02.mp3");
    }

    #[test]
    fn files_outside_sources_use_the_full_path() {
        assert_eq!(key_for("C:\\Users\\me\\Downloads\\Cindrix\\Podcasts\\Show\\e1.mp3", &[]), "C:/Users/me/Downloads/Cindrix/Podcasts/Show/e1.mp3");
    }

    #[test]
    fn records_and_resumes() {
        let dir = temp("a");
        let h = HistoryStore::load(dir.join("playback_history.json"));
        assert_eq!(h.resume_at("k"), 0);
        h.record("k", 5055);
        assert_eq!(h.resume_at("k"), 5055);
        // survives a restart
        let h2 = HistoryStore::load(dir.join("playback_history.json"));
        assert_eq!(h2.resume_at("k"), 5055);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn finished_items_restart_and_tiny_positions_are_ignored() {
        let dir = temp("b");
        let h = HistoryStore::load(dir.join("playback_history.json"));
        h.record("k", 3);
        assert_eq!(h.resume_at("k"), 0, "first few seconds are not progress");
        h.record("k", 900);
        h.mark_played("k");
        assert_eq!(h.resume_at("k"), 0);
        assert!(h.get("k").unwrap().played);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn periodic_saves_are_throttled() {
        let dir = temp("c");
        let h = HistoryStore::load(dir.join("playback_history.json"));
        h.record("k", 100);
        h.record_periodic("k", 200, 15);
        assert_eq!(h.resume_at("k"), 100, "too soon to save again");
        h.record_periodic("k", 200, 0);
        assert_eq!(h.resume_at("k"), 200);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn corrupt_file_gives_empty_history() {
        let dir = temp("d");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("playback_history.json"), "{ nope").unwrap();
        let h = HistoryStore::load(dir.join("playback_history.json"));
        assert_eq!(h.resume_at("x"), 0);
        let _ = std::fs::remove_dir_all(dir);
    }
}

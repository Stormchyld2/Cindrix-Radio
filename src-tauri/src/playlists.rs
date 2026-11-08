//! Playlists and favorites, kept as a plain JSON file in Downloads/Cindrix/Playlists so they
//! survive a reinstall and can be copied to another device. Favorite radio stations are the
//! saved stations in the Radio folder, so they are not duplicated here.
use crate::player::Track;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub items: Vec<Track>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Data {
    pub playlists: Vec<Playlist>,
    pub favorites: Vec<Track>,
}

pub struct PlaylistStore { data: Mutex<Data>, path: PathBuf }

fn clean(name: &str) -> String {
    let n: String = name.trim().chars().filter(|c| !c.is_control()).take(80).collect();
    if n.is_empty() { "Untitled playlist".into() } else { n }
}

impl PlaylistStore {
    pub fn load(path: PathBuf) -> PlaylistStore {
        let data = std::fs::read_to_string(&path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
        PlaylistStore { data: Mutex::new(data), path }
    }

    fn save(&self, d: &Data) {
        if let Some(p) = self.path.parent() { let _ = std::fs::create_dir_all(p); }
        let tmp = self.path.with_extension("json.tmp");
        if let Ok(body) = serde_json::to_string_pretty(d) {
            if std::fs::write(&tmp, body).is_ok() { let _ = std::fs::rename(tmp, &self.path); }
        }
    }

    fn edit(&self, f: impl FnOnce(&mut Data)) -> Data {
        let mut d = self.data.lock().unwrap();
        f(&mut d);
        self.save(&d);
        d.clone()
    }

    pub fn get(&self) -> Data { self.data.lock().unwrap().clone() }

    pub fn create(&self, name: &str) -> Data {
        let name = clean(name);
        self.edit(|d| {
            // ids never repeat, even after deletes
            let next = d.playlists.iter().filter_map(|p| p.id.strip_prefix('p').and_then(|n| n.parse::<u32>().ok())).max().unwrap_or(0) + 1;
            d.playlists.push(Playlist { id: format!("p{next}"), name, items: vec![] });
        })
    }

    pub fn rename(&self, id: &str, name: &str) -> Data {
        let name = clean(name);
        self.edit(|d| if let Some(p) = d.playlists.iter_mut().find(|p| p.id == id) { p.name = name; })
    }

    pub fn delete(&self, id: &str) -> Data { self.edit(|d| d.playlists.retain(|p| p.id != id)) }

    /// Adds a track unless the playlist already has it.
    pub fn add(&self, id: &str, track: Track) -> Data {
        self.edit(|d| if let Some(p) = d.playlists.iter_mut().find(|p| p.id == id) {
            if !p.items.iter().any(|t| t.id == track.id) { p.items.push(track); }
        })
    }

    pub fn remove(&self, id: &str, index: usize) -> Data {
        self.edit(|d| if let Some(p) = d.playlists.iter_mut().find(|p| p.id == id) { if index < p.items.len() { p.items.remove(index); } })
    }

    /// Move one entry up (delta -1) or down (+1).
    pub fn shift(&self, id: &str, index: usize, delta: i32) -> Data {
        self.edit(|d| if let Some(p) = d.playlists.iter_mut().find(|p| p.id == id) {
            let to = index as i64 + delta as i64;
            if index < p.items.len() && to >= 0 && (to as usize) < p.items.len() { p.items.swap(index, to as usize); }
        })
    }

    pub fn toggle_favorite(&self, track: Track) -> Data {
        self.edit(|d| match d.favorites.iter().position(|t| t.id == track.id) {
            Some(i) => { d.favorites.remove(i); }
            None => d.favorites.push(track),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::Kind;

    fn tr(id: &str) -> Track { Track { id: id.into(), title: id.into(), subtitle: String::new(), kind: Kind::Music, duration_secs: None, path: Some(format!("/m/{id}.mp3")), url: None } }
    fn temp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("cindrix-radio-pl-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d.join("playlists.json")
    }

    #[test]
    fn playlists_add_remove_reorder_and_survive_restart() {
        let path = temp("a");
        let s = PlaylistStore::load(path.clone());
        let d = s.create("  Road trip  ");
        assert_eq!(d.playlists[0].name, "Road trip");
        let id = d.playlists[0].id.clone();
        s.add(&id, tr("a")); s.add(&id, tr("b")); s.add(&id, tr("c"));
        let d = s.add(&id, tr("a"));
        assert_eq!(d.playlists[0].items.len(), 3, "no duplicates");
        let d = s.shift(&id, 2, -1);
        assert_eq!(d.playlists[0].items.iter().map(|t| t.id.as_str()).collect::<Vec<_>>(), ["a", "c", "b"]);
        let d = s.shift(&id, 0, -1);
        assert_eq!(d.playlists[0].items[0].id, "a", "moving past the top does nothing");
        let d = s.remove(&id, 1);
        assert_eq!(d.playlists[0].items.len(), 2);
        let again = PlaylistStore::load(path.clone()).get();
        assert_eq!(again.playlists[0].items.len(), 2, "saved to disk");
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn ids_are_not_reused_and_names_are_cleaned() {
        let s = PlaylistStore::load(temp("b"));
        s.create("One"); s.create("Two");
        s.delete("p1");
        let d = s.create("   ");
        assert_eq!(d.playlists.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(), ["p2", "p3"]);
        assert_eq!(d.playlists[1].name, "Untitled playlist");
        let d = s.rename("p2", "Mix");
        assert_eq!(d.playlists[0].name, "Mix");
    }

    #[test]
    fn favorites_toggle_on_and_off() {
        let s = PlaylistStore::load(temp("c"));
        assert_eq!(s.toggle_favorite(tr("x")).favorites.len(), 1);
        assert_eq!(s.toggle_favorite(tr("y")).favorites.len(), 2);
        let d = s.toggle_favorite(tr("x"));
        assert_eq!(d.favorites.len(), 1);
        assert_eq!(d.favorites[0].id, "y");
    }
}

//! Library: up to 5 user-chosen drives/folders, scanned in place (files are referenced,
//! never copied, moved or deleted). The index lives in the app data folder.
use crate::player::Kind;
use lofty::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub const MAX_SOURCES: usize = 5;
const AUDIO: [&str; 10] = ["mp3", "flac", "ogg", "oga", "wav", "m4a", "m4b", "aac", "opus", "wma"];
const VIDEO: [&str; 6] = ["mp4", "mkv", "webm", "avi", "mov", "m4v"];

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Source {
    pub id: String,
    pub path: String,
    /// Which kinds of media to pull from this source: Music, Podcast, Audiobook, Video.
    pub kinds: Vec<Kind>,
    /// The app's own Downloads/Cindrix folder: always present, not counted in the 5, not removable.
    #[serde(default)]
    pub builtin: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Item {
    pub id: String,
    pub source_id: String,
    pub path: String,
    pub kind: Kind,
    pub title: String,
    pub artist: String,
    pub album: String,
    /// Genre from the tag, if any.
    pub genre: Option<String>,
    /// Genre taken from the top-level folder under the source (Genre/Artist/Album/track).
    pub folder_genre: Option<String>,
    /// Parent folder name: the book for audiobooks, the show for podcasts.
    pub group: String,
    pub duration_secs: Option<u32>,
    /// From the folder's metadata.txt, when present.
    #[serde(default)]
    pub published: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub available: bool,
}

#[derive(Serialize, Deserialize, Default, Clone)]
struct Data { sources: Vec<Source>, items: Vec<Item> }

#[derive(Serialize, Clone, Debug)]
pub struct SourceStatus { pub source: Source, pub online: bool, pub items: usize }

#[derive(Serialize, Clone, Debug, Default)]
pub struct ScanSummary { pub scanned: usize, pub added: usize, pub skipped_offline: Vec<String> }

#[derive(Serialize, Clone, Debug)]
pub struct Page { pub items: Vec<Item>, pub total: usize }

#[derive(Serialize, Clone, Debug)]
pub struct AlbumRow { pub album: String, pub artist: String, pub count: usize }

#[derive(Serialize, Clone, Debug)]
pub struct Counts { pub music: usize, pub podcast: usize, pub audiobook: usize, pub video: usize }

fn fnv(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() { h ^= b as u64; h = h.wrapping_mul(0x100000001b3); }
    h
}

fn is_media_kind(k: &Kind) -> bool { matches!(k, Kind::Music | Kind::Podcast | Kind::Audiobook | Kind::Video) }

/// Decide what an audio file is, from the source's tags, the extension and folder names.
pub fn classify(path: &Path, root: &Path, kinds: &[Kind]) -> Option<Kind> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    if VIDEO.contains(&ext.as_str()) {
        return kinds.contains(&Kind::Video).then_some(Kind::Video);
    }
    if !AUDIO.contains(&ext.as_str()) { return None; }
    let rel = path.strip_prefix(root).unwrap_or(path).to_string_lossy().to_ascii_lowercase();
    if kinds.contains(&Kind::Audiobook) && (ext == "m4b" || rel.contains("audiobook")) { return Some(Kind::Audiobook); }
    if kinds.contains(&Kind::Podcast) && rel.contains("podcast") { return Some(Kind::Podcast); }
    for k in [Kind::Music, Kind::Podcast, Kind::Audiobook] {
        if kinds.contains(&k) { return Some(k); }
    }
    None
}

fn skip_dir(name: &str) -> bool {
    name.starts_with('.') || name == "$RECYCLE.BIN" || name == "System Volume Information"
}

fn walk(dir: &Path, depth: u32, out: &mut Vec<PathBuf>) {
    if depth > 12 { return; }
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let Ok(ft) = e.file_type() else { continue };
        let p = e.path();
        if ft.is_symlink() { continue; } // avoid loops
        if ft.is_dir() {
            if !skip_dir(&e.file_name().to_string_lossy()) { walk(&p, depth + 1, out); }
        } else if ft.is_file() {
            out.push(p);
        }
    }
}

#[derive(Default)]
struct Tags { title: Option<String>, artist: Option<String>, album: Option<String>, genre: Option<String>, secs: Option<u32> }

fn read_tags(path: &Path) -> Tags {
    let Ok(tagged) = lofty::read_from_path(path) else { return Tags::default() };
    let secs = Some(tagged.properties().duration().as_secs() as u32).filter(|s| *s > 0);
    let tag = tagged.primary_tag().or_else(|| tagged.first_tag());
    let clean = |s: Option<std::borrow::Cow<str>>| s.map(|c| c.trim().to_string()).filter(|c| !c.is_empty());
    match tag {
        Some(t) => Tags { title: clean(t.title()), artist: clean(t.artist()), album: clean(t.album()), genre: clean(t.genre()), secs },
        None => Tags { secs, ..Default::default() },
    }
}

fn scan_source(src: &Source) -> Vec<Item> {
    let root = Path::new(&src.path);
    let mut files = Vec::new();
    walk(root, 0, &mut files);
    let mut items = Vec::new();
    let mut meta_cache: std::collections::HashMap<PathBuf, std::collections::HashMap<String, crate::meta::Meta>> = std::collections::HashMap::new();
    for f in files {
        let Some(kind) = classify(&f, root, &src.kinds) else { continue };
        let tags = if kind == Kind::Video { Tags::default() } else { read_tags(&f) };
        let stem = f.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let rel = f.strip_prefix(root).unwrap_or(&f);
        let comps: Vec<String> = rel.components().map(|c| c.as_os_str().to_string_lossy().to_string()).collect();
        // Radio recordings live in Downloads/Cindrix/Radio and are not library music
        if src.builtin && comps.first().map_or(false, |c| c.eq_ignore_ascii_case("radio")) { continue; }
        // genre folder = first folder under the source root (under the Music/Videos folder for
        // the built-in downloads source), only when the file is nested at least 2 deep
        let g = if src.builtin && comps.len() > 1 { &comps[1..] } else { &comps[..] };
        let folder_genre = if g.len() >= 3 { Some(g[0].clone()) } else { None };
        let group = f.parent().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let path = f.to_string_lossy().to_string();
        let dir = f.parent().unwrap_or(root).to_path_buf();
        let meta = meta_cache.entry(dir.clone()).or_insert_with(|| crate::meta::load_dir(&dir));
        let fname = f.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
        let m = meta.get(&fname).cloned().unwrap_or_default();
        items.push(Item {
            id: format!("{:016x}", fnv(&path)),
            source_id: src.id.clone(),
            path,
            kind,
            // metadata.txt first, then embedded tags, then the file name
            title: m.title.or(tags.title).unwrap_or(stem),
            artist: tags.artist.unwrap_or_default(),
            album: tags.album.unwrap_or_default(),
            genre: tags.genre,
            folder_genre,
            group,
            duration_secs: tags.secs,
            published: m.date,
            description: m.description,
            available: true,
        });
    }
    items
}

#[derive(Clone)]
pub struct LibraryStore { data: Arc<Mutex<Data>>, path: PathBuf }

impl LibraryStore {
    pub fn load(path: PathBuf) -> LibraryStore {
        let data = std::fs::read_to_string(&path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
        LibraryStore { data: Arc::new(Mutex::new(data)), path }
    }

    fn save(&self, d: &Data) {
        if let Some(p) = self.path.parent() { let _ = std::fs::create_dir_all(p); }
        let tmp = self.path.with_extension("json.tmp");
        if std::fs::write(&tmp, serde_json::to_string(d).unwrap()).is_ok() { let _ = std::fs::rename(tmp, &self.path); }
    }

    /// Make sure the built-in Downloads/Cindrix source exists and points at the current folder.
    pub fn ensure_builtin(&self, root: &str) {
        let mut d = self.data.lock().unwrap();
        let all = vec![Kind::Music, Kind::Podcast, Kind::Audiobook, Kind::Video];
        match d.sources.iter_mut().find(|s| s.id == "downloads") {
            Some(s) => { s.path = root.to_string(); s.builtin = true; s.kinds = all; }
            None => d.sources.insert(0, Source { id: "downloads".into(), path: root.to_string(), kinds: all, builtin: true }),
        }
        self.save(&d);
    }

    pub fn source_list(&self) -> Vec<Source> { self.data.lock().unwrap().sources.clone() }

    pub fn statuses(&self) -> Vec<SourceStatus> {
        let d = self.data.lock().unwrap();
        d.sources.iter().map(|s| SourceStatus {
            online: Path::new(&s.path).is_dir(),
            items: d.items.iter().filter(|i| i.source_id == s.id).count(),
            source: s.clone(),
        }).collect()
    }

    pub fn add_source(&self, path: &str, kinds: Vec<Kind>) -> Result<(), String> {
        let kinds: Vec<Kind> = kinds.into_iter().filter(is_media_kind).collect();
        if kinds.is_empty() { return Err("Pick at least one kind of media for this source".into()); }
        if !Path::new(path).is_dir() { return Err("That folder was not found".into()); }
        let mut d = self.data.lock().unwrap();
        if d.sources.iter().filter(|s| !s.builtin).count() >= MAX_SOURCES { return Err(format!("You can use up to {MAX_SOURCES} sources. Remove one first.")); }
        if d.sources.iter().any(|s| s.path == path) { return Err("That folder is already a source".into()); }
        d.sources.push(Source { id: format!("{:08x}", fnv(path) as u32), path: path.to_string(), kinds, builtin: false });
        self.save(&d);
        Ok(())
    }

    /// Removes the source from the app only. Files on the drive are never touched.
    pub fn remove_source(&self, id: &str) {
        let mut d = self.data.lock().unwrap();
        if d.sources.iter().any(|s| s.id == id && s.builtin) { return; }
        d.sources.retain(|s| s.id != id);
        d.items.retain(|i| i.source_id != id);
        self.save(&d);
    }

    pub fn set_kinds(&self, id: &str, kinds: Vec<Kind>) -> Result<(), String> {
        let kinds: Vec<Kind> = kinds.into_iter().filter(is_media_kind).collect();
        if kinds.is_empty() { return Err("Pick at least one kind of media".into()); }
        let mut d = self.data.lock().unwrap();
        if let Some(s) = d.sources.iter_mut().find(|s| s.id == id && !s.builtin) { s.kinds = kinds; }
        self.save(&d);
        Ok(())
    }

    /// Rescan one source (or all when id is None). Offline sources keep their old index
    /// and are reported, never wiped.
    pub fn scan(&self, id: Option<&str>) -> ScanSummary {
        let sources: Vec<Source> = self.data.lock().unwrap().sources.iter()
            .filter(|s| id.map_or(true, |i| i == s.id)).cloned().collect();
        let mut sum = ScanSummary::default();
        for s in sources {
            if !Path::new(&s.path).is_dir() { sum.skipped_offline.push(s.path.clone()); continue; }
            let items = scan_source(&s); // slow part runs without holding the lock
            let mut d = self.data.lock().unwrap();
            let before = d.items.iter().filter(|i| i.source_id == s.id).count();
            d.items.retain(|i| i.source_id != s.id);
            sum.scanned += items.len();
            sum.added += items.len().saturating_sub(before);
            d.items.extend(items);
            self.save(&d);
        }
        sum
    }

    fn online_ids(d: &Data) -> Vec<String> {
        d.sources.iter().filter(|s| Path::new(&s.path).is_dir()).map(|s| s.id.clone()).collect()
    }

    pub fn search(&self, q: &str, kind: Option<Kind>, genre: Option<&str>, artist: Option<&str>, album: Option<&str>, limit: usize, offset: usize) -> Page {
        let d = self.data.lock().unwrap();
        let online = Self::online_ids(&d);
        let q = q.trim().to_lowercase();
        let hits: Vec<&Item> = d.items.iter().filter(|i| {
            kind.as_ref().map_or(true, |k| &i.kind == k)
                && genre.map_or(true, |g| i.folder_genre.as_deref().or(i.genre.as_deref()) == Some(g))
                && artist.map_or(true, |a| i.artist == a)
                && album.map_or(true, |a| i.album == a)
                && (q.is_empty()
                    || [&i.title, &i.artist, &i.album, &i.group, &i.path].iter().any(|f| f.to_lowercase().contains(&q))
                    || i.genre.iter().chain(i.folder_genre.iter()).any(|g| g.to_lowercase().contains(&q)))
        }).collect();
        let total = hits.len();
        let mut items: Vec<Item> = hits.into_iter().skip(offset).take(limit).cloned().collect();
        for i in items.iter_mut() { i.available = online.contains(&i.source_id); }
        Page { items, total }
    }

    pub fn artists(&self, kind: Option<Kind>, q: &str) -> Vec<(String, usize)> {
        let d = self.data.lock().unwrap();
        let q = q.trim().to_lowercase();
        let mut m: BTreeMap<String, usize> = BTreeMap::new();
        for i in d.items.iter().filter(|i| kind.as_ref().map_or(true, |k| &i.kind == k) && !i.artist.is_empty()) {
            if q.is_empty() || i.artist.to_lowercase().contains(&q) { *m.entry(i.artist.clone()).or_default() += 1; }
        }
        m.into_iter().collect()
    }

    pub fn albums(&self, kind: Option<Kind>, q: &str, artist: Option<&str>) -> Vec<AlbumRow> {
        let d = self.data.lock().unwrap();
        let q = q.trim().to_lowercase();
        let mut m: BTreeMap<(String, String), usize> = BTreeMap::new();
        for i in d.items.iter().filter(|i| kind.as_ref().map_or(true, |k| &i.kind == k) && !i.album.is_empty()) {
            if artist.map_or(true, |a| i.artist == a) && (q.is_empty() || i.album.to_lowercase().contains(&q)) {
                *m.entry((i.album.clone(), i.artist.clone())).or_default() += 1;
            }
        }
        m.into_iter().map(|((album, artist), count)| AlbumRow { album, artist, count }).collect()
    }

    pub fn genres(&self, kind: Option<Kind>) -> Vec<(String, usize)> {
        let d = self.data.lock().unwrap();
        let mut m: BTreeMap<String, usize> = BTreeMap::new();
        for i in d.items.iter().filter(|i| kind.as_ref().map_or(true, |k| &i.kind == k)) {
            if let Some(g) = i.folder_genre.as_deref().or(i.genre.as_deref()) { *m.entry(g.to_string()).or_default() += 1; }
        }
        let mut v: Vec<_> = m.into_iter().collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        v
    }

    pub fn counts(&self) -> Counts {
        let d = self.data.lock().unwrap();
        let n = |k: Kind| d.items.iter().filter(|i| i.kind == k).count();
        Counts { music: n(Kind::Music), podcast: n(Kind::Podcast), audiobook: n(Kind::Audiobook), video: n(Kind::Video) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("cindrix-lib-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }
    fn touch(p: &Path) { fs::create_dir_all(p.parent().unwrap()).unwrap(); fs::write(p, b"x").unwrap(); }

    #[test]
    fn classifies_by_extension_and_source_tags() {
        let root = Path::new("/r");
        let music = [Kind::Music];
        assert_eq!(classify(Path::new("/r/a.mp3"), root, &music), Some(Kind::Music));
        assert_eq!(classify(Path::new("/r/a.mp4"), root, &music), None, "video not enabled");
        assert_eq!(classify(Path::new("/r/a.mp4"), root, &[Kind::Video]), Some(Kind::Video));
        assert_eq!(classify(Path::new("/r/a.txt"), root, &music), None);
        let all = [Kind::Music, Kind::Podcast, Kind::Audiobook, Kind::Video];
        assert_eq!(classify(Path::new("/r/Audiobooks/Book/01.mp3"), root, &all), Some(Kind::Audiobook));
        assert_eq!(classify(Path::new("/r/Podcasts/Show/e1.mp3"), root, &all), Some(Kind::Podcast));
        assert_eq!(classify(Path::new("/r/book.m4b"), root, &all), Some(Kind::Audiobook));
    }

    #[test]
    fn scan_finds_media_uses_folder_genre_and_search_works() {
        let root = temp("scan");
        touch(&root.join("Country/Hank/Greatest/Lonesome Whistle.mp3"));
        touch(&root.join("Rock/Song One.flac"));
        touch(&root.join("notes.txt"));
        touch(&root.join("clips/Camp Setup.mp4"));
        let store = LibraryStore::load(root.join("idx.json"));
        store.add_source(root.to_str().unwrap(), vec![Kind::Music, Kind::Video]).unwrap();
        let sum = store.scan(None);
        assert_eq!(sum.scanned, 3);
        let all = store.search("", None, None, None, None, 100, 0);
        assert_eq!(all.total, 3);
        let whistle = all.items.iter().find(|i| i.title == "Lonesome Whistle").unwrap();
        assert_eq!(whistle.folder_genre.as_deref(), Some("Country"));
        let rock = all.items.iter().find(|i| i.title == "Song One").unwrap();
        assert_eq!(rock.folder_genre, None, "file directly in a top folder is not genre-nested");
        assert_eq!(store.search("whistle", None, None, None, None, 10, 0).total, 1);
        assert_eq!(store.search("", Some(Kind::Video), None, None, None, 10, 0).total, 1);
        assert_eq!(store.search("", None, Some("Country"), None, None, 10, 0).total, 1);
        let c = store.counts();
        assert_eq!((c.music, c.video), (2, 1));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn metadata_txt_overrides_titles() {
        let root = temp("meta");
        touch(&root.join("Podcasts/Show/ep1.mp3"));
        touch(&root.join("Podcasts/Show/ep2.mp3"));
        fs::write(root.join("Podcasts/Show/metadata.txt"), "EP1.mp3|Clean Title|2026-11-08|About it").unwrap();
        let store = LibraryStore::load(root.join("idx.json"));
        store.add_source(root.to_str().unwrap(), vec![Kind::Podcast]).unwrap();
        store.scan(None);
        let all = store.search("", None, None, None, None, 10, 0);
        let one = all.items.iter().find(|i| i.path.ends_with("ep1.mp3")).unwrap();
        assert_eq!(one.title, "Clean Title");
        assert_eq!(one.published.as_deref(), Some("2026-11-08"));
        assert_eq!(one.description.as_deref(), Some("About it"));
        let two = all.items.iter().find(|i| i.path.ends_with("ep2.mp3")).unwrap();
        assert_eq!(two.title, "ep2", "no entry: falls back to the file name");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn offline_source_keeps_its_index() {
        let root = temp("offline");
        let drive = root.join("USB");
        touch(&drive.join("A/B/song.mp3"));
        let store = LibraryStore::load(root.join("idx.json"));
        store.add_source(drive.to_str().unwrap(), vec![Kind::Music]).unwrap();
        store.scan(None);
        fs::remove_dir_all(&drive).unwrap(); // drive unplugged
        let sum = store.scan(None);
        assert_eq!(sum.skipped_offline.len(), 1);
        let page = store.search("", None, None, None, None, 10, 0);
        assert_eq!(page.total, 1, "index kept");
        assert!(!page.items[0].available, "marked unavailable, not deleted");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn at_most_five_sources_and_no_duplicates() {
        let root = temp("max");
        let store = LibraryStore::load(root.join("idx.json"));
        for i in 0..5 {
            let d = root.join(format!("d{i}"));
            fs::create_dir_all(&d).unwrap();
            store.add_source(d.to_str().unwrap(), vec![Kind::Music]).unwrap();
        }
        let extra = root.join("d5");
        fs::create_dir_all(&extra).unwrap();
        assert!(store.add_source(extra.to_str().unwrap(), vec![Kind::Music]).is_err());
        let again = root.join("d0");
        store.remove_source(&store.statuses()[0].source.id.clone());
        assert!(store.add_source(again.to_str().unwrap(), vec![Kind::Music]).is_ok());
        assert!(store.add_source(again.to_str().unwrap(), vec![Kind::Music]).is_err(), "duplicate");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn music_search_covers_genre_artist_and_album() {
        let root = temp("music");
        touch(&root.join("Country/Hank/Greatest/a.mp3"));
        touch(&root.join("Country/Hank/Greatest/b.mp3"));
        touch(&root.join("Rock/Band/Debut/c.mp3"));
        let store = LibraryStore::load(root.join("idx.json"));
        store.add_source(root.to_str().unwrap(), vec![Kind::Music]).unwrap();
        store.scan(None);
        assert_eq!(store.search("country", None, None, None, None, 10, 0).total, 2, "genre is searchable");
        assert_eq!(store.search("", None, None, None, Some("Greatest"), 10, 0).total, 0, "no tags in test files, so no album names");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn builtin_downloads_source_is_always_there_and_protected() {
        let root = temp("builtin");
        let dl = root.join("Cindrix");
        touch(&dl.join("Music/Country/Hank/Greatest/song.mp3"));
        touch(&dl.join("Radio/Station 2026.mp3"));
        touch(&dl.join("Podcasts/Show/ep.mp3"));
        let store = LibraryStore::load(root.join("idx.json"));
        store.ensure_builtin(dl.to_str().unwrap());
        store.scan(None);
        let all = store.search("", None, None, None, None, 10, 0);
        assert_eq!(all.total, 2, "radio recordings are not library music");
        let song = all.items.iter().find(|i| i.title == "song").unwrap();
        assert_eq!(song.kind, Kind::Music);
        assert_eq!(song.folder_genre.as_deref(), Some("Country"), "the Music folder is not a genre");
        assert_eq!(all.items.iter().find(|i| i.title == "ep").unwrap().kind, Kind::Podcast);
        store.remove_source("downloads");
        assert_eq!(store.statuses().len(), 1, "built-in source cannot be removed");
        for i in 0..5 {
            let d = root.join(format!("d{i}"));
            fs::create_dir_all(&d).unwrap();
            store.add_source(d.to_str().unwrap(), vec![Kind::Music]).unwrap();
        }
        assert_eq!(store.statuses().len(), 6, "built-in does not use up one of the 5 slots");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn source_needs_a_media_kind() {
        let root = temp("kinds");
        let store = LibraryStore::load(root.join("idx.json"));
        assert!(store.add_source(root.to_str().unwrap(), vec![Kind::Radio]).is_err());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn removing_a_source_never_touches_files() {
        let root = temp("remove");
        let f = root.join("A/B/keep.mp3");
        touch(&f);
        let store = LibraryStore::load(root.join("idx.json"));
        store.add_source(root.to_str().unwrap(), vec![Kind::Music]).unwrap();
        store.scan(None);
        store.remove_source(&store.statuses()[0].source.id.clone());
        assert!(f.exists());
        assert_eq!(store.search("", None, None, None, None, 10, 0).total, 0);
        let _ = fs::remove_dir_all(root);
    }
}

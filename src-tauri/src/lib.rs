#[cfg(target_os = "android")]
mod android;
mod admin;
mod dsp;
mod engine;
mod history;
mod hls;
mod library;
mod lyrics;
mod meta;
mod paths;
mod podcasts;
mod player;
mod playlists;
mod radio;
mod settings;
mod stream;
mod videos;

use player::{Player, PlayerState, Track};
use std::sync::Mutex;
use engine::Engine;
use history::HistoryStore;
use library::{Counts, LibraryStore, Page, ScanSummary, SourceStatus};
use player::Kind;
use radio::{Station, StationStore};
use podcasts::{Feed, PodStore};
use serde::Serialize;
use tauri::State;
use videos::{VideoHit, VideoSource};
use lyrics::{LyricsStore, Summary as LyricsSummary};
use playlists::{Data as PlaylistData, PlaylistStore};

pub struct AppSettings {
    pub inner: Mutex<settings::Settings>,
    pub path: std::path::PathBuf,
}

impl AppSettings {
    pub fn update_pub(&self, f: impl FnOnce(&mut settings::Settings)) -> settings::Settings { self.update(f) }

    fn update(&self, f: impl FnOnce(&mut settings::Settings)) -> settings::Settings {
        let mut g = self.inner.lock().unwrap();
        f(&mut g);
        let _ = g.save(&self.path); // best effort; UI keeps working if disk is read-only
        g.clone()
    }
}

#[tauri::command]
fn get_folders() -> Result<paths::Folders, String> {
    let f = paths::default_folders();
    paths::ensure(&f).map_err(|e| e.to_string())?;
    Ok(f)
}

fn resumable(t: &Track) -> bool { matches!(t.kind, Kind::Podcast | Kind::Audiobook) && t.path.is_some() }

fn key_of(lib: &LibraryStore, t: &Track) -> Option<String> {
    t.path.as_deref().map(|p| history::key_for(p, &lib.source_list()))
}

/// Remember where the outgoing podcast/audiobook was.
fn save_outgoing(h: &HistoryStore, lib: &LibraryStore, s: &PlayerState) {
    if let Some(t) = s.current.as_ref().filter(|t| resumable(t)) {
        if s.position_secs > 0 { if let Some(k) = key_of(lib, t) { h.record(&k, s.position_secs); } }
    }
}

/// Start whatever the player state says is current, resuming a saved position if there is one.
/// Streams arrive with Radio.
fn begin(engine: &Engine, h: &HistoryStore, lib: &LibraryStore, s: &mut PlayerState) {
    let (path, url) = match s.current.as_ref() { Some(t) => (t.path.clone(), t.url.clone()), None => (None, None) };
    match (path, url) {
        (Some(path), _) => engine.play_file(&path),
        (None, Some(url)) => { engine.play_stream(&url); return; }
        _ => { engine.stop(); return; }
    }
    if let Some(t) = s.current.as_ref().filter(|t| resumable(t)) {
        if let Some(k) = key_of(lib, t) {
            let at = h.resume_at(&k);
            if at > 0 { engine.seek(at); s.position_secs = at; }
        }
    }
}

fn online_or_err(s: &AppSettings) -> Result<(), String> {
    if s.inner.lock().unwrap().grid_down {
        Err("Grid-Down mode is on, so the app is not using the network. Switch to Online mode to do this.".into())
    } else { Ok(()) }
}

#[tauri::command]
fn player_state(p: State<Player>, e: State<Engine>, h: State<HistoryStore>, lib: State<LibraryStore>) -> PlayerState {
    let mut s = p.0.lock().unwrap();
    let st = e.status();
    if s.playing && s.current.is_some() { s.position_secs = st.position_secs; }
    s.error = st.error;
    s.now_playing = st.now_playing;
    s.recording = st.recording;
    s.connecting = st.connecting;
    if s.playing {
        if let Some(t) = s.current.as_ref().filter(|t| resumable(t)) {
            if let Some(k) = key_of(&lib, t) { h.record_periodic(&k, s.position_secs, 15); }
        }
    }
    // song finished on its own: mark it played and move on to the next queued track
    if e.take_ended() && s.playing {
        if let Some(t) = s.current.as_ref().filter(|t| resumable(t)) {
            if let Some(k) = key_of(&lib, t) { h.mark_played(&k); }
        }
        s.next();
        begin(&e, &h, &lib, &mut s);
    }
    #[cfg(target_os = "android")]
    android::media_sync(&s);
    s.clone()
}

/// A command from the lock screen, a Bluetooth/headset button, the notification, or audio focus.
#[cfg(target_os = "android")]
pub(crate) fn media_command(app: &tauri::AppHandle, cmd: &str) {
    use tauri::Manager;
    let playing = app.state::<Player>().0.lock().unwrap().playing;
    match cmd {
        "play" if !playing => { player_toggle(app.state(), app.state(), app.state(), app.state()); }
        "pause" if playing => { player_toggle(app.state(), app.state(), app.state(), app.state()); }
        "toggle" => { player_toggle(app.state(), app.state(), app.state(), app.state()); }
        "next" => { player_next(app.state(), app.state(), app.state(), app.state()); }
        "prev" => { player_seek(app.state(), app.state(), 0); }
        "noisy" if playing && app.state::<AppSettings>().inner.lock().unwrap().pause_on_disconnect => {
            player_toggle(app.state(), app.state(), app.state(), app.state());
        }
        _ => {}
    }
}

/// Open the phone's Bluetooth settings (Android only).
#[cfg(target_os = "android")]
#[tauri::command]
fn open_bluetooth_settings() -> Result<(), String> { android::open_bluetooth_settings() }

#[cfg(not(target_os = "android"))]
#[tauri::command]
fn open_bluetooth_settings() -> Result<(), String> {
    Err("Bluetooth devices are managed in your system settings on this platform.".into())
}


#[tauri::command]
fn player_play(settings: State<AppSettings>, p: State<Player>, e: State<Engine>, h: State<HistoryStore>, lib: State<LibraryStore>, track: Track) -> PlayerState {
    let mut s = p.0.lock().unwrap();
    if track.url.is_some() && online_or_err(&settings).is_err() {
        e.set_error("Grid-Down mode is on, so live streams are off. Switch to Online mode to listen.");
        return s.clone();
    }
    save_outgoing(&h, &lib, &s);
    s.play(track);
    begin(&e, &h, &lib, &mut s);
    s.clone()
}

#[tauri::command]
fn player_toggle(p: State<Player>, e: State<Engine>, h: State<HistoryStore>, lib: State<LibraryStore>) -> PlayerState {
    let mut s = p.0.lock().unwrap();
    s.toggle();
    if s.current.is_some() {
        if s.playing { e.resume() } else { e.pause(); save_outgoing(&h, &lib, &s); }
    }
    s.clone()
}

#[tauri::command]
fn player_next(p: State<Player>, e: State<Engine>, h: State<HistoryStore>, lib: State<LibraryStore>) -> PlayerState {
    let mut s = p.0.lock().unwrap();
    save_outgoing(&h, &lib, &s);
    s.next();
    begin(&e, &h, &lib, &mut s);
    s.clone()
}

#[tauri::command]
fn player_enqueue(p: State<Player>, track: Track) -> PlayerState {
    let mut s = p.0.lock().unwrap(); s.enqueue(track); s.clone()
}

/// Jump to a position in the current file. Live streams cannot be seeked.
#[tauri::command]
fn player_seek(p: State<Player>, e: State<Engine>, secs: u32) -> PlayerState {
    let mut s = p.0.lock().unwrap();
    if s.current.as_ref().map_or(false, |t| t.path.is_some()) {
        let to = s.current.as_ref().and_then(|t| t.duration_secs).map_or(secs, |d| secs.min(d.saturating_sub(1)));
        e.seek(to);
        s.position_secs = to;
    }
    s.clone()
}

#[tauri::command]
fn player_volume(p: State<Player>, e: State<Engine>, volume: f32) -> PlayerState {
    let mut s = p.0.lock().unwrap();
    s.set_volume(volume);
    e.set_volume(s.volume);
    s.clone()
}

#[tauri::command]
fn get_settings(s: State<AppSettings>) -> settings::Settings { s.inner.lock().unwrap().clone() }

#[tauri::command]
fn set_lyrics_viewer(s: State<AppSettings>, on: bool) -> settings::Settings {
    s.update(|g| g.lyrics_viewer = on)
}

#[tauri::command]
fn set_grid_down(s: State<AppSettings>, on: bool) -> settings::Settings {
    s.update(|g| g.grid_down = on)
}

#[tauri::command]
fn accept_terms(s: State<AppSettings>, version: String) -> settings::Settings {
    s.update(|g| g.terms_version = Some(version))
}

#[tauri::command]
fn set_eq(s: State<AppSettings>, e: State<Engine>, eq: dsp::EqSettings) -> settings::Settings {
    e.set_eq(eq.clone());
    s.update(|g| g.eq = eq.clamped())
}

#[tauri::command]
fn set_vocal_reduction(s: State<AppSettings>, e: State<Engine>, amount: f32) -> settings::Settings {
    let out = s.update(|g| { g.eq.vocal_reduction = amount; g.eq = g.eq.clone().clamped(); });
    e.set_eq(out.eq.clone());
    out
}

fn lyrics_on(s: &AppSettings) -> Result<(), String> {
    if s.inner.lock().unwrap().lyrics_viewer { Ok(()) } else { Err("Turn on the lyrics viewer in Settings first.".into()) }
}

/// Lyrics are only handed out while the lyrics viewer is on.
#[tauri::command]
fn lyrics_get(s: State<AppSettings>, l: State<LyricsStore>, path: String) -> Option<String> {
    if lyrics_on(&s).is_err() { return None; }
    l.get(&path)
}

#[tauri::command]
fn lyrics_set(s: State<AppSettings>, l: State<LyricsStore>, path: String, title: String, text: String) -> Result<Vec<LyricsSummary>, String> {
    lyrics_on(&s)?;
    l.set(&path, &title, &text)?;
    Ok(l.list())
}

#[tauri::command]
fn lyrics_list(s: State<AppSettings>, l: State<LyricsStore>) -> Vec<LyricsSummary> {
    if lyrics_on(&s).is_err() { return vec![]; }
    l.list()
}

#[derive(Serialize)]
struct ImportResult { feeds: Vec<Feed>, failed: Vec<String> }

type Blocking<T> = Result<T, String>;

pub(crate) async fn blocking_pub<T: Send + 'static>(f: impl FnOnce() -> Blocking<T> + Send + 'static) -> Blocking<T> { blocking(f).await }

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> Blocking<T> + Send + 'static) -> Blocking<T> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(|e| e.to_string())?
}

#[tauri::command]
fn podcast_list(store: State<PodStore>) -> Vec<Feed> { store.feeds.lock().unwrap().clone() }

#[tauri::command]
async fn podcast_add(settings: State<'_, AppSettings>, store: State<'_, PodStore>, url: String) -> Result<Vec<Feed>, String> {
    online_or_err(&settings)?;
    let store = store.inner().clone();
    blocking(move || Ok(store.upsert(podcasts::fetch_feed(&url)?))).await
}

#[tauri::command]
async fn podcast_import_opml(settings: State<'_, AppSettings>, store: State<'_, PodStore>, xml: String) -> Result<ImportResult, String> {
    online_or_err(&settings)?;
    let store = store.inner().clone();
    blocking(move || {
        let urls = podcasts::parse_opml(&xml);
        if urls.is_empty() { return Err("No podcast feeds found in that file".to_string()); }
        let mut failed = Vec::new();
        for u in urls {
            match podcasts::fetch_feed(&u) { Ok(f) => { store.upsert(f); } Err(e) => failed.push(format!("{u}: {e}")) }
        }
        Ok(ImportResult { feeds: store.feeds.lock().unwrap().clone(), failed })
    }).await
}

#[tauri::command]
async fn podcast_refresh(settings: State<'_, AppSettings>, store: State<'_, PodStore>) -> Result<ImportResult, String> {
    online_or_err(&settings)?;
    let store = store.inner().clone();
    blocking(move || {
        let urls: Vec<String> = store.feeds.lock().unwrap().iter().map(|f| f.url.clone()).collect();
        let mut failed = Vec::new();
        for u in urls {
            match podcasts::fetch_feed(&u) { Ok(f) => { store.upsert(f); } Err(e) => failed.push(format!("{u}: {e}")) }
        }
        Ok(ImportResult { feeds: store.feeds.lock().unwrap().clone(), failed })
    }).await
}

#[tauri::command]
fn podcast_remove(store: State<PodStore>, id: String) -> Vec<Feed> {
    // removes the subscription only; downloaded files are never deleted
    let mut g = store.feeds.lock().unwrap();
    g.retain(|f| f.id != id);
    store.save(&g);
    g.clone()
}

#[tauri::command]
async fn podcast_download(settings: State<'_, AppSettings>, store: State<'_, PodStore>, feed_id: String, guid: String) -> Result<Vec<Feed>, String> {
    online_or_err(&settings)?;
    let store = store.inner().clone();
    blocking(move || {
        let (feed, ep) = {
            let g = store.feeds.lock().unwrap();
            let feed = g.iter().find(|f| f.id == feed_id).ok_or("Podcast not found")?.clone();
            let ep = feed.episodes.iter().find(|e| e.guid == guid).ok_or("Episode not found")?.clone();
            (feed, ep)
        };
        let root = std::path::PathBuf::from(paths::default_folders().podcasts);
        let saved = podcasts::download_episode(&root, &feed, &ep)?;
        let mut g = store.feeds.lock().unwrap();
        if let Some(e) = g.iter_mut().find(|f| f.id == feed_id).and_then(|f| f.episodes.iter_mut().find(|e| e.guid == guid)) {
            e.file = Some(saved.to_string_lossy().into_owned());
        }
        store.save(&g);
        Ok(g.clone())
    }).await
}

#[tauri::command]
fn podcast_mark_played(store: State<PodStore>, feed_id: String, guid: String, played: bool) -> Vec<Feed> {
    let mut g = store.feeds.lock().unwrap();
    if let Some(e) = g.iter_mut().find(|f| f.id == feed_id).and_then(|f| f.episodes.iter_mut().find(|e| e.guid == guid)) {
        e.played = played;
    }
    store.save(&g);
    g.clone()
}

#[tauri::command]
fn library_sources(lib: State<LibraryStore>) -> Vec<SourceStatus> { lib.statuses() }

#[tauri::command]
fn library_add_source(lib: State<LibraryStore>, path: String, kinds: Vec<Kind>) -> Result<Vec<SourceStatus>, String> {
    lib.add_source(&path, kinds)?;
    Ok(lib.statuses())
}

#[tauri::command]
fn library_remove_source(lib: State<LibraryStore>, id: String) -> Vec<SourceStatus> {
    lib.remove_source(&id); // files are never touched
    lib.statuses()
}

#[tauri::command]
fn library_set_kinds(lib: State<LibraryStore>, id: String, kinds: Vec<Kind>) -> Result<Vec<SourceStatus>, String> {
    lib.set_kinds(&id, kinds)?;
    Ok(lib.statuses())
}

#[tauri::command]
async fn library_scan(lib: State<'_, LibraryStore>, id: Option<String>) -> Result<ScanSummary, String> {
    let lib = lib.inner().clone();
    blocking(move || Ok(lib.scan(id.as_deref()))).await
}

/// Edit a file's title, date and description. Saved in the folder's metadata.txt, never in the
/// file itself, then that source is rescanned so the Library shows it.
#[tauri::command]
async fn library_set_details(lib: State<'_, LibraryStore>, path: String, title: String, date: Option<String>, description: Option<String>) -> Result<(), String> {
    let lib = lib.inner().clone();
    blocking(move || {
        let p = std::path::Path::new(&path);
        let (Some(dir), Some(name)) = (p.parent(), p.file_name().and_then(|n| n.to_str())) else { return Err("Bad file path".to_string()) };
        if !p.exists() { return Err("That file is not available right now. Is its drive connected?".into()); }
        let date = date.map(|d| d.trim().to_string()).filter(|d| !d.is_empty());
        if let Some(d) = &date { if meta::iso_date(d).is_none() || d.len() != 10 { return Err("Use the date format YYYY-MM-DD, like 2026-11-08.".into()); } }
        meta::set_entry(dir, name, &title, date.as_deref(), description.as_deref()).map_err(|e| format!("Could not save the details: {e}"))?;
        if let Some(id) = lib.source_of(&path) { lib.scan(Some(&id)); }
        Ok(())
    }).await
}

#[tauri::command]
fn library_search(lib: State<LibraryStore>, query: String, kind: Option<Kind>, genre: Option<String>, artist: Option<String>, album: Option<String>, limit: usize, offset: usize) -> Page {
    lib.search(&query, kind, genre.as_deref(), artist.as_deref(), album.as_deref(), limit.min(2000), offset)
}

#[tauri::command]
fn playlists_get(p: State<PlaylistStore>) -> PlaylistData { p.get() }
#[tauri::command]
fn playlist_create(p: State<PlaylistStore>, name: String) -> PlaylistData { p.create(&name) }
#[tauri::command]
fn playlist_rename(p: State<PlaylistStore>, id: String, name: String) -> PlaylistData { p.rename(&id, &name) }
#[tauri::command]
fn playlist_delete(p: State<PlaylistStore>, id: String) -> PlaylistData { p.delete(&id) }
#[tauri::command]
fn playlist_add(p: State<PlaylistStore>, id: String, track: Track) -> PlaylistData { p.add(&id, track) }
#[tauri::command]
fn playlist_remove(p: State<PlaylistStore>, id: String, index: usize) -> PlaylistData { p.remove(&id, index) }
#[tauri::command]
fn playlist_shift(p: State<PlaylistStore>, id: String, index: usize, delta: i32) -> PlaylistData { p.shift(&id, index, delta) }
#[tauri::command]
fn favorite_toggle(p: State<PlaylistStore>, track: Track) -> PlaylistData { p.toggle_favorite(track) }

#[tauri::command]
fn video_sources() -> Vec<VideoSource> { videos::sources() }

#[tauri::command]
async fn video_search(app: tauri::AppHandle, settings: State<'_, AppSettings>, source: String, query: String, limit: u32, offset: u32) -> Result<Vec<VideoHit>, String> {
    use tauri::Manager;
    online_or_err(&settings)?;
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    blocking(move || videos::search(&dir, &source, &query, limit.clamp(1, 40), offset)).await
}

/// Save a search result for offline viewing. Returns the saved path.
#[tauri::command]
async fn video_download(app: tauri::AppHandle, settings: State<'_, AppSettings>, hit: VideoHit) -> Result<String, String> {
    use tauri::Manager;
    online_or_err(&settings)?;
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let root = std::path::PathBuf::from(paths::default_folders().videos);
    blocking(move || videos::download(&dir, &hit, &root).map(|p| p.to_string_lossy().into_owned())).await
}

/// The installed YouTube helper version, or None when it is missing or not in this build.
#[tauri::command]
fn ytdl_status(app: tauri::AppHandle) -> Option<String> {
    use tauri::Manager;
    videos::ytdl_version(&app.path().app_data_dir().ok()?)
}

#[tauri::command]
async fn ytdl_install(app: tauri::AppHandle, settings: State<'_, AppSettings>) -> Result<String, String> {
    use tauri::Manager;
    online_or_err(&settings)?;
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    blocking(move || videos::ytdl_install(&dir)).await
}

#[tauri::command]
fn library_artists(lib: State<LibraryStore>, kind: Option<Kind>, query: String) -> Vec<(String, usize)> { lib.artists(kind, &query) }

#[tauri::command]
fn library_albums(lib: State<LibraryStore>, kind: Option<Kind>, query: String, artist: Option<String>) -> Vec<library::AlbumRow> {
    lib.albums(kind, &query, artist.as_deref())
}

#[tauri::command]
fn library_genres(lib: State<LibraryStore>, kind: Option<Kind>) -> Vec<(String, usize)> { lib.genres(kind) }

#[tauri::command]
fn library_counts(lib: State<LibraryStore>) -> Counts { lib.counts() }

#[derive(Serialize)]
struct ImportStations { stations: Vec<Station>, added: usize }

#[tauri::command]
fn radio_stations(st: State<StationStore>) -> Vec<Station> { st.list() }

#[tauri::command]
fn radio_add(st: State<StationStore>, station: Station) -> Result<Vec<Station>, String> { st.add(station) }

#[tauri::command]
fn radio_remove(st: State<StationStore>, id: String) -> Vec<Station> { st.remove(&id) }

#[tauri::command]
fn radio_import(st: State<StationStore>, text: String) -> Result<ImportStations, String> {
    let (stations, added) = st.import(&text);
    if added == 0 && stations.is_empty() { return Err("No stream addresses found in that file".into()); }
    Ok(ImportStations { stations, added })
}

#[tauri::command]
async fn radio_probe(settings: State<'_, AppSettings>, url: String) -> Result<stream::StreamInfo, String> {
    online_or_err(&settings)?;
    blocking(move || stream::probe(&url)).await
}

#[tauri::command]
async fn radio_search(settings: State<'_, AppSettings>, query: String, tags: Vec<String>, order: Option<String>, country: Option<String>, limit: u32, offset: u32) -> Result<Vec<Station>, String> {
    online_or_err(&settings)?;
    blocking(move || radio::search(&query, &tags, order.as_deref(), country.as_deref(), limit.min(100), offset)).await
}

#[tauri::command]
fn radio_play(settings: State<AppSettings>, p: State<Player>, e: State<Engine>, h: State<HistoryStore>, lib: State<LibraryStore>, station: Station) -> Result<PlayerState, String> {
    online_or_err(&settings)?;
    if let Some(u) = station.rb_uuid.as_deref() { radio::count_click(u); }
    let track = Track {
        id: station.id.clone(),
        title: station.name.clone(),
        subtitle: station.tags.clone().unwrap_or_else(|| "Live radio".into()),
        kind: Kind::Radio,
        duration_secs: None,
        path: None,
        url: Some(station.url.clone()),
    };
    let mut s = p.0.lock().unwrap();
    save_outgoing(&h, &lib, &s);
    s.play(track);
    begin(&e, &h, &lib, &mut s);
    Ok(s.clone())
}

/// Start or stop saving the live stream. Recordings go to the Radio folder by default.
#[tauri::command]
fn radio_record(e: State<Engine>, start: bool, name: String) -> Result<Option<String>, String> {
    if start {
        let dir = std::path::PathBuf::from(paths::default_folders().radio);
        e.start_recording(&dir, &name).map(|p| Some(p.to_string_lossy().into_owned()))
    } else {
        e.stop_recording();
        Ok(None)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // On the phone, a panic is written where the Kotlin side shows it on the next start.
    #[cfg(target_os = "android")]
    std::panic::set_hook(Box::new(|info| {
        let text = format!("Rust panic: {info}\n{}", std::backtrace::Backtrace::force_capture());
        let _ = std::fs::write("/data/data/app.cindrix.radio/files/rust-crash.txt", text);
    }));
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Player::new())
        .setup(|app| {
            use tauri::Manager;
            let dir = app.path().app_data_dir().expect("app data dir");
            let path = dir.join("settings.json");
            let loaded = settings::Settings::load(&path);
            paths::set_root_override(loaded.download_root.clone());
            app.manage(Engine::start(loaded.eq.clone(), 0.8));
            app.manage(StationStore::load(std::path::PathBuf::from(paths::default_folders().radio).join("radio_stations.json")));
            app.manage(HistoryStore::load(dir.join("playback_history.json")));
            let lib = LibraryStore::load(dir.join("library.json"));
            let folders = paths::default_folders();
            let _ = paths::ensure(&folders);
            lib.ensure_builtin(&folders.root);
            app.manage(lib);
            app.manage(LyricsStore::load(dir.join("lyrics.json")));
            app.manage(PlaylistStore::load(std::path::PathBuf::from(&folders.root).join("Playlists").join("playlists.json")));
            app.manage(PodStore::load(dir.join("podcasts.json")));
            app.manage(AppSettings { inner: Mutex::new(loaded), path });
            #[cfg(target_os = "android")]
            android::set_app(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_folders, player_state, player_play, player_toggle,
            player_next, player_enqueue, player_volume, player_seek,
            get_settings, set_vocal_reduction, lyrics_get, lyrics_set, lyrics_list, set_lyrics_viewer, set_grid_down, accept_terms, set_eq,
            podcast_list, podcast_add, podcast_import_opml, podcast_refresh, podcast_remove,
            podcast_download, podcast_mark_played,
            library_sources, library_add_source, library_remove_source, library_set_kinds,
            library_scan, library_set_details, library_search, library_genres, library_counts, library_artists, library_albums,
            radio_stations, radio_add, radio_remove, radio_import, radio_probe, radio_search, radio_play, radio_record,
            playlists_get, playlist_create, playlist_rename, playlist_delete, playlist_add, playlist_remove, playlist_shift, favorite_toggle,
            video_sources, video_search, video_download, ytdl_status, ytdl_install, open_bluetooth_settings,
            admin::storage_report, admin::set_prefs, admin::set_pause_on_disconnect, admin::set_download_root, admin::privacy_clear,
            admin::backup_make, admin::backup_list, admin::backup_restore, admin::backup_delete, admin::diagnostics
        ])
        .run(tauri::generate_context!())
        .expect("error while running Cindrix Radio");
}

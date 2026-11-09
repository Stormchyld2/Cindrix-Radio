//! Settings-page backends: storage report, backup/restore, privacy clears, diagnostics, preferences.
use crate::{blocking_pub, paths, settings, AppSettings};
use crate::history::HistoryStore;
use crate::library::LibraryStore;
use crate::lyrics::LyricsStore;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::{Manager, State};

#[allow(dead_code)]
fn dir_size(p: &Path) -> u64 { dir_stats(p).0 }

/// (bytes, files) under a folder.
fn dir_stats(p: &Path) -> (u64, u64) {
    let mut total = 0;
    let mut files = 0;
    let mut stack = vec![p.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            match e.metadata() {
                Ok(m) if m.is_dir() => stack.push(e.path()),
                Ok(m) => { total += m.len(); files += 1; }
                Err(_) => {}
            }
        }
    }
    (total, files)
}

#[derive(Serialize)]
pub struct StorageRow { pub label: String, pub path: String, pub bytes: u64, pub files: u64, pub exists: bool }

#[derive(Serialize)]
pub struct StorageReport { pub rows: Vec<StorageRow>, pub free: u64, pub total: u64, pub root_exists: bool }

#[tauri::command]
pub async fn storage_report() -> Result<StorageReport, String> {
    blocking_pub(|| {
        let f = paths::default_folders();
        let rows = [("Music", f.music), ("Podcasts", f.podcasts), ("Audiobooks", f.audiobooks), ("Videos", f.videos), ("Radio & recordings", f.radio)];
        let rows = rows.into_iter().map(|(l, p)| {
            let exists = Path::new(&p).is_dir();
            let (bytes, files) = dir_stats(Path::new(&p));
            StorageRow { label: l.into(), path: p, bytes, files, exists }
        }).collect();
        // free and total space on the drive that holds the save folder (or its nearest existing parent)
        let mut probe = std::path::PathBuf::from(&f.root);
        while !probe.exists() && probe.pop() {}
        let free = fs2::available_space(&probe).unwrap_or(0);
        let total = fs2::total_space(&probe).unwrap_or(0);
        Ok(StorageReport { rows, free, total, root_exists: Path::new(&f.root).is_dir() })
    }).await
}

#[tauri::command]
pub fn set_prefs(s: State<AppSettings>, prefs: settings::Prefs) -> settings::Settings {
    s.update_pub(|g| g.prefs = prefs.clamped())
}

#[tauri::command]
pub fn set_pause_on_disconnect(s: State<AppSettings>, on: bool) -> settings::Settings {
    s.update_pub(|g| g.pause_on_disconnect = on)
}

/// Choose where downloads are saved. Takes effect after the app restarts.
#[tauri::command]
pub fn set_download_root(s: State<AppSettings>, path: Option<String>) -> Result<settings::Settings, String> {
    if let Some(p) = path.as_deref().filter(|p| !p.trim().is_empty()) {
        std::fs::create_dir_all(p).map_err(|e| format!("That folder cannot be used: {e}"))?;
    }
    Ok(s.update_pub(|g| g.download_root = path.filter(|p| !p.trim().is_empty())))
}

#[tauri::command]
pub fn privacy_clear(what: String, h: State<HistoryStore>, l: State<LyricsStore>, lib: State<LibraryStore>) -> Result<(), String> {
    match what.as_str() {
        "history" => h.clear(),
        "lyrics" => l.clear(),
        "library" => lib.clear_index(),
        _ => return Err("Unknown item".into()),
    }
    Ok(())
}

// ---- backup / restore ----
// A backup is one JSON file holding the app's small data files (settings, playlists and
// favorites, stations, podcast subscriptions, lyrics, listening positions). Media is not included.

fn data_files(app: &tauri::AppHandle) -> Vec<(&'static str, PathBuf)> {
    let data = app.path().app_data_dir().unwrap_or_default();
    let root = PathBuf::from(paths::default_folders().root);
    vec![
        ("settings.json", data.join("settings.json")),
        ("lyrics.json", data.join("lyrics.json")),
        ("podcasts.json", data.join("podcasts.json")),
        ("playback_history.json", data.join("playback_history.json")),
        ("playlists.json", root.join("Playlists").join("playlists.json")),
        ("radio_stations.json", PathBuf::from(paths::default_folders().radio).join("radio_stations.json")),
    ]
}

fn backups_dir() -> PathBuf { PathBuf::from(paths::default_folders().root).join("Backups") }

#[derive(Serialize)]
pub struct BackupInfo { pub name: String, pub bytes: u64, pub made: u64 }

#[tauri::command]
pub async fn backup_make(app: tauri::AppHandle) -> Result<BackupInfo, String> {
    blocking_pub(move || {
        let mut files = serde_json::Map::new();
        for (name, path) in data_files(&app) {
            if let Ok(text) = std::fs::read_to_string(&path) { files.insert(name.to_string(), serde_json::Value::String(text)); }
        }
        let made = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let body = serde_json::json!({ "app": "Cindrix Radio", "format": 1, "made": made, "files": files });
        let dir = backups_dir();
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let name = format!("cindrix-radio-backup-{made}.json");
        let text = serde_json::to_string_pretty(&body).map_err(|e| e.to_string())?;
        std::fs::write(dir.join(&name), &text).map_err(|e| e.to_string())?;
        Ok(BackupInfo { name, bytes: text.len() as u64, made })
    }).await
}

#[tauri::command]
pub fn backup_list() -> Vec<BackupInfo> {
    let mut v: Vec<BackupInfo> = std::fs::read_dir(backups_dir()).map(|rd| rd.flatten().filter_map(|e| {
        let name = e.file_name().to_string_lossy().into_owned();
        if !(name.starts_with("cindrix-radio-backup-") && name.ends_with(".json")) { return None; }
        let m = e.metadata().ok()?;
        let made = name.trim_start_matches("cindrix-radio-backup-").trim_end_matches(".json").parse().unwrap_or(0);
        Some(BackupInfo { name, bytes: m.len(), made })
    }).collect()).unwrap_or_default();
    v.sort_by(|a, b| b.made.cmp(&a.made));
    v
}

/// Put a backup's files back. The app must be restarted to load them.
#[tauri::command]
pub async fn backup_restore(app: tauri::AppHandle, name: String) -> Result<usize, String> {
    if name.contains('/') || name.contains('\\') || name.contains("..") { return Err("Bad backup name".into()); }
    blocking_pub(move || {
        let text = std::fs::read_to_string(backups_dir().join(&name)).map_err(|e| format!("Could not read that backup: {e}"))?;
        let v: serde_json::Value = serde_json::from_str(&text).map_err(|_| "That file is not a Cindrix Radio backup".to_string())?;
        if v["app"] != "Cindrix Radio" { return Err("That file is not a Cindrix Radio backup".into()); }
        let files = v["files"].as_object().ok_or("That backup has no data in it")?;
        let mut n = 0;
        for (fname, path) in data_files(&app) {
            if let Some(body) = files.get(fname).and_then(|b| b.as_str()) {
                if serde_json::from_str::<serde_json::Value>(body).is_err() { continue; }
                if let Some(d) = path.parent() { let _ = std::fs::create_dir_all(d); }
                let tmp = path.with_extension("restore.tmp");
                if std::fs::write(&tmp, body).is_ok() && std::fs::rename(&tmp, &path).is_ok() { n += 1; }
            }
        }
        Ok(n)
    }).await
}

#[tauri::command]
pub fn backup_delete(name: String) -> Result<(), String> {
    if name.contains('/') || name.contains('\\') || name.contains("..") { return Err("Bad backup name".into()); }
    std::fs::remove_file(backups_dir().join(name)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn engine_trace() -> Vec<String> {
    #[allow(unused_mut)]
    let mut v = crate::engine::trace_dump();
    #[cfg(target_os = "android")]
    {
        v.push("---- Android audio route ----".into());
        v.extend(crate::android::media_log().lines().map(|l| l.to_string()));
    }
    v
}

// ---- diagnostics ----
#[derive(Serialize)]
pub struct Diagnostics { pub version: String, pub os: String, pub arch: String, pub data_dir: String, pub save_root: String, pub library_items: usize, pub private_build: bool }

#[tauri::command]
pub fn diagnostics(app: tauri::AppHandle, lib: State<LibraryStore>) -> Diagnostics {
    Diagnostics {
        version: env!("CARGO_PKG_VERSION").into(),
        os: std::env::consts::OS.into(),
        arch: std::env::consts::ARCH.into(),
        data_dir: app.path().app_data_dir().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default(),
        save_root: paths::default_folders().root,
        library_items: lib.item_count(),
        private_build: cfg!(feature = "ytdl"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dir_size_counts_nested_files() {
        let d = std::env::temp_dir().join(format!("cindrix-size-{}", std::process::id()));
        std::fs::create_dir_all(d.join("a/b")).unwrap();
        std::fs::write(d.join("x"), [0u8; 10]).unwrap();
        std::fs::write(d.join("a/b/y"), [0u8; 5]).unwrap();
        assert_eq!(dir_size(&d), 15);
        let _ = std::fs::remove_dir_all(&d);
    }
}

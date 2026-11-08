mod dsp;
mod paths;
mod player;
mod settings;

use player::{Player, PlayerState, Track};
use std::sync::Mutex;
use tauri::State;

pub struct AppSettings {
    pub inner: Mutex<settings::Settings>,
    pub path: std::path::PathBuf,
}

impl AppSettings {
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

#[tauri::command]
fn player_state(p: State<Player>) -> PlayerState { p.0.lock().unwrap().clone() }

#[tauri::command]
fn player_play(p: State<Player>, track: Track) -> PlayerState {
    let mut s = p.0.lock().unwrap(); s.play(track); s.clone()
}

#[tauri::command]
fn player_toggle(p: State<Player>) -> PlayerState {
    let mut s = p.0.lock().unwrap(); s.toggle(); s.clone()
}

#[tauri::command]
fn player_next(p: State<Player>) -> PlayerState {
    let mut s = p.0.lock().unwrap(); s.next(); s.clone()
}

#[tauri::command]
fn player_enqueue(p: State<Player>, track: Track) -> PlayerState {
    let mut s = p.0.lock().unwrap(); s.enqueue(track); s.clone()
}

#[tauri::command]
fn player_volume(p: State<Player>, volume: f32) -> PlayerState {
    let mut s = p.0.lock().unwrap(); s.set_volume(volume); s.clone()
}

#[tauri::command]
fn get_settings(s: State<AppSettings>) -> settings::Settings { s.inner.lock().unwrap().clone() }

#[tauri::command]
fn set_lyrics_viewer(s: State<AppSettings>, on: bool) -> settings::Settings {
    s.update(|g| g.lyrics_viewer = on)
}

#[tauri::command]
fn accept_terms(s: State<AppSettings>, version: String) -> settings::Settings {
    s.update(|g| g.terms_version = Some(version))
}

#[tauri::command]
fn set_eq(s: State<AppSettings>, eq: dsp::EqSettings) -> settings::Settings {
    s.update(|g| g.eq = eq.clamped())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Player::new())
        .setup(|app| {
            use tauri::Manager;
            let dir = app.path().app_data_dir().expect("app data dir");
            let path = dir.join("settings.json");
            let loaded = settings::Settings::load(&path);
            app.manage(AppSettings { inner: Mutex::new(loaded), path });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_folders, player_state, player_play, player_toggle,
            player_next, player_enqueue, player_volume,
            get_settings, set_lyrics_viewer, accept_terms, set_eq
        ])
        .run(tauri::generate_context!())
        .expect("error while running Cindrix Radio");
}

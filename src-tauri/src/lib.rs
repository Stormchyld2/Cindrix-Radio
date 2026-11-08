mod dsp;
mod engine;
mod paths;
mod player;
mod settings;

use player::{Player, PlayerState, Track};
use std::sync::Mutex;
use engine::Engine;
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

/// Start whatever the player state says is current. Streams arrive with Radio.
fn start_current(engine: &Engine, s: &PlayerState) {
    match s.current.as_ref().and_then(|t| t.path.as_deref()) {
        Some(path) => engine.play_file(path),
        None => engine.stop(),
    }
}

#[tauri::command]
fn player_state(p: State<Player>, e: State<Engine>) -> PlayerState {
    let mut s = p.0.lock().unwrap();
    let st = e.status();
    if s.playing && s.current.is_some() { s.position_secs = st.position_secs; }
    s.error = st.error;
    // song finished on its own: move on to the next queued track
    if e.take_ended() && s.playing {
        s.next();
        start_current(&e, &s);
    }
    s.clone()
}

#[tauri::command]
fn player_play(p: State<Player>, e: State<Engine>, track: Track) -> PlayerState {
    let mut s = p.0.lock().unwrap();
    s.play(track);
    start_current(&e, &s);
    s.clone()
}

#[tauri::command]
fn player_toggle(p: State<Player>, e: State<Engine>) -> PlayerState {
    let mut s = p.0.lock().unwrap();
    s.toggle();
    if s.current.is_some() { if s.playing { e.resume() } else { e.pause() } }
    s.clone()
}

#[tauri::command]
fn player_next(p: State<Player>, e: State<Engine>) -> PlayerState {
    let mut s = p.0.lock().unwrap();
    s.next();
    start_current(&e, &s);
    s.clone()
}

#[tauri::command]
fn player_enqueue(p: State<Player>, track: Track) -> PlayerState {
    let mut s = p.0.lock().unwrap(); s.enqueue(track); s.clone()
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
fn accept_terms(s: State<AppSettings>, version: String) -> settings::Settings {
    s.update(|g| g.terms_version = Some(version))
}

#[tauri::command]
fn set_eq(s: State<AppSettings>, e: State<Engine>, eq: dsp::EqSettings) -> settings::Settings {
    e.set_eq(eq.clone());
    s.update(|g| g.eq = eq.clamped())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Player::new())
        .setup(|app| {
            use tauri::Manager;
            let dir = app.path().app_data_dir().expect("app data dir");
            let path = dir.join("settings.json");
            let loaded = settings::Settings::load(&path);
            app.manage(Engine::start(loaded.eq.clone(), 0.8));
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

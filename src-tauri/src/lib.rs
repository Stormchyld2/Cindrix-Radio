mod paths;
mod player;

use player::{Player, PlayerState, Track};
use tauri::State;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Player::new())
        .invoke_handler(tauri::generate_handler![
            get_folders, player_state, player_play, player_toggle,
            player_next, player_enqueue, player_volume
        ])
        .run(tauri::generate_context!())
        .expect("error while running Cindrix Radio");
}

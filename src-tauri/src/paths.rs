//! Default save locations. Everything lives under Downloads/Cindrix/<Kind>,
//! and the user can override the root or any single kind in settings.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Folders {
    pub root: String,
    pub radio: String,
    pub music: String,
    pub podcasts: String,
    pub audiobooks: String,
    pub videos: String,
    pub recordings: String,
}

static ROOT_OVERRIDE: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

/// A user-chosen save location replacing Downloads/Cindrix (applied at startup).
pub fn set_root_override(p: Option<String>) {
    *ROOT_OVERRIDE.lock().unwrap() = p.filter(|s| !s.trim().is_empty());
}

fn downloads_root() -> PathBuf {
    if let Some(p) = ROOT_OVERRIDE.lock().unwrap().clone() { return PathBuf::from(p); }
    #[cfg(target_os = "android")]
    return PathBuf::from("/storage/emulated/0/Download/Cindrix");
    #[cfg(not(target_os = "android"))]
    dirs::download_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join("Downloads")))
        .unwrap_or_else(|| PathBuf::from("Downloads"))
        .join("Cindrix")
}

pub fn default_folders() -> Folders {
    let root = downloads_root();
    let p = |s: &str| root.join(s).to_string_lossy().into_owned();
    Folders {
        root: root.to_string_lossy().into_owned(),
        radio: p("Radio"),
        music: p("Music"),
        podcasts: p("Podcasts"),
        audiobooks: p("Audiobooks"),
        videos: p("Videos"),
        recordings: p("Radio"), // recordings default beside favorites, per user
    }
}

pub fn ensure(f: &Folders) -> std::io::Result<()> {
    for d in [&f.root, &f.radio, &f.music, &f.podcasts, &f.audiobooks, &f.videos, &f.recordings] {
        std::fs::create_dir_all(d)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_live_under_cindrix_root() {
        let f = default_folders();
        assert!(f.radio.ends_with("Radio"));
        assert!(f.radio.contains("Cindrix"));
        assert_eq!(f.recordings, f.radio);
        assert!(f.audiobooks.ends_with("Audiobooks"));
        assert!(f.podcasts.ends_with("Podcasts"));
    }
}

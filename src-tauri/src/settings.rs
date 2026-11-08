use crate::dsp::EqSettings;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Look-and-feel and small behaviour choices.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Prefs {
    /// 0.9 to 1.5; scales all text.
    pub text_scale: f32,
    /// blue, green, amber or violet.
    pub accent: String,
    pub high_contrast: bool,
    pub reduce_motion: bool,
    /// Jump back / forward buttons, in seconds.
    pub skip_back: u32,
    pub skip_fwd: u32,
    /// Refresh podcast feeds when the app opens (Online mode only).
    pub auto_refresh_podcasts: bool,
    /// Radio search starts with this country (ISO code), when set.
    pub radio_country: Option<String>,
    /// Radio search starts with this order, when set.
    pub radio_order: Option<String>,
    /// Keep the screen on while the player is open.
    pub keep_awake: bool,
}

impl Default for Prefs {
    fn default() -> Self {
        Prefs { text_scale: 1.0, accent: "blue".into(), high_contrast: false, reduce_motion: false, skip_back: 15, skip_fwd: 30,
                auto_refresh_podcasts: true, radio_country: None, radio_order: None, keep_awake: false }
    }
}

impl Prefs {
    pub fn clamped(mut self) -> Prefs {
        self.text_scale = self.text_scale.clamp(0.9, 1.5);
        if !["blue", "green", "amber", "violet"].contains(&self.accent.as_str()) { self.accent = "blue".into(); }
        self.skip_back = self.skip_back.clamp(5, 120);
        self.skip_fwd = self.skip_fwd.clamp(5, 300);
        self
    }
}

/// Everything the user can set, saved as one JSON file in the app data folder.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Settings {
    /// Lyrics viewer (and pasting lyrics) only works when this is on. Default off.
    pub lyrics_viewer: bool,
    /// Version of the legal documents the user accepted; None until accepted.
    pub terms_version: Option<String>,
    pub eq: EqSettings,
    /// Grid-Down mode: no network use at all, only local and downloaded content. Default: Online.
    pub grid_down: bool,
    /// Pause when headphones or a Bluetooth speaker disconnect (Android). Default on.
    pub pause_on_disconnect: bool,
    pub prefs: Prefs,
    /// Replaces Downloads/Cindrix as the save location after a restart.
    pub download_root: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { lyrics_viewer: false, terms_version: None, eq: EqSettings::default(), grid_down: false, pause_on_disconnect: true, prefs: Prefs::default(), download_root: None }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Settings {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str::<Settings>(&s).ok())
            .map(|mut s| { s.eq = s.eq.clone().clamped(); s.prefs = s.prefs.clone().clamped(); s })
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() { std::fs::create_dir_all(dir)?; }
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(self).unwrap())?;
        std::fs::rename(tmp, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lyrics_viewer_defaults_off() {
        assert!(!Settings::default().lyrics_viewer);
    }

    #[test]
    fn starts_in_online_mode() {
        assert!(!Settings::default().grid_down);
    }

    #[test]
    fn terms_not_accepted_by_default() {
        assert!(Settings::default().terms_version.is_none());
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = std::env::temp_dir().join(format!("cindrix-test-{}", std::process::id()));
        let path = dir.join("settings.json");
        let mut s = Settings::default();
        s.lyrics_viewer = true;
        s.terms_version = Some("1.0".into());
        s.eq.gains_db[3] = 4.5;
        s.grid_down = true;
        s.save(&path).unwrap();
        assert_eq!(Settings::load(&path), s);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn missing_or_corrupt_file_gives_defaults() {
        let dir = std::env::temp_dir().join(format!("cindrix-bad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.json");
        std::fs::write(&path, "{ not json").unwrap();
        assert_eq!(Settings::load(&path), Settings::default());
        let _ = std::fs::remove_dir_all(dir);
    }
}

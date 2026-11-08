use crate::dsp::EqSettings;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Everything the user can set, saved as one JSON file in the app data folder.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Settings {
    /// Lyrics viewer (and pasting lyrics) only works when this is on. Default off.
    pub lyrics_viewer: bool,
    /// Version of the legal documents the user accepted; None until accepted.
    pub terms_version: Option<String>,
    pub eq: EqSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { lyrics_viewer: false, terms_version: None, eq: EqSettings::default() }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Settings {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str::<Settings>(&s).ok())
            .map(|mut s| { s.eq = s.eq.clone().clamped(); s })
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

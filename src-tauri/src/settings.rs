use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Settings {
    /// Lyrics viewer (and pasting lyrics) only works when this is on. Default off.
    pub lyrics_viewer: bool,
}

impl Default for Settings {
    fn default() -> Self { Settings { lyrics_viewer: false } }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lyrics_viewer_defaults_off() {
        assert!(!Settings::default().lyrics_viewer);
    }
}

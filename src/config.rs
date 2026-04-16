use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::util;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub mpvpaper: MpvpaperConfig,
    pub monitors: MonitorAssignments,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub video_folder: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MpvpaperConfig {
    pub panscan: f64,
    pub loop_video: bool,
    pub gpu_api: String,
    pub hwdec: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorAssignments {
    pub assignments: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                video_folder: util::default_video_folder()
                    .to_string_lossy()
                    .into_owned(),
            },
            mpvpaper: MpvpaperConfig {
                panscan: 1.0,
                loop_video: true,
                gpu_api: "vulkan".into(),
                hwdec: "auto".into(),
            },
            monitors: MonitorAssignments {
                assignments: HashMap::new(),
            },
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(contents) => match toml::from_str(&contents) {
                    Ok(config) => return config,
                    Err(e) => log::warn!("Failed to parse config: {e}"),
                },
                Err(e) => log::warn!("Failed to read config: {e}"),
            }
        }
        let config = Config::default();
        config.save();
        config
    }

    pub fn save(&self) {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        match toml::to_string_pretty(self) {
            Ok(contents) => {
                if let Err(e) = fs::write(&path, contents) {
                    log::error!("Failed to write config: {e}");
                }
            }
            Err(e) => log::error!("Failed to serialize config: {e}"),
        }
    }

    fn path() -> PathBuf {
        util::config_dir().join("config.toml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = Config::default();
        assert_eq!(cfg.mpvpaper.panscan, 1.0);
        assert!(cfg.mpvpaper.loop_video);
        assert_eq!(cfg.mpvpaper.gpu_api, "vulkan");
        assert_eq!(cfg.mpvpaper.hwdec, "auto");
        assert!(cfg.monitors.assignments.is_empty());
        assert!(cfg.general.video_folder.contains("VideoWallpapers"));
    }

    #[test]
    fn config_roundtrip() {
        let mut cfg = Config::default();
        cfg.mpvpaper.panscan = 0.5;
        cfg.mpvpaper.hwdec = "nvdec".into();
        cfg.monitors
            .assignments
            .insert("DP-1".into(), "/tmp/test.mp4".into());

        let toml_str = toml::to_string_pretty(&cfg).unwrap();
        let loaded: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(loaded.mpvpaper.panscan, 0.5);
        assert_eq!(loaded.mpvpaper.hwdec, "nvdec");
        assert_eq!(
            loaded.monitors.assignments.get("DP-1").unwrap(),
            "/tmp/test.mp4"
        );
    }

    #[test]
    fn config_deserializes_partial() {
        let toml_str = r#"
[general]
video_folder = "/custom/path"

[mpvpaper]
panscan = 0.8
loop_video = false
gpu_api = "opengl"
hwdec = "vaapi"

[monitors.assignments]
"#;
        let cfg: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.general.video_folder, "/custom/path");
        assert_eq!(cfg.mpvpaper.panscan, 0.8);
        assert!(!cfg.mpvpaper.loop_video);
        assert_eq!(cfg.mpvpaper.gpu_api, "opengl");
        assert!(cfg.monitors.assignments.is_empty());
    }
}

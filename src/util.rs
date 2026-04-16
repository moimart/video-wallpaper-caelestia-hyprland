use std::path::PathBuf;

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("video-wallpaper")
}

pub fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("~/.cache"))
        .join("video-wallpaper")
}

pub fn thumbnail_dir() -> PathBuf {
    cache_dir().join("thumbnails")
}

pub fn frames_dir() -> PathBuf {
    cache_dir().join("frames")
}

pub fn caelestia_state_dir() -> PathBuf {
    dirs::state_dir()
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join(".local/state")
        })
        .join("caelestia")
}

pub fn scheme_json_path() -> PathBuf {
    caelestia_state_dir().join("scheme.json")
}

pub fn default_video_folder() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("VideoWallpapers")
}

pub fn check_command_exists(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

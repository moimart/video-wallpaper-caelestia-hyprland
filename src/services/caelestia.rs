use std::fs;
use std::path::Path;
use std::process::Command;

use crate::util;

pub fn update_wallpaper_scheme(video_path: &Path) -> Result<(), String> {
    let frames_dir = util::frames_dir();
    fs::create_dir_all(&frames_dir).map_err(|e| format!("mkdir frames: {e}"))?;

    let frame_path = frames_dir.join("current_frame.png");

    // Extract a representative frame
    let status = Command::new("ffmpeg")
        .args(["-y", "-ss", "00:00:02", "-i"])
        .arg(video_path)
        .args(["-frames:v", "1", "-q:v", "2"])
        .arg(&frame_path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("ffmpeg frame extraction: {e}"))?;

    if !status.success() {
        return Err("Failed to extract frame from video".into());
    }

    // Use caelestia's own wallpaper command — it handles state files,
    // color extraction, and scheme refresh all in one.
    let output = Command::new("caelestia")
        .args(["wallpaper", "-f"])
        .arg(&frame_path)
        .output();

    match output {
        Ok(o) if o.status.success() => {
            log::info!("Set caelestia wallpaper from video frame");
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            log::warn!("caelestia wallpaper returned error: {stderr}");
        }
        Err(e) => {
            log::warn!("caelestia not available: {e}");
        }
    }

    Ok(())
}

use std::fs;
use std::path::Path;
use std::process::Command;

use crate::util;

pub fn update_wallpaper_scheme(video_path: &Path) -> Result<(), String> {
    let frames_dir = util::frames_dir();
    fs::create_dir_all(&frames_dir).map_err(|e| format!("mkdir frames: {e}"))?;

    let frame_path = frames_dir.join("current_frame.png");

    // Probe video duration to pick a good seek point
    let seek = probe_seek_time(video_path);

    let status = Command::new("ffmpeg")
        .args(["-y", "-ss", &seek, "-i"])
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

/// Probe video duration and return a seek time that avoids black intro frames.
/// Falls back to "00:00:01" if probing fails.
fn probe_seek_time(video_path: &Path) -> String {
    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "csv=p=0",
        ])
        .arg(video_path)
        .output();

    if let Ok(o) = output {
        if let Ok(text) = std::str::from_utf8(&o.stdout) {
            if let Ok(duration) = text.trim().parse::<f64>() {
                // Seek to 10% of duration, clamped to 1..10 seconds
                let seek_secs = (duration * 0.1).clamp(1.0, 10.0);
                return format!("{seek_secs:.1}");
            }
        }
    }

    "1.0".to_string()
}

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::services::video_scanner::VideoEntry;
use crate::util;

pub fn thumbnail_path_for(video_path: &Path) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    video_path.to_string_lossy().hash(&mut hasher);
    let hash = hasher.finish();
    util::thumbnail_dir().join(format!("{hash:016x}.jpg"))
}

pub fn needs_generation(video: &VideoEntry) -> bool {
    let thumb = thumbnail_path_for(&video.path);
    if !thumb.exists() {
        return true;
    }
    if let Ok(meta) = fs::metadata(&thumb) {
        if let Ok(thumb_modified) = meta.modified() {
            return video.modified > thumb_modified;
        }
    }
    true
}

/// Probe video duration and return a seek time string.
/// Picks 10% of duration, clamped to [0.5, 5.0] seconds.
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
                let seek = (duration * 0.1).clamp(0.5, 5.0);
                return format!("{seek:.2}");
            }
        }
    }

    "1.0".to_string()
}

pub fn generate_thumbnail(video_path: &Path) -> Result<PathBuf, String> {
    let thumb_path = thumbnail_path_for(video_path);
    if let Some(parent) = thumb_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;
    }

    let seek = probe_seek_time(video_path);

    let status = Command::new("ffmpeg")
        .args(["-y", "-ss", &seek, "-i"])
        .arg(video_path)
        .args([
            "-frames:v", "1",
            "-vf", "scale=320:-1",
            "-q:v", "6",
        ])
        .arg(&thumb_path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match status {
        Ok(s) if s.success() && thumb_path.exists() => Ok(thumb_path),
        _ => Err(format!(
            "ffmpeg failed to extract thumbnail from {}",
            video_path.display()
        )),
    }
}

pub fn ensure_thumbnails(videos: &[VideoEntry]) -> Vec<(VideoEntry, PathBuf)> {
    videos
        .iter()
        .filter_map(|video| {
            let thumb = if needs_generation(video) {
                match generate_thumbnail(&video.path) {
                    Ok(p) => p,
                    Err(e) => {
                        log::warn!(
                            "Thumbnail generation failed for {}: {e}",
                            video.path.display()
                        );
                        return None;
                    }
                }
            } else {
                thumbnail_path_for(&video.path)
            };
            Some((video.clone(), thumb))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[test]
    fn thumbnail_path_is_deterministic() {
        let p1 = thumbnail_path_for(Path::new("/home/user/Videos/test.mp4"));
        let p2 = thumbnail_path_for(Path::new("/home/user/Videos/test.mp4"));
        assert_eq!(p1, p2);
    }

    #[test]
    fn thumbnail_path_differs_per_video() {
        let p1 = thumbnail_path_for(Path::new("/a.mp4"));
        let p2 = thumbnail_path_for(Path::new("/b.mp4"));
        assert_ne!(p1, p2);
    }

    #[test]
    fn thumbnail_path_is_jpg() {
        let p = thumbnail_path_for(Path::new("/test.mp4"));
        assert_eq!(p.extension().unwrap(), "jpg");
    }

    #[test]
    fn needs_generation_when_missing() {
        let video = VideoEntry {
            path: PathBuf::from("/tmp/nonexistent_video_xyz.mp4"),
            file_name: "test".into(),
            modified: SystemTime::now(),
        };
        assert!(needs_generation(&video));
    }
}

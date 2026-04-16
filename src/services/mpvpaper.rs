use std::path::Path;
use std::process::Command;

use crate::config::MpvpaperConfig;

pub fn apply_wallpaper(
    monitor: &str,
    video_path: &Path,
    config: &MpvpaperConfig,
) -> Result<(), String> {
    kill_existing(monitor);

    // Small delay to let wlroots release the surface
    std::thread::sleep(std::time::Duration::from_millis(150));

    let loop_flag = if config.loop_video { " --loop" } else { "" };

    let mpv_opts = format!(
        "no-border --vo=gpu-next --gpu-api={} --gpu-context=wayland --hwdec={} --vd-lavc-dr=yes --panscan={}{loop_flag}",
        config.gpu_api,
        config.hwdec,
        config.panscan,
    );

    let status = Command::new("mpvpaper")
        .arg("-sf")
        .arg("-o")
        .arg(&mpv_opts)
        .arg(monitor)
        .arg(video_path)
        .spawn()
        .map_err(|e| format!("Failed to spawn mpvpaper: {e}"))?;

    log::info!(
        "mpvpaper started (pid {}) on {monitor} with {}",
        status.id(),
        video_path.display()
    );
    Ok(())
}

pub fn restore_wallpapers(
    assignments: &std::collections::HashMap<String, String>,
    config: &MpvpaperConfig,
) -> Result<(), String> {
    for (monitor, video) in assignments {
        let path = Path::new(video);
        if !path.exists() {
            log::warn!("Saved wallpaper not found: {video}");
            continue;
        }
        apply_wallpaper(monitor, path, config)?;
    }
    Ok(())
}

fn kill_existing(monitor: &str) {
    let output = Command::new("pgrep")
        .args(["-a", "mpvpaper"])
        .output();

    let Ok(output) = output else { return };

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if line.contains(monitor) {
            if let Some(pid_str) = line.split_whitespace().next() {
                if let Ok(pid) = pid_str.parse::<u32>() {
                    let _ = Command::new("kill").arg(pid.to_string()).status();
                    log::info!("Killed mpvpaper pid {pid} for monitor {monitor}");
                }
            }
        }
    }
}

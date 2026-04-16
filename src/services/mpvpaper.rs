use std::path::Path;

use crate::config::MpvpaperConfig;
use super::process;

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

    let video_str = video_path.to_string_lossy();
    let args: Vec<&str> = vec!["-sf", "-o", &mpv_opts, monitor, &video_str];
    let pid = process::spawn("mpvpaper", &args)?;

    log::info!(
        "mpvpaper started (pid {pid}) on {monitor} with {}",
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
    let Ok(output) = process::run("pgrep", &["-a", "mpvpaper"]) else {
        return;
    };

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if line.contains(monitor) {
            if let Some(pid_str) = line.split_whitespace().next() {
                if let Ok(pid) = pid_str.parse::<u32>() {
                    let _ = process::run("kill", &[&pid.to_string()]);
                    log::info!("Killed mpvpaper pid {pid} for monitor {monitor}");
                }
            }
        }
    }
}

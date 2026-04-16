# Video Wallpaper Selector

A GTK4 video wallpaper selector for Hyprland/Wayland desktops. Browse your video collection in a coverflow-style gallery, pick a wallpaper, and it's applied instantly via [mpvpaper](https://github.com/GhostNaN/mpvpaper) with automatic GPU detection.

Built with Rust, GTK4, and libadwaita. Integrates with [caelestia-shell](https://github.com/caelestia) for dynamic theming.

## Features

- **Coverflow gallery** -- horizontally scrollable tiles that scale and fade based on distance from focus, with smooth animated transitions
- **Video preview on focus** -- the focused tile plays the video inline (muted, no controls), debounced to avoid GStreamer thrashing
- **GPU auto-detection** -- detects NVIDIA (`nvdec`), AMD (`vaapi`), or Intel (`vaapi`) and configures mpvpaper accordingly
- **Per-monitor wallpapers** -- assign different wallpapers to each monitor on multi-monitor setups
- **Caelestia theming** -- reads `~/.local/state/caelestia/scheme.json` and applies Material 3 colors live, with file watcher for instant updates
- **Dynamic color scheme** -- extracts a frame from the selected video and feeds it to `caelestia wallpaper -f` so the desktop palette matches
- **Chromeless UI** -- no title bar, no scrollbar, just the gallery. Floats centered via Hyprland windowrule
- **Persistent config** -- remembers wallpaper assignments, mpvpaper settings, and video folder in `~/.config/video-wallpaper/config.toml`
- **Boot restore** -- `video-wallpaper --restore` re-applies saved wallpapers (for use with a systemd user service)

## Dependencies

- `mpvpaper`
- `ffmpeg` / `ffprobe`
- `hyprctl` (Hyprland)
- `caelestia` (optional, for theming)
- GTK4 >= 4.18, libadwaita >= 1.6

## Building

```sh
cargo build --release
```

## Installation

```sh
# Binary
sudo install -Dm755 target/release/video-wallpaper /usr/bin/video-wallpaper

# Desktop entry
install -Dm644 data/sh.martinez.VideoWallpaper.desktop ~/.local/share/applications/

# Boot restore service (optional)
install -Dm644 data/video-wallpaper-restore.service ~/.config/systemd/user/
systemctl --user enable video-wallpaper-restore.service
```

## Hyprland setup

Add a windowrule to float and center the app:

```
windowrule {
  name = windowrule-video-wallpaper
  float = on
  noborder = on
  size = 1400 360
  match:class = sh.martinez.VideoWallpaper
}
```

Add a keybind:

```
bind = $mod, W, exec, video-wallpaper
```

## Usage

1. Put `.mp4`, `.mkv`, or `.webm` files in `~/VideoWallpapers/` (configurable via the gear icon)
2. Launch the app (keybind or `video-wallpaper`)
3. Navigate with arrow keys or mouse wheel
4. Press Enter to apply the focused wallpaper
5. Press Escape to close without changing

## Configuration

Stored in `~/.config/video-wallpaper/config.toml`:

```toml
[general]
video_folder = "/home/user/VideoWallpapers"

[mpvpaper]
panscan = 1.0
loop_video = true
gpu_api = "vulkan"
hwdec = "nvdec"    # auto-detected, overridable

[monitors.assignments]
HDMI-A-1 = "/home/user/VideoWallpapers/waterfall.mp4"
```

## Tests

```sh
cargo test
```

## License

MIT

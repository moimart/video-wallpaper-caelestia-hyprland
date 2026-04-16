# Video Wallpaper Selector — PRD

## Overview

A GTK4 application written in Rust that lets the user browse and select video wallpapers from a gallery, then applies them via `mpvpaper`. The app is designed for Hyprland/Wayland desktops with caelestia-shell theming integration.

---

## Core Behavior

- **Single instance**: Only one instance of the app can run at a time. If invoked again (e.g., from a Hyprland keybind), the existing instance should be brought to focus or toggled via GApplication/D-Bus activation.
- **Dismiss on selection**: Once the user selects a wallpaper, the app closes immediately. mpvpaper continues running in the background.
- **Invocation**: The app is meant to be launched via a keybind in Hyprland. It is not a persistent/tray application.
- **Chromeless window**: No title bar, no window decorations. The window floats centered on screen via Hyprland windowrule. Escape key closes the app.

---

## Hard Dependencies

- `mpvpaper` — must be installed. The app should fail with a clear error at startup if missing.
- `ffmpeg` / `ffprobe` — required for thumbnail generation.

---

## mpvpaper Configuration

Base flags:

```
-sf -o "no-border --vo=gpu-next --gpu-api=vulkan --gpu-context=wayland --hwdec=<detected> --vd-lavc-dr=yes --panscan=1.0 --loop"
```

### GPU Detection & hwdec

Detect the GPU vendor at runtime and set `--hwdec` accordingly:

| Vendor | `--hwdec` value |
|--------|----------------|
| NVIDIA | `nvdec` |
| AMD    | `vaapi` |
| Intel  | `vaapi` |

---

## Monitor Support

- Detect available Wayland/Hyprland monitors (e.g., via `hyprctl monitors -j`).
- If **one monitor**: no selector shown, apply directly.
- If **multiple monitors**: show a monitor dropdown in the UI. The user can assign a **different wallpaper per monitor**.
- Persist the per-monitor wallpaper assignments.

---

## Video Gallery

### Source

- Default video folder: `~/VideoWallpapers`
- Configurable in the settings dialog (accessed via gear icon).
- Supported formats: `.mp4`, `.mkv`, `.webm`

### Thumbnails

- Generate thumbnails on first scan via `ffmpeg` (320px wide, JPEG, tries seek at 2s/1s/0s).
- Cache thumbnails to `~/.cache/video-wallpaper/thumbnails/` for fast gallery loading.
- Regenerate if source video is newer than cached thumbnail.

### Layout & Interaction — Coverflow

- Horizontal gallery of video tiles (single row), **no scrollbar**.
- Tiles are squares with **rounded corners**, no text labels.
- **Coverflow effect** (inspired by Apple's Cover Flow):
  - The **focused tile** is enlarged (`scale(1.15)`) with full opacity and a highlighted border.
  - Adjacent tiles are progressively smaller and more transparent:
    - Distance 1: `scale(0.88)`, 75% opacity
    - Distance 2: `scale(0.72)`, 55% opacity
    - Distance 3+: `scale(0.6)`, 45% opacity
  - All transitions animate smoothly (350ms ease-out cubic for scroll, 350ms CSS transitions for scale/opacity).
- **Focus triggers playback**: when a tile becomes the focused center tile, its video starts playing inline (muted, no controls/slider). Uses `GtkPicture` with `GtkMediaFile` as paintable. The thumbnail stays visible until the first video frame is decoded (`notify::prepared`), avoiding a blank flash on transition. Playback stops when focus moves away.
- **Navigation**: arrow keys or mouse wheel navigate focus left/right (not pixel-scroll). The gallery smoothly animates to center the newly focused tile.
- **Selection**: press Enter or click on a tile. Click first focuses the tile (with coverflow animation); Enter confirms selection.
- **Edge padding**: the tile strip has large horizontal padding so the first and last tiles can be centered in the viewport without hitting the window edges.
- **Initial focus**: on launch, the currently assigned wallpaper tile is focused. If none, the first tile.
- On selection, the wallpaper is applied to the chosen monitor (or the only monitor), and the app **exits**.

---

## Persistence

- Remember the last-selected wallpaper **per monitor**.
- On login/boot, automatically re-apply the saved wallpapers via `video-wallpaper --restore` (systemd user service).
- Store configuration in `~/.config/video-wallpaper/config.toml`.

---

## Settings

Accessed via a gear icon in the top-right corner of the gallery, which opens an `adw::Dialog` overlay.

### Video folder
- Path to the video wallpaper directory (default: `~/VideoWallpapers`).

### Simplified mpvpaper settings
- **Panscan** (0.0 – 1.0 slider, default 1.0)
- **Loop** (toggle, default on)
- **GPU API** (dropdown: `vulkan`, `opengl` — default `vulkan`)
- **Hardware decoding** (auto-detected, but overridable: `nvdec`, `vaapi`, `auto`, `no`)

---

## Theming — Caelestia Integration

### Reading the active scheme

The app reads and applies colors from the caelestia scheme system:

- **Source**: `~/.local/state/caelestia/scheme.json`
- **Format**: JSON with a `colours` map containing Material 3 color tokens as hex strings.
- The app watches this file via `gio::FileMonitor` and updates its colors live.

### Color mapping (Material 3 tokens to GTK)

| UI element | Caelestia token |
|------------|----------------|
| App background | `background` |
| Tile background | `surfaceContainer` |
| Tile border | `outlineVariant` |
| Tile border (hover) | `outline` |
| Selected/focused tile border | `primary` |
| Gear button | `surfaceContainer` / `primaryContainer` on hover |
| Settings dialog | `surfaceContainer` |

### Implementation

Apply theming via GTK4 CSS provider, dynamically regenerated from `scheme.json` values. No hardcoded colors. Static structural CSS (transitions, border-radius) is loaded separately.

---

## Dominant Color Extraction (Caelestia Dynamic Scheme)

When the user selects a video wallpaper:

1. Extract a representative frame from the video via `ffmpeg`.
2. Symlink that frame to `~/.local/state/caelestia/wallpaper/current` and write the path to `path.txt`.
3. Invoke `caelestia scheme set -n dynamic` to trigger scheme regeneration.

This makes the video wallpaper feed the desktop color scheme just like a static wallpaper would.

---

## Hyprland Integration

### Windowrule (in `hyprland/rules.conf`)

```
windowrule {
  name = windowrule-video-wallpaper
  float = on
  size = 1400 360
  match:class = sh.martinez.VideoWallpaper
}
```

### Keybind example

```
bind = $mod, W, exec, video-wallpaper
```

---

## Technical Notes

- **GTK4 + libadwaita** via `gtk4-rs 0.11` / `libadwaita-rs 0.9`.
- **Video preview**: `GtkPicture` with `GtkMediaFile` as paintable (GStreamer backend). MediaFile is created when a tile gains focus and dropped when focus moves away — only one video plays at a time.
- **Single-instance**: `adw::Application` with D-Bus activation (`sh.martinez.VideoWallpaper`).
- **No async runtime**: Uses GLib main loop, `gio::FileMonitor` for file watching, `gio::spawn_blocking` for I/O.
- **Config format**: TOML in `~/.config/video-wallpaper/config.toml`.
- **Boot restore**: systemd user service calling `video-wallpaper --restore`.

---

## Out of Scope (for now)

- Playlist / slideshow rotation of wallpapers.
- Audio from wallpapers.
- Non-Wayland / non-Hyprland compositors.
- Video wallpaper recording or downloading.

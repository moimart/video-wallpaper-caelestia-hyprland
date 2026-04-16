# Maintainer: Moises Martinez <moises@martinez.sh>

pkgname=video-wallpaper
pkgver=0.1.0
pkgrel=1
pkgdesc='GTK4 coverflow video wallpaper selector for Hyprland with caelestia theming'
arch=('x86_64')
url='https://github.com/moimart/video-wallpaper'
license=('MIT')
depends=('gtk4' 'libadwaita' 'gstreamer' 'gst-plugins-base' 'gst-plugins-good'
         'mpvpaper' 'ffmpeg' 'hyprland')
makedepends=('rust' 'cargo')
optdepends=('caelestia-cli: dynamic color scheme from video wallpapers')
source=()

build() {
    cd "$startdir"
    cargo build --release --locked
}

package() {
    cd "$startdir"
    install -Dm755 target/release/video-wallpaper "$pkgdir/usr/bin/video-wallpaper"
    install -Dm644 data/sh.martinez.VideoWallpaper.desktop "$pkgdir/usr/share/applications/sh.martinez.VideoWallpaper.desktop"
    install -Dm644 data/video-wallpaper-restore.service "$pkgdir/usr/lib/systemd/user/video-wallpaper-restore.service"
}

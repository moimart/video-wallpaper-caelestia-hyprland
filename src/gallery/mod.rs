pub mod monitor_selector;
pub mod video_tile;

use gtk::prelude::*;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use crate::config::Config;
use crate::services::{monitors::MonitorInfo, thumbnails, video_scanner};
use video_tile::VideoTile;

const TILE_STRIDE: f64 = 252.0; // tile size (240) + spacing (12)
const EDGE_PAD: f64 = 580.0; // must match .tile-strip CSS padding

pub struct GalleryView {
    pub container: gtk::Box,
    tiles: Rc<RefCell<Vec<VideoTile>>>,
    tile_box: gtk::Box,
    scrolled: gtk::ScrolledWindow,
    monitor_selector: monitor_selector::MonitorSelector,
    monitors: Vec<MonitorInfo>,
    focused_index: Rc<RefCell<Option<usize>>>,
    preview_timer: Rc<RefCell<Option<glib::SourceId>>>,
}

impl GalleryView {
    pub fn new(
        config: Rc<RefCell<Config>>,
        monitors: Vec<MonitorInfo>,
        on_wallpaper_selected: Rc<dyn Fn()>,
    ) -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.set_focusable(true);

        let monitor_selector = monitor_selector::MonitorSelector::new(&monitors);
        if monitors.len() > 1 {
            container.append(&monitor_selector.widget);
        }

        let scrolled = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::External)
            .vscrollbar_policy(gtk::PolicyType::Never)
            .css_classes(vec!["gallery-scroll"])
            .vexpand(true)
            .valign(gtk::Align::Center)
            .propagate_natural_height(true)
            .build();

        let tile_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .halign(gtk::Align::Start)
            .valign(gtk::Align::Center)
            .css_classes(vec!["tile-strip"])
            .build();

        scrolled.set_child(Some(&tile_box));
        container.append(&scrolled);

        let tiles: Rc<RefCell<Vec<VideoTile>>> = Rc::new(RefCell::new(Vec::new()));
        let focused_index: Rc<RefCell<Option<usize>>> = Rc::new(RefCell::new(None));
        let preview_timer: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));

        // Mouse wheel -> navigate focus
        let scroll_ctrl =
            gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        {
            let tiles = tiles.clone();
            let focused = focused_index.clone();
            let scrolled = scrolled.clone();
            let timer = preview_timer.clone();
            scroll_ctrl.connect_scroll(move |_, _, dy| {
                let count = tiles.borrow().len();
                if count == 0 {
                    return glib::Propagation::Proceed;
                }
                let cur = focused.borrow().unwrap_or(0);
                let next = if dy > 0.0 {
                    (cur + 1).min(count - 1)
                } else if cur > 0 {
                    cur - 1
                } else {
                    cur
                };
                if next != cur {
                    Self::focus_tile(&tiles, &focused, &scrolled, &timer, next);
                }
                glib::Propagation::Stop
            });
        }
        container.add_controller(scroll_ctrl);

        // Keyboard navigation
        let key_ctrl = gtk::EventControllerKey::new();
        {
            let tiles = tiles.clone();
            let focused = focused_index.clone();
            let scrolled = scrolled.clone();
            let timer = preview_timer.clone();
            let config = config.clone();
            let monitors = monitors.clone();
            let monitor_sel = monitor_selector.clone();
            let on_selected = on_wallpaper_selected.clone();

            key_ctrl.connect_key_pressed(move |_, key, _, _| {
                let count = tiles.borrow().len();
                if count == 0 {
                    return glib::Propagation::Proceed;
                }
                let cur = focused.borrow().unwrap_or(0);
                match key {
                    gdk::Key::Left => {
                        let next = if cur > 0 { cur - 1 } else { cur };
                        Self::focus_tile(&tiles, &focused, &scrolled, &timer, next);
                        glib::Propagation::Stop
                    }
                    gdk::Key::Right => {
                        let next = (cur + 1).min(count - 1);
                        Self::focus_tile(&tiles, &focused, &scrolled, &timer, next);
                        glib::Propagation::Stop
                    }
                    gdk::Key::Return | gdk::Key::KP_Enter => {
                        let tiles_borrow = tiles.borrow();
                        Self::select_tile(
                            &tiles_borrow,
                            cur,
                            &config,
                            &monitors,
                            &monitor_sel,
                            &on_selected,
                        );
                        glib::Propagation::Stop
                    }
                    _ => glib::Propagation::Proceed,
                }
            });
        }
        container.add_controller(key_ctrl);

        Self {
            container,
            tiles,
            tile_box,
            scrolled,
            monitor_selector,
            monitors,
            focused_index,
            preview_timer,
        }
    }

    /// Coverflow: update CSS classes instantly, debounce video preview 350ms.
    fn focus_tile(
        tiles_rc: &Rc<RefCell<Vec<VideoTile>>>,
        focused: &Rc<RefCell<Option<usize>>>,
        scrolled: &gtk::ScrolledWindow,
        preview_timer: &Rc<RefCell<Option<glib::SourceId>>>,
        idx: usize,
    ) {
        let old = *focused.borrow();
        let tiles = tiles_rc.borrow();

        // Cancel pending preview
        if let Some(id) = preview_timer.borrow_mut().take() {
            id.remove();
        }

        // Stop preview on old focused tile
        if let Some(old_idx) = old {
            if old_idx != idx {
                if let Some(tile) = tiles.get(old_idx) {
                    tile.stop_preview();
                }
            }
        }

        // Apply distance-based CSS classes (instant, lightweight)
        let classes = [
            "coverflow-center",
            "coverflow-near",
            "coverflow-far",
            "coverflow-distant",
        ];
        for (i, tile) in tiles.iter().enumerate() {
            for cls in &classes {
                tile.widget.remove_css_class(cls);
            }
            tile.widget.remove_css_class("focused");

            let dist = (i as isize - idx as isize).unsigned_abs();
            tile.widget.add_css_class(match dist {
                0 => "coverflow-center",
                1 => "coverflow-near",
                2 => "coverflow-far",
                _ => "coverflow-distant",
            });
        }
        if let Some(tile) = tiles.get(idx) {
            tile.widget.add_css_class("focused");
        }

        *focused.borrow_mut() = Some(idx);
        drop(tiles); // release borrow before scheduling callback

        // Smooth-scroll to center
        let adj = scrolled.hadjustment();
        let page_size = adj.page_size();
        let tile_center = EDGE_PAD + (idx as f64) * TILE_STRIDE + TILE_STRIDE / 2.0;
        let target = (tile_center - page_size / 2.0).max(0.0);
        let start = adj.value();
        let delta = target - start;

        if delta.abs() > 1.0 {
            let start_time: std::cell::Cell<i64> = std::cell::Cell::new(0);
            let first_frame = std::cell::Cell::new(true);
            let adj_ref = adj.clone();
            scrolled.add_tick_callback(move |_, clock| {
                let now = clock.frame_time();
                if first_frame.get() {
                    start_time.set(now);
                    first_frame.set(false);
                }
                let elapsed = (now - start_time.get()) as f64 / 300_000.0;
                let t = elapsed.min(1.0);
                let ease = 1.0 - (1.0 - t).powi(3);
                adj_ref.set_value(start + delta * ease);
                if t >= 1.0 {
                    glib::ControlFlow::Break
                } else {
                    glib::ControlFlow::Continue
                }
            });
        }

        // Debounce video preview: wait 350ms after last navigation before playing.
        let tiles_rc = tiles_rc.clone();
        let focused = focused.clone();
        let timer_ref = preview_timer.clone();

        let source_id = glib::timeout_add_local_once(
            std::time::Duration::from_millis(350),
            move || {
                *timer_ref.borrow_mut() = None;
                if focused.borrow().unwrap_or(usize::MAX) != idx {
                    return;
                }
                let tiles = tiles_rc.borrow();
                if let Some(tile) = tiles.get(idx) {
                    tile.start_preview();
                }
            },
        );
        *preview_timer.borrow_mut() = Some(source_id);
    }

    fn select_tile(
        tiles: &[VideoTile],
        idx: usize,
        config: &Rc<RefCell<Config>>,
        monitors: &[MonitorInfo],
        monitor_sel: &monitor_selector::MonitorSelector,
        on_selected: &Rc<dyn Fn()>,
    ) {
        let Some(tile) = tiles.get(idx) else { return };
        let video_path = tile.video_path.clone();
        let mut cfg = config.borrow_mut();

        let monitor_name = if monitors.len() > 1 {
            monitor_sel.selected_monitor()
        } else {
            monitors.first().map(|m| m.name.clone())
        };
        let Some(monitor) = monitor_name else { return };

        if let Err(e) =
            crate::services::mpvpaper::apply_wallpaper(&monitor, &video_path, &cfg.mpvpaper)
        {
            log::error!("Failed to apply wallpaper: {e}");
            return;
        }

        cfg.monitors
            .assignments
            .insert(monitor.clone(), video_path.to_string_lossy().into_owned());
        cfg.save();

        // Update caelestia scheme before closing
        if let Err(e) = crate::services::caelestia::update_wallpaper_scheme(&video_path) {
            log::warn!("Caelestia update failed: {e}");
        }

        on_selected();
    }

    pub fn load_videos(&self, config: &Config) {
        let folder = Path::new(&config.general.video_folder);
        let videos = video_scanner::scan_folder(folder);

        if videos.is_empty() {
            self.show_empty_state();
            return;
        }

        let entries = thumbnails::ensure_thumbnails(&videos);

        let current_monitor = if self.monitors.len() > 1 {
            self.monitor_selector.selected_monitor()
        } else {
            self.monitors.first().map(|m| m.name.clone())
        };
        let current_wallpaper = current_monitor
            .as_ref()
            .and_then(|m| config.monitors.assignments.get(m))
            .cloned();

        let initial_focus = entries
            .iter()
            .position(|(v, _)| {
                current_wallpaper
                    .as_ref()
                    .is_some_and(|cw| cw == &v.path.to_string_lossy().as_ref())
            })
            .unwrap_or(0);

        while let Some(child) = self.tile_box.first_child() {
            self.tile_box.remove(&child);
        }

        let mut tiles = self.tiles.borrow_mut();
        tiles.clear();

        for (i, (video, thumb_path)) in entries.iter().enumerate() {
            let is_selected = current_wallpaper
                .as_ref()
                .is_some_and(|cw| cw == &video.path.to_string_lossy().as_ref());
            let tile = VideoTile::new(video, thumb_path, is_selected);

            let gesture = gtk::GestureClick::new();
            let tiles_ref = self.tiles.clone();
            let focused_ref = self.focused_index.clone();
            let scrolled_ref = self.scrolled.clone();
            let timer_ref = self.preview_timer.clone();
            let tile_idx = i;
            gesture.connect_released(move |_, _, _, _| {
                Self::focus_tile(&tiles_ref, &focused_ref, &scrolled_ref, &timer_ref, tile_idx);
            });
            tile.widget.add_controller(gesture);

            self.tile_box.append(&tile.widget);
            tiles.push(tile);
        }
        drop(tiles);

        let tiles = self.tiles.clone();
        let scrolled = self.scrolled.clone();
        let focused = self.focused_index.clone();
        let timer = self.preview_timer.clone();
        *focused.borrow_mut() = None;

        glib::idle_add_local_once(move || {
            if tiles.borrow().is_empty() {
                return;
            }
            Self::focus_tile(&tiles, &focused, &scrolled, &timer, initial_focus);
        });
    }

    /// Release all GStreamer media objects before window teardown.
    pub fn release_media(&self) {
        // Cancel any pending preview timer
        if let Some(id) = self.preview_timer.borrow_mut().take() {
            id.remove();
        }
        for tile in self.tiles.borrow().iter() {
            tile.release_media();
        }
    }

    fn show_empty_state(&self) {
        while let Some(child) = self.tile_box.first_child() {
            self.tile_box.remove(&child);
        }
        let label = gtk::Label::builder()
            .label("No videos found.\nAdd .mp4, .mkv, or .webm files to your video folder.")
            .css_classes(vec!["empty-state"])
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .build();
        self.tile_box.append(&label);
    }
}

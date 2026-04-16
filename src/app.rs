use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::Config;
use crate::services::{gpu, monitors};
use crate::theming;
use crate::util;
use crate::window;

pub struct VideoWallpaperApp {
    app: adw::Application,
}

impl VideoWallpaperApp {
    pub fn new() -> Self {
        let app = adw::Application::builder()
            .application_id("sh.martinez.VideoWallpaper")
            .flags(gio::ApplicationFlags::default())
            .build();

        app.connect_activate(move |app| {
            // If window already exists, just present it
            if let Some(win) = app.active_window() {
                win.present();
                return;
            }

            Self::activate(app);
        });

        Self { app }
    }

    pub fn run(&self) -> glib::ExitCode {
        // Check for --restore flag
        let args: Vec<String> = std::env::args().collect();
        if args.iter().any(|a| a == "--restore") {
            return Self::restore();
        }

        self.app.run()
    }

    fn activate(app: &adw::Application) {
        // Check dependencies
        let missing: Vec<&str> = ["mpvpaper", "ffmpeg", "ffprobe", "hyprctl"]
            .into_iter()
            .filter(|cmd| !util::check_command_exists(cmd))
            .collect();

        if !missing.is_empty() {
            let msg = format!(
                "Missing required dependencies:\n\n{}",
                missing.join(", ")
            );
            let dialog = adw::AlertDialog::builder()
                .heading("Missing Dependencies")
                .body(&msg)
                .build();
            dialog.add_response("close", "Close");
            dialog.set_default_response(Some("close"));

            let app_ref = app.clone();
            dialog.connect_response(None, move |_, _| {
                app_ref.quit();
            });

            // Need a temporary window to show the dialog
            let tmp_win = adw::ApplicationWindow::builder()
                .application(app)
                .build();
            dialog.present(Some(&tmp_win));
            return;
        }

        // Load config
        let mut config = Config::load();

        // Detect GPU and resolve "auto" hwdec
        if config.mpvpaper.hwdec == "auto" {
            let vendor = gpu::detect_gpu();
            config.mpvpaper.hwdec = vendor.hwdec().to_string();
            log::info!("Detected GPU: {:?} -> hwdec={}", vendor, config.mpvpaper.hwdec);
        }

        let config = Rc::new(RefCell::new(config));

        // Detect monitors
        let detected_monitors = monitors::detect_monitors();
        if detected_monitors.is_empty() {
            log::error!("No monitors detected");
        }

        // Setup theming
        if let Some(display) = gdk::Display::default() {
            // Load static base CSS
            let base_css = gtk::CssProvider::new();
            base_css.load_from_string(include_str!("../style/base.css"));
            gtk::style_context_add_provider_for_display(
                &display,
                &base_css,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );

            // Load dynamic caelestia theme
            let theme_provider = theming::setup_theme(&display);
            theming::watch_scheme(theme_provider);
        }

        // Build window
        let win = window::build_window(app, config, detected_monitors);
        win.present();
    }

    fn restore() -> glib::ExitCode {
        let mut config = Config::load();

        if config.mpvpaper.hwdec == "auto" {
            let vendor = gpu::detect_gpu();
            config.mpvpaper.hwdec = vendor.hwdec().to_string();
        }

        if config.monitors.assignments.is_empty() {
            log::info!("No wallpapers to restore");
            return glib::ExitCode::SUCCESS;
        }

        match crate::services::mpvpaper::restore_wallpapers(
            &config.monitors.assignments,
            &config.mpvpaper,
        ) {
            Ok(()) => {
                log::info!("Wallpapers restored successfully");
                glib::ExitCode::SUCCESS
            }
            Err(e) => {
                log::error!("Failed to restore wallpapers: {e}");
                glib::ExitCode::FAILURE
            }
        }
    }
}

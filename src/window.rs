use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::Config;
use crate::gallery::GalleryView;
use crate::services::monitors::MonitorInfo;
use crate::settings::SettingsView;

pub fn build_window(
    app: &adw::Application,
    config: Rc<RefCell<Config>>,
    monitors: Vec<MonitorInfo>,
) -> adw::ApplicationWindow {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Video Wallpaper")
        .default_width(1400)
        .default_height(360)
        .decorated(false)
        .build();

    // Close-on-select callback
    let window_ref = window.clone();
    let on_selected: Rc<dyn Fn()> = Rc::new(move || {
        window_ref.close();
    });

    let gallery = GalleryView::new(config.clone(), monitors, on_selected);
    gallery.load_videos(&config.borrow());

    let gallery_ref = Rc::new(gallery);

    // Main layout: gallery fills the window, gear icon floats top-right
    let overlay = gtk::Overlay::new();
    overlay.set_child(Some(&gallery_ref.container));

    // Settings gear button
    let gear = gtk::Button::builder()
        .icon_name("emblem-system-symbolic")
        .css_classes(vec!["circular", "settings-gear"])
        .halign(gtk::Align::End)
        .valign(gtk::Align::Start)
        .margin_top(12)
        .margin_end(12)
        .tooltip_text("Settings")
        .build();

    let config_settings = config.clone();
    let gallery_settings = gallery_ref.clone();
    let window_for_dialog = window.clone();
    gear.connect_clicked(move |_| {
        let config_for_reload = config_settings.clone();
        let gallery_for_reload = gallery_settings.clone();
        let on_folder_changed: Rc<dyn Fn()> = Rc::new(move || {
            gallery_for_reload.load_videos(&config_for_reload.borrow());
        });

        let settings = SettingsView::new(config_settings.clone(), on_folder_changed);

        let dialog = adw::Dialog::builder()
            .title("Settings")
            .content_width(480)
            .content_height(400)
            .build();
        dialog.set_child(Some(&settings.widget));
        dialog.present(Some(&window_for_dialog));
    });

    overlay.add_overlay(&gear);
    window.set_content(Some(&overlay));

    // Before window destruction, release all GStreamer media objects.
    // This prevents the NVIDIA GL driver from crashing when g_object_unref
    // races with GStreamer's background GL thread during GTK teardown.
    let gallery_cleanup = gallery_ref.clone();
    window.connect_close_request(move |_| {
        gallery_cleanup.release_media();
        glib::Propagation::Proceed
    });

    // Escape to close
    let esc = gtk::ShortcutController::new();
    let window_close = window.clone();
    esc.add_shortcut(
        gtk::Shortcut::builder()
            .trigger(&gtk::ShortcutTrigger::parse_string("Escape").unwrap())
            .action(&gtk::CallbackAction::new(move |_, _| {
                window_close.close();
                glib::Propagation::Stop
            }))
            .build(),
    );
    window.add_controller(esc);

    // Grab focus on the gallery so keyboard nav works immediately
    let gallery_container = gallery_ref.container.clone();
    window.connect_show(move |_| {
        gallery_container.grab_focus();
    });

    window
}

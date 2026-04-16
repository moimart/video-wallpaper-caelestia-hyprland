pub mod mpvpaper_prefs;

use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::Config;

pub struct SettingsView {
    pub widget: adw::PreferencesPage,
}

impl SettingsView {
    pub fn new(config: Rc<RefCell<Config>>, on_folder_changed: Rc<dyn Fn()>) -> Self {
        let page = adw::PreferencesPage::new();

        // --- Video Source group ---
        let source_group = adw::PreferencesGroup::builder()
            .title("Video Source")
            .build();

        let folder_row = adw::ActionRow::builder()
            .title("Video Folder")
            .subtitle(&config.borrow().general.video_folder)
            .activatable(true)
            .build();

        let folder_button = gtk::Button::builder()
            .icon_name("folder-open-symbolic")
            .valign(gtk::Align::Center)
            .build();

        let config_folder = config.clone();
        let folder_row_ref = folder_row.clone();
        let on_folder = on_folder_changed.clone();

        folder_button.connect_clicked(move |btn| {
            let dialog = gtk::FileDialog::builder()
                .title("Select Video Folder")
                .modal(true)
                .build();

            let initial = gio::File::for_path(&config_folder.borrow().general.video_folder);
            dialog.set_initial_folder(Some(&initial));

            let window = btn.root().and_downcast::<gtk::Window>();
            let config_ref = config_folder.clone();
            let row_ref = folder_row_ref.clone();
            let on_folder_ref = on_folder.clone();

            dialog.select_folder(
                window.as_ref(),
                gio::Cancellable::NONE,
                move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            let path_str = path.to_string_lossy().into_owned();
                            config_ref.borrow_mut().general.video_folder = path_str.clone();
                            config_ref.borrow().save();
                            row_ref.set_subtitle(&path_str);
                            on_folder_ref();
                        }
                    }
                },
            );
        });

        folder_row.add_suffix(&folder_button);
        source_group.add(&folder_row);
        page.add(&source_group);

        // --- mpvpaper settings ---
        let mpv_group = mpvpaper_prefs::build_mpvpaper_group(config.clone());
        page.add(&mpv_group);

        Self { widget: page }
    }
}

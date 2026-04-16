use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::Config;

pub fn build_mpvpaper_group(config: Rc<RefCell<Config>>) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("mpvpaper Settings")
        .build();

    // Panscan slider
    let panscan_row = adw::ActionRow::builder()
        .title("Panscan")
        .subtitle("How much to crop to fill the screen (0.0 = none, 1.0 = full)")
        .build();

    let panscan_scale = gtk::Scale::builder()
        .orientation(gtk::Orientation::Horizontal)
        .width_request(200)
        .valign(gtk::Align::Center)
        .build();
    panscan_scale.set_range(0.0, 1.0);
    panscan_scale.set_increments(0.1, 0.1);
    panscan_scale.set_value(config.borrow().mpvpaper.panscan);
    panscan_scale.set_draw_value(true);

    let config_ps = config.clone();
    panscan_scale.connect_value_changed(move |scale| {
        let val = (scale.value() * 10.0).round() / 10.0;
        config_ps.borrow_mut().mpvpaper.panscan = val;
        config_ps.borrow().save();
    });

    panscan_row.add_suffix(&panscan_scale);
    group.add(&panscan_row);

    // Loop toggle
    let loop_row = adw::SwitchRow::builder()
        .title("Loop Video")
        .active(config.borrow().mpvpaper.loop_video)
        .build();

    let config_loop = config.clone();
    loop_row.connect_active_notify(move |row| {
        config_loop.borrow_mut().mpvpaper.loop_video = row.is_active();
        config_loop.borrow().save();
    });

    group.add(&loop_row);

    // GPU API dropdown
    let gpu_api_row = adw::ComboRow::builder()
        .title("GPU API")
        .build();

    let gpu_apis = gtk::StringList::new(&["vulkan", "opengl"]);
    gpu_api_row.set_model(Some(&gpu_apis));

    let current_api = &config.borrow().mpvpaper.gpu_api;
    gpu_api_row.set_selected(if current_api == "opengl" { 1 } else { 0 });

    let config_gpu = config.clone();
    gpu_api_row.connect_selected_notify(move |row| {
        let idx = row.selected();
        let api = if idx == 1 { "opengl" } else { "vulkan" };
        config_gpu.borrow_mut().mpvpaper.gpu_api = api.into();
        config_gpu.borrow().save();
    });

    group.add(&gpu_api_row);

    // Hardware decoding dropdown
    let hwdec_row = adw::ComboRow::builder()
        .title("Hardware Decoding")
        .build();

    let hwdec_opts = gtk::StringList::new(&["auto", "nvdec", "vaapi", "no"]);
    hwdec_row.set_model(Some(&hwdec_opts));

    let current_hwdec = &config.borrow().mpvpaper.hwdec;
    let hwdec_idx = match current_hwdec.as_str() {
        "nvdec" => 1,
        "vaapi" => 2,
        "no" => 3,
        _ => 0,
    };
    hwdec_row.set_selected(hwdec_idx);

    let config_hw = config.clone();
    hwdec_row.connect_selected_notify(move |row| {
        let hwdec = match row.selected() {
            1 => "nvdec",
            2 => "vaapi",
            3 => "no",
            _ => "auto",
        };
        config_hw.borrow_mut().mpvpaper.hwdec = hwdec.into();
        config_hw.borrow().save();
    });

    group.add(&hwdec_row);

    group
}

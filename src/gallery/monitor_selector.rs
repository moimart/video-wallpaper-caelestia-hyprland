use gtk::prelude::*;

use crate::services::monitors::MonitorInfo;

#[derive(Clone)]
pub struct MonitorSelector {
    pub widget: gtk::Box,
    dropdown: gtk::DropDown,
    monitors: Vec<MonitorInfo>,
}

impl MonitorSelector {
    pub fn new(monitors: &[MonitorInfo]) -> Self {
        let widget = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .css_classes(vec!["monitor-bar"])
            .spacing(8)
            .halign(gtk::Align::Center)
            .build();

        let label = gtk::Label::builder().label("Monitor:").build();

        let display_names: Vec<String> = monitors
            .iter()
            .map(|m| {
                if m.description.is_empty() {
                    format!("{} ({}x{})", m.name, m.width, m.height)
                } else {
                    format!("{} — {}", m.name, m.description)
                }
            })
            .collect();

        let names_arr: Vec<&str> = display_names.iter().map(String::as_str).collect();
        let dropdown = gtk::DropDown::from_strings(&names_arr);
        dropdown.set_selected(0);

        widget.append(&label);
        widget.append(&dropdown);

        Self {
            widget,
            dropdown,
            monitors: monitors.to_vec(),
        }
    }

    pub fn selected_monitor(&self) -> Option<String> {
        let idx = self.dropdown.selected() as usize;
        self.monitors.get(idx).map(|m| m.name.clone())
    }
}

use gio::prelude::*;
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

use crate::util;

#[derive(Debug, Clone)]
struct SchemeColors {
    background: String,
    surface_container: String,
    surface_container_high: String,
    on_surface: String,
    on_surface_variant: String,
    outline_variant: String,
    primary: String,
    primary_container: String,
    on_primary_container: String,
    secondary_container: String,
    on_secondary_container: String,
    error: String,
}

fn hex_to_css(hex: &str) -> String {
    format!("#{hex}")
}

fn load_scheme_colors() -> Option<SchemeColors> {
    let path = util::scheme_json_path();
    let data = fs::read_to_string(&path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&data).ok()?;
    let c = json.get("colours")?;

    let get = |key: &str| -> String {
        c.get(key)
            .and_then(|v| v.as_str())
            .map(|s| hex_to_css(s))
            .unwrap_or_else(|| "#888888".into())
    };

    Some(SchemeColors {
        background: get("background"),
        surface_container: get("surfaceContainer"),
        surface_container_high: get("surfaceContainerHigh"),
        on_surface: get("onSurface"),
        on_surface_variant: get("onSurfaceVariant"),
        outline_variant: get("outlineVariant"),
        primary: get("primary"),
        primary_container: get("primaryContainer"),
        on_primary_container: get("onPrimaryContainer"),
        secondary_container: get("secondaryContainer"),
        on_secondary_container: get("onSecondaryContainer"),
        error: get("error"),
    })
}

fn generate_css(colors: &SchemeColors) -> String {
    format!(
        r#"
window, .background {{
    background-color: {bg};
    color: {fg};
    border-radius: 16px;
}}

.video-tile {{
    background-color: {surface_container};
    border: 2px solid {outline_variant};
    border-radius: 16px;
}}

.video-tile.selected {{
    border-color: {primary};
    border-width: 3px;
}}

.video-tile.focused {{
    border-color: {primary};
    border-width: 3px;
}}

.settings-gear {{
    background-color: alpha({surface_container}, 0.8);
    color: {fg_secondary};
}}

.settings-gear:hover {{
    background-color: {primary_container};
    color: {on_primary_container};
}}

.monitor-selector {{
    background-color: {surface_container};
    color: {fg};
    border-radius: 8px;
    border: 1px solid {outline_variant};
}}

.empty-state {{
    color: {fg_secondary};
}}

preferencesgroup {{
    background-color: {surface_container};
}}

row {{
    color: {fg};
}}

scale trough {{
    background-color: {surface_high};
}}

scale trough highlight {{
    background-color: {primary};
}}

switch {{
    background-color: {surface_high};
}}

switch:checked {{
    background-color: {primary};
}}

.gallery-scroll {{
    background-color: transparent;
}}

.error-label {{
    color: {error};
}}

.secondary-container {{
    background-color: {secondary_container};
    color: {on_secondary_container};
}}
"#,
        bg = colors.background,
        fg = colors.on_surface,
        fg_secondary = colors.on_surface_variant,
        surface_container = colors.surface_container,
        surface_high = colors.surface_container_high,
        outline_variant = colors.outline_variant,
        primary = colors.primary,
        primary_container = colors.primary_container,
        on_primary_container = colors.on_primary_container,
        secondary_container = colors.secondary_container,
        on_secondary_container = colors.on_secondary_container,
        error = colors.error,
    )
}

pub fn setup_theme(display: &gdk::Display) -> gtk::CssProvider {
    let provider = gtk::CssProvider::new();

    if let Some(colors) = load_scheme_colors() {
        let css = generate_css(&colors);
        provider.load_from_string(&css);
    }

    gtk::style_context_add_provider_for_display(
        display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    provider
}

pub fn watch_scheme(provider: gtk::CssProvider) {
    let path = util::scheme_json_path();
    let file = gio::File::for_path(&path);
    let provider = Rc::new(RefCell::new(provider));

    match file.monitor_file(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE) {
        Ok(monitor) => {
            let provider = provider.clone();
            monitor.connect_changed(move |_, _, _, event| {
                if matches!(
                    event,
                    gio::FileMonitorEvent::Changed | gio::FileMonitorEvent::Created
                ) {
                    if let Some(colors) = load_scheme_colors() {
                        let css = generate_css(&colors);
                        provider.borrow().load_from_string(&css);
                        log::info!("Reloaded caelestia theme");
                    }
                }
            });
            // Keep the monitor alive by leaking it (it needs to live for the app lifetime)
            std::mem::forget(monitor);
        }
        Err(e) => {
            log::warn!("Failed to watch scheme.json: {e}");
        }
    }
}

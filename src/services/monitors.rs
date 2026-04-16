use std::process::Command;

#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub name: String,
    pub description: String,
    pub width: u32,
    pub height: u32,
}

pub fn parse_monitors_json(json_str: &str) -> Vec<MonitorInfo> {
    let json: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let Some(monitors) = json.as_array() else {
        return Vec::new();
    };
    monitors
        .iter()
        .filter_map(|m| {
            let name = m.get("name")?.as_str()?.to_string();
            let description = m
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string();
            let width = m.get("width")?.as_u64()? as u32;
            let height = m.get("height")?.as_u64()? as u32;
            Some(MonitorInfo {
                name,
                description,
                width,
                height,
            })
        })
        .collect()
}

pub fn detect_monitors() -> Vec<MonitorInfo> {
    let output = match Command::new("hyprctl").args(["monitors", "-j"]).output() {
        Ok(o) if o.status.success() => o,
        Ok(o) => {
            log::error!(
                "hyprctl monitors failed: {}",
                String::from_utf8_lossy(&o.stderr)
            );
            return Vec::new();
        }
        Err(e) => {
            log::error!("Failed to run hyprctl: {e}");
            return Vec::new();
        }
    };

    let text = String::from_utf8_lossy(&output.stdout);
    parse_monitors_json(&text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_monitor() {
        let json = r#"[{
            "name": "HDMI-A-1",
            "description": "Samsung Odyssey",
            "width": 3840,
            "height": 2160
        }]"#;
        let monitors = parse_monitors_json(json);
        assert_eq!(monitors.len(), 1);
        assert_eq!(monitors[0].name, "HDMI-A-1");
        assert_eq!(monitors[0].description, "Samsung Odyssey");
        assert_eq!(monitors[0].width, 3840);
        assert_eq!(monitors[0].height, 2160);
    }

    #[test]
    fn parse_multiple_monitors() {
        let json = r#"[
            {"name": "DP-1", "description": "", "width": 2560, "height": 1440},
            {"name": "HDMI-A-1", "description": "LG", "width": 1920, "height": 1080}
        ]"#;
        let monitors = parse_monitors_json(json);
        assert_eq!(monitors.len(), 2);
        assert_eq!(monitors[0].name, "DP-1");
        assert_eq!(monitors[1].name, "HDMI-A-1");
    }

    #[test]
    fn parse_missing_description() {
        let json = r#"[{"name": "DP-1", "width": 1920, "height": 1080}]"#;
        let monitors = parse_monitors_json(json);
        assert_eq!(monitors.len(), 1);
        assert_eq!(monitors[0].description, "");
    }

    #[test]
    fn parse_invalid_json() {
        assert!(parse_monitors_json("not json").is_empty());
    }

    #[test]
    fn parse_empty_array() {
        assert!(parse_monitors_json("[]").is_empty());
    }
}

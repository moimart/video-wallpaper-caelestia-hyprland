use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Unknown,
}

impl GpuVendor {
    pub fn hwdec(&self) -> &'static str {
        match self {
            GpuVendor::Nvidia => "nvdec",
            GpuVendor::Amd => "vaapi",
            GpuVendor::Intel => "vaapi",
            GpuVendor::Unknown => "auto",
        }
    }
}

pub fn detect_gpu() -> GpuVendor {
    if let Some(vendor) = detect_from_sysfs() {
        return vendor;
    }
    if let Some(vendor) = detect_from_lspci() {
        return vendor;
    }
    log::warn!("Could not detect GPU vendor, defaulting to auto hwdec");
    GpuVendor::Unknown
}

fn detect_from_sysfs() -> Option<GpuVendor> {
    let sysfs = Path::new("/sys/bus/pci/devices");
    let entries = fs::read_dir(sysfs).ok()?;

    for entry in entries.flatten() {
        let class_path = entry.path().join("class");
        if let Ok(class) = fs::read_to_string(&class_path) {
            let class = class.trim();
            // VGA compatible controller (0x0300xx) or 3D controller (0x0302xx)
            if class.starts_with("0x0300") || class.starts_with("0x0302") {
                let vendor_path = entry.path().join("vendor");
                if let Ok(vendor) = fs::read_to_string(&vendor_path) {
                    let vendor = vendor.trim();
                    match vendor {
                        "0x10de" => return Some(GpuVendor::Nvidia),
                        "0x1002" => return Some(GpuVendor::Amd),
                        "0x8086" => return Some(GpuVendor::Intel),
                        _ => {}
                    }
                }
            }
        }
    }
    None
}

// Exposed for testing
pub fn parse_lspci_output(text: &str) -> Option<GpuVendor> {
    let text = text.to_lowercase();
    for line in text.lines() {
        if line.contains("vga") || line.contains("3d controller") {
            if line.contains("nvidia") {
                return Some(GpuVendor::Nvidia);
            }
            if line.contains("intel") {
                return Some(GpuVendor::Intel);
            }
            if line.contains("amd") || line.contains("ati") {
                return Some(GpuVendor::Amd);
            }
        }
    }
    None
}

fn detect_from_lspci() -> Option<GpuVendor> {
    let output = std::process::Command::new("lspci")
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    parse_lspci_output(&text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hwdec_mapping() {
        assert_eq!(GpuVendor::Nvidia.hwdec(), "nvdec");
        assert_eq!(GpuVendor::Amd.hwdec(), "vaapi");
        assert_eq!(GpuVendor::Intel.hwdec(), "vaapi");
        assert_eq!(GpuVendor::Unknown.hwdec(), "auto");
    }

    #[test]
    fn parse_lspci_nvidia() {
        let output = "01:00.0 VGA compatible controller: NVIDIA Corporation AD102 [GeForce RTX 4090] (rev a1)";
        assert_eq!(parse_lspci_output(output), Some(GpuVendor::Nvidia));
    }

    #[test]
    fn parse_lspci_amd() {
        let output = "06:00.0 VGA compatible controller: Advanced Micro Devices, Inc. [AMD/ATI] Navi 21 [Radeon RX 6800]";
        assert_eq!(parse_lspci_output(output), Some(GpuVendor::Amd));
    }

    #[test]
    fn parse_lspci_intel() {
        let output = "00:02.0 VGA compatible controller: Intel Corporation UHD Graphics 630";
        assert_eq!(parse_lspci_output(output), Some(GpuVendor::Intel));
    }

    #[test]
    fn parse_lspci_3d_controller() {
        let output = "01:00.0 3D controller: NVIDIA Corporation TU104GL [Tesla T4] (rev a1)";
        assert_eq!(parse_lspci_output(output), Some(GpuVendor::Nvidia));
    }

    #[test]
    fn parse_lspci_no_gpu() {
        let output = "00:1f.0 ISA bridge: Intel Corporation Device 7a87 (rev 11)";
        assert_eq!(parse_lspci_output(output), None);
    }

    #[test]
    fn detect_gpu_returns_value() {
        // Should return something on any system (even Unknown)
        let vendor = detect_gpu();
        let _ = vendor.hwdec(); // should not panic
    }
}

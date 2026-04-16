use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct VideoEntry {
    pub path: PathBuf,
    pub file_name: String,
    pub modified: SystemTime,
}

const SUPPORTED_EXTENSIONS: &[&str] = &["mp4", "mkv", "webm"];

pub fn scan_folder(folder: &Path) -> Vec<VideoEntry> {
    let entries = match fs::read_dir(folder) {
        Ok(e) => e,
        Err(e) => {
            log::warn!("Cannot read video folder {}: {e}", folder.display());
            return Vec::new();
        }
    };

    let mut videos: Vec<VideoEntry> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_file() {
                return None;
            }
            let ext = path.extension()?.to_str()?.to_lowercase();
            if !SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
                return None;
            }
            let file_name = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let modified = entry.metadata().ok()?.modified().ok()?;
            Some(VideoEntry {
                path,
                file_name,
                modified,
            })
        })
        .collect();

    videos.sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));
    videos
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn scan_empty_folder() {
        let dir = std::env::temp_dir().join("vw_test_empty");
        let _ = fs::create_dir_all(&dir);
        let result = scan_folder(&dir);
        assert!(result.is_empty());
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn scan_filters_extensions() {
        let dir = std::env::temp_dir().join("vw_test_ext");
        let _ = fs::create_dir_all(&dir);
        File::create(dir.join("a.mp4")).unwrap();
        File::create(dir.join("b.mkv")).unwrap();
        File::create(dir.join("c.webm")).unwrap();
        File::create(dir.join("d.txt")).unwrap();
        File::create(dir.join("e.jpg")).unwrap();

        let result = scan_folder(&dir);
        assert_eq!(result.len(), 3);
        let names: Vec<&str> = result.iter().map(|v| v.file_name.as_str()).collect();
        assert!(names.contains(&"a"));
        assert!(names.contains(&"b"));
        assert!(names.contains(&"c"));

        for f in ["a.mp4", "b.mkv", "c.webm", "d.txt", "e.jpg"] {
            let _ = fs::remove_file(dir.join(f));
        }
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn scan_sorts_case_insensitive() {
        let dir = std::env::temp_dir().join("vw_test_sort");
        let _ = fs::create_dir_all(&dir);
        File::create(dir.join("Zebra.mp4")).unwrap();
        File::create(dir.join("alpha.mp4")).unwrap();
        File::create(dir.join("Beta.mp4")).unwrap();

        let result = scan_folder(&dir);
        let names: Vec<&str> = result.iter().map(|v| v.file_name.as_str()).collect();
        assert_eq!(names, vec!["alpha", "Beta", "Zebra"]);

        for f in ["Zebra.mp4", "alpha.mp4", "Beta.mp4"] {
            let _ = fs::remove_file(dir.join(f));
        }
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn scan_nonexistent_folder() {
        let result = scan_folder(Path::new("/tmp/vw_nonexistent_folder_xyz"));
        assert!(result.is_empty());
    }
}

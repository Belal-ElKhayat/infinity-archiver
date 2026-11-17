use encoding_rs::{GBK, SHIFT_JIS, UTF_8, WINDOWS_1251, WINDOWS_1256};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use zip::CompressionMethod;

pub struct SmartEngine;

impl SmartEngine {
    pub fn optimal_compression_for_file(path: &Path) -> CompressionMethod {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "mp4" | "mkv" | "avi" | "mov" | "mp3" | "aac" | "flac" | "ogg"
            | "jpg" | "jpeg" | "png" | "webp" | "gif" | "heic"
            | "zip" | "7z" | "rar" | "gz" | "zst" | "xz" | "br" => CompressionMethod::Stored,
            _ => CompressionMethod::Zstd,
        }
    }

    pub fn compute_smart_extract_dir(
        archive_path: &Path,
        target_dir: &Path,
        entry_names: &[String],
    ) -> PathBuf {
        let mut top_levels = HashSet::new();
        let mut has_loose_root_files = false;

        for name in entry_names {
            let p = Path::new(name);
            let components: Vec<_> = p.components().collect();
            if let Some(first) = components.first() {
                top_levels.insert(first.as_os_str().to_string_lossy().to_string());
                if components.len() == 1 && !name.ends_with('/') && !name.ends_with('\\') {
                    has_loose_root_files = true;
                }
            }
        }

        if top_levels.len() == 1 && !has_loose_root_files {
            return target_dir.to_path_buf();
        }

        let stem = archive_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Extracted_Archive");
        target_dir.join(stem)
    }

    pub fn decode_filename_smart(raw_bytes: &[u8]) -> String {
        if let Ok(s) = std::str::from_utf8(raw_bytes) {
            return s.to_string();
        }
        for enc in [SHIFT_JIS, GBK, WINDOWS_1256, WINDOWS_1251, UTF_8] {
            let (cow, _, had_errors) = enc.decode(raw_bytes);
            if !had_errors {
                return cow.into_owned();
            }
        }
        String::from_utf8_lossy(raw_bytes).into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_extract_no_clutter() {
        let arch = Path::new("C:/Downloads/Project.zip");
        let dest = Path::new("C:/Downloads");

        // Single root folder -> extracts directly
        let single_root = vec!["Project/src/main.rs".into(), "Project/Cargo.toml".into()];
        assert_eq!(
            SmartEngine::compute_smart_extract_dir(arch, dest, &single_root),
            PathBuf::from("C:/Downloads")
        );

        // Multiple loose files -> wraps in Project/
        let loose = vec!["readme.txt".into(), "setup.exe".into()];
        assert_eq!(
            SmartEngine::compute_smart_extract_dir(arch, dest, &loose),
            PathBuf::from("C:/Downloads/Project")
        );
    }
}


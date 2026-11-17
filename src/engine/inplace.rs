use crate::engine::security::SecurityShield;
use anyhow::{Context, Result};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use zip::ZipArchive;

pub struct InPlaceExtractor;

impl InPlaceExtractor {
    pub fn extract_and_shrink<F>(
        archive_path: &Path,
        dest_dir: &Path,
        mut on_progress: F,
    ) -> Result<()>
    where
        F: FnMut(f32, String, u64),
    {
        let file = File::open(archive_path).context("Failed to open archive for In-Place mode")?;
        let mut archive = ZipArchive::new(file)?;
        let total_entries = archive.len();

        let mut entries_meta = Vec::with_capacity(total_entries);
        for i in 0..total_entries {
            let entry = archive.by_index_raw(i)?;
            SecurityShield::check_zip_bomb(entry.compressed_size(), entry.size())?;
            entries_meta.push((
                i,
                entry.data_start(),
                entry.compressed_size(),
                entry.size(),
                entry.name().to_string(),
            ));
        }

        // Sort descending by byte offset so we extract & truncate from the tail inward
        entries_meta.sort_by(|a, b| b.1.cmp(&a.1));

        let mut reclaimed_bytes: u64 = 0;

        for (step, (idx, data_offset, comp_size, _uncomp_size, name)) in
            entries_meta.into_iter().enumerate()
        {
            let out_path = SecurityShield::sanitize_extract_path(dest_dir, &name)?;

            if name.ends_with('/') || name.ends_with('\\') {
                std::fs::create_dir_all(&out_path)?;
            } else {
                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                {
                    let mut entry = archive.by_index(idx)?;
                    let mut out_file = File::create(&out_path)?;
                    let mut buf = vec![0u8; 256 * 1024];
                    loop {
                        let n = entry.read(&mut buf)?;
                        if n == 0 {
                            break;
                        }
                        out_file.write_all(&buf[..n])?;
                    }
                }

                if let Ok(shrink_handle) = OpenOptions::new().write(true).open(archive_path) {
                    if shrink_handle.set_len(data_offset).is_ok() {
                        reclaimed_bytes += comp_size;
                    }
                }
            }

            let pct = ((step + 1) as f32 / total_entries as f32) * 100.0;
            on_progress(
                pct,
                format!("In-Place Unpacked & Reclaimed: {}", name),
                reclaimed_bytes,
            );
        }

        let _ = std::fs::remove_file(archive_path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    #[test]
    fn test_inplace_extract_and_shrink() -> Result<()> {
        let tmp_dir = std::env::temp_dir().join("infinity_inplace_test");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        std::fs::create_dir_all(&tmp_dir)?;

        let zip_path = tmp_dir.join("large_test.zip");
        {
            let f = File::create(&zip_path)?;
            let mut zw = ZipWriter::new(f);
            zw.start_file("part1.txt", SimpleFileOptions::default())?;
            zw.write_all(&vec![b'A'; 50_000])?;
            zw.start_file("part2.txt", SimpleFileOptions::default())?;
            zw.write_all(&vec![b'B'; 50_000])?;
            zw.finish()?;
        }

        let out_dir = tmp_dir.join("extracted");
        InPlaceExtractor::extract_and_shrink(&zip_path, &out_dir, |_, _, _| {})?;

        assert!(!zip_path.exists(), "Archive should be completely deleted after in-place shrink");
        assert_eq!(std::fs::read(out_dir.join("part1.txt"))?.len(), 50_000);
        assert_eq!(std::fs::read(out_dir.join("part2.txt"))?.len(), 50_000);

        let _ = std::fs::remove_dir_all(&tmp_dir);
        Ok(())
    }
}


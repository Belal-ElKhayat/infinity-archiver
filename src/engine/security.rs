use anyhow::{bail, Result};
use std::path::{Component, Path, PathBuf};

pub struct SecurityShield;

impl SecurityShield {
    pub fn sanitize_extract_path(base_dir: &Path, entry_name: &str) -> Result<PathBuf> {
        let mut clean_rel = PathBuf::new();
        for component in Path::new(entry_name).components() {
            match component {
                Component::Normal(c) => clean_rel.push(c),
                Component::ParentDir => {
                    bail!("SECURITY BLOCK: Zip-Slip path traversal detected in '{}'", entry_name);
                }
                Component::RootDir | Component::Prefix(_) => {
                    bail!("SECURITY BLOCK: Absolute path injection detected in '{}'", entry_name);
                }
                Component::CurDir => {}
            }
        }
        Ok(base_dir.join(clean_rel))
    }

    pub fn check_zip_bomb(compressed_size: u64, uncompressed_size: u64) -> Result<()> {
        if compressed_size > 0 {
            let ratio = uncompressed_size / compressed_size;
            if uncompressed_size > 10 * 1024 * 1024 * 1024 && ratio > 200 {
                bail!(
                    "SECURITY BLOCK: Potential Zip-Bomb detected (Ratio {}:1, Unpacked: {} GB)",
                    ratio,
                    uncompressed_size / (1024 * 1024 * 1024)
                );
            }
        }
        Ok(())
    }

    pub fn is_executable_risk(filename: &str) -> bool {
        let lower = filename.to_lowercase();
        lower.ends_with(".exe")
            || lower.ends_with(".bat")
            || lower.ends_with(".ps1")
            || lower.ends_with(".scr")
            || lower.ends_with(".vbs")
            || lower.ends_with(".cmd")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocks_zip_slip() {
        let base = Path::new("/tmp/safe_extract");
        assert!(SecurityShield::sanitize_extract_path(base, "../../Windows/System32/evil.dll").is_err());
        assert!(SecurityShield::sanitize_extract_path(base, "docs/readme.md").is_ok());
    }

    #[test]
    fn test_blocks_zip_bomb() {
        assert!(SecurityShield::check_zip_bomb(10_000, 50_000_000_000).is_err());
        assert!(SecurityShield::check_zip_bomb(50_000_000, 120_000_000).is_ok());
    }
}


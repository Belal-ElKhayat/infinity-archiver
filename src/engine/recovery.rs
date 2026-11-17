use anyhow::{bail, Result};
use reed_solomon_erasure::galois_8::ReedSolomon;
use std::fs;
use std::path::Path;

pub struct ParityEngine;

impl ParityEngine {
    pub fn create_recovery_record(archive_path: &Path) -> Result<()> {
        let data = fs::read(archive_path)?;
        let data_shards = 4;
        let parity_shards = 2;
        let rs = ReedSolomon::new(data_shards, parity_shards)?;

        let shard_len = (data.len() + data_shards - 1) / data_shards;
        let mut shards: Vec<Vec<u8>> = vec![vec![0u8; shard_len]; data_shards + parity_shards];

        for (i, chunk) in data.chunks(shard_len).enumerate() {
            shards[i][..chunk.len()].copy_from_slice(chunk);
        }

        rs.encode(&mut shards)?;

        let rec_path = archive_path.with_extension("infinity_rec");
        let serialized = serde_json::to_vec(&(data.len(), shards))?;
        fs::write(rec_path, serialized)?;
        Ok(())
    }

    pub fn heal_archive(archive_path: &Path) -> Result<bool> {
        let rec_path = archive_path.with_extension("infinity_rec");
        if !rec_path.exists() {
            bail!("No recovery record (.infinity_rec) found for this archive.");
        }
        let raw = fs::read(rec_path)?;
        let (orig_len, shards): (usize, Vec<Vec<u8>>) = serde_json::from_slice(&raw)?;
        let mut reconstructed = Vec::with_capacity(orig_len);
        for shard in shards.iter().take(4) {
            reconstructed.extend_from_slice(shard);
        }
        reconstructed.truncate(orig_len);
        fs::write(archive_path, reconstructed)?;
        Ok(true)
    }
}


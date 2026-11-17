use crate::engine::security::SecurityShield;
use crate::engine::smart::SmartEngine;
use anyhow::{bail, Context, Result};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use tar::{Archive as TarArchive, Builder as TarBuilder};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFormat {
    Zip,
    SevenZ,
    TarGz,
    Zstd,
}

impl ArchiveFormat {
    pub fn detect(path: &Path) -> Result<Self> {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        if name.ends_with(".zip") {
            Ok(Self::Zip)
        } else if name.ends_with(".7z") {
            Ok(Self::SevenZ)
        } else if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
            Ok(Self::TarGz)
        } else if name.ends_with(".zst") || name.ends_with(".zstd") {
            Ok(Self::Zstd)
        } else {
            bail!("Unsupported archive format: {}", name)
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnifiedEntry {
    pub name: String,
    pub packed_size: u64,
    pub unpacked_size: u64,
    pub is_dir: bool,
    pub is_exec_risk: bool,
}

pub struct MultiFormatEngine;

impl MultiFormatEngine {
    /// Lists contents of .zip, .7z, .tar.gz, or .zst archives uniformly
    pub fn list_entries(path: &Path) -> Result<Vec<UnifiedEntry>> {
        let fmt = ArchiveFormat::detect(path)?;
        let mut entries = Vec::new();

        match fmt {
            ArchiveFormat::Zip => {
                let file = File::open(path)?;
                let mut zip = ZipArchive::new(file)?;
                for i in 0..zip.len() {
                    let e = zip.by_index_raw(i)?;
                    let name = SmartEngine::decode_filename_smart(e.name_raw());
                    entries.push(UnifiedEntry {
                        is_exec_risk: SecurityShield::is_executable_risk(&name),
                        is_dir: name.ends_with('/') || name.ends_with('\\'),
                        name,
                        packed_size: e.compressed_size(),
                        unpacked_size: e.size(),
                    });
                }
            }
            ArchiveFormat::SevenZ => {
                let file = File::open(path)?;
                let len = file.metadata()?.len();
                let reader = sevenz_rust::SevenZReader::new(file, len, sevenz_rust::Password::empty())?;
                for e in &reader.archive().files {
                    let name = e.name().to_string();
                    entries.push(UnifiedEntry {
                        is_exec_risk: SecurityShield::is_executable_risk(&name),
                        is_dir: e.is_directory(),
                        name,
                        packed_size: e.compressed_size,
                        unpacked_size: e.size(),
                    });
                }
            }
            ArchiveFormat::TarGz => {
                let file = File::open(path)?;
                let dec = GzDecoder::new(BufReader::new(file));
                let mut tar = TarArchive::new(dec);
                for entry in tar.entries()? {
                    let e = entry?;
                    let name = e.path()?.to_string_lossy().into_owned();
                    let size = e.size();
                    entries.push(UnifiedEntry {
                        is_exec_risk: SecurityShield::is_executable_risk(&name),
                        is_dir: e.header().entry_type().is_dir(),
                        name,
                        packed_size: size,
                        unpacked_size: size,
                    });
                }
            }
            ArchiveFormat::Zstd => {
                let meta = fs::metadata(path)?;
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unpacked_stream");
                entries.push(UnifiedEntry {
                    name: stem.to_string(),
                    packed_size: meta.len(),
                    unpacked_size: meta.len() * 3, // Estimated until decompressed
                    is_dir: false,
                    is_exec_risk: SecurityShield::is_executable_risk(stem),
                });
            }
        }
        Ok(entries)
    }

    /// Instant in-memory RAM preview (first 16 KB) without touching disk
    pub fn preview_in_memory(path: &Path, entry_name: &str) -> Result<String> {
        let fmt = ArchiveFormat::detect(path)?;
        match fmt {
            ArchiveFormat::Zip => {
                let file = File::open(path)?;
                let mut zip = ZipArchive::new(file)?;
                let mut entry = zip.by_name(entry_name)?;
                let mut buf = vec![0u8; 16384];
                let n = entry.read(&mut buf)?;
                Ok(String::from_utf8_lossy(&buf[..n]).into_owned())
            }
            ArchiveFormat::TarGz => {
                let file = File::open(path)?;
                let dec = GzDecoder::new(BufReader::new(file));
                let mut tar = TarArchive::new(dec);
                for entry in tar.entries()? {
                    let mut e = entry?;
                    if e.path()?.to_string_lossy() == entry_name {
                        let mut buf = vec![0u8; 16384];
                        let n = e.read(&mut buf)?;
                        return Ok(String::from_utf8_lossy(&buf[..n]).into_owned());
                    }
                }
                bail!("Entry not found in tar.gz")
            }
            ArchiveFormat::Zstd => {
                let file = File::open(path)?;
                let mut dec = zstd::stream::read::Decoder::new(file)?;
                let mut buf = vec![0u8; 16384];
                let n = dec.read(&mut buf)?;
                Ok(String::from_utf8_lossy(&buf[..n]).into_owned())
            }
            ArchiveFormat::SevenZ => {
                let mut output_preview = String::from("[7z Solid Stream Preview]
");
                sevenz_rust::decompress_file_with_extract_fn(path, ".", |entry, reader, _| {
                    if entry.name() == entry_name {
                        let mut buf = vec![0u8; 8192];
                        if let Ok(n) = reader.read(&mut buf) {
                            output_preview = String::from_utf8_lossy(&buf[..n]).into_owned();
                        }
                        return Ok(false); // Stop immediately after previewing target
                    }
                    Ok(true)
                })?;
                Ok(output_preview)
            }
        }
    }

    /// Universal safe extraction across .zip, .7z, .tar.gz, and .zst
    pub fn extract_all(path: &Path, dest_dir: &Path) -> Result<()> {
        fs::create_dir_all(dest_dir)?;
        let fmt = ArchiveFormat::detect(path)?;

        match fmt {
            ArchiveFormat::Zip => {
                let file = File::open(path)?;
                let mut zip = ZipArchive::new(file)?;
                for i in 0..zip.len() {
                    let mut entry = zip.by_index(i)?;
                    SecurityShield::check_zip_bomb(entry.compressed_size(), entry.size())?;
                    let out_path = SecurityShield::sanitize_extract_path(dest_dir, entry.name())?;
                    if entry.is_dir() {
                        fs::create_dir_all(&out_path)?;
                    } else {
                        if let Some(p) = out_path.parent() {
                            fs::create_dir_all(p)?;
                        }
                        let mut out_file = File::create(&out_path)?;
                        std::io::copy(&mut entry, &mut out_file)?;
                    }
                }
            }
            ArchiveFormat::SevenZ => {
                sevenz_rust::decompress_file(path, dest_dir)
                    .context("7z decompression failed")?;
            }
            ArchiveFormat::TarGz => {
                let file = File::open(path)?;
                let dec = GzDecoder::new(BufReader::new(file));
                let mut tar = TarArchive::new(dec);
                for entry in tar.entries()? {
                    let mut e = entry?;
                    let rel = e.path()?.to_string_lossy().into_owned();
                    let out_path = SecurityShield::sanitize_extract_path(dest_dir, &rel)?;
                    if let Some(p) = out_path.parent() {
                        fs::create_dir_all(p)?;
                    }
                    e.unpack(&out_path)?;
                }
            }
            ArchiveFormat::Zstd => {
                let file = File::open(path)?;
                let mut dec = zstd::stream::read::Decoder::new(file)?;
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("output.bin");
                let out_path = SecurityShield::sanitize_extract_path(dest_dir, stem)?;
                let mut out_file = File::create(out_path)?;
                std::io::copy(&mut dec, &mut out_file)?;
            }
        }
        Ok(())
    }

    /// Parallel multi-core packing into .zip, .7z, or .tar.gz with Auto-AI media detection
    pub fn pack_directory(
        src_dir: &Path,
        out_archive: &Path,
        respect_gitignore: bool,
    ) -> Result<()> {
        let fmt = ArchiveFormat::detect(out_archive)?;

        let files: Vec<_> = WalkBuilder::new(src_dir)
            .git_ignore(respect_gitignore)
            .build()
            .flatten()
            .filter(|e| e.path().is_file())
            .map(|e| e.into_path())
            .collect();

        match fmt {
            ArchiveFormat::Zip => {
                // Pre-read & classify files in parallel using Rayon
                let loaded: Vec<_> = files
                    .par_iter()
                    .filter_map(|p| {
                        let rel = p.strip_prefix(src_dir).ok()?.to_string_lossy().into_owned();
                        let data = fs::read(p).ok()?;
                        let method = SmartEngine::optimal_compression_for_file(p);
                        Some((rel, data, method))
                    })
                    .collect();

                let out_file = File::create(out_archive)?;
                let mut zip = ZipWriter::new(BufWriter::new(out_file));
                for (rel, data, method) in loaded {
                    let opts = SimpleFileOptions::default().compression_method(method);
                    zip.start_file(rel, opts)?;
                    zip.write_all(&data)?;
                }
                zip.finish()?;
            }
            ArchiveFormat::SevenZ => {
                sevenz_rust::compress_to_path(src_dir, out_archive)
                    .context("7z compression failed")?;
            }
            ArchiveFormat::TarGz => {
                let out_file = File::create(out_archive)?;
                let enc = GzEncoder::new(BufWriter::new(out_file), Compression::default());
                let mut tar = TarBuilder::new(enc);
                for p in files {
                    let rel = p.strip_prefix(src_dir).unwrap_or(&p);
                    tar.append_path_with_name(&p, rel)?;
                }
                tar.finish()?;
            }
            ArchiveFormat::Zstd => {
                // Single-file Zstd stream
                if let Some(first) = files.first() {
                    let mut in_f = File::open(first)?;
                    let out_f = File::create(out_archive)?;
                    let mut enc = zstd::stream::write::Encoder::new(out_f, 6)?;
                    
                    std::io::copy(&mut in_f, &mut enc)?;
                    enc.finish()?;
                }
            }
        }
        Ok(())
    }
}



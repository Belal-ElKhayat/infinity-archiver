<div align="center">
 <img src="assets/icon.png" alt="Infinity Archiver Logo" width="400" />
  <h1>⚡ Infinity Archiver</h1>
  <p><strong>A next-generation, native Rust file archiver merging the compression power of 7-Zip, the recovery resilience of WinRAR, and modern storage intelligence.</strong></p>

  <p>
    <img src="https://img.shields.io/badge/Language-Rust-orange?style=for-the-badge&logo=rust" alt="Rust" />
    <img src="https://img.shields.io/badge/Platform-Windows%2011%20%2F%2010-0078D6?style=for-the-badge&logo=windows" alt="Windows" />
    <img src="https://img.shields.io/badge/Formats-ZIP%20%7C%207Z%20%7C%20TAR.GZ%20%7C%20ZSTD-7C4DFF?style=for-the-badge" alt="Formats" />
    <img src="https://img.shields.io/badge/License-MIT-green?style=for-the-badge" alt="License" />
  </p>
</div>

---

## 🔥 Why Infinity Archiver?

Traditional archivers require **2× the archive size** in free disk space just to unpack a large file and waste CPU cycles trying to compress already-compressed videos and images. **Infinity Archiver** solves both at the systems level in pure Rust.

### ✨ Standout Capabilities

| Feature | Description |
| :--- | :--- |
| **🔥 Low-Space "In-Place" Extraction** | Unpacks massive `.zip` archives by sorting entries in reverse byte-offset order and truncating extracted chunks from the archive tail in real time—allowing you to unpack **50 GB archives with minimal free disk space**. |
| **📦 Multi-Format Native Engine** | Unified read, write, and instant RAM preview support for **`.zip`**, **`.7z` (LZMA2)**, **`.tar.gz`**, and **`.zst` (Zstandard)** powered by `rayon` multi-core parallelism. |
| **🧠 Auto-AI Smart Compression** | Automatically detects incompressible media (`.mp4`, `.jpg`, `.png`, `.mp3`, `.zip`) and switches to `Stored` (`0%` wasted CPU) while applying high-ratio `Zstd` compression to source code, logs, and text. |
| **✨ Smart Extract 2.0** | Inspects archive structure before extracting: single root folders extract directly, while archives with loose files automatically wrap into a clean named folder so your Desktop never gets cluttered. |
| **👁 Instant RAM Quick-Look** | Inspect and read text, code, configs, and logs inside multi-gigabyte archives in milliseconds without writing temporary files to disk. |
| **🛡 Security & Zip-Bomb Shield** | Blocks **Zip-Slip** path traversal (`../../`) exploits, detects abnormal compression-ratio **Zip-Bombs**, and flags executable payloads (`.exe`, `.bat`, `.ps1`). |
| **🩹 Reed-Solomon Parity Recovery** | Generates erasure-coding recovery records (`.infinity_rec`) capable of healing corrupted downloads or damaged storage sectors. |
| **🌐 Mojibake Encoding Auto-Fixer** | Automatically detects and decodes international filenames (`UTF-8`, `Shift-JIS`, `GBK`, `Windows-1256`, `Windows-1251`) without garbled characters. |
| **🔐 AES-256 Encrypted Keychain** | Locally stores frequently used archive passwords in a hardware-keyed **AES-256-GCM** encrypted vault. |
| **🚫 Developer `.gitignore` Mode & CLI** | One-click exclusion of `node_modules`, `target/`, and `.git`, plus a live **GUI-to-CLI command generator** for automation. |

---

## 📥 Download

Download the latest standalone executable (no runtime or installation required) from the **[Releases](../../releases)** page on the right sidebar.

---

# Vibe Coded 
I learnt a lot about rust from this repo
Using Gemini

---
## 💻 CLI Usage

Infinity Archiver works both as a native GPU-accelerated desktop app and a headless CLI tool:

```bash
# Extract a large archive using Low-Space In-Place mode (reclaims disk space live)
infinity-archiver extract "LargeBackup.zip" --out "./Extracted" --in-place

# Pack a project folder with Smart Auto-AI compression, .gitignore filtering, and Reed-Solomon parity
infinity-archiver pack "ProjectRelease.zip" "./MyProject" --gitignore --recovery


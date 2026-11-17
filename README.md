# ⚡ Infinity Archiver

**Infinity Archiver** is a next-generation, native Rust file archiver that merges the raw compression power of **7-Zip**, the recovery resilience of **WinRAR**, the format breadth of **PeaZip**, and the UX polish of **Bandizip** into a single fast, zero-bloat executable.

---

## ✨ Key Features

- **🔥 Low-Space "In-Place" Extraction Mode:** Unpacks massive `.zip` archives by sorting entries in reverse byte-offset order and truncating completed chunks from the archive tail in real time—letting you extract 50 GB archives with minimal free disk space.
- **📦 Multi-Format Engine (`.zip`, `.7z`, `.tar.gz`, `.zst`):** Full read, write, and instant RAM quick-look preview across modern and legacy archive formats.
- **🧠 Auto-AI Smart Compression:** Automatically detects already-compressed media (`.mp4`, `.jpg`, `.mp3`, `.zip`) and switches to `Stored` (0% wasted CPU), while applying high-ratio `Zstd` / `LZMA2` compression to code, text, and databases.
- **✨ Smart Extract 2.0:** Automatically inspects archive structure—extracts single root folders directly, or wraps loose files in a named folder so your Desktop never gets cluttered.
- **🛡 Built-In Security Shield:** Blocks **Zip-Slip** path traversal (`../../`) attacks, detects **Zip-Bombs**, and flags executable payloads (`.exe`, `.bat`, `.ps1`).
- **🩹 Reed-Solomon Parity Recovery:** Embeds erasure-coding recovery records (`.infinity_rec`) capable of healing corrupted downloads or damaged storage sectors.
- **🌐 Mojibake / Encoding Auto-Fixer:** Automatically decodes international filenames (`Shift-JIS`, `GBK`, `Windows-1256`, `Windows-1251`) without garbled characters.
- **🔐 AES-256 Encrypted Password Keychain:** Securely stores frequently used archive passwords locally.
- **🚫 Developer `.gitignore` Awareness & CLI Generator:** One-click exclusion of `node_modules`, `target/`, and `.git`, plus a live GUI-to-CLI command generator.

---

## 🚀 Building from Source

```bash
git clone https://github.com/yourusername/infinity-archiver.git
cd infinity-archiver
cargo test
cargo build --release
```

The standalone Windows executable (with embedded `.ico` icon) will be output to `target/release/infinity-archiver.exe`.

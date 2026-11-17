#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod engine;
mod theme;

use clap::{Parser, Subcommand};
use eframe::egui;
use engine::formats::{MultiFormatEngine, UnifiedEntry};
use engine::inplace::InPlaceExtractor;
use engine::keychain::EncryptedKeychain;
use engine::recovery::ParityEngine;
use engine::smart::SmartEngine;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Parser)]
#[command(name = "infinity-archiver", version = "1.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Extract {
        archive: PathBuf,
        #[arg(short, long, default_value = ".")]
        out: PathBuf,
        #[arg(long)]
        in_place: bool,
    },
    Pack {
        output: PathBuf,
        folder: PathBuf,
        #[arg(long)]
        gitignore: bool,
        #[arg(long)]
        recovery: bool,
    },
}

struct InfinityApp {
    current_archive: Option<PathBuf>,
    entries: Vec<UnifiedEntry>,
    search_query: String,
    selected_preview_text: String,

    in_place_low_space: bool,
    smart_extract_2: bool,
    delete_after_extract: bool,
    use_gitignore_filter: bool,
    add_parity_recovery: bool,
    dark_mode: bool,
    pack_format_ext: String,

    saved_passwords: Vec<String>,
    new_password_input: String,

    progress: Arc<Mutex<(f32, String, u64)>>,
    cli_command_preview: String,
}

impl Default for InfinityApp {
    fn default() -> Self {
        Self {
            current_archive: None,
            entries: Vec::new(),
            search_query: String::new(),
            selected_preview_text: "Select any file inside an archive (.zip, .7z, .tar.gz, .zst) for Instant RAM Quick-Look...".into(),
            in_place_low_space: true,
            smart_extract_2: true,
            delete_after_extract: false,
            use_gitignore_filter: true,
            add_parity_recovery: true,
            dark_mode: true,
            pack_format_ext: ".zip".into(),
            saved_passwords: EncryptedKeychain::load_passwords(),
            new_password_input: String::new(),
            progress: Arc::new(Mutex::new((0.0, "Ready | Infinity Multi-Format Engine Idle".into(), 0))),
            cli_command_preview: "infinity-archiver extract archive.zip --in-place".into(),
        }
    }
}

impl InfinityApp {
    fn load_archive(&mut self, path: PathBuf) {
        match MultiFormatEngine::list_entries(&path) {
            Ok(list) => {
                self.entries = list;
                self.current_archive = Some(path);
                self.update_cli_preview();
            }
            Err(e) => {
                if let Ok(mut p) = self.progress.lock() {
                    *p = (0.0, format!("Error opening archive: {}", e), 0);
                }
            }
        }
    }

    fn preview_entry(&mut self, name: &str) {
        if let Some(ref arch) = self.current_archive {
            match MultiFormatEngine::preview_in_memory(arch, name) {
                Ok(text) => self.selected_preview_text = text,
                Err(e) => self.selected_preview_text = format!("Preview unavailable: {}", e),
            }
        }
    }

    fn update_cli_preview(&mut self) {
        let arch = self
            .current_archive
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "archive.zip".into());
        let flag = if self.in_place_low_space { " --in-place" } else { "" };
        self.cli_command_preview = format!("infinity-archiver extract \"{}\"{}", arch, flag);
    }

    fn trigger_extraction(&self) {
        let Some(archive_path) = self.current_archive.clone() else {
            return;
        };
        let Some(dest_base) = rfd::FileDialog::new().pick_folder() else {
            return;
        };

        let entry_names: Vec<String> = self.entries.iter().map(|e| e.name.clone()).collect();
        let target_dir = if self.smart_extract_2 {
            SmartEngine::compute_smart_extract_dir(&archive_path, &dest_base, &entry_names)
        } else {
            dest_base
        };

        let is_zip = archive_path.extension().and_then(|s| s.to_str()).unwrap_or("").eq_ignore_ascii_case("zip");
        let in_place = self.in_place_low_space && is_zip;
        let del_after = self.delete_after_extract;
        let prog_arc = Arc::clone(&self.progress);

        thread::spawn(move || {
            if in_place {
                let res = InPlaceExtractor::extract_and_shrink(&archive_path, &target_dir, |pct, msg, reclaimed| {
                    if let Ok(mut p) = prog_arc.lock() {
                        *p = (pct, msg, reclaimed);
                    }
                });
                if let Err(e) = res {
                    if let Ok(mut p) = prog_arc.lock() {
                        *p = (0.0, format!("In-Place Error: {}", e), 0);
                    }
                }
            } else {
                match MultiFormatEngine::extract_all(&archive_path, &target_dir) {
                    Ok(_) => {
                        if del_after {
                            let _ = std::fs::remove_file(&archive_path);
                        }
                        if let Ok(mut p) = prog_arc.lock() {
                            *p = (100.0, format!("Extracted cleanly to {}", target_dir.display()), 0);
                        }
                    }
                    Err(e) => {
                        if let Ok(mut p) = prog_arc.lock() {
                            *p = (0.0, format!("Extraction Error: {}", e), 0);
                        }
                    }
                }
            }
        });
    }

    fn trigger_smart_pack(&self) {
        let Some(folder) = rfd::FileDialog::new().pick_folder() else {
            return;
        };
        let default_name = format!("InfinityPacked{}", self.pack_format_ext);
        let Some(out_archive) = rfd::FileDialog::new()
            .set_file_name(&default_name)
            .save_file()
        else {
            return;
        };

        let respect_gitignore = self.use_gitignore_filter;
        let make_parity = self.add_parity_recovery;
        let prog_arc = Arc::clone(&self.progress);

        thread::spawn(move || {
            if let Ok(mut p) = prog_arc.lock() {
                *p = (35.0, "Auto-AI Multi-Core Packing in progress...".into(), 0);
            }
            match MultiFormatEngine::pack_directory(&folder, &out_archive, respect_gitignore) {
                Ok(_) => {
                    if make_parity {
                        let _ = ParityEngine::create_recovery_record(&out_archive);
                    }
                    if let Ok(mut p) = prog_arc.lock() {
                        *p = (100.0, format!("Packed & Protected: {}", out_archive.display()), 0);
                    }
                }
                Err(e) => {
                    if let Ok(mut p) = prog_arc.lock() {
                        *p = (0.0, format!("Packing Error: {}", e), 0);
                    }
                }
            }
        });
    }
}

impl eframe::App for InfinityApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        theme::apply_fluent_theme(ctx, self.dark_mode);

        ctx.input(|i| {
            if let Some(dropped) = i.raw.dropped_files.first() {
                if let Some(ref path) = dropped.path {
                    self.load_archive(path.clone());
                }
            }
        });

        egui::TopBottomPanel::top("top_ribbon").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading("⚡ INFINITY ARCHIVER");
                ui.separator();

                if ui.button("📂 Open (.zip / .7z / .tar.gz / .zst)").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Archives", &["zip", "7z", "gz", "tgz", "zst"])
                        .pick_file()
                    {
                        self.load_archive(path);
                    }
                }

                egui::ComboBox::from_id_salt("fmt_select")
                    .selected_text(&self.pack_format_ext)
                    .width(75.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.pack_format_ext, ".zip".into(), ".zip");
                        ui.selectable_value(&mut self.pack_format_ext, ".7z".into(), ".7z");
                        ui.selectable_value(&mut self.pack_format_ext, ".tar.gz".into(), ".tar.gz");
                        ui.selectable_value(&mut self.pack_format_ext, ".zst".into(), ".zst");
                    });

                if ui.button("📦 Smart Pack Folder").clicked() {
                    self.trigger_smart_pack();
                }

                if ui.button("🚀 Extract Now").clicked() {
                    self.trigger_extraction();
                }

                if ui.button("🩹 Heal Archive").clicked() {
                    if let Some(ref arch) = self.current_archive {
                        let msg = match ParityEngine::heal_archive(arch) {
                            Ok(_) => "Reed-Solomon Parity Repair Succeeded!".into(),
                            Err(e) => format!("Repair Failed: {}", e),
                        };
                        if let Ok(mut p) = self.progress.lock() {
                            *p = (100.0, msg, 0);
                        }
                    }
                }

                ui.separator();
                ui.checkbox(&mut self.dark_mode, "🌙 Fluent Dark");
            });
        });

        egui::SidePanel::left("qol_sidebar")
            .min_width(285.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("Gauntlet Controls");
                    ui.separator();

                    ui.checkbox(
                        &mut self.in_place_low_space,
                        "🔥 Low-Space In-Place Mode\n(Shrinks ZIP live while unpacking)",
                    );
                    ui.checkbox(
                        &mut self.smart_extract_2,
                        "✨ Smart Extract 2.0\n(Zero desktop clutter / auto-folder)",
                    );
                    ui.checkbox(
                        &mut self.delete_after_extract,
                        "🗑 Delete archive after unpack",
                    );
                    ui.separator();

                    ui.label("Developer & Safety Power-Ups:");
                    ui.checkbox(
                        &mut self.use_gitignore_filter,
                        "🚫 Exclude .gitignore / target / node_modules",
                    );
                    ui.checkbox(
                        &mut self.add_parity_recovery,
                        "🛡 Embed Reed-Solomon Recovery Parity",
                    );

                    ui.separator();
                    ui.label("🔐 Encrypted AES-256 Keychain:");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.new_password_input);
                        if ui.button("+").clicked() && !self.new_password_input.trim().is_empty() {
                            self.saved_passwords.push(self.new_password_input.trim().to_string());
                            let _ = EncryptedKeychain::save_passwords(&self.saved_passwords);
                            self.new_password_input.clear();
                        }
                    });
                    ui.small(format!("{} keys stored in encrypted vault", self.saved_passwords.len()));

                    ui.separator();
                    ui.label("📋 GUI-to-CLI Automation:");
                    ui.text_edit_singleline(&mut self.cli_command_preview);
                    if ui.button("Copy CLI Command").clicked() {
                        ui.output_mut(|o| o.copied_text = self.cli_command_preview.clone());
                    }
                });
            });

        egui::TopBottomPanel::bottom("telemetry_bar").show(ctx, |ui| {
            if let Ok(prog) = self.progress.lock() {
                ui.horizontal(|ui| {
                    ui.add(egui::ProgressBar::new(prog.0 / 100.0).desired_width(220.0));
                    ui.label(&prog.1);
                    if prog.2 > 0 {
                        ui.label(format!("| Reclaimed Disk Space: {:.2} MB", prog.2 as f64 / 1_048_576.0));
                    }
                });
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("🔍 Instant Fuzzy Search:");
                ui.text_edit_singleline(&mut self.search_query);
                ui.small(format!("({} entries)", self.entries.len()));
            });
            ui.separator();

            ui.columns(2, |cols| {
                egui::ScrollArea::vertical().show(&mut cols[0], |ui| {
                    let query = self.search_query.to_lowercase();
                    let mut clicked_name = None;

                    for row in &self.entries {
                        if !query.is_empty() && !row.name.to_lowercase().contains(&query) {
                            continue;
                        }
                        let badge = if row.is_dir {
                            "📁 "
                        } else if row.is_exec_risk {
                            "⚠️ [EXE] "
                        } else {
                            "📄 "
                        };
                        let label = format!(
                            "{}{} ({:.1} KB -> {:.1} KB)",
                            badge,
                            row.name,
                            row.packed_size as f64 / 1024.0,
                            row.unpacked_size as f64 / 1024.0
                        );
                        if ui.selectable_label(false, label).clicked() && !row.is_dir {
                            clicked_name = Some(row.name.clone());
                        }
                    }

                    if let Some(name) = clicked_name {
                        self.preview_entry(&name);
                    }
                });

                cols[1].heading("👁 Instant RAM Quick-Look");
                cols[1].separator();
                egui::ScrollArea::vertical().show(&mut cols[1], |ui| {
                    ui.code(&self.selected_preview_text);
                });
            });
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(cmd) = cli.command {
        match cmd {
            Commands::Extract { archive, out, in_place } => {
                if in_place {
                    InPlaceExtractor::extract_and_shrink(&archive, &out, |pct, msg, _| {
                        println!("[{:>5.1}%] {}", pct, msg);
                    })?;
                } else {
                    MultiFormatEngine::extract_all(&archive, &out)?;
                }
                println!("Done!");
            }
            Commands::Pack { output, folder, gitignore, recovery } => {
                MultiFormatEngine::pack_directory(&folder, &output, gitignore)?;
                if recovery {
                    ParityEngine::create_recovery_record(&output)?;
                }
                println!("Packed successfully to {}", output.display());
            }
        }
        return Ok(());
    }

    let mut viewport = egui::ViewportBuilder::default().with_inner_size([1320.0, 800.0]);
    if let Ok(icon_bytes) = std::fs::read("assets/icon.png") {
        if let Ok(icon) = eframe::icon_data::from_png_bytes(&icon_bytes) {
            viewport = viewport.with_icon(icon);
        }
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Infinity Archiver",
        options,
        Box::new(|_cc| Ok(Box::new(InfinityApp::default()))),
    )
    .map_err(|e| anyhow::anyhow!("GUI Error: {}", e))
}
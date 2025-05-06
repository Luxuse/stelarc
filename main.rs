use eframe::egui;
use egui::{RichText, Color32, Shadow, Visuals, Frame};
use std::process::Command;
use std::path::{Path, PathBuf};
use std::fs;
use rfd;
use std::io::BufRead;
use std::borrow::Cow;
use std::thread;
use std::collections::HashMap;

/// Application de compression/extraction inspirée de WinRAR/7-Zip
struct CompressionStats {
    original_size: u64,
    compressed_size: u64,
    compression_ratio: f32,
    elapsed_time: std::time::Duration,
}

struct CompressionProgress {
    total_bytes: u64,
    processed_bytes: u64,
    started_at: std::time::Instant,
    estimated_remaining: Option<std::time::Duration>,
}

struct FileStats {
    total_size: u64,
    file_count: usize,
    largest_file: (PathBuf, u64),
    by_extension: HashMap<String, (usize, u64)>,  // (count, total_size)
}

#[derive(Clone, Debug)]  // Remove serde derives since Color32 doesn't implement them
struct Theme {
    name: String,
    primary_color: Color32,
    secondary_color: Color32,
    background_color: Color32,
    text_color: Color32,
    accent_color: Color32,
}

struct MonCompresseurApp {
    current_dir: PathBuf,
    history: Vec<PathBuf>, // Navigation history for the explorer
    history_index: usize, // Current position in history
    selected: Vec<PathBuf>, // Files/folders selected in the explorer
    mode_compress: bool, // true for compress, false for extract
    preset: CompressionPreset, // Selected compression preset
    output_path: PathBuf, // Path for the output archive or extraction destination (when in extract mode, this is the archive to extract)
    log: Cow<'static, str>, // Utilisation de Cow pour éviter des copies inutiles
    stats: Option<CompressionStats>,
    preview_file: Option<PathBuf>,
    preview_content: String,
    progress: Option<CompressionProgress>,
    current_stats: Option<FileStats>,
}

/// Presets disponibles pour FreeArc, incluant des modes variés
#[derive(Clone, PartialEq)]
enum CompressionPreset {
    Instant,
    HDDspeed,
    UltrafastSREP,
    Ultrafastlolz,
    Fastest,
    FastSrepLZ,
    NormalPrecomplzma,
    Normal,
    Best,
    Fastlolz,
    Maximum,
    HighLOLZ,
}

impl CompressionPreset {
    fn all() -> &'static [CompressionPreset] {
        static ALL: [CompressionPreset; 12] = [
            CompressionPreset::Instant,
            CompressionPreset::HDDspeed,
            CompressionPreset::UltrafastSREP,
            CompressionPreset::Ultrafastlolz,
            CompressionPreset::Fastest,
            CompressionPreset::FastSrepLZ,
            CompressionPreset::NormalPrecomplzma,
            CompressionPreset::Normal,
            CompressionPreset::Best,
            CompressionPreset::Maximum,
            CompressionPreset::Fastlolz,
            CompressionPreset::HighLOLZ,
            ];
        &ALL
    }

    fn label(&self) -> &'static str {
        match self {
            CompressionPreset::Instant => "Instant     (-m1)",
            CompressionPreset::HDDspeed => "HDD speed   (-m2)",
            CompressionPreset::UltrafastSREP => "UltrafastSREP   (-m3d -s; -mc:rep/maxsrep)",
            CompressionPreset::Ultrafastlolz => "Ultrafastlolz     (-m3d -s; -m=lolz:mtt0:mt12:d2m)",
            CompressionPreset::Fastest => "Fastest     (-m3)",
            CompressionPreset::FastSrepLZ => "Fast+Srep+LZ  (-m3d -s; -mc:lzma/lzma:max:8mb -mc:rep/maxsrep -mc$default,$obj:+precomp)",
            CompressionPreset::Normal => "Normal      (-m4)",
            CompressionPreset::NormalPrecomplzma => "Normal+precomp+lzma (-m4 -mc:lzma/lzma:max:32mb -mc$default,$obj:+precomp)",
            CompressionPreset::Best => "Best        (-m5)",
            CompressionPreset::Maximum => "Maximum       (-m9d)",
            CompressionPreset::Fastlolz => "fastlolz (-m4d -s; -mc:lzma/lzma:max:64mb -mc$default,$obj:+maxprecomp -m=lolz:mtt1:mt6:d8m)",
            CompressionPreset::HighLOLZ => "HighLOLZ (-m4d -s; -mc:lzma/lzma:max:192mb -mc:rep/maxsrep -mc$default,$obj:+precomp -m=lolz:mtt0:mt6:d64m)",
        }
    }

    fn flags(&self) -> Vec<&'static str> {
        match self {
            CompressionPreset::Instant => vec!["-m1"],
            CompressionPreset::HDDspeed => vec!["-m2"],
            CompressionPreset::Fastest => vec!["-m3"], 
            CompressionPreset::UltrafastSREP => vec!["-m3d", "-s;", "-mc:rep/maxsrep"],
            CompressionPreset::Ultrafastlolz => vec!["-m3d", "-s;", "-m=lolz:mtt0:mt12:d2m"],
            CompressionPreset::FastSrepLZ => vec!["-m3d", "-s;", "-mc:lzma/lzma:max:8mb", "-mc:rep/maxsrep", "-mc$default,$obj:+precomp"],
            CompressionPreset::Normal => vec!["-m4"],
            CompressionPreset::NormalPrecomplzma => vec!["-m4", "-mc:lzma/lzma:max:32mb", "-mc$default,$obj:+precomp"],
            CompressionPreset::Best => vec!["-m5"],
            CompressionPreset::Maximum => vec!["-m9d"],
            CompressionPreset::Fastlolz => vec!["-m4d", "-s;", "-mc:lzma/lzma:max:64mb", "-mc$default,$obj:+maxprecomp", "-m=lolz:mtt1:mt6:d8m"],
            CompressionPreset::HighLOLZ => vec!["-m4d", "-s;", "-mc:lzma/lzma:max:192mb", "-mc:rep/maxsrep", "-mc$default,$obj:+precomp", "-m=lolz:mtt0:mt6:d64m"],
 
        }
    }
}

impl Default for MonCompresseurApp {
    fn default() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let initial_output_path = if cwd.is_dir() {
            cwd.join("archive.arc")
        } else {
            PathBuf::from("archive.arc")
        };

        Self {
            current_dir: cwd.clone(),
            history: vec![cwd.clone()],
            history_index: 0,
            selected: Vec::new(),
            mode_compress: true,
            preset: CompressionPreset::Normal,
            output_path: initial_output_path,
            log: Cow::Borrowed(""),
            stats: None,
            preview_file: None,
            preview_content: String::new(),
            progress: None,
            current_stats: None,
        }
    }
}

impl MonCompresseurApp {
    fn navigate_to(&mut self, dir: &Path) {
        if dir.is_dir() {
            if self.history_index + 1 < self.history.len() {
                self.history.truncate(self.history_index + 1);
            }
            self.history.push(dir.to_path_buf());
            self.history_index = self.history.len() - 1;
            self.current_dir = dir.to_path_buf();
            self.selected.clear();
            // Sélectionner automatiquement le dossier pour la compression
            if self.mode_compress {
                self.selected.push(dir.to_path_buf());
            }
            self.log.to_mut().push_str(&format!("Navigated to: {}\n", dir.display()));
        } else {
            self.log.to_mut().push_str(&format!("Error: Cannot navigate to non-directory path: {}\n", dir.display()));
        }
    }

    fn go_back(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.current_dir = self.history[self.history_index].clone();
            self.selected.clear();
            self.log.to_mut().push_str(&format!("Navigated back to: {}\n", self.current_dir.display()));
        }
    }

    fn go_forward(&mut self) {
        if self.history_index + 1 < self.history.len() {
            self.history_index += 1;
            self.current_dir = self.history[self.history_index].clone();
            self.selected.clear();
            self.log.to_mut().push_str(&format!("Navigated forward to: {}\n", self.current_dir.display()));
        }
    }

    fn show_directory(&mut self, ui: &mut egui::Ui) {
        if let Some(parent) = self.current_dir.parent() {
            let p = parent.to_path_buf();
            let name = "..";
            let is_selected = self.selected.contains(&p);
            let color = if is_selected { Color32::LIGHT_BLUE } else { ui.visuals().text_color() };

            let response = ui
                .colored_label(color, RichText::new(name))
                .on_hover_text("Double-clic: aller au dossier parent");

            if response.double_clicked() {
                self.navigate_to(&p);
                // Sélectionner automatiquement le nouveau dossier pour la compression
                if self.mode_compress {
                    self.selected.clear();
                    self.selected.push(p);
                }
            }
            ui.label("Dossier");
            ui.end_row();
        }

        if let Ok(entries) = fs::read_dir(&self.current_dir) {
            let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            entries.sort_by_key(|e| e.file_name());

            egui::Grid::new("entries_grid").striped(true).show(ui, |ui| {
                ui.label(RichText::new("Nom").strong());
                ui.label(RichText::new("Type").strong());
                ui.end_row();

                for entry in entries {
                    let p = entry.path();
                    if p == self.current_dir {
                        continue;
                    }

                    let name = p.file_name().unwrap_or_default().to_string_lossy();
                    let is_selected = self.selected.contains(&p);
                    let color = if is_selected { Color32::LIGHT_BLUE } else { ui.visuals().text_color() };

                    let mut rich_text_name = RichText::new(name.as_ref());
                    if p.is_dir() {
                        rich_text_name = rich_text_name.strong();
                    }

                    let response = ui
                        .colored_label(color, rich_text_name)
                        .on_hover_text("Double-clic: ouvrir dossier | Clic: sélectionner");

                    if response.double_clicked() {
                        if p.is_dir() {
                            self.navigate_to(&p);
                        }
                    } else if response.clicked() {
                        if ui.ctx().input(|i| i.modifiers.ctrl) {
                            if is_selected {
                                self.selected.retain(|x| x != &p);
                            } else {
                                self.selected.push(p.clone());
                            }
                        } else {
                            self.selected.clear();
                            self.selected.push(p.clone());
                        }
                    }
                    ui.label(if p.is_dir() { "Dossier" } else { "Fichier" });
                    ui.end_row();
                }
            });
        } else {
            ui.label(RichText::new(format!("Impossible de lire le répertoire : {}", self.current_dir.display())).color(Color32::RED));
        }
    }

    fn execute_command(&mut self, mut cmd: Command, action: &str, ctx: &egui::Context) {
        println!("Commande exécutée : {:?}", cmd);
        self.log.to_mut().push_str(&format!("Exécution de la commande : {:?}\n", cmd));

        let action = action.to_string();
        let ctx_clone = ctx.clone();

        thread::spawn(move || {
            cmd.stdout(std::process::Stdio::piped())
               .stderr(std::process::Stdio::piped());

            match cmd.spawn() {
                Ok(mut child) => {
                    let stdout = child.stdout.take().unwrap();
                    let stderr = child.stderr.take().unwrap();

                    let stdout_reader = std::io::BufReader::new(stdout);
                    let stderr_reader = std::io::BufReader::new(stderr);

                    // Lire stdout
                    for line in stdout_reader.lines() {
                        if let Ok(line) = line {
                            println!("{}", line); // Afficher dans la console
                            ctx_clone.request_repaint(); // Repeindre l'interface
                        }
                    }

                    // Lire stderr
                    for line in stderr_reader.lines() {
                        if let Ok(line) = line {
                            eprintln!("{}", line); // Afficher les erreurs dans la console
                            ctx_clone.request_repaint(); // Repeindre l'interface
                        }
                    }

                    match child.wait() {
                        Ok(status) => {
                            if status.success() {
                                println!("{} terminée avec succès", action);
                            } else {
                                eprintln!("Erreur lors de {} (code: {:?})", action, status.code());
                            }
                        }
                        Err(e) => {
                            eprintln!("Erreur lors de l'attente du processus : {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Erreur lors du lancement de la commande : {}", e);
                }
            }
        });
    }

    fn handle_action(&mut self, ctx: &egui::Context) {
        self.log.to_mut().clear();
        let exe = if cfg!(windows) { ".\\FreeArc\\bin\\arc.exe" } else { "./FreeArc/bin/arc" };

        if !std::path::Path::new(exe).exists() {
            self.log.to_mut().push_str("Erreur : FreeArc n'est pas installé correctement\n");
            return;
        }

        if self.mode_compress {
            // Mode compression
            if self.selected.is_empty() {
                self.log.to_mut().push_str("Erreur : Aucune source sélectionnée pour la compression.\n");
                return;
            }
            if self.output_path.file_name().is_none() {
                self.log.to_mut().push_str("Erreur : Chemin de sortie invalide.\n");
                return;
            }

            self.log.to_mut().push_str(&format!(
                "Compression des fichiers : {:?}\nVers : {}\n",
                self.selected,
                self.output_path.display()
            ));

            let mut cmd = Command::new(exe);
            cmd.arg("a");
            cmd.arg(&self.output_path);
            for item in &self.selected {
                cmd.arg(item);
            }
            for flag in self.preset.flags() {
                cmd.arg(flag);
            }
            if self.output_path.extension().and_then(|ext| ext.to_str()) == Some("sfx") {
                cmd.arg("-sfx");
            }

            self.execute_command(cmd, "la compression", ctx);
        } else {
            // Mode extraction
            if self.selected.len() != 1 {
                self.log.to_mut().push_str("Erreur : Veuillez sélectionner une seule archive pour l'extraction.\n");
                return;
            }

            let archive_to_extract = &self.selected[0];
            if !archive_to_extract.exists() {
                self.log.to_mut().push_str(&format!("Erreur : L'archive {} n'existe pas\n", archive_to_extract.display()));
                return;
            }

            if let Some(dest) = rfd::FileDialog::new().set_title("Choisir le dossier d'extraction").pick_folder() {
                self.log.to_mut().push_str(&format!(
                    "Extraction de l'archive : {}\nVers : {}\n",
                    archive_to_extract.display(),
                    dest.display()
                ));

                let mut cmd = Command::new(exe);
                cmd.args(&[
                    "x",
                    archive_to_extract.to_str().unwrap(),
                    &format!("-dp{}", dest.display()),
                    "-o+",
                    "-y",
                ]);

                self.execute_command(cmd, "l'extraction", ctx);
            }
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Modern theme configuration
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        let mut visuals = Visuals::dark();
        visuals.window_shadow = Shadow {
            offset: [0, 4], // Use integer array for offset
            blur: 8,        // Use integer for blur
            spread: 2,      // Use integer for spread
            color: Color32::from_black_alpha(96),
        };
        visuals.panel_fill = Color32::from_rgb(32, 33, 36);
        visuals.window_fill = Color32::from_rgb(40, 41, 45);
        style.visuals = visuals;
        ctx.set_style(style);

        // Toolbar panel
        egui::TopBottomPanel::top("main_toolbar")
            .frame(Frame::default()
                .fill(Color32::from_rgb(28, 29, 32))
                .outer_margin(10.0)
                .corner_radius(8.0)
                .shadow(Shadow {
                    offset: [0, 2],
                    blur: 4,
                    spread: 1,
                    color: Color32::from_black_alpha(60),
                }))
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new("← Retour")
                            .fill(if self.history_index > 0 {Color32::from_rgb(66, 133, 244)} else {Color32::from_rgb(50, 51, 55)})
                            .corner_radius(5.0))
                            .clicked() && self.history_index > 0 { 
                            self.go_back();
                        }
                            
                        if ui.add(egui::Button::new("→")
                            .fill(if self.history_index + 1 < self.history.len() {Color32::from_rgb(66, 133, 244)} else {Color32::from_rgb(50, 51, 55)})
                            .corner_radius(5.0))
                            .clicked() && self.history_index + 1 < self.history.len() { 
                            self.go_forward();
                        }

                        if ui.add(egui::Button::new("↑")
                            .fill(Color32::from_rgb(66, 133, 244))
                            .corner_radius(5.0))
                            .clicked() {
                            if let Some(parent) = self.current_dir.parent() {
                                self.navigate_to(&parent.to_path_buf());
                            }
                        }
                    });
                    
                    ui.add_space(20.0);
                    
                    // Modern mode selector
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.mode_compress, true, 
                            RichText::new("📦 Compresser").size(16.0));
                        ui.selectable_value(&mut self.mode_compress, false, 
                            RichText::new("📂 Extraire").size(16.0));
                    });

                    ui.add_space(20.0);

                    // Modern preset selector
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Preset:").size(16.0));
                        egui::ComboBox::new("preset_selector", "")
                            .selected_text(RichText::new(self.preset.label()).size(16.0))
                            .show_ui(ui, |ui| {
                                for preset in CompressionPreset::all() {
                                    ui.selectable_value(&mut self.preset, preset.clone(),
                                        RichText::new(preset.label()).size(16.0));
                                }
                            });
                    });

                    ui.add_space(20.0);

                    // Extension selector
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Extension:").size(16.0));
                        let current_ext = self.output_path.extension()
                            .and_then(|ext| ext.to_str())
                            .unwrap_or("arc");
                        let stem = self.output_path.file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default();
                        let parent = self.output_path.parent().unwrap_or(Path::new(""));
                        let mut new_ext: Option<&str> = None;  // Add type annotation
                        
                        egui::ComboBox::from_label("")
                            .selected_text(format!(".{}", current_ext))
                            .show_ui(ui, |ui| {
                                for ext in ["arc", "bin", "pak", "dat", "sfx"] {
                                    let selected = current_ext == ext;
                                    if ui.selectable_label(selected, format!(".{}", ext)).clicked() && !selected {
                                        new_ext = Some(ext);
                                    }
                                }
                            });

                        if let Some(ext) = new_ext {
                            self.output_path = parent.join(format!("{}.{}", stem, ext));
                        }
                    });
                });
                ui.add_space(8.0);
            });

        // File explorer panel
        egui::SidePanel::left("file_explorer")
            .resizable(true)
            .min_width(250.0)
            .frame(Frame::default()
                .fill(Color32::from_rgb(35, 36, 40))
                .corner_radius(8.0)
                .outer_margin(10.0)
                .shadow(Shadow {
                    offset: [2, 0],
                    blur: 4,
                    spread: 1,
                    color: Color32::from_black_alpha(60),
                }))
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.heading(RichText::new("📁 Explorateur").size(20.0).strong());
                ui.add_space(4.0);
                ui.label(RichText::new(format!("📍 {}", self.current_dir.display())).size(14.0));
                ui.add_space(8.0);
                
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        self.show_directory(ui);
                    });
            });

        // Central panel
        egui::CentralPanel::default()
            .frame(Frame::default()
                .fill(Color32::from_rgb(35, 36, 40))
                .corner_radius(8.0)
                .outer_margin(10.0)
                .shadow(Shadow {
                    offset: [0, 2],
                    blur: 4,
                    spread: 1,
                    color: Color32::from_black_alpha(60),
                }))
            .show(ctx, |ui| {
                ui.add_space(8.0);
                // Action buttons
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new(RichText::new("📂 Parcourir...").size(16.0))
                        .fill(Color32::from_rgb(66, 133, 244))
                        .min_size(egui::vec2(120.0, 32.0)))
                        .clicked() {
                        if let Some(dir) = rfd::FileDialog::new()
                            .set_title("Sélectionner un dossier")
                            .set_directory(&self.current_dir)
                            .pick_folder() {
                            self.current_dir = dir.clone();
                            self.history.push(dir.clone());
                            self.history_index = self.history.len() - 1;
                            self.selected.clear();
                            // Sélectionner automatiquement le dossier pour la compression
                            self.selected.push(dir);
                            self.log.to_mut().push_str(&format!("Dossier sélectionné : {}\n", self.current_dir.display()));
                        }
                    }
                    
                    if ui.add(egui::Button::new(RichText::new("💾 Destination...").size(16.0))
                        .fill(Color32::from_rgb(66, 133, 244))
                        .min_size(egui::vec2(120.0, 32.0)))
                        .clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_title("Choisir la destination")
                            .set_file_name(self.output_path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("archive.arc"))
                            .save_file() {
                            self.output_path = path;
                        }
                    }

                    // Extension selector
                    egui::ComboBox::from_label(RichText::new("📑 Extension").size(16.0))
                        .selected_text(format!(".{}", self.output_path.extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("arc")))
                        .show_ui(ui, |ui| {
                            // ... existing extension logic ...
                        });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(egui::Button::new(RichText::new("▶ Exécuter").size(16.0))
                            .fill(Color32::from_rgb(52, 168, 83))
                            .min_size(egui::vec2(120.0, 32.0)))
                            .clicked() {
                            self.handle_action(ctx);
                        }
                    });
                });

                ui.add_space(16.0);
                
                // Logs avec style moderne
                ui.heading(RichText::new("📝 Logs").size(20.0).strong());
                ui.add_space(8.0);
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.log)
                                .desired_rows(12)
                                .desired_width(f32::INFINITY)
                                .font(egui::TextStyle::Monospace)
                                .interactive(false)
                        );
                    });

                // Progress bar
                self.show_progress(ui);
            });

        // Status bar
        egui::TopBottomPanel::bottom("status_bar")
            .frame(Frame::default()
                .fill(Color32::from_rgb(28, 29, 32))
                .corner_radius(8.0)
                .outer_margin(10.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("⚡ Mode:").strong());
                    ui.label(if self.mode_compress {"Compression"} else {"Extraction"});
                    ui.separator();
                    ui.label(RichText::new("📊 Sélection:").strong());
                    ui.label(format!("{} fichier(s)", self.selected.len()));
                });
            });
    }

    fn show_preview(&mut self, ui: &mut egui::Ui) {
        if let Some(path) = &self.preview_file {
            if path.is_file() {
                // Limiter la taille de prévisualisation
                if let Ok(content) = fs::read_to_string(path) {
                    ui.group(|ui| {
                        ui.heading("Prévisualisation");
                        ui.text_edit_multiline(&mut content.as_str());
                    });
                }
            }
        }
    }

    fn show_progress(&mut self, ui: &mut egui::Ui) {
        if let Some(progress) = &self.progress {
            let percentage = (progress.processed_bytes as f32 / progress.total_bytes as f32) * 100.0;
            ui.add(egui::ProgressBar::new(percentage / 100.0)
                .text(format!("{:.1}% - Temps restant estimé: {:?}", percentage, progress.estimated_remaining)));
        }
    }

    fn update_stats(&mut self) {
        let stats = FileStats { // Removed mut as it's not needed
            total_size: 0,
            file_count: 0,
            largest_file: (PathBuf::new(), 0),
            by_extension: HashMap::new(),
        };

        for _path in &self.selected { // Added underscore to unused variable
            // Calculer les statistiques...
        }

        self.current_stats = Some(stats);
    }

    fn apply_theme(&self, ctx: &egui::Context, theme: &Theme) {
        let mut style = (*ctx.style()).clone();
        let mut visuals = style.visuals.clone();
        
        visuals.window_fill = theme.background_color;
        visuals.panel_fill = theme.secondary_color;
        visuals.widgets.noninteractive.fg_stroke.color = theme.text_color;
        visuals.selection.bg_fill = theme.accent_color;
        
        style.visuals = visuals;
        ctx.set_style(style);
    }
}

impl eframe::App for MonCompresseurApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.update(ctx, frame);
    }
}

fn main() -> Result<(), eframe::Error> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "--compress" => {
                if args.len() > 2 {
                    let path = &args[2];
                    println!("Compression demandée pour : {}", path);

                    // Exemple de logique de compression
                    let output_path = format!("{}.arc", path);
                    let mut cmd = Command::new(".\\FreeArc\\bin\\arc.exe");
                    cmd.args(&["a", &output_path, path]);

                    match cmd.output() {
                        Ok(output) => {
                            if output.status.success() {
                                println!("Compression réussie : {}", output_path);
                            } else {
                                eprintln!(
                                    "Erreur lors de la compression : {}",
                                    String::from_utf8_lossy(&output.stderr)
                                );
                            }
                        }
                        Err(e) => {
                            eprintln!("Erreur lors de l'exécution de la commande : {}", e);
                        }
                    }
                } else {
                    eprintln!("Erreur : Aucun chemin fourni pour la compression.");
                }
            }
            "--extract" => {
                if args.len() > 2 {
                    let path = &args[2];
                    println!("Extraction demandée pour : {}", path);
                    // Ajoutez ici la logique d'extraction
                } else {
                    eprintln!("Erreur : Aucun chemin fourni pour l'extraction.");
                }
            }
            _ => {
                eprintln!("Argument inconnu : {}", args[1]);
            }
        }
        return Ok(());
    }

    // Updated window configuration
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([800.0, 600.0])
            .with_decorations(true)
            .with_transparent(false),
        ..Default::default()
    };

    eframe::run_native(
        "stelarc 0.4.46-beta",
        native_options,
        Box::new(|_creation_context| Ok(Box::new(MonCompresseurApp::default()))),
    )
}
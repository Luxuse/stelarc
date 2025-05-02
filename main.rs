use eframe::egui;
use eframe::App; // Import the App trait
use egui::{RichText, Color32};
use std::process::Command;
use std::path::PathBuf;
use std::fs;
use rfd;

/// Application de compression/extraction inspirée de WinRAR/7-Zip
struct MonCompresseurApp {
    current_dir: PathBuf,
    history: Vec<PathBuf>, // Navigation history for the explorer
    history_index: usize, // Current position in history
    selected: Vec<PathBuf>, // Files/folders selected in the explorer
    mode_compress: bool, // true for compress, false for extract
    preset: CompressionPreset, // Selected compression preset
    output_path: PathBuf, // Path for the output archive or extraction destination (when in extract mode, this is the archive to extract)
    log: String, // Log area content
    drives: Vec<PathBuf>, // List of system drives (Windows specific)
}

/// Presets disponibles pour FreeArc, incluant des modes variés
#[derive(Clone, PartialEq)]
enum CompressionPreset {
    Instant,
    HDDspeed,
    Fastest,
    NormalPrecomplzma,
    Normal,
    Best,
}

impl CompressionPreset {
    fn all() -> &'static [CompressionPreset] {
        static ALL: [CompressionPreset; 6] = [
            CompressionPreset::Instant,
            CompressionPreset::HDDspeed,
            CompressionPreset::Fastest,
            CompressionPreset::NormalPrecomplzma,
            CompressionPreset::Normal,
            CompressionPreset::Best,
        ];
        &ALL
    }

    fn label(&self) -> &'static str {
        match self {
            CompressionPreset::Instant => "Instant     (-m1)",
            CompressionPreset::HDDspeed => "HDD speed   (-m2)",
            CompressionPreset::Fastest => "Fastest     (-m3)",
            CompressionPreset::Normal => "Normal      (-m4)",
            CompressionPreset::NormalPrecomplzma => "Normal+precomp+lzma (-m4 -mc:lzma/lzma:max:32mb -mc$default,$obj:+precomp)",
            CompressionPreset::Best => "Best        (-m5)",
        }
    }

    fn flag(&self) -> &'static str {
        match self {
            CompressionPreset::Instant => "-m1",
            CompressionPreset::HDDspeed => "-m2",
            CompressionPreset::Fastest => "-m3",
            CompressionPreset::Normal => "-m4",
            CompressionPreset::NormalPrecomplzma => "-m4 -mc:lzma/lzma:max:32mb -mc$default,$obj:+precomp",
            CompressionPreset::Best => "-m5",
        }
    }

    fn flags(&self) -> Vec<&'static str> {
        match self {
            CompressionPreset::Instant => vec!["-m1"],
            CompressionPreset::HDDspeed => vec!["-m2"],
            CompressionPreset::Fastest => vec!["-m3"],
            CompressionPreset::Normal => vec!["-m4"],
            CompressionPreset::NormalPrecomplzma => vec!["-m4", "-mc:lzma/lzma:max:32mb", "-mc$default,$obj:+precomp"],
            CompressionPreset::Best => vec!["-m5"],
        }
    }
}

impl Default for MonCompresseurApp {
    fn default() -> Self {
        let drives = if cfg!(target_os = "windows") {
            ('A'..='Z')
                .filter_map(|c| {
                    let d = format!("{}:/", c);
                    let pb = PathBuf::from(&d);
                    if pb.exists() && fs::metadata(&pb).map_or(false, |m| m.is_dir()) {
                        Some(pb)
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            vec![PathBuf::from("/")]
        };

        let cwd = std::env::current_dir().unwrap_or_else(|_| {
            drives.get(0).cloned().unwrap_or_else(|| PathBuf::from("."))
        });

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
            log: String::new(),
            drives,
        }
    }
}

impl MonCompresseurApp {
    fn navigate_to(&mut self, dir: PathBuf) {
        if dir.is_dir() {
            if self.history_index + 1 < self.history.len() {
                self.history.truncate(self.history_index + 1);
            }
            self.history.push(dir.clone());
            self.history_index = self.history.len() - 1;
            self.current_dir = dir;
            self.selected.clear();
            self.log.push_str(&format!("Navigated to: {}\n", self.current_dir.display()));
        } else {
            self.log.push_str(&format!("Error: Cannot navigate to non-directory path: {}\n", dir.display()));
        }
    }

    fn go_back(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.current_dir = self.history[self.history_index].clone();
            self.selected.clear();
            self.log.push_str(&format!("Navigated back to: {}\n", self.current_dir.display()));
        }
    }

    fn go_forward(&mut self) {
        if self.history_index + 1 < self.history.len() {
            self.history_index += 1;
            self.current_dir = self.history[self.history_index].clone();
            self.selected.clear();
            self.log.push_str(&format!("Navigated forward to: {}\n", self.current_dir.display()));
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
                self.navigate_to(p);
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
                            self.navigate_to(p.clone());
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

    fn execute_command(&mut self, cmd: &mut Command, action: &str) {
        self.log.push_str(&format!("Exécution de la commande : {:?}\n", cmd));
        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    self.log.push_str(&format!("{} réussie !\n{}", action, String::from_utf8_lossy(&output.stdout)));
                } else {
                    self.log.push_str(&format!("Erreur lors de {} :\n{}", action, String::from_utf8_lossy(&output.stderr)));
                }
            }
            Err(e) => {
                self.log.push_str(&format!("Erreur lors de l'exécution de la commande : {}\n", e));
            }
        }
    }

    fn handle_action(&mut self) {
        self.log.clear();
        let exe = if cfg!(windows) { ".\\FreeArc\\bin\\arc.exe" } else { "./FreeArc/bin/arc" };

        if self.mode_compress {
            // Mode compression
            if self.selected.is_empty() {
                self.log.push_str("Erreur : Aucune source sélectionnée pour la compression.\n");
                return;
            }
            if self.output_path.file_name().is_none() {
                self.log.push_str("Erreur : Chemin de sortie invalide.\n");
                return;
            }

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

            self.execute_command(&mut cmd, "la compression");
        } else {
            // Mode extraction
            if self.selected.len() != 1 {
                self.log.push_str("Erreur : Veuillez sélectionner une seule archive pour l'extraction.\n");
                return;
            }

            let archive_to_extract = &self.selected[0];
            if let Some(dest) = rfd::FileDialog::new().pick_folder() {
                let mut cmd = Command::new(exe);
                cmd.args(&["x", archive_to_extract.to_str().unwrap(), &format!("-dp{}", dest.display()), "-o+", "-y"]);

                self.log.push_str(&format!(
                    "[EXTRACTION] Archive sélectionnée : {}\n",
                    archive_to_extract.display()
                ));
                self.log.push_str(&format!("Dossier de destination : {}\n", dest.display()));
                self.log.push_str(&format!("Commande exécutée : {:?}\n", cmd));

                match cmd.output() {
                    Ok(output) => {
                        if output.status.success() {
                            self.log.push_str(&format!(
                                "Extraction réussie pour l'archive : {}\n",
                                archive_to_extract.display()
                            ));
                        } else {
                            self.log.push_str(&format!(
                                "Erreur lors de l'extraction :\n{}\n",
                                String::from_utf8_lossy(&output.stderr)
                            ));
                        }
                    }
                    Err(e) => {
                        self.log.push_str(&format!(
                            "Erreur lors de l'exécution de la commande : {}\n",
                            e
                        ));
                    }
                }
            } else {
                self.log.push_str("Extraction annulée : dossier de destination non sélectionné.\n");
            }
        }
    }
}

impl App for MonCompresseurApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui.button("← Retour").clicked() {
                    self.go_back();
                }
                if ui.add_enabled(self.history_index + 1 < self.history.len(), egui::Button::new("Avancer →")).clicked() {
                    self.go_forward();
                }
                if ui.button("Haut ↑").clicked() {
                    if let Some(parent) = self.current_dir.parent() {
                        self.navigate_to(parent.to_path_buf());
                    }
                }
                ui.separator();

                // Réintégration de la sélection du mode de compression
                ui.selectable_value(&mut self.mode_compress, true, "Compresser");
                ui.selectable_value(&mut self.mode_compress, false, "Extraire");
                ui.separator();

                // Réintégration de la sélection des presets de compression
                if self.mode_compress {
                    ui.label("Preset :");
                    egui::ComboBox::from_label("")
                        .selected_text(self.preset.label())
                        .show_ui(ui, |ui| {
                            for p in CompressionPreset::all() {
                                ui.selectable_value(&mut self.preset, p.clone(), p.label());
                            }
                        });
                }
                ui.separator();

                // Bouton pour ajouter des fichiers/dossiers via une boîte de dialogue
                if ui.button("Parcourir...").clicked() {
                    if let Some(paths) = rfd::FileDialog::new().pick_folders() {
                        for path in paths {
                            if !self.selected.contains(&path) {
                                self.selected.push(path);
                            }
                        }
                        self.log.push_str("Fichiers/dossiers ajoutés via la boîte de dialogue.\n");
                    }
                }
                ui.separator();

                // Bouton pour sélectionner le chemin de sortie
                if ui.button("Chemin de sortie...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_directory(&self.current_dir)
                        .set_file_name(&*self.output_path.file_name().unwrap_or_default().to_string_lossy())
                        .save_file()
                    {
                        self.output_path = path;
                        self.log.push_str(&format!("Chemin de sortie sélectionné : {}\n", self.output_path.display()));
                    }
                }

                // Liste déroulante pour choisir l'extension du fichier de sortie
                let mut current_ext = self.output_path.extension().unwrap_or_default().to_string_lossy().into_owned();
                egui::ComboBox::from_label("Extension")
                    .selected_text(&current_ext)
                    .show_ui(ui, |ui| {
                        for ext in ["arc", "bin", "pak", "dat", "sfx"] {
                            if ui.selectable_value(&mut current_ext, ext.to_string(), ext).clicked() {
                                if let Some(stem) = self.output_path.file_stem() {
                                    self.output_path = self.output_path.with_file_name(format!("{}.{}", stem.to_string_lossy(), ext));
                                }
                            }
                        }
                    });

                ui.separator();

                // Bouton pour exécuter l'action principale
                if ui.button("Exécuter").clicked() {
                    self.handle_action();
                }
            });
        });

        egui::SidePanel::left("explorer").resizable(true).show(ctx, |ui| {
            ui.heading("Explorateur de fichiers");
            ui.label(format!("Dossier actuel : {}", self.current_dir.display()));
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.show_directory(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Logs");
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(egui::TextEdit::multiline(&mut self.log).desired_rows(12).interactive(false));
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native(
        "stelarc-v1.35-BETA",
        native_options,
        Box::new(|_creation_context| Ok(Box::new(MonCompresseurApp::default()))),
    )
}
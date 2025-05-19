use eframe::egui;
use egui::{RichText, Color32, Shadow, Visuals, Frame, pos2};
use egui::menu::MenuState;
use std::process::Command;
use std::path::{Path, PathBuf};
use std::fs;
use rfd;
use std::io::BufRead;
use std::thread;
use std::collections::HashMap;
use std::sync::mpsc;
use rodio;
use rodio::Source;
use sysinfo::{System, Process}; // <-- Correction de l'import


// Import unique du trait Digest via sha3
use sha3::digest::Digest;

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

impl Theme {
    fn default_themes() -> Vec<Theme> {
        vec![
            Theme {
                name: "Dark".to_string(),
                primary_color: Color32::from_rgb(28, 29, 32),
                secondary_color: Color32::from_rgb(35, 36, 40),
                background_color: Color32::from_rgb(32, 33, 36),
                text_color: Color32::WHITE,
                accent_color: Color32::from_rgb(66, 133, 244),
            },
            Theme {
                name: "Light".to_string(),
                primary_color: Color32::from_rgb(245, 245, 245),
                secondary_color: Color32::from_rgb(255, 255, 255),
                background_color: Color32::from_rgb(250, 250, 250),
                text_color: Color32::BLACK,
                accent_color: Color32::from_rgb(26, 115, 232),
            },
        ]
    }
}

#[derive(Debug, Clone)] // Add Clone derive to Notification
struct Notification {
    message: String,
    level: NotificationLevel,
    timestamp: std::time::Instant,
}

#[derive(Debug, PartialEq, Clone)] // Add Clone derive to NotificationLevel
enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}

// Ajout des structures pour le calcul de hash
#[derive(Debug, Clone, PartialEq)]
enum HashType {
    CRC32,
    Blake3,
    MD5,
    SHA256,
    SHA3_256
}

impl HashType {
    fn label(&self) -> &'static str {
        match self {
            HashType::CRC32 => "CRC32",
            HashType::Blake3 => "BLAKE3",
            HashType::MD5 => "MD5",
            HashType::SHA256 => "SHA-256",
            HashType::SHA3_256 => "SHA3-256"
        }
    }

    fn all() -> &'static [HashType] {
        static ALL: [HashType; 5] = [
            HashType::CRC32,
            HashType::Blake3,
            HashType::MD5,
            HashType::SHA256,
            HashType::SHA3_256,
        ];
        &ALL
    }
}

// Nouveau type de message pour la communication depuis le thread de commande
#[derive(Debug)]
enum CommandUpdate {
    LogOutput(String),      // Une ligne de log (stdout ou stderr)
    Progress(f32),          // Progression en pourcentage (0.0 à 1.0)
    ProcessCompleted(Result<String, String>), // Résultat: Ok(message_succès) ou Err(message_erreur)
}

struct MonCompresseurApp {
    current_dir: PathBuf,
    history: Vec<PathBuf>,
    history_index: usize,
    selected: Vec<PathBuf>,
    mode_compress: bool,
    preset: CompressionPreset,
    output_path: PathBuf,
    log_lines: Vec<String>,       // Pour les logs en temps réel
    stats: Option<CompressionStats>,
    preview_file: Option<PathBuf>,
    preview_content: String,
    progress_value: f32, // 0.0 à 1.0 pour la barre de progression
    operation_status: String, // Ex: "Compression en cours...", "Terminé", "Erreur"
    is_processing: bool,  // True si une commande est en cours
    current_stats: Option<FileStats>,
    notification: Option<Notification>,
    current_theme: Theme,
    command_rx: Option<mpsc::Receiver<CommandUpdate>>, // Pour recevoir les logs/progressions
    command_tx: Option<mpsc::Sender<CommandUpdate>>,   // Pour envoyer depuis le thread (gardé temporairement)
    show_hash_window: bool,
    selected_hash_type: HashType,
    hash_result: Option<String>,

    // Pour CPU/RAM
    sys: System,
    cpu_usage: f32,        // 0.0 à 1.0
    ram_usage_mb: u64,
    ram_total_mb: u64,
    arc_pid: Option<u32>,
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
    NormalPrecomplzmadelta,
    Normal,
    Best,
    Fastlolz,
    MediumLOLZ,
    Maximum,
    MaximumLOLZ,
    MXtoolLolz,
    Bxtool,
    XtoolC,
    XtoolD,
    XtoolE,
}

impl CompressionPreset {
    fn all() -> &'static [CompressionPreset] {
        static ALL: [CompressionPreset; 18] = [
            CompressionPreset::Instant,
            CompressionPreset::HDDspeed,
            CompressionPreset::UltrafastSREP,
            CompressionPreset::Ultrafastlolz,
            CompressionPreset::Fastest,
            CompressionPreset::FastSrepLZ,
            CompressionPreset::NormalPrecomplzmadelta,
            CompressionPreset::Normal,
            CompressionPreset::Fastlolz,
            CompressionPreset::Best,
            CompressionPreset::Maximum,
            CompressionPreset::MediumLOLZ,
            CompressionPreset::Bxtool,
            CompressionPreset::MaximumLOLZ,
            CompressionPreset::MXtoolLolz,
            CompressionPreset::XtoolC,
            CompressionPreset::XtoolD,
            CompressionPreset::XtoolE,
            ];
        &ALL
    }

    fn label(&self) -> &'static str {
        match self {
            CompressionPreset::Instant => "Instant     (-m1)",
            CompressionPreset::HDDspeed => "HDD speed   (-m2)",
            CompressionPreset::UltrafastSREP => "UltrafastSREP   (M3+lzma+srep)",
            CompressionPreset::Ultrafastlolz => "Ultrafastlolz     (M3+lzma+lolz)",
            CompressionPreset::Fastest => "Fastest     (-m3)",
            CompressionPreset::FastSrepLZ => "Fast+Srep+LZ  (M4+lzma+srep+precomp)",
            CompressionPreset::Normal => "Normal      (-m4)",
            CompressionPreset::NormalPrecomplzmadelta => "Normal+precomp+lzma (M4+lzma+precomp+delta)",
            CompressionPreset::Best => "Best        (-m5)",
            CompressionPreset::Maximum => "Maximum       (-m9d)",
            CompressionPreset::Fastlolz => "fastlolz (M4+lzma+lolz+srep+precomp)",
            CompressionPreset::MediumLOLZ => "MediumLOLZ (-M5+lzma+lolz+srep+precomp+lzma)",
            CompressionPreset::Bxtool => "xtool (M6+lzma+xtool+srep+precomp)",
            CompressionPreset::MaximumLOLZ => "MaximumLOLZ (M6+lzma+lolz+srep+precomp)",
            CompressionPreset::MXtoolLolz => "xtoolfast+ (M6+lzma+xtool+lolz+srep+precomp)",
            CompressionPreset::XtoolC => "xtoolmedium+       (M6+precomp+xtool+crilayla+razorx)",
            CompressionPreset::XtoolD => "xtoolhigh       (M6+precomp+xtool+crilayla+ZSTD)",
            CompressionPreset::XtoolE => "xtoolhigh       (m6+precomp+xtool+crilayla+lolz)",
        }
    }
    
    fn flags(&self) -> Vec<&'static str> {
        match self {
            CompressionPreset::Instant => vec!["-m1"],
            CompressionPreset::HDDspeed => vec!["-m2"],
            CompressionPreset::Fastest => vec!["-m3"],
            CompressionPreset::UltrafastSREP => vec!["-m3d", "-s;", "-mc:lzma/lzma:max:8mb", "-mc:rep/maxsrep"],
            CompressionPreset::Ultrafastlolz => vec!["-m3d", "-s;", "-mc:lzma/lzma:max:8mb", "-m=lolz:mtt0:mt12:d2m"],
            CompressionPreset::FastSrepLZ => vec!["-m3d", "-s;", "-mc:lzma/lzma:max:8mb", "-mc:rep/maxsrep", "-mc$default,$obj:+precomp048"],
            CompressionPreset::Normal => vec!["-m4"],
            CompressionPreset::NormalPrecomplzmadelta => vec!["-m4", "-mc:lzma/lzma:max:32mb", "-mc$default,$obj:+precomp048", "-mc-delta"],
            CompressionPreset::Best => vec!["-m5"],
            CompressionPreset::Maximum => vec!["-m9d"],
            CompressionPreset::Fastlolz => vec!["-m4d", "-s;", "-mc:lzma/lzma:max:64mb",  "-mc$default,$obj:+precomp048", "-m=rep/maxsrep+lolz:mtt1:mt6:d8m"],
            CompressionPreset::MaximumLOLZ => vec!["-m6d", "-s;", "-mc:lzma/lzma:max:192mb",  "-mc$default,$obj:+precomp048", "-m=rep/maxsrep+lolz:mtt0:mt6:d128m"],
            CompressionPreset::MediumLOLZ => vec!["-m5d", "-s;", "-mc:lzma/lzma:max:96mb",  "-mc$default,$obj:+precomp048", "-m=rep/maxsrep+lolz:mtt0:mt6:d48m"],
            CompressionPreset::MXtoolLolz => vec!["-m6d", "-s;", "-mc$default,$obj:+precomp048", "-mc:lzma/lzma:max:192mb", "-mxtool:c256mb:mpreflate:mzlib+xtool:dd3+lolz:mtt0:mt6:d64m"],
            CompressionPreset::Bxtool => vec!["-m6d", "-s;", "-mc$default,$obj:+precomp048", "-mc:lzma/lzma:max:32mb", "-mxtool:c64mb:mpreflate:mzlib+xtool:dd3"],
            CompressionPreset::XtoolC => vec!["-m6d", "-s;", "-mc$default,$obj:+precomp048",  "-mxtool:o:c64mb:t75p:g90p:mkraken:mmermaid:3:mzlib:mlz4f,l6:mzstd:c32mb:mcrilayla:mreflate:l3,d16mb+razorx"],
            CompressionPreset::XtoolD => vec!["-m6d", "-s;", "-mc$default,$obj:+precomp048", "-mxtool:o:c1024mb:t90p:g100p:mkraken:mmermaid:9:mzlib:mlz4f,l12:mzstd:c32mb:mcrilayla:mreflate:l10,d1024mb+zstd:-ultra:22:T0"],
            CompressionPreset::XtoolE => vec!["-m6d", "-s;", "-mc$default,$obj:+precomp048", "-mxtool:o:c1024mb:t90p:g100p:mkraken:mmermaid:9:mzlib:mlz4f,l8:mzstd:c32mb:mcrilayla:mpreflate:mbrunsli:dd3:l6,d1024mb+lolz:mtt0:mt6:d256m"],
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
            log_lines: Vec::new(),
            stats: None,
            preview_file: None,
            preview_content: String::new(),
            progress_value: 0.0,
            operation_status: String::new(),
            is_processing: false,
            current_stats: None,
            notification: None,
            current_theme: Theme::default_themes()[0].clone(),
            command_rx: None,
            command_tx: None,
            show_hash_window: false,
            selected_hash_type: HashType::CRC32,
            hash_result: None,
            sys: System::new_all(),
            cpu_usage: 0.0,
            ram_usage_mb: 0,
            ram_total_mb: 0,
            arc_pid: None,
        }
    }
}

impl MonCompresseurApp {
    fn list_available_drives() -> Vec<PathBuf> {
        let mut drives = Vec::new();

        if cfg!(windows) {
            // Windows: check drives from C: to Z:
            for letter in b'C'..=b'Z' {
                let drive = format!("{}:\\", letter as char);
                if Path::new(&drive).exists() {
                    drives.push(PathBuf::from(drive));
                }
            }
        } else {
            // Linux/Unix: just add root
            drives.push(PathBuf::from("/"));
        }

        drives
    }

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
            self.log_lines.push(format!("Navigated to: {}\n", dir.display()));
        } else {
            self.log_lines.push(format!("Error: Cannot navigate to non-directory path: {}\n", dir.display()));
        }
    }

    fn go_back(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.current_dir = self.history[self.history_index].clone();
            self.selected.clear();
            self.log_lines.push(format!("Navigated back to: {}\n", self.current_dir.display()));
        }
    }

    fn go_forward(&mut self) {
        if self.history_index + 1 < self.history.len() {
            self.history_index += 1;
            self.current_dir = self.history[self.history_index].clone();
            self.selected.clear();
            self.log_lines.push(format!("Navigated forward to: {}\n", self.current_dir.display()));
        }
    }

    fn show_directory(&mut self, ui: &mut egui::Ui) {
        // Ajout du sélecteur de disques
        ui.horizontal(|ui| {
            ui.label(RichText::new("💽 Disques:").strong());
            let drives = Self::list_available_drives();
            for drive in drives {
                if ui.button(drive.display().to_string()).clicked() {
                    self.navigate_to(&drive);
                }
            }
        });
        ui.add_space(8.0);

        // Affichage du dossier parent
        if let Some(parent) = self.current_dir.parent() {
            let p = parent.to_path_buf();
            let name = "..";
            let is_selected = self.selected.contains(&p);
            let color = if is_selected { Color32::LIGHT_BLUE } else { ui.visuals().text_color() };

            ui.horizontal(|ui| {
                let response = ui
                    .colored_label(color, RichText::new(name))
                    .on_hover_text("Double-clic: aller au dossier parent");

                if response.double_clicked() {
                    self.navigate_to(&p);
                    if self.mode_compress {
                        self.selected.clear();
                        self.selected.push(p);
                    }
                }
                ui.label("Dossier");
            });
            ui.add_space(4.0);
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

                    ui.horizontal(|ui| {
                        let mut rich_text_name = RichText::new(name.as_ref());
                        if p.is_dir() {
                            rich_text_name = rich_text_name.strong();
                        }

                        let response = ui
                            .colored_label(color, rich_text_name)
                            .on_hover_text("Double-clic: ouvrir | Clic droit: menu");

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

                        // Afficher le menu contextuel
                        self.show_context_menu(ui, &p);

                        ui.label(if p.is_dir() { "Dossier" } else { "Fichier" });
                    });
                    ui.end_row();
                }
            });
        } else {
            ui.label(RichText::new(format!("Impossible de lire le répertoire : {}", self.current_dir.display())).color(Color32::RED));
        }
    }

    fn play_notification_sound() {
        static NOTIFICATION_BYTES: &[u8] = include_bytes!("notification_sound.wav");
        if let Ok((_stream, stream_handle)) = rodio::OutputStream::try_default() {
            let cursor = std::io::Cursor::new(NOTIFICATION_BYTES);
            if let Ok(source) = rodio::Decoder::new(cursor) {
                let _ = stream_handle.play_raw(source.convert_samples());
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }
    }

    fn execute_command(&mut self, mut cmd: Command, action: &str, ctx: &egui::Context) {
        println!("Commande exécutée : {:?}", cmd);
        self.log_lines.push(format!("Exécution de la commande : {:?}\n", cmd));

        let action = action.to_string();
        let ctx_clone = ctx.clone();
        let (tx, rx) = mpsc::channel();
        self.command_rx = Some(rx);
        self.command_tx = Some(tx.clone());

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
                            println!("{}", line);
                            tx.send(CommandUpdate::LogOutput(line)).ok();
                            ctx_clone.request_repaint();
                        }
                    }

                    // Lire stderr
                    for line in stderr_reader.lines() {
                        if let Ok(line) = line {
                            eprintln!("{}", line);
                            tx.send(CommandUpdate::LogOutput(line)).ok();
                            ctx_clone.request_repaint();
                        }
                    }

                    match child.wait() {
                        Ok(status) => {
                            if status.success() {
                                Self::play_notification_sound();
                                let notification = Notification {
                                    message: format!("{} terminée avec succès", action),
                                    level: NotificationLevel::Success,
                                    timestamp: std::time::Instant::now(),
                                };
                                tx.send(CommandUpdate::ProcessCompleted(Ok(notification.message))).ok();
                            } else {
                                Self::play_notification_sound();
                                let notification = Notification {
                                    message: format!("Erreur lors de {}", action),
                                    level: NotificationLevel::Error,
                                    timestamp: std::time::Instant::now(),
                                };
                                tx.send(CommandUpdate::ProcessCompleted(Err(notification.message))).ok();
                            }
                        }
                        Err(e) => {
                            eprintln!("Erreur lors de l'attente du processus : {}", e);
                            let notification = Notification {
                                message: format!("Erreur: {}", e),
                                level: NotificationLevel::Error,
                                timestamp: std::time::Instant::now(),
                            };
                            tx.send(CommandUpdate::ProcessCompleted(Err(notification.message))).ok();
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Erreur lors du lancement de la commande : {}", e);
                    let notification = Notification {
                        message: format!("Erreur: {}", e),
                        level: NotificationLevel::Error,
                        timestamp: std::time::Instant::now(),
                    };
                    tx.send(CommandUpdate::ProcessCompleted(Err(notification.message))).ok();
                }
            }
        });
    }

    fn handle_action(&mut self, ctx: &egui::Context) {
        self.log_lines.clear();
        let exe = if cfg!(windows) { ".\\FreeArc\\arc.exe" } else { "./FreeArc/bin/arc" };

        if !std::path::Path::new(exe).exists() {
            self.log_lines.push("Erreur : FreeArc n'est pas installé correctement\n".to_string());
            return;
        }

        if self.mode_compress {
            // Mode compression
            if self.selected.is_empty() {
                self.log_lines.push("Erreur : Aucune source sélectionnée pour la compression.\n".to_string());
                return;
            }
            if self.output_path.file_name().is_none() {
                self.log_lines.push("Erreur : Chemin de sortie invalide.\n".to_string());
                return;
            }

            self.log_lines.push(format!(
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
                self.log_lines.push("Erreur : Veuillez sélectionner une seule archive pour l'extraction.\n".to_string());
                return;
            }

            let archive_to_extract = &self.selected[0];
            if !archive_to_extract.exists() {
                self.log_lines.push(format!("Erreur : L'archive {} n'existe pas\n", archive_to_extract.display()));
                return;
            }

            if let Some(dest) = rfd::FileDialog::new().set_title("Choisir le dossier d'extraction").pick_folder() {
                self.log_lines.push(format!(
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
        // Check for notifications at the start of the update
        if let Some(rx) = &self.command_rx {
            if let Ok(update) = rx.try_recv() {
                match update {
                    CommandUpdate::LogOutput(log) => {
                        self.log_lines.push(log);
                    },
                    CommandUpdate::Progress(progress) => {
                        self.progress_value = progress;
                    },
                    CommandUpdate::ProcessCompleted(result) => {
                        match result {
                            Ok(message) => {
                                self.notification = Some(Notification {
                                    message,
                                    level: NotificationLevel::Success,
                                    timestamp: std::time::Instant::now(),
                                });
                            },
                            Err(message) => {
                                self.notification = Some(Notification {
                                    message,
                                    level: NotificationLevel::Error,
                                    timestamp: std::time::Instant::now(),
                                });
                            }
                        }
                    }
                }
            }
        }

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        let visuals = Visuals::dark();  // Utiliser uniquement le thème sombre
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
                ui.horizontal_wrapped(|ui| {  // Changed to horizontal_wrapped
                    // Navigation buttons group
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            if ui.add_enabled(
                                self.history_index > 0,
                                egui::Button::new("←").fill(Color32::from_rgb(66, 133, 244))
                            ).clicked() {
                                self.go_back();
                            }

                            if ui.add_enabled(
                                self.history_index + 1 < self.history.len(),
                                egui::Button::new("→").fill(Color32::from_rgb(66, 133, 244))
                            ).clicked() {
                                self.go_forward();
                            }

                            if ui.button("↑").clicked() {
                                // Clone the parent path before the mutable borrow
                                if let Some(parent) = self.current_dir.parent().map(|p| p.to_path_buf()) {
                                    self.navigate_to(&parent);
                                }
                            }
                        });
                    });

                    ui.add_space(10.0);

                    // Mode selection group
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.selectable_value(&mut self.mode_compress, true,
                                RichText::new("📦 Compresser").size(16.0));
                            ui.selectable_value(&mut self.mode_compress, false,
                                RichText::new("📂 Extraire").size(16.0));
                        });
                    });

                    ui.add_space(10.0);

                    // Preset selector group
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Preset:").size(16.0));
                            egui::ComboBox::new("preset_selector", "")
                                .selected_text(self.preset.label())
                                .show_ui(ui, |ui| {
                                    for preset in CompressionPreset::all() {
                                        ui.selectable_value(
                                            &mut self.preset,
                                            preset.clone(),
                                            preset.label()
                                        );
                                    }
                                });
                        });
                    });
                });
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
                            self.log_lines.push(format!("Dossier sélectionné : {}\n", self.current_dir.display()));
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

                    // vers la ? 	// Extension selector
egui::ComboBox::from_label(RichText::new("📑 Extension").size(16.0))
.selected_text(format!(".{}", self.output_path.extension()
    .and_then(|e| e.to_str())
    .unwrap_or("arc")))
.show_ui(ui, |ui| {
    let extensions = ["arc", "bin", "doi", "bbv", "pak", "sfx"];
    for ext in extensions.iter() {
        // Calculate the potential path *before* calling selectable_value
        let target_path = self.output_path.with_extension(ext);

        // Now call selectable_value, passing the pre-calculated path
        // This doesn't require an immutable borrow of self.output_path inside the call
        if ui.selectable_value(
            &mut self.output_path, // Mutable borrow
            target_path,           // The target value (a PathBuf)
            format!(".{}", ext)    // The label to display
        ).clicked() {
            // The selectable_value function already updates self.output_path
            // to target_path if it was clicked and different,
            // so you don't need this line anymore:
            // self.output_path = self.output_path.with_extension(ext);
        }
    }
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
                            egui::TextEdit::multiline(&mut self.log_lines.join("\n"))
                                .desired_rows(12)
                                .desired_width(f32::INFINITY)
                                .font(egui::TextStyle::Monospace)
                                .interactive(false)
                        );
                    });

                // Progress bar
                self.show_progress(ui);

                // Notification
                self.show_notification(ui);
            });

        // Status bar
        egui::TopBottomPanel::bottom("status_bar")
            .frame(Frame::default()
                .fill(Color32::from_rgb(28, 29, 32))
                .corner_radius(8.0)
                .outer_margin(10.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.label(RichText::new("⚡ Mode:").strong());
                        ui.label(if self.mode_compress {"Compression"} else {"Extraction"});
                        ui.separator();
                        ui.label(RichText::new("📊 Sélection:").strong());
                        ui.label(format!("{} fichier(s)", self.selected.len()));
                    });
                });
            });

        // Ajouter l'affichage de la fenêtre de hash
        self.show_hash_window(ctx);
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
        ui.add(egui::ProgressBar::new(self.progress_value)
            .text(format!("{:.1}%", self.progress_value * 100.0)));
    }

    fn show_notification(&mut self, ui: &mut egui::Ui) {
        if let Some(notification) = &self.notification {
            let age = notification.timestamp.elapsed();
            if age.as_secs() < 5 {
                let (text_color, bg_color) = match notification.level {
                    NotificationLevel::Info => (Color32::WHITE, Color32::from_rgb(66, 133, 244)),
                    NotificationLevel::Warning => (Color32::BLACK, Color32::from_rgb(251, 188, 4)),
                    NotificationLevel::Error => (Color32::WHITE, Color32::from_rgb(234, 67, 53)),
                    NotificationLevel::Success => (Color32::WHITE, Color32::from_rgb(52, 168, 83)),
                };

                egui::Window::new("Notification")
                    .fixed_pos(ui.available_rect_before_wrap().right_top())
                    .frame(Frame::new()
                        .fill(bg_color)
                        .corner_radius(8.0)
                        .outer_margin(8.0))
                    .show(ui.ctx(), |ui| {
                        ui.colored_label(text_color, &notification.message);
                    });
            }
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

    fn calculate_directory_size(&self, path: &Path) -> u64 {
        let mut total_size = 0;
        if path.is_file() {
            if let Ok(metadata) = fs::metadata(path) {
                return metadata.len();
            }
        } else if path.is_dir() {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    total_size += self.calculate_directory_size(&entry.path());
                }
            }
        }
        total_size
    }

    fn calculate_file_hash(path: &Path) -> Result<(HashType, String), String> {
        if !path.is_file() {
            return Err("Le fichier spécifié n'existe pas".to_string());
        }

        match std::fs::read(path) {
            Ok(data) => {
                // CRC32 (pas besoin de Digest)
                let crc = crc32fast::hash(&data);
                Ok((HashType::CRC32, format!("{:08X}", crc)))
            }
            Err(e) => Err(format!("Erreur lors de la lecture du fichier: {}", e))
        }
    }

    fn show_hash_window(&mut self, ctx: &egui::Context) {
        if self.show_hash_window {
            let mut show = true;
            egui::Window::new("Calculateur de Hash")
                .open(&mut show)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.heading("Sélectionnez le type de hash");

                    ui.horizontal(|ui| {
                        if ui.selectable_label(self.selected_hash_type == HashType::CRC32, "CRC32").clicked() {
                            self.selected_hash_type = HashType::CRC32;
                        }
                        if ui.selectable_label(self.selected_hash_type == HashType::Blake3, "BLAKE3").clicked() {
                            self.selected_hash_type = HashType::Blake3;
                        }
                        if ui.selectable_label(self.selected_hash_type == HashType::MD5, "MD5").clicked() {
                            self.selected_hash_type = HashType::MD5;
                        }
                        if ui.selectable_label(self.selected_hash_type == HashType::SHA256, "SHA-256").clicked() {
                            self.selected_hash_type = HashType::SHA256;
                        }
                        if ui.selectable_label(self.selected_hash_type == HashType::SHA3_256, "SHA3-256").clicked() {
                            self.selected_hash_type = HashType::SHA3_256;
                        }
                    });

                    ui.add_space(10.0);

                    let selected_file = self.selected.first().cloned();
                    if let Some(file_path) = selected_file {
                        ui.label(format!("Fichier sélectionné : {}", file_path.display()));
                        if ui.button("Calculer le hash").clicked() {
                            match Self::calculate_file_hash(&file_path) {
                                Ok((hash_type, hash)) => {
                                    self.hash_result = Some(hash);
                                    self.notification = Some(Notification {
                                        message: format!("Hash {} calculé avec succès", hash_type.label()),
                                        level: NotificationLevel::Success,
                                        timestamp: std::time::Instant::now(),
                                    });
                                }
                                Err(e) => {
                                    self.notification = Some(Notification {
                                        message: e,
                                        level: NotificationLevel::Error,
                                        timestamp: std::time::Instant::now(),
                                    });
                                }
                            }
                        }
                    } else {
                        ui.label("Aucun fichier sélectionné");
                    }

                    if let Some(hash) = &self.hash_result {
                        ui.add_space(10.0);
                        ui.group(|ui| {
                            ui.label(format!("{} :", self.selected_hash_type.label()));
                            let _text_response = ui.add(
                                egui::TextEdit::singleline(&mut hash.as_str())
                                    .desired_width(f32::INFINITY)
                                    .font(egui::TextStyle::Monospace)
                            );

                            if ui.button("Copier").clicked() {
                                ctx.copy_text(hash.clone());
                                self.notification = Some(Notification {
                                    message: "Hash copié dans le presse-papiers".to_string(),
                                    level: NotificationLevel::Info,
                                    timestamp: std::time::Instant::now(),
                                });
                            }
                        });
                    }
                });
            self.show_hash_window = show;
        }
    }

    fn show_context_menu(&mut self, ui: &mut egui::Ui, path: &Path) {
        let response = ui.interact(
            ui.min_rect(),
            ui.id().with("context_menu"),
            egui::Sense::click(),
        );

        if response.secondary_clicked() {
            let rect = response.rect;
            let _menu = MenuState::new(pos2(rect.left(), rect.bottom()));
            if path.is_file() {
                if ui.button("Calculer le hash...").clicked() {
                    self.selected = vec![path.to_path_buf()];
                    self.show_hash_window = true;
                }
            }
        }
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
            "--hash" => {
                if args.len() > 2 {
                    let path = PathBuf::from(&args[2]);
                    if !path.exists() {
                        eprintln!("Erreur : Le fichier n'existe pas");
                        return Ok(());
                    }

                    // Récupérer le type de hash demandé
                    let hash_type = if args.len() > 4 && args[3] == "--type" {
                        match args[4].as_str() {
                            "crc32" => vec![HashType::CRC32],
                            "blake3" => vec![HashType::Blake3],
                            "md5" => vec![HashType::MD5],
                            "sha256" => vec![HashType::SHA256],
                            "sha3" => vec![HashType::SHA3_256],
                            "all" => HashType::all().to_vec(),
                            _ => {
                                eprintln!("Type de hash non reconnu");
                                return Ok(());
                            }
                        }
                    } else {
                        HashType::all().to_vec()
                    };

                    if let Ok(data) = std::fs::read(&path) {
                        println!("Calcul du hash pour : {}", path.display());
                        println!("----------------------------------------");

                        // Ne calculer que le hash sélectionné
                        for hash_type in hash_type.iter() {
                            match hash_type {
                                HashType::CRC32 => {
                                    let crc = crc32fast::hash(&data);
                                    println!("CRC32    : {:08X}", crc);
                                },
                                HashType::Blake3 => {
                                    let hash = blake3::hash(&data);
                                    println!("BLAKE3   : {}", hash.to_hex());
                                },
                                HashType::MD5 => {
                                    let mut hasher = md5::Md5::new();
                                    hasher.update(&data);
                                    println!("MD5      : {:x}", hasher.finalize());
                                },
                                HashType::SHA256 => {
                                    let mut hasher = sha2::Sha256::new();
                                    hasher.update(&data);
                                    println!("SHA-256  : {:x}", hasher.finalize());
                                },
                                HashType::SHA3_256 => {
                                    let mut hasher = sha3::Sha3_256::new();
                                    hasher.update(&data);
                                    println!("SHA3-256 : {:x}", hasher.finalize());
                                }
                            }
                        }

                        println!("----------------------------------------");
                        println!("Appuyez sur une touche pour continuer...");
                        let mut input = String::new();
                        let _ = std::io::stdin().read_line(&mut input);
                    } else {
                        eprintln!("Erreur : Impossible de lire le fichier");
                    }
                    return Ok(());
                }
            }
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
        "stelarc V2",
        native_options,
        Box::new(|_creation_context| Ok(Box::new(MonCompresseurApp::default()))),
    )
}

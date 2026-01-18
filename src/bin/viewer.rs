use eframe::egui;
use image::DynamicImage;
use std::path::PathBuf;
use wk_format::{WkDecoder, WkMetadata};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("WK Image Viewer"),
        ..Default::default()
    };

    eframe::run_native(
        "WK Image Viewer",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(WkViewerApp::default()))
        }),
    )
}

#[derive(Default)]
struct WkViewerApp {
    image: Option<DynamicImage>,
    metadata: Option<WkMetadata>,
    file_path: Option<PathBuf>,
    file_size: Option<u64>,
    texture: Option<egui::TextureHandle>,
    error_message: Option<String>,
    success_message: Option<String>,
    show_metadata: bool,
}

impl WkViewerApp {
    fn load_wk_file(&mut self, path: PathBuf, ctx: &egui::Context) {
        self.error_message = None;
        self.success_message = None;

        match self.try_load_wk_file(&path) {
            Ok((image, metadata, file_size)) => {
                // Convert image to texture
                let size = [image.width() as usize, image.height() as usize];
                let rgba_image = image.to_rgba8();
                let pixels = rgba_image.as_flat_samples();

                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

                self.texture = Some(ctx.load_texture(
                    "wk-image",
                    color_image,
                    egui::TextureOptions::default(),
                ));

                self.image = Some(image);
                self.metadata = Some(metadata);
                self.file_path = Some(path.clone());
                self.file_size = Some(file_size);
                self.success_message = Some(format!("Loaded: {}", path.display()));
                self.show_metadata = true;
            }
            Err(e) => {
                self.error_message = Some(format!("Error loading file: {}", e));
            }
        }
    }

    fn try_load_wk_file(&self, path: &PathBuf) -> Result<(DynamicImage, WkMetadata, u64), String> {
        let file_size = std::fs::metadata(path)
            .map_err(|e| format!("Failed to get file size: {}", e))?
            .len();

        let mut file =
            std::fs::File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;

        let decoder = WkDecoder::new();
        let (image, metadata) = decoder
            .decode(&mut file)
            .map_err(|e| format!("Failed to decode WK file: {}", e))?;

        Ok((image, metadata, file_size))
    }

    fn save_as_png(&self) {
        if let Some(image) = &self.image {
            if let Some(result) = rfd::FileDialog::new()
                .add_filter("PNG Image", &["png"])
                .set_file_name("output.png")
                .save_file()
            {
                if let Err(e) = image.save(&result) {
                    eprintln!("Error saving PNG: {}", e);
                }
            }
        }
    }

    fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

impl eframe::App for WkViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading("🖼️ WK Image Viewer");
                ui.separator();

                if ui.button("📁 Open WK File").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("WK Image", &["wk"])
                        .pick_file()
                    {
                        self.load_wk_file(path, ctx);
                    }
                }

                ui.add_enabled_ui(self.image.is_some(), |ui| {
                    if ui.button("💾 Save as PNG").clicked() {
                        self.save_as_png();
                    }

                    if ui.button("🗑️ Clear").clicked() {
                        *self = Self::default();
                    }
                });

                ui.separator();
                ui.checkbox(&mut self.show_metadata, "📋 Show Info");
            });
            ui.add_space(4.0);
        });

        // Error/Success messages
        if let Some(error) = &self.error_message {
            egui::TopBottomPanel::top("error_panel").show(ctx, |ui| {
                ui.colored_label(egui::Color32::RED, format!("❌ {}", error));
            });
        }

        if let Some(success) = &self.success_message {
            egui::TopBottomPanel::top("success_panel").show(ctx, |ui| {
                ui.colored_label(egui::Color32::GREEN, format!("✓ {}", success));
            });
        }

        // Metadata side panel
        if self.show_metadata && self.image.is_some() {
            egui::SidePanel::right("info_panel")
                .default_width(300.0)
                .show(ctx, |ui| {
                    ui.heading("📊 Image Information");
                    ui.separator();

                    if let Some(image) = &self.image {
                        egui::Grid::new("image_info")
                            .num_columns(2)
                            .spacing([10.0, 8.0])
                            .show(ui, |ui| {
                                ui.label("Width:");
                                ui.label(format!("{} px", image.width()));
                                ui.end_row();

                                ui.label("Height:");
                                ui.label(format!("{} px", image.height()));
                                ui.end_row();

                                ui.label("Color Type:");
                                ui.label(format!("{:?}", image.color()));
                                ui.end_row();
                            });
                    }

                    if let Some(file_size) = self.file_size {
                        ui.separator();
                        ui.heading("📦 File Information");
                        ui.separator();

                        egui::Grid::new("file_info")
                            .num_columns(2)
                            .spacing([10.0, 8.0])
                            .show(ui, |ui| {
                                ui.label("File Size:");
                                ui.label(Self::format_bytes(file_size));
                                ui.end_row();

                                if let Some(path) = &self.file_path {
                                    ui.label("File Name:");
                                    if let Some(name) = path.file_name() {
                                        ui.label(name.to_string_lossy().to_string());
                                    }
                                    ui.end_row();
                                }
                            });
                    }

                    if let Some(metadata) = &self.metadata {
                        ui.separator();
                        ui.heading("📝 Metadata");
                        ui.separator();

                        egui::Grid::new("metadata")
                            .num_columns(2)
                            .spacing([10.0, 8.0])
                            .show(ui, |ui| {
                                if let Some(created) = &metadata.created_at {
                                    ui.label("Created:");
                                    ui.label(created);
                                    ui.end_row();
                                }

                                if let Some(software) = &metadata.software {
                                    ui.label("Software:");
                                    ui.label(software);
                                    ui.end_row();
                                }

                                if let Some(author) = &metadata.author {
                                    ui.label("Author:");
                                    ui.label(author);
                                    ui.end_row();
                                }

                                if let Some(desc) = &metadata.description {
                                    ui.label("Description:");
                                    ui.label(desc);
                                    ui.end_row();
                                }
                            });

                        if !metadata.custom_fields.is_empty() {
                            ui.separator();
                            ui.label("Custom Fields:");
                            for (key, value) in &metadata.custom_fields {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{}:", key));
                                    ui.label(value);
                                });
                            }
                        }
                    }
                });
        }

        // Central panel with image
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(texture) = &self.texture {
                ui.centered_and_justified(|ui| {
                    let available_size = ui.available_size();
                    let image_size = texture.size_vec2();

                    // Calculate scale to fit
                    let scale = (available_size.x / image_size.x)
                        .min(available_size.y / image_size.y)
                        .min(1.0);

                    let display_size = image_size * scale;

                    ui.image((texture.id(), display_size));
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(50.0);
                        ui.heading("📁 No image loaded");
                        ui.add_space(10.0);
                        ui.label("Click 'Open WK File' to load an image");
                        ui.add_space(20.0);

                        if ui.button("📂 Browse Files").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("WK Image", &["wk"])
                                .pick_file()
                            {
                                self.load_wk_file(path, ctx);
                            }
                        }
                    });
                });
            }
        });

        // Handle file drop
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                if let Some(file) = i.raw.dropped_files.first() {
                    if let Some(path) = &file.path {
                        if path.extension().and_then(|s| s.to_str()) == Some("wk") {
                            self.load_wk_file(path.clone(), ctx);
                        } else {
                            self.error_message = Some("Please drop a .wk file".to_string());
                        }
                    }
                }
            }
        });
    }
}

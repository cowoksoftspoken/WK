#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::path::PathBuf;
use wk_format::metadata::exif::ExifBuilder;
use wk_format::metadata::icc::IccProfile;
use wk_format::{WkDecoder, WkEncoder, WkMetadata, WkResult};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 750.0])
            .with_title("WK Image Viewer v2.0"),
        ..Default::default()
    };

    eframe::run_native(
        "WK Viewer",
        options,
        Box::new(|cc| Ok(Box::new(WkViewerApp::new(cc)))),
    )
}

struct WkViewerApp {
    current_image: Option<LoadedImage>,
    error_message: Option<String>,
    success_message: Option<String>,
    dropped_file: Option<PathBuf>,
    convert_quality: u8,
    convert_lossless: bool,
    pending_convert: bool,
}

struct LoadedImage {
    texture: egui::TextureHandle,
    width: u32,
    height: u32,
    file_name: String,
    file_path: PathBuf,
    file_size: u64,
    color_type: String,
    compression: String,
    quality: u8,
    is_wk: bool,
}

impl WkViewerApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            current_image: None,
            error_message: None,
            success_message: None,
            dropped_file: None,
            convert_quality: 85,
            convert_lossless: false,
            pending_convert: false,
        }
    }

    fn load_file(&mut self, path: &std::path::Path, ctx: &egui::Context) {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        if ext.eq_ignore_ascii_case("wk") {
            self.load_wk_file(path, ctx);
        } else {
            self.load_other_format(path, ctx);
        }
    }

    fn load_wk_file(&mut self, path: &std::path::Path, ctx: &egui::Context) {
        match self.try_load_wk(path, ctx) {
            Ok(img) => {
                self.current_image = Some(img);
                self.error_message = None;
                self.success_message = Some("WK file loaded!".to_string());
            }
            Err(e) => {
                self.error_message = Some(format!("Error loading WK: {}", e));
                self.current_image = None;
            }
        }
    }

    fn load_other_format(&mut self, path: &std::path::Path, ctx: &egui::Context) {
        match image::open(path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let pixels = rgba.as_flat_samples();

                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                let texture = ctx.load_texture(
                    format!("image_{}", path.display()),
                    color_image,
                    egui::TextureOptions::LINEAR,
                );

                let file_name = path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();

                let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("unknown");

                self.current_image = Some(LoadedImage {
                    texture,
                    width: img.width(),
                    height: img.height(),
                    file_name,
                    file_path: path.to_path_buf(),
                    file_size,
                    color_type: format!("{} image", ext.to_uppercase()),
                    compression: "N/A".to_string(),
                    quality: 0,
                    is_wk: false,
                });
                self.error_message = None;
                self.success_message =
                    Some(format!("{} loaded - Ready to convert!", ext.to_uppercase()));
            }
            Err(e) => {
                self.error_message = Some(format!("Error loading image: {}", e));
            }
        }
    }

    fn try_load_wk(&self, path: &std::path::Path, ctx: &egui::Context) -> WkResult<LoadedImage> {
        let file = std::fs::File::open(path)?;
        let file_size = file.metadata()?.len();
        let decoder = WkDecoder::new();
        let decoded = decoder.decode(std::io::BufReader::new(file))?;

        let rgba = decoded.image.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        let pixels = rgba.as_flat_samples();

        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
        let texture = ctx.load_texture(
            format!("wk_image_{}", path.display()),
            color_image,
            egui::TextureOptions::LINEAR,
        );

        let file_name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        Ok(LoadedImage {
            texture,
            width: decoded.header.width,
            height: decoded.header.height,
            file_name,
            file_path: path.to_path_buf(),
            file_size,
            color_type: format!("{:?}", decoded.header.color_type),
            compression: format!("{:?}", decoded.header.compression_mode),
            quality: decoded.header.quality,
            is_wk: true,
        })
    }

    fn do_convert(
        input: &std::path::Path,
        output: &std::path::Path,
        quality: u8,
        lossless: bool,
    ) -> WkResult<u64> {
        let img = image::open(input)?;

        let exif = ExifBuilder::new().software("WK Viewer v2.0").build();

        let metadata = WkMetadata::new()
            .with_exif(exif)
            .with_icc(IccProfile::srgb());

        let use_lossless = lossless || quality >= 100;

        let encoder = if use_lossless {
            WkEncoder::lossless().with_metadata(metadata)
        } else {
            WkEncoder::lossy(quality).with_metadata(metadata)
        };

        let mut file = std::fs::File::create(output)?;
        encoder.encode(&img, &mut file)?;

        Ok(std::fs::metadata(output)?.len())
    }
}

impl eframe::App for WkViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(path) = self.dropped_file.take() {
            self.load_file(&path, ctx);
        }

        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                if let Some(path) = i.raw.dropped_files[0].path.clone() {
                    self.dropped_file = Some(path);
                }
            }
        });

        if self.pending_convert {
            self.pending_convert = false;

            if let Some(ref img) = self.current_image {
                if !img.is_wk {
                    let source_path = img.file_path.clone();
                    let file_size = img.file_size;
                    let mut output_path = source_path.clone();
                    output_path.set_extension("wk");

                    if let Some(path) = rfd::FileDialog::new()
                        .set_file_name(output_path.file_name().unwrap().to_string_lossy().as_ref())
                        .add_filter("WK Image", &["wk"])
                        .save_file()
                    {
                        let quality = self.convert_quality;
                        let lossless = self.convert_lossless;

                        match Self::do_convert(&source_path, &path, quality, lossless) {
                            Ok(output_size) => {
                                let ratio = output_size as f64 / file_size as f64 * 100.0;
                                self.success_message = Some(format!(
                                    "✓ Converted! {} → {} bytes ({:.1}%)",
                                    file_size, output_size, ratio
                                ));
                                self.load_file(&path, ctx);
                            }
                            Err(e) => {
                                self.error_message = Some(format!("Conversion failed: {}", e));
                            }
                        }
                    }
                }
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🖼️ WK Image Viewer");
                ui.separator();
                ui.label("v2.0");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("📂 Open").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter(
                                "Images",
                                &["wk", "png", "jpg", "jpeg", "webp", "bmp", "gif", "tiff"],
                            )
                            .add_filter("WK Image", &["wk"])
                            .pick_file()
                        {
                            self.load_file(&path, ctx);
                        }
                    }

                    if self.current_image.is_some() {
                        if ui.button("🗑️ Clear").clicked() {
                            self.current_image = None;
                            self.success_message = None;
                        }
                    }
                });
            });
        });

        let mut should_convert = false;

        if let Some(ref img) = self.current_image {
            egui::SidePanel::right("info_panel")
                .resizable(true)
                .min_width(220.0)
                .show(ctx, |ui| {
                    ui.heading("📊 Image Info");
                    ui.separator();

                    egui::Grid::new("info_grid")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("File:");
                            ui.label(&img.file_name);
                            ui.end_row();

                            ui.label("Size:");
                            ui.label(format!("{} bytes", img.file_size));
                            ui.end_row();

                            ui.label("Dimensions:");
                            ui.label(format!("{}x{}", img.width, img.height));
                            ui.end_row();

                            ui.label("Format:");
                            ui.label(&img.color_type);
                            ui.end_row();

                            if img.is_wk {
                                ui.label("Compression:");
                                ui.label(&img.compression);
                                ui.end_row();

                                ui.label("Quality:");
                                ui.label(format!("{}", img.quality));
                                ui.end_row();
                            }
                        });

                    if !img.is_wk {
                        ui.add_space(20.0);
                        ui.separator();
                        ui.heading("🔄 Convert to WK");
                        ui.add_space(5.0);

                        ui.horizontal(|ui| {
                            ui.checkbox(&mut self.convert_lossless, "Lossless");
                        });

                        if !self.convert_lossless {
                            ui.horizontal(|ui| {
                                ui.label("Quality:");
                                ui.add(egui::Slider::new(&mut self.convert_quality, 1..=100));
                            });
                        }

                        ui.add_space(10.0);

                        if ui.button("⚡ Convert to .wk").clicked() {
                            should_convert = true;
                        }
                    }

                    ui.add_space(20.0);
                    ui.separator();
                    ui.heading("ℹ️ About WK");
                    ui.label("• Predictive Compression");
                    ui.label("• 8x8 DCT Transform");
                    ui.label("• Adaptive Quantization");
                    ui.label("• EXIF/ICC/XMP Metadata");
                    ui.label("• Animation Support");
                });
        }

        if should_convert {
            self.pending_convert = true;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(ref error) = self.error_message {
                ui.colored_label(egui::Color32::RED, error);
            }
            if let Some(ref success) = self.success_message {
                ui.colored_label(egui::Color32::GREEN, success);
            }

            if let Some(ref img) = self.current_image {
                egui::ScrollArea::both().show(ui, |ui| {
                    ui.centered_and_justified(|ui| {
                        ui.image(&img.texture);
                    });
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(100.0);
                        ui.heading("📁 Drop image here");
                        ui.label("Supports: WK, PNG, JPEG, WebP, BMP, GIF, TIFF");
                        ui.add_space(10.0);
                        ui.label("Drop any image to convert to WK format");
                    });
                });
            }
        });
    }
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::path::PathBuf;
use std::time::Instant;
use wk_format::metadata::exif::ExifBuilder;
use wk_format::metadata::icc::IccProfile;
use wk_format::{WkDecoder, WkEncoder, WkMetadata, WkResult};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 900.0])
            .with_title("WK Image Viewer v3.0 Pro"),
        ..Default::default()
    };
    eframe::run_native(
        "WK Viewer",
        options,
        Box::new(|cc| Ok(Box::new(WkViewerApp::new(cc)))),
    )
}

#[derive(Clone, Copy, PartialEq)]
enum ViewMode {
    Single,
    Batch,
}

struct WkViewerApp {
    current_image: Option<LoadedImage>,
    batch_files: Vec<PathBuf>,
    batch_progress: usize,
    batch_total: usize,
    error_message: Option<String>,
    success_message: Option<String>,
    dropped_file: Option<PathBuf>,
    convert_quality: u8,
    convert_lossless: bool,
    pending_convert: bool,
    zoom: f32,
    pan_offset: egui::Vec2,
    show_histogram: bool,
    histogram_data: Option<HistogramData>,
    decode_time_ms: f64,
    view_mode: ViewMode,
    use_cabac: bool,
    use_intra_prediction: bool,
    use_adaptive_quant: bool,
    exif_camera: String,
    exif_software: String,
    show_exif_editor: bool,
    show_stats: bool,
    fps: f32,
    last_frame_time: Instant,
    frame_count: u32,
}

#[derive(Clone)]
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

#[derive(Clone)]
struct HistogramData {
    red: [u32; 256],
    green: [u32; 256],
    blue: [u32; 256],
}

impl HistogramData {
    fn from_rgba(pixels: &[u8]) -> Self {
        let mut red = [0u32; 256];
        let mut green = [0u32; 256];
        let mut blue = [0u32; 256];
        for chunk in pixels.chunks(4) {
            if chunk.len() >= 3 {
                red[chunk[0] as usize] += 1;
                green[chunk[1] as usize] += 1;
                blue[chunk[2] as usize] += 1;
            }
        }
        Self { red, green, blue }
    }
    fn max_value(&self) -> u32 {
        self.red
            .iter()
            .chain(self.green.iter())
            .chain(self.blue.iter())
            .copied()
            .max()
            .unwrap_or(1)
            .max(1)
    }
}

fn draw_histogram(ui: &mut egui::Ui, hist: &HistogramData) {
    let max_val = hist.max_value() as f32;
    let height = 80.0;
    let (response, painter) = ui.allocate_painter(
        egui::vec2(ui.available_width(), height),
        egui::Sense::hover(),
    );
    let rect = response.rect;
    painter.rect_filled(rect, 4.0, egui::Color32::from_gray(15));
    let bar_width = rect.width() / 256.0;
    for i in 0..256 {
        let x = rect.left() + i as f32 * bar_width;
        let r_h = (hist.red[i] as f32 / max_val) * height * 0.9;
        let g_h = (hist.green[i] as f32 / max_val) * height * 0.9;
        let b_h = (hist.blue[i] as f32 / max_val) * height * 0.9;
        painter.line_segment(
            [
                egui::pos2(x, rect.bottom()),
                egui::pos2(x, rect.bottom() - r_h),
            ],
            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 50, 50, 120)),
        );
        painter.line_segment(
            [
                egui::pos2(x, rect.bottom()),
                egui::pos2(x, rect.bottom() - g_h),
            ],
            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(50, 255, 50, 120)),
        );
        painter.line_segment(
            [
                egui::pos2(x, rect.bottom()),
                egui::pos2(x, rect.bottom() - b_h),
            ],
            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(50, 50, 255, 120)),
        );
    }
}

fn do_convert(
    input: &std::path::Path,
    output: &std::path::Path,
    quality: u8,
    lossless: bool,
    camera: &str,
    software: &str,
) -> WkResult<(u64, f64)> {
    let start = Instant::now();
    let img = image::open(input)?;
    let mut exif = ExifBuilder::new();
    if !camera.is_empty() {
        exif = exif.make(camera);
    }
    if !software.is_empty() {
        exif = exif.software(software);
    }
    let metadata = WkMetadata::new()
        .with_exif(exif.build())
        .with_icc(IccProfile::srgb());
    let encoder = if lossless || quality >= 100 {
        WkEncoder::lossless().with_metadata(metadata)
    } else {
        WkEncoder::lossy(quality).with_metadata(metadata)
    };
    let mut file = std::fs::File::create(output)?;
    encoder.encode(&img, &mut file)?;
    Ok((
        std::fs::metadata(output)?.len(),
        start.elapsed().as_secs_f64() * 1000.0,
    ))
}

impl WkViewerApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            current_image: None,
            batch_files: Vec::new(),
            batch_progress: 0,
            batch_total: 0,
            error_message: None,
            success_message: None,
            dropped_file: None,
            convert_quality: 85,
            convert_lossless: false,
            pending_convert: false,
            zoom: 1.0,
            pan_offset: egui::Vec2::ZERO,
            show_histogram: false,
            histogram_data: None,
            decode_time_ms: 0.0,
            view_mode: ViewMode::Single,
            use_cabac: true,
            use_intra_prediction: true,
            use_adaptive_quant: true,
            exif_camera: String::new(),
            exif_software: "WK Viewer v3.0".to_string(),
            show_exif_editor: false,
            show_stats: true,
            fps: 0.0,
            last_frame_time: Instant::now(),
            frame_count: 0,
        }
    }

    fn load_file(&mut self, path: &std::path::Path, ctx: &egui::Context) {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext.eq_ignore_ascii_case("wk") {
            self.load_wk_file(path, ctx);
        } else {
            self.load_other_format(path, ctx);
        }
        self.zoom = 1.0;
        self.pan_offset = egui::Vec2::ZERO;
    }

    fn load_wk_file(&mut self, path: &std::path::Path, ctx: &egui::Context) {
        let start = Instant::now();
        match std::fs::File::open(path) {
            Ok(file) => {
                let file_size = file.metadata().map(|m| m.len()).unwrap_or(0);
                match WkDecoder::new().decode(std::io::BufReader::new(file)) {
                    Ok(decoded) => {
                        self.decode_time_ms = start.elapsed().as_secs_f64() * 1000.0;
                        let rgba = decoded.image.to_rgba8();
                        let raw = rgba.as_raw();
                        self.histogram_data = Some(HistogramData::from_rgba(raw));
                        let size = [rgba.width() as usize, rgba.height() as usize];
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, raw);
                        let texture = ctx.load_texture(
                            format!("wk_{}", path.display()),
                            color_image,
                            egui::TextureOptions::LINEAR,
                        );
                        self.current_image = Some(LoadedImage {
                            texture,
                            width: decoded.header.width,
                            height: decoded.header.height,
                            file_name: path
                                .file_name()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_default(),
                            file_path: path.to_path_buf(),
                            file_size,
                            color_type: format!("{:?}", decoded.header.color_type),
                            compression: format!("{:?}", decoded.header.compression_mode),
                            quality: decoded.header.quality,
                            is_wk: true,
                        });
                        self.error_message = None;
                        self.success_message =
                            Some(format!("WK loaded in {:.2}ms", self.decode_time_ms));
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Decode error: {}", e));
                    }
                }
            }
            Err(e) => {
                self.error_message = Some(format!("File error: {}", e));
            }
        }
    }

    fn load_other_format(&mut self, path: &std::path::Path, ctx: &egui::Context) {
        match image::open(path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let raw = rgba.as_raw();
                self.histogram_data = Some(HistogramData::from_rgba(raw));
                let size = [rgba.width() as usize, rgba.height() as usize];
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, raw);
                let texture = ctx.load_texture(
                    format!("img_{}", path.display()),
                    color_image,
                    egui::TextureOptions::LINEAR,
                );
                self.current_image = Some(LoadedImage {
                    texture,
                    width: img.width(),
                    height: img.height(),
                    file_name: path
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    file_path: path.to_path_buf(),
                    file_size: std::fs::metadata(path).map(|m| m.len()).unwrap_or(0),
                    color_type: format!(
                        "{} image",
                        path.extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("")
                            .to_uppercase()
                    ),
                    compression: "N/A".to_string(),
                    quality: 0,
                    is_wk: false,
                });
                self.error_message = None;
                self.success_message = Some("Ready to convert!".to_string());
            }
            Err(e) => {
                self.error_message = Some(format!("Error: {}", e));
            }
        }
    }
}

impl eframe::App for WkViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame_count += 1;
        if self.last_frame_time.elapsed().as_secs_f32() >= 1.0 {
            self.fps = self.frame_count as f32;
            self.frame_count = 0;
            self.last_frame_time = Instant::now();
        }

        if let Some(path) = self.dropped_file.take() {
            self.load_file(&path, ctx);
        }
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                if let Some(path) = i.raw.dropped_files[0].path.clone() {
                    self.dropped_file = Some(path);
                }
            }
            self.zoom = (self.zoom + i.smooth_scroll_delta.y * 0.01).clamp(0.1, 10.0);
        });

        if self.pending_convert {
            self.pending_convert = false;
            if let Some(ref img) = self.current_image.clone() {
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
                        match do_convert(
                            &source_path,
                            &path,
                            self.convert_quality,
                            self.convert_lossless,
                            &self.exif_camera,
                            &self.exif_software,
                        ) {
                            Ok((output_size, encode_time)) => {
                                let ratio = output_size as f64 / file_size as f64 * 100.0;
                                self.success_message = Some(format!(
                                    "✓ Converted in {:.1}ms ({:.1}%)",
                                    encode_time, ratio
                                ));
                                self.load_file(&path, ctx);
                            }
                            Err(e) => {
                                self.error_message = Some(format!("Failed: {}", e));
                            }
                        }
                    }
                }
            }
        }

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🖼️ WK Viewer Pro");
                ui.separator();
                ui.label("v3.0");
                ui.separator();
                ui.selectable_value(&mut self.view_mode, ViewMode::Single, "📷 Single");
                ui.selectable_value(&mut self.view_mode, ViewMode::Batch, "📁 Batch");
                ui.separator();
                if self.show_stats {
                    ui.label(format!(
                        "Zoom: {:.0}% | FPS: {:.0}",
                        self.zoom * 100.0,
                        self.fps
                    ));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("📂 Open").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter(
                                "Images",
                                &["wk", "png", "jpg", "jpeg", "webp", "bmp", "gif", "tiff"],
                            )
                            .pick_file()
                        {
                            self.load_file(&path, ctx);
                        }
                    }
                    if self.current_image.is_some() && ui.button("🗑️ Clear").clicked() {
                        self.current_image = None;
                        self.histogram_data = None;
                        self.success_message = None;
                    }
                    if ui.button("🔄 Reset").clicked() {
                        self.zoom = 1.0;
                        self.pan_offset = egui::Vec2::ZERO;
                    }
                });
            });
        });

        let mut should_convert = false;
        let img_clone = self.current_image.clone();
        let hist_clone = self.histogram_data.clone();

        if let Some(ref img) = img_clone {
            egui::SidePanel::right("info")
                .resizable(true)
                .min_width(280.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
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
                                ui.label(format!("{:.2} KB", img.file_size as f64 / 1024.0));
                                ui.end_row();
                                ui.label("Dimensions:");
                                ui.label(format!("{}×{}", img.width, img.height));
                                ui.end_row();
                                ui.label("Pixels:");
                                ui.label(format!(
                                    "{:.2} MP",
                                    (img.width * img.height) as f64 / 1_000_000.0
                                ));
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
                                    ui.label("Decode:");
                                    ui.label(format!("{:.2}ms", self.decode_time_ms));
                                    ui.end_row();
                                }
                            });

                        ui.add_space(10.0);
                        ui.checkbox(&mut self.show_histogram, "📈 Histogram");
                        if self.show_histogram {
                            if let Some(ref hist) = hist_clone {
                                draw_histogram(ui, hist);
                            }
                        }
                        ui.checkbox(&mut self.show_stats, "📊 Show Stats");

                        if !img.is_wk {
                            ui.add_space(15.0);
                            ui.separator();
                            ui.heading("🔄 Convert to WK");
                            ui.checkbox(&mut self.convert_lossless, "Lossless");
                            if !self.convert_lossless {
                                ui.horizontal(|ui| {
                                    ui.label("Quality:");
                                    ui.add(egui::Slider::new(&mut self.convert_quality, 1..=100));
                                });
                            }
                            ui.collapsing("⚙️ v3.0 Features", |ui| {
                                ui.checkbox(&mut self.use_cabac, "CABAC Entropy");
                                ui.checkbox(&mut self.use_intra_prediction, "Intra Prediction");
                                ui.checkbox(&mut self.use_adaptive_quant, "Adaptive Quantization");
                            });
                            ui.checkbox(&mut self.show_exif_editor, "📸 Edit EXIF");
                            if self.show_exif_editor {
                                ui.horizontal(|ui| {
                                    ui.label("Camera:");
                                    ui.text_edit_singleline(&mut self.exif_camera);
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Software:");
                                    ui.text_edit_singleline(&mut self.exif_software);
                                });
                            }
                            ui.add_space(10.0);
                            if ui.button("⚡ Convert to .wk").clicked() {
                                should_convert = true;
                            }
                        }

                        ui.add_space(15.0);
                        ui.separator();
                        ui.heading("ℹ️ WK v3.0");
                        ui.label("• Multi-block DCT (8×8, 16×16)");
                        ui.label("• CABAC Arithmetic Coding");
                        ui.label("• 11 Intra Prediction Modes");
                        ui.label("• WebP-style Adaptive Quant");
                        ui.label("• HDR (10/12-bit PQ/HLG)");
                        ui.label("• SIMD Acceleration");
                    });
                });
        }
        if should_convert {
            self.pending_convert = true;
        }

        if self.view_mode == ViewMode::Batch {
            egui::SidePanel::left("batch")
                .resizable(true)
                .min_width(200.0)
                .show(ctx, |ui| {
                    ui.heading("📁 Batch Convert");
                    ui.separator();
                    if ui.button("➕ Add Files").clicked() {
                        if let Some(paths) = rfd::FileDialog::new()
                            .add_filter(
                                "Images",
                                &["png", "jpg", "jpeg", "webp", "bmp", "gif", "tiff"],
                            )
                            .pick_files()
                        {
                            self.batch_files.extend(paths);
                        }
                    }
                    ui.label(format!("{} files queued", self.batch_files.len()));
                    if !self.batch_files.is_empty() {
                        egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                for f in &self.batch_files {
                                    ui.label(
                                        f.file_name()
                                            .unwrap_or_default()
                                            .to_string_lossy()
                                            .as_ref(),
                                    );
                                }
                            });
                        if ui.button("🚀 Convert All").clicked() {
                            if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                                self.batch_total = self.batch_files.len();
                                for file in &self.batch_files.clone() {
                                    let mut out = dir.join(file.file_name().unwrap_or_default());
                                    out.set_extension("wk");
                                    let _ = do_convert(
                                        file,
                                        &out,
                                        self.convert_quality,
                                        self.convert_lossless,
                                        &self.exif_camera,
                                        &self.exif_software,
                                    );
                                    self.batch_progress += 1;
                                }
                                self.success_message =
                                    Some(format!("Batch converted {} files!", self.batch_total));
                                self.batch_files.clear();
                            }
                        }
                        if ui.button("🗑️ Clear").clicked() {
                            self.batch_files.clear();
                        }
                    }
                    if self.batch_total > 0 {
                        ui.add(egui::ProgressBar::new(
                            self.batch_progress as f32 / self.batch_total as f32,
                        ));
                    }
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(ref error) = self.error_message {
                ui.colored_label(egui::Color32::RED, error);
            }
            if let Some(ref success) = self.success_message {
                ui.colored_label(egui::Color32::GREEN, success);
            }
            if let Some(ref img) = self.current_image {
                let img_size =
                    egui::vec2(img.width as f32 * self.zoom, img.height as f32 * self.zoom);
                egui::ScrollArea::both().show(ui, |ui| {
                    let response = ui.allocate_response(img_size, egui::Sense::drag());
                    if response.dragged() {
                        self.pan_offset += response.drag_delta();
                    }
                    let rect = response.rect.translate(self.pan_offset);
                    ui.painter().image(
                        img.texture.id(),
                        rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(100.0);
                        ui.heading("📁 Drop image here");
                        ui.label("Supports: WK, PNG, JPEG, WebP, BMP, GIF, TIFF");
                        ui.label("Mouse wheel to zoom, drag to pan");
                        ui.add_space(20.0);
                        ui.label("WK v3.0 Features:");
                        ui.label("• CABAC • Intra-Prediction • Adaptive Quantization");
                    });
                });
            }
        });
    }
}

//! Interface gráfica do processador de imagens (egui + eframe)
//! Suporta imagens coloridas e em escala de cinza.

use eframe::egui;
use image_proc::*;
use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_title("Image Proc — Processador de Imagens (PDI)"),
        ..Default::default()
    };

    eframe::run_native(
        "Image Proc",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(App::default())
        }),
    )
}

struct App {
    // Imagens principais (trabalhamos sempre com RGB; cinza é convertido)
    original: Option<RgbImage>,
    current: Option<RgbImage>,
    background: Option<RgbImage>, // para chroma-key

    texture_original: Option<egui::TextureHandle>,
    texture_current: Option<egui::TextureHandle>,

    // Parâmetros das operações
    threshold_val: u8,
    brightness_val: i16,
    gamma_val: f64,
    kernel_size: usize,
    gaussian_sigma: f64,
    highboost_amp: f64,
    scale_w: u32,
    scale_h: u32,
    rotate_angle: f64,
    pivot_x: f64,
    pivot_y: f64,
    interp_nearest: bool,

    // Cor
    hue_delta: f64,
    sat_delta: f64,
    intensity_delta: f64,
    channel_r: i16,
    channel_g: i16,
    channel_b: i16,
    chroma_distance: f64,
    chroma_key_r: u8,
    chroma_key_g: u8,
    chroma_key_b: u8,

    // Esteganografia
    stego_message: String,
    stego_extracted: String,

    // Histograma
    show_histogram: bool,
    hist_channel: usize, // 0=R, 1=G, 2=B, 3=I
    hist_r: [u32; 256],
    hist_g: [u32; 256],
    hist_b: [u32; 256],
    hist_i: [u32; 256],

    status: String,
    last_path: Option<PathBuf>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            original: None,
            current: None,
            background: None,
            texture_original: None,
            texture_current: None,
            threshold_val: 128,
            brightness_val: 0,
            gamma_val: 1.0,
            kernel_size: 3,
            gaussian_sigma: 1.0,
            highboost_amp: 1.5,
            scale_w: 0,
            scale_h: 0,
            rotate_angle: 0.0,
            pivot_x: 0.0,
            pivot_y: 0.0,
            interp_nearest: true,
            hue_delta: 0.0,
            sat_delta: 0.0,
            intensity_delta: 0.0,
            channel_r: 0,
            channel_g: 0,
            channel_b: 0,
            chroma_distance: 80.0,
            chroma_key_r: 0,
            chroma_key_g: 255,
            chroma_key_b: 0,
            stego_message: String::new(),
            stego_extracted: String::new(),
            show_histogram: false,
            hist_channel: 3,
            hist_r: [0; 256],
            hist_g: [0; 256],
            hist_b: [0; 256],
            hist_i: [0; 256],
            status: String::new(),
            last_path: None,
        }
    }
}

impl App {
    fn load_image(&mut self, path: PathBuf, ctx: &egui::Context) {
        match RgbImage::load(&path) {
            Ok(img) => {
                self.scale_w = img.width;
                self.scale_h = img.height;
                self.pivot_x = img.width as f64 / 2.0;
                self.pivot_y = img.height as f64 / 2.0;
                self.update_histograms(&img);
                self.original = Some(img.clone());
                self.current = Some(img);
                self.last_path = Some(path.clone());
                self.status = format!("Carregado: {}", path.display());
                self.update_textures(ctx);
            }
            Err(e) => {
                self.status = format!("Erro ao carregar: {}", e);
            }
        }
    }

    fn load_background(&mut self, path: PathBuf) {
        match RgbImage::load(&path) {
            Ok(img) => {
                self.background = Some(img);
                self.status = format!("Fundo (chroma-key) carregado: {}", path.display());
            }
            Err(e) => {
                self.status = format!("Erro ao carregar fundo: {}", e);
            }
        }
    }

    fn update_histograms(&mut self, img: &RgbImage) {
        let (hr, hg, hb, hi) = compute_histograms_rgb(img);
        self.hist_r = hr;
        self.hist_g = hg;
        self.hist_b = hb;
        self.hist_i = hi;
    }

    fn update_textures(&mut self, ctx: &egui::Context) {
        if let Some(ref img) = self.original {
            let dyn_img = img.to_dynamic();
            let size = [dyn_img.width() as usize, dyn_img.height() as usize];
            let rgba = dyn_img.to_rgba8();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
            self.texture_original =
                Some(ctx.load_texture("original", color_image, egui::TextureOptions::NEAREST));
        }
        // Clona a referência necessária antes de chamar update_histograms
        // (evita borrow imutável + mutável ao mesmo tempo)
        let current_for_hist = self.current.clone();
        if let Some(ref img) = self.current {
            let dyn_img = img.to_dynamic();
            let size = [dyn_img.width() as usize, dyn_img.height() as usize];
            let rgba = dyn_img.to_rgba8();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
            self.texture_current =
                Some(ctx.load_texture("current", color_image, egui::TextureOptions::NEAREST));
        }
        if let Some(ref img) = current_for_hist {
            self.update_histograms(img);
        }
    }

    fn apply_op<F>(&mut self, ctx: &egui::Context, op: F)
    where
        F: FnOnce(&RgbImage) -> RgbImage,
    {
        if let Some(ref img) = self.current {
            let result = op(img);
            self.current = Some(result);
            self.update_textures(ctx);
            self.status = "Operação aplicada".into();
        }
    }

    fn reset(&mut self, ctx: &egui::Context) {
        if let Some(ref orig) = self.original {
            self.current = Some(orig.clone());
            self.update_textures(ctx);
            self.status = "Resetado para original".into();
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Defaults dos parâmetros
        if self.threshold_val == 0 && self.gamma_val == 0.0 {
            self.threshold_val = 128;
            self.brightness_val = 0;
            self.gamma_val = 1.0;
            self.kernel_size = 3;
            self.gaussian_sigma = 1.0;
            self.highboost_amp = 1.5;
            self.interp_nearest = true;
            self.chroma_distance = 80.0;
            self.chroma_key_g = 255; // verde padrão
            self.hist_channel = 3; // Intensidade
        }

        egui::SidePanel::left("controls")
            .resizable(true)
            .default_width(340.0)
            .show(ctx, |ui| {
                ui.heading("Image Proc — PDI");
                ui.separator();

                // Carregar / Salvar
                ui.horizontal(|ui| {
                    if ui.button("📂 Carregar").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Imagens", &["png", "jpg", "jpeg", "bmp", "gif", "webp"])
                            .pick_file()
                        {
                            self.load_image(path, ctx);
                        }
                    }
                    if ui.button("💾 Salvar").clicked() {
                        if let Some(ref img) = self.current {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("PNG", &["png"])
                                .add_filter("JPEG", &["jpg", "jpeg"])
                                .add_filter("BMP", &["bmp"])
                                .save_file()
                            {
                                match img.save(&path) {
                                    Ok(_) => self.status = format!("Salvo: {}", path.display()),
                                    Err(e) => self.status = format!("Erro ao salvar: {}", e),
                                }
                            }
                        }
                    }
                    if ui.button("↺ Reset").clicked() {
                        self.reset(ctx);
                    }
                });

                ui.separator();
                ui.label(egui::RichText::new(&self.status).small());

                if self.current.is_none() {
                    ui.label("Carregue uma imagem para começar.");
                    return;
                }

                // ============================================================
                // Operações ponto a ponto
                // ============================================================
                ui.separator();
                ui.heading("Operações Ponto a Ponto");

                ui.horizontal(|ui| {
                    ui.label("Limiar:");
                    ui.add(egui::Slider::new(&mut self.threshold_val, 0..=255));
                    if ui.button("Aplicar").clicked() {
                        let t = self.threshold_val;
                        self.apply_op(ctx, |img| threshold_rgb(img, t));
                    }
                });

                if ui.button("Negativo").clicked() {
                    self.apply_op(ctx, negative_rgb);
                }

                ui.horizontal(|ui| {
                    ui.label("Brilho:");
                    ui.add(egui::Slider::new(&mut self.brightness_val, -255..=255));
                    if ui.button("Aplicar").clicked() {
                        let v = self.brightness_val;
                        self.apply_op(ctx, |img| brightness_rgb(img, v));
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Gama:");
                    ui.add(egui::Slider::new(&mut self.gamma_val, 0.1..=5.0));
                    if ui.button("Aplicar").clicked() {
                        let g = self.gamma_val;
                        self.apply_op(ctx, |img| gamma_rgb(img, g));
                    }
                });

                if ui.button("Linear por partes (contraste)").clicked() {
                    let points = [(0, 0), (64, 32), (192, 223), (255, 255)];
                    self.apply_op(ctx, |img| piecewise_linear_rgb(img, &points));
                }

                // ============================================================
                // Escala de cinza e cor
                // ============================================================
                ui.separator();
                ui.heading("Escala de Cinza / Cor");

                ui.horizontal(|ui| {
                    if ui.button("Cinza (média)").clicked() {
                        self.apply_op(ctx, |img| {
                            let g = img.to_gray_average();
                            RgbImage::from_gray(&g)
                        });
                    }
                    if ui.button("Cinza (ponderada)").clicked() {
                        self.apply_op(ctx, |img| {
                            let g = img.to_gray_weighted();
                            RgbImage::from_gray(&g)
                        });
                    }
                });

                if ui.button("Sépia").clicked() {
                    self.apply_op(ctx, sepia);
                }

                ui.horizontal(|ui| {
                    ui.label("Δ Matiz (°):");
                    ui.add(egui::Slider::new(&mut self.hue_delta, -180.0..=180.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Δ Saturação:");
                    ui.add(egui::Slider::new(&mut self.sat_delta, -1.0..=1.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Δ Intensidade:");
                    ui.add(egui::Slider::new(&mut self.intensity_delta, -1.0..=1.0));
                });
                if ui.button("Ajustar HSI").clicked() {
                    let dh = self.hue_delta;
                    let ds = self.sat_delta;
                    let di = self.intensity_delta;
                    self.apply_op(ctx, |img| adjust_hsi(img, dh, ds, di));
                }

                ui.horizontal(|ui| {
                    ui.label("Canal R:");
                    ui.add(egui::Slider::new(&mut self.channel_r, -255..=255));
                });
                ui.horizontal(|ui| {
                    ui.label("Canal G:");
                    ui.add(egui::Slider::new(&mut self.channel_g, -255..=255));
                });
                ui.horizontal(|ui| {
                    ui.label("Canal B:");
                    ui.add(egui::Slider::new(&mut self.channel_b, -255..=255));
                });
                if ui.button("Ajustar Canais").clicked() {
                    let dr = self.channel_r;
                    let dg = self.channel_g;
                    let db = self.channel_b;
                    self.apply_op(ctx, |img| adjust_channels(img, dr, dg, db));
                }

                // ============================================================
                // Chroma-Key
                // ============================================================
                ui.separator();
                ui.heading("Chroma-Key");

                ui.horizontal(|ui| {
                    ui.label("Chave R:");
                    ui.add(egui::Slider::new(&mut self.chroma_key_r, 0..=255));
                });
                ui.horizontal(|ui| {
                    ui.label("Chave G:");
                    ui.add(egui::Slider::new(&mut self.chroma_key_g, 0..=255));
                });
                ui.horizontal(|ui| {
                    ui.label("Chave B:");
                    ui.add(egui::Slider::new(&mut self.chroma_key_b, 0..=255));
                });
                ui.horizontal(|ui| {
                    ui.label("Distância:");
                    ui.add(egui::Slider::new(&mut self.chroma_distance, 0.0..=441.0));
                });
                if ui.button("Carregar imagem de fundo").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Imagens", &["png", "jpg", "jpeg", "bmp"])
                        .pick_file()
                    {
                        self.load_background(path);
                    }
                }
                if ui.button("Aplicar Chroma-Key").clicked() {
                    if let Some(ref bg) = self.background {
                        let key = (self.chroma_key_r, self.chroma_key_g, self.chroma_key_b);
                        let dist = self.chroma_distance;
                        let bg = bg.clone();
                        self.apply_op(ctx, |img| chroma_key(img, key, dist, &bg));
                    } else {
                        self.status = "Carregue uma imagem de fundo primeiro".into();
                    }
                }

                // ============================================================
                // Histograma
                // ============================================================
                ui.separator();
                ui.heading("Histograma");

                if ui.button("Equalizar (HSI)").clicked() {
                    self.apply_op(ctx, equalize_histogram_color);
                }
                ui.checkbox(&mut self.show_histogram, "Mostrar histograma");
                ui.horizontal(|ui| {
                    ui.label("Canal:");
                    ui.selectable_value(&mut self.hist_channel, 0, "R");
                    ui.selectable_value(&mut self.hist_channel, 1, "G");
                    ui.selectable_value(&mut self.hist_channel, 2, "B");
                    ui.selectable_value(&mut self.hist_channel, 3, "I");
                });

                // ============================================================
                // Filtros / Convolução
                // ============================================================
                ui.separator();
                ui.heading("Filtros / Convolução");

                ui.horizontal(|ui| {
                    ui.label("Tamanho do kernel:");
                    ui.add(egui::Slider::new(&mut self.kernel_size, 3..=15).step_by(2.0));
                });

                ui.horizontal(|ui| {
                    if ui.button("Média").clicked() {
                        let k = self.kernel_size;
                        self.apply_op(ctx, |img| apply_mean_rgb(img, k));
                    }
                    if ui.button("Gaussiano").clicked() {
                        let k = self.kernel_size;
                        let s = self.gaussian_sigma;
                        self.apply_op(ctx, |img| apply_gaussian_rgb(img, k, s));
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("σ Gauss:");
                    ui.add(egui::Slider::new(&mut self.gaussian_sigma, 0.5..=5.0));
                });

                if ui.button("Mediana").clicked() {
                    let k = self.kernel_size;
                    self.apply_op(ctx, |img| median_filter_rgb(img, k));
                }

                ui.horizontal(|ui| {
                    if ui.button("Laplaciano (norm)").clicked() {
                        self.apply_op(ctx, |img| apply_laplacian_rgb(img, true));
                    }
                    if ui.button("Aguçar (Lap)").clicked() {
                        self.apply_op(ctx, sharpen_laplacian_rgb);
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("High-Boost A:");
                    ui.add(egui::Slider::new(&mut self.highboost_amp, 1.0..=3.0));
                    if ui.button("Aplicar").clicked() {
                        let k = self.kernel_size;
                        let a = self.highboost_amp;
                        self.apply_op(ctx, |img| high_boost_rgb(img, k, a));
                    }
                });

                // ============================================================
                // Bordas
                // ============================================================
                ui.separator();
                ui.heading("Bordas");

                ui.horizontal(|ui| {
                    if ui.button("Sobel X").clicked() {
                        self.apply_op(ctx, sobel_x_rgb);
                    }
                    if ui.button("Sobel Y").clicked() {
                        self.apply_op(ctx, sobel_y_rgb);
                    }
                    if ui.button("Magnitude").clicked() {
                        self.apply_op(ctx, gradient_magnitude_rgb);
                    }
                });

                // ============================================================
                // Geometria
                // ============================================================
                ui.separator();
                ui.heading("Geometria");

                ui.horizontal(|ui| {
                    ui.label("Largura:");
                    ui.add(egui::DragValue::new(&mut self.scale_w).speed(1.0));
                    ui.label("Altura:");
                    ui.add(egui::DragValue::new(&mut self.scale_h).speed(1.0));
                });
                ui.checkbox(&mut self.interp_nearest, "Vizinho mais próximo (senão bilinear)");

                if ui.button("Escalar").clicked() {
                    let w = self.scale_w;
                    let h = self.scale_h;
                    let interp = if self.interp_nearest {
                        Interpolation::Nearest
                    } else {
                        Interpolation::Bilinear
                    };
                    self.apply_op(ctx, |img| scale_rgb(img, w, h, interp));
                }

                ui.horizontal(|ui| {
                    ui.label("Ângulo (°):");
                    ui.add(egui::Slider::new(&mut self.rotate_angle, -180.0..=180.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Pivô X:");
                    ui.add(egui::DragValue::new(&mut self.pivot_x));
                    ui.label("Y:");
                    ui.add(egui::DragValue::new(&mut self.pivot_y));
                });
                if ui.button("Rotacionar").clicked() {
                    let angle = self.rotate_angle;
                    let px = self.pivot_x;
                    let py = self.pivot_y;
                    let interp = if self.interp_nearest {
                        Interpolation::Nearest
                    } else {
                        Interpolation::Bilinear
                    };
                    self.apply_op(ctx, |img| rotate_rgb(img, angle, px, py, interp));
                }

                // ============================================================
                // Esteganografia
                // ============================================================
                ui.separator();
                ui.heading("Esteganografia (LSB)");

                ui.text_edit_multiline(&mut self.stego_message);
                if ui.button("Esconder mensagem").clicked() {
                    if let Some(ref rgb) = self.current {
                        let stego = embed_message(rgb, &self.stego_message);
                        self.current = Some(stego);
                        self.update_textures(ctx);
                        self.status = "Mensagem escondida no canal R (LSB)".into();
                    }
                }
                if ui.button("Extrair mensagem").clicked() {
                    if let Some(ref rgb) = self.current {
                        self.stego_extracted = extract_message(rgb);
                        self.status = "Mensagem extraída".into();
                    }
                }
                if !self.stego_extracted.is_empty() {
                    ui.label("Extraído:");
                    ui.label(&self.stego_extracted);
                }
            });

        // Painel central — imagens
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading("Original");
                    if let Some(ref tex) = self.texture_original {
                        let max_h = ui.available_height() - 40.0;
                        let size = tex.size_vec2();
                        let scale = (max_h / size.y).min(1.0);
                        ui.image((tex.id(), size * scale));
                    }
                });
                ui.separator();
                ui.vertical(|ui| {
                    ui.heading("Resultado");
                    if let Some(ref tex) = self.texture_current {
                        let max_h = ui.available_height() - 40.0;
                        let size = tex.size_vec2();
                        let scale = (max_h / size.y).min(1.0);
                        ui.image((tex.id(), size * scale));
                    }
                });
            });

            // Histograma
            if self.show_histogram {
                ui.separator();
                let title = match self.hist_channel {
                    0 => "Histograma — Canal R",
                    1 => "Histograma — Canal G",
                    2 => "Histograma — Canal B",
                    _ => "Histograma — Intensidade (I)",
                };
                ui.heading(title);

                let hist = match self.hist_channel {
                    0 => &self.hist_r,
                    1 => &self.hist_g,
                    2 => &self.hist_b,
                    _ => &self.hist_i,
                };
                let max_count = *hist.iter().max().unwrap_or(&1) as f32;
                let color = match self.hist_channel {
                    0 => egui::Color32::from_rgb(220, 80, 80),
                    1 => egui::Color32::from_rgb(80, 180, 80),
                    2 => egui::Color32::from_rgb(80, 120, 220),
                    _ => egui::Color32::from_gray(180),
                };

                let desired_size = egui::vec2(ui.available_width(), 120.0);
                let (response, painter) =
                    ui.allocate_painter(desired_size, egui::Sense::hover());

                let rect = response.rect;
                let bar_w = rect.width() / 256.0;

                for i in 0..256 {
                    let h = (hist[i] as f32 / max_count) * rect.height();
                    let x = rect.left() + i as f32 * bar_w;
                    painter.rect_filled(
                        egui::Rect::from_min_size(
                            egui::pos2(x, rect.bottom() - h),
                            egui::vec2(bar_w.max(1.0), h),
                        ),
                        0.0,
                        color,
                    );
                }
            }
        });
    }
}

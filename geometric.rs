//! Transformações geométricas: escala e rotação com interpolação.
//! Versões para GrayImage e RgbImage.

use crate::{GrayImage, RgbImage, utils::clamp_u8};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Interpolation {
    Nearest,
    Bilinear,
}

// ============================================================================
// GrayImage
// ============================================================================

/// Escala a imagem para novas dimensões.
pub fn scale(img: &GrayImage, new_width: u32, new_height: u32, interp: Interpolation) -> GrayImage {
    if new_width == 0 || new_height == 0 {
        return GrayImage::new(0, 0);
    }

    let mut out = GrayImage::new(new_width, new_height);
    let x_ratio = img.width as f64 / new_width as f64;
    let y_ratio = img.height as f64 / new_height as f64;

    for y in 0..new_height {
        for x in 0..new_width {
            let src_x = x as f64 * x_ratio;
            let src_y = y as f64 * y_ratio;

            let val = match interp {
                Interpolation::Nearest => {
                    let ix = src_x.round().clamp(0.0, (img.width - 1) as f64) as u32;
                    let iy = src_y.round().clamp(0.0, (img.height - 1) as f64) as u32;
                    img.get(ix, iy)
                }
                Interpolation::Bilinear => bilinear_sample(img, src_x, src_y),
            };
            out.set(x, y, val);
        }
    }
    out
}

/// Rotação em torno de um pivô (px, py).
/// Ângulo em graus (sentido anti-horário).
/// A imagem de saída tem o mesmo tamanho da original (pixels fora ficam pretos).
pub fn rotate(
    img: &GrayImage,
    angle_deg: f64,
    pivot_x: f64,
    pivot_y: f64,
    interp: Interpolation,
) -> GrayImage {
    let mut out = GrayImage::new(img.width, img.height);
    let angle = angle_deg.to_radians();
    let cos = angle.cos();
    let sin = angle.sin();

    for y in 0..img.height {
        for x in 0..img.width {
            let dx = x as f64 - pivot_x;
            let dy = y as f64 - pivot_y;

            // Rotação inversa (destino → origem)
            let src_x = cos * dx + sin * dy + pivot_x;
            let src_y = -sin * dx + cos * dy + pivot_y;

            if src_x >= 0.0 && src_x < img.width as f64 && src_y >= 0.0 && src_y < img.height as f64 {
                let val = match interp {
                    Interpolation::Nearest => {
                        let ix = src_x.round() as u32;
                        let iy = src_y.round() as u32;
                        img.get(ix.min(img.width - 1), iy.min(img.height - 1))
                    }
                    Interpolation::Bilinear => bilinear_sample(img, src_x, src_y),
                };
                out.set(x, y, val);
            }
        }
    }
    out
}

fn bilinear_sample(img: &GrayImage, x: f64, y: f64) -> u8 {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let x1 = x0 + 1;
    let y1 = y0 + 1;

    let dx = x - x0 as f64;
    let dy = y - y0 as f64;

    let get_safe = |xx: i32, yy: i32| -> f64 {
        if xx < 0 || yy < 0 || xx >= img.width as i32 || yy >= img.height as i32 {
            0.0
        } else {
            img.get(xx as u32, yy as u32) as f64
        }
    };

    let p00 = get_safe(x0, y0);
    let p10 = get_safe(x1, y0);
    let p01 = get_safe(x0, y1);
    let p11 = get_safe(x1, y1);

    let top = p00 * (1.0 - dx) + p10 * dx;
    let bottom = p01 * (1.0 - dx) + p11 * dx;
    let val = top * (1.0 - dy) + bottom * dy;

    clamp_u8(val)
}

// ============================================================================
// RgbImage
// ============================================================================

pub fn scale_rgb(img: &RgbImage, new_width: u32, new_height: u32, interp: Interpolation) -> RgbImage {
    if new_width == 0 || new_height == 0 {
        return RgbImage::new(0, 0);
    }

    let mut out = RgbImage::new(new_width, new_height);
    let x_ratio = img.width as f64 / new_width as f64;
    let y_ratio = img.height as f64 / new_height as f64;

    for y in 0..new_height {
        for x in 0..new_width {
            let src_x = x as f64 * x_ratio;
            let src_y = y as f64 * y_ratio;

            let (r, g, b) = match interp {
                Interpolation::Nearest => {
                    let ix = src_x.round().clamp(0.0, (img.width - 1) as f64) as u32;
                    let iy = src_y.round().clamp(0.0, (img.height - 1) as f64) as u32;
                    img.get_pixel(ix, iy)
                }
                Interpolation::Bilinear => bilinear_sample_rgb(img, src_x, src_y),
            };
            out.set_pixel(x, y, r, g, b);
        }
    }
    out
}

pub fn rotate_rgb(
    img: &RgbImage,
    angle_deg: f64,
    pivot_x: f64,
    pivot_y: f64,
    interp: Interpolation,
) -> RgbImage {
    let mut out = RgbImage::new(img.width, img.height);
    let angle = angle_deg.to_radians();
    let cos = angle.cos();
    let sin = angle.sin();

    for y in 0..img.height {
        for x in 0..img.width {
            let dx = x as f64 - pivot_x;
            let dy = y as f64 - pivot_y;

            let src_x = cos * dx + sin * dy + pivot_x;
            let src_y = -sin * dx + cos * dy + pivot_y;

            if src_x >= 0.0 && src_x < img.width as f64 && src_y >= 0.0 && src_y < img.height as f64 {
                let (r, g, b) = match interp {
                    Interpolation::Nearest => {
                        let ix = src_x.round() as u32;
                        let iy = src_y.round() as u32;
                        img.get_pixel(ix.min(img.width - 1), iy.min(img.height - 1))
                    }
                    Interpolation::Bilinear => bilinear_sample_rgb(img, src_x, src_y),
                };
                out.set_pixel(x, y, r, g, b);
            }
        }
    }
    out
}

fn bilinear_sample_rgb(img: &RgbImage, x: f64, y: f64) -> (u8, u8, u8) {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let x1 = x0 + 1;
    let y1 = y0 + 1;

    let dx = x - x0 as f64;
    let dy = y - y0 as f64;

    let get_safe = |xx: i32, yy: i32| -> (f64, f64, f64) {
        if xx < 0 || yy < 0 || xx >= img.width as i32 || yy >= img.height as i32 {
            (0.0, 0.0, 0.0)
        } else {
            let (r, g, b) = img.get_pixel(xx as u32, yy as u32);
            (r as f64, g as f64, b as f64)
        }
    };

    let (r00, g00, b00) = get_safe(x0, y0);
    let (r10, g10, b10) = get_safe(x1, y0);
    let (r01, g01, b01) = get_safe(x0, y1);
    let (r11, g11, b11) = get_safe(x1, y1);

    let lerp = |a: f64, b: f64, t: f64| a * (1.0 - t) + b * t;

    let r = lerp(lerp(r00, r10, dx), lerp(r01, r11, dx), dy);
    let g = lerp(lerp(g00, g10, dx), lerp(g01, g11, dx), dy);
    let b = lerp(lerp(b00, b10, dx), lerp(b01, b11, dx), dy);

    (clamp_u8(r), clamp_u8(g), clamp_u8(b))
}

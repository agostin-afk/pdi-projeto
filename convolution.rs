//! Convolução genérica e kernels clássicos.
//! Funciona em GrayImage e RgbImage (canal a canal).

use crate::{
    GrayImage, RgbImage,
    utils::{validate_kernel_size, gray_from_f64, clamp_u8},
};

/// Aplica um kernel de convolução (tamanho ímpar) sobre a imagem cinza.
/// Mantém o tamanho original (padding por replicação de borda).
/// Se `normalize_display` for true, normaliza o resultado para [0,255].
pub fn convolve(img: &GrayImage, kernel: &[f64], ksize: usize, normalize_display: bool) -> GrayImage {
    let ksize = validate_kernel_size(ksize);
    let radius = (ksize / 2) as i32;
    let kw = ksize * ksize;
    assert_eq!(kernel.len(), kw, "Kernel deve ter tamanho ksize*ksize");

    let w = img.width as i32;
    let h = img.height as i32;
    let mut result = vec![0.0f64; (w * h) as usize];

    for y in 0..h {
        for x in 0..w {
            let mut acc = 0.0;
            for ky in 0..ksize as i32 {
                for kx in 0..ksize as i32 {
                    let px = (x + kx - radius).clamp(0, w - 1);
                    let py = (y + ky - radius).clamp(0, h - 1);
                    let pixel = img.get(px as u32, py as u32) as f64;
                    let k = kernel[(ky as usize * ksize) + kx as usize];
                    acc += pixel * k;
                }
            }
            result[(y * w + x) as usize] = acc;
        }
    }

    gray_from_f64(img.width, img.height, &result, normalize_display)
}

/// Convolução em imagem RGB (aplica o mesmo kernel em cada canal).
pub fn convolve_rgb(img: &RgbImage, kernel: &[f64], ksize: usize, normalize_display: bool) -> RgbImage {
    let ksize = validate_kernel_size(ksize);
    let radius = (ksize / 2) as i32;
    assert_eq!(kernel.len(), ksize * ksize);

    let w = img.width as i32;
    let h = img.height as i32;
    let mut out = RgbImage::new(img.width, img.height);

    // Processa cada canal separadamente
    for c in 0..3 {
        let mut result = vec![0.0f64; (w * h) as usize];
        for y in 0..h {
            for x in 0..w {
                let mut acc = 0.0;
                for ky in 0..ksize as i32 {
                    for kx in 0..ksize as i32 {
                        let px = (x + kx - radius).clamp(0, w - 1) as u32;
                        let py = (y + ky - radius).clamp(0, h - 1) as u32;
                        let idx = ((py * img.width + px) * 3 + c) as usize;
                        let pixel = img.data[idx] as f64;
                        let k = kernel[(ky as usize * ksize) + kx as usize];
                        acc += pixel * k;
                    }
                }
                result[(y * w + x) as usize] = acc;
            }
        }

        // Normaliza ou clampa o canal
        if normalize_display {
            let mut min = f64::INFINITY;
            let mut max = f64::NEG_INFINITY;
            for &v in &result {
                if v < min { min = v; }
                if v > max { max = v; }
            }
            let range = (max - min).max(1e-9);
            for y in 0..h {
                for x in 0..w {
                    let v = result[(y * w + x) as usize];
                    let norm = ((v - min) / range * 255.0).round() as u8;
                    let idx = ((y as u32 * img.width + x as u32) * 3 + c) as usize;
                    out.data[idx] = norm;
                }
            }
        } else {
            for y in 0..h {
                for x in 0..w {
                    let v = result[(y * w + x) as usize];
                    let idx = ((y as u32 * img.width + x as u32) * 3 + c) as usize;
                    out.data[idx] = clamp_u8(v);
                }
            }
        }
    }
    out
}

/// Kernel da média (box filter)
pub fn kernel_mean(ksize: usize) -> Vec<f64> {
    let ksize = validate_kernel_size(ksize);
    let val = 1.0 / (ksize * ksize) as f64;
    vec![val; ksize * ksize]
}

/// Kernel gaussiano 2D
pub fn kernel_gaussian(ksize: usize, sigma: f64) -> Vec<f64> {
    let ksize = validate_kernel_size(ksize);
    let radius = (ksize / 2) as i32;
    let mut kernel = vec![0.0; ksize * ksize];
    let mut sum = 0.0;
    let sigma2 = 2.0 * sigma * sigma;

    for y in 0..ksize as i32 {
        for x in 0..ksize as i32 {
            let dx = (x - radius) as f64;
            let dy = (y - radius) as f64;
            let v = (-((dx * dx + dy * dy) / sigma2)).exp();
            kernel[(y as usize * ksize) + x as usize] = v;
            sum += v;
        }
    }
    for v in &mut kernel {
        *v /= sum;
    }
    kernel
}

/// Kernel Laplaciano clássico 3x3
pub fn kernel_laplacian() -> Vec<f64> {
    vec![
        0.0, -1.0,  0.0,
       -1.0,  4.0, -1.0,
        0.0, -1.0,  0.0,
    ]
}

/// Kernel Laplaciano com diagonais
pub fn kernel_laplacian_diag() -> Vec<f64> {
    vec![
       -1.0, -1.0, -1.0,
       -1.0,  8.0, -1.0,
       -1.0, -1.0, -1.0,
    ]
}

pub fn apply_mean(img: &GrayImage, ksize: usize) -> GrayImage {
    let k = kernel_mean(ksize);
    convolve(img, &k, ksize, false)
}

pub fn apply_mean_rgb(img: &RgbImage, ksize: usize) -> RgbImage {
    let k = kernel_mean(ksize);
    convolve_rgb(img, &k, ksize, false)
}

pub fn apply_gaussian(img: &GrayImage, ksize: usize, sigma: f64) -> GrayImage {
    let k = kernel_gaussian(ksize, sigma);
    convolve(img, &k, ksize, false)
}

pub fn apply_gaussian_rgb(img: &RgbImage, ksize: usize, sigma: f64) -> RgbImage {
    let k = kernel_gaussian(ksize, sigma);
    convolve_rgb(img, &k, ksize, false)
}

pub fn apply_laplacian(img: &GrayImage, normalize: bool) -> GrayImage {
    let k = kernel_laplacian();
    convolve(img, &k, 3, normalize)
}

pub fn apply_laplacian_rgb(img: &RgbImage, normalize: bool) -> RgbImage {
    let k = kernel_laplacian();
    convolve_rgb(img, &k, 3, normalize)
}

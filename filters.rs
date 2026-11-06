//! Filtros específicos: mediana, Sobel, high-boost, gradiente.
//! Versões para GrayImage e RgbImage.

use crate::{
    GrayImage, RgbImage,
    convolution::{convolve, convolve_rgb, kernel_mean, kernel_laplacian},
    utils::{clamp_u8, gray_from_f64, validate_kernel_size},
};

/// Filtro da mediana (não-linear). Excelente contra ruído sal-e-pimenta.
pub fn median_filter(img: &GrayImage, ksize: usize) -> GrayImage {
    let ksize = validate_kernel_size(ksize);
    let radius = (ksize / 2) as i32;
    let w = img.width as i32;
    let h = img.height as i32;
    let mut out = GrayImage::new(img.width, img.height);
    let mut window = vec![0u8; ksize * ksize];

    for y in 0..h {
        for x in 0..w {
            let mut idx = 0;
            for ky in 0..ksize as i32 {
                for kx in 0..ksize as i32 {
                    let px = (x + kx - radius).clamp(0, w - 1) as u32;
                    let py = (y + ky - radius).clamp(0, h - 1) as u32;
                    window[idx] = img.get(px, py);
                    idx += 1;
                }
            }
            window.sort_unstable();
            out.set(x as u32, y as u32, window[window.len() / 2]);
        }
    }
    out
}

/// Mediana em RGB (canal a canal)
pub fn median_filter_rgb(img: &RgbImage, ksize: usize) -> RgbImage {
    let ksize = validate_kernel_size(ksize);
    let radius = (ksize / 2) as i32;
    let w = img.width as i32;
    let h = img.height as i32;
    let mut out = RgbImage::new(img.width, img.height);
    let mut window = vec![0u8; ksize * ksize];

    for c in 0..3 {
        for y in 0..h {
            for x in 0..w {
                let mut idx = 0;
                for ky in 0..ksize as i32 {
                    for kx in 0..ksize as i32 {
                        let px = (x + kx - radius).clamp(0, w - 1) as u32;
                        let py = (y + ky - radius).clamp(0, h - 1) as u32;
                        let pidx = ((py * img.width + px) * 3 + c) as usize;
                        window[idx] = img.data[pidx];
                        idx += 1;
                    }
                }
                window.sort_unstable();
                let oidx = ((y as u32 * img.width + x as u32) * 3 + c) as usize;
                out.data[oidx] = window[window.len() / 2];
            }
        }
    }
    out
}

/// Kernels de Sobel
pub fn kernel_sobel_x() -> Vec<f64> {
    vec![
        -1.0, 0.0, 1.0,
        -2.0, 0.0, 2.0,
        -1.0, 0.0, 1.0,
    ]
}

pub fn kernel_sobel_y() -> Vec<f64> {
    vec![
        -1.0, -2.0, -1.0,
         0.0,  0.0,  0.0,
         1.0,  2.0,  1.0,
    ]
}

/// Sobel X (normalizado para visualização)
pub fn sobel_x(img: &GrayImage) -> GrayImage {
    let k = kernel_sobel_x();
    convolve(img, &k, 3, true)
}

pub fn sobel_x_rgb(img: &RgbImage) -> RgbImage {
    let k = kernel_sobel_x();
    convolve_rgb(img, &k, 3, true)
}

/// Sobel Y (normalizado para visualização)
pub fn sobel_y(img: &GrayImage) -> GrayImage {
    let k = kernel_sobel_y();
    convolve(img, &k, 3, true)
}

pub fn sobel_y_rgb(img: &RgbImage) -> RgbImage {
    let k = kernel_sobel_y();
    convolve_rgb(img, &k, 3, true)
}

/// Magnitude do gradiente (não-linear): sqrt(Gx² + Gy²)
pub fn gradient_magnitude(img: &GrayImage) -> GrayImage {
    let gx_k = kernel_sobel_x();
    let gy_k = kernel_sobel_y();

    let w = img.width as i32;
    let h = img.height as i32;
    let radius = 1i32;
    let mut mag = vec![0.0f64; (w * h) as usize];

    for y in 0..h {
        for x in 0..w {
            let mut gx = 0.0;
            let mut gy = 0.0;
            for ky in 0..3 {
                for kx in 0..3 {
                    let px = (x + kx - radius).clamp(0, w - 1);
                    let py = (y + ky - radius).clamp(0, h - 1);
                    let p = img.get(px as u32, py as u32) as f64;
                    gx += p * gx_k[(ky * 3 + kx) as usize];
                    gy += p * gy_k[(ky * 3 + kx) as usize];
                }
            }
            mag[(y * w + x) as usize] = (gx * gx + gy * gy).sqrt();
        }
    }

    gray_from_f64(img.width, img.height, &mag, true)
}

/// Magnitude do gradiente em RGB (aplica na intensidade e replica)
pub fn gradient_magnitude_rgb(img: &RgbImage) -> RgbImage {
    let gray = img.to_gray_weighted();
    let mag = gradient_magnitude(&gray);
    RgbImage::from_gray(&mag)
}

/// High-Boost filtering: A * original - blurred
pub fn high_boost(img: &GrayImage, ksize: usize, amp: f64) -> GrayImage {
    let blurred = {
        let k = kernel_mean(ksize);
        convolve(img, &k, ksize, false)
    };

    let mut out = GrayImage::new(img.width, img.height);
    for i in 0..img.data.len() {
        let original = img.data[i] as f64;
        let blur = blurred.data[i] as f64;
        let val = amp * original - blur;
        out.data[i] = clamp_u8(val);
    }
    out
}

pub fn high_boost_rgb(img: &RgbImage, ksize: usize, amp: f64) -> RgbImage {
    let blurred = {
        let k = kernel_mean(ksize);
        convolve_rgb(img, &k, ksize, false)
    };

    let mut out = RgbImage::new(img.width, img.height);
    for i in 0..img.data.len() {
        let original = img.data[i] as f64;
        let blur = blurred.data[i] as f64;
        out.data[i] = clamp_u8(amp * original - blur);
    }
    out
}

/// Aguçamento clássico com Laplaciano: original - Laplaciano
pub fn sharpen_laplacian(img: &GrayImage) -> GrayImage {
    let k = kernel_laplacian();
    let w = img.width as i32;
    let h = img.height as i32;
    let mut lap = vec![0.0f64; (w * h) as usize];
    let radius = 1;

    for y in 0..h {
        for x in 0..w {
            let mut acc = 0.0;
            for ky in 0..3 {
                for kx in 0..3 {
                    let px = (x + kx - radius).clamp(0, w - 1);
                    let py = (y + ky - radius).clamp(0, h - 1);
                    acc += img.get(px as u32, py as u32) as f64 * k[(ky * 3 + kx) as usize];
                }
            }
            lap[(y * w + x) as usize] = acc;
        }
    }

    let mut out = GrayImage::new(img.width, img.height);
    for i in 0..img.data.len() {
        let val = img.data[i] as f64 - lap[i];
        out.data[i] = clamp_u8(val);
    }
    out
}

pub fn sharpen_laplacian_rgb(img: &RgbImage) -> RgbImage {
    let k = kernel_laplacian();
    let mut out = RgbImage::new(img.width, img.height);
    let w = img.width as i32;
    let h = img.height as i32;
    let radius = 1;

    for c in 0..3 {
        let mut lap = vec![0.0f64; (w * h) as usize];
        for y in 0..h {
            for x in 0..w {
                let mut acc = 0.0;
                for ky in 0..3 {
                    for kx in 0..3 {
                        let px = (x + kx - radius).clamp(0, w - 1) as u32;
                        let py = (y + ky - radius).clamp(0, h - 1) as u32;
                        let idx = ((py * img.width + px) * 3 + c) as usize;
                        acc += img.data[idx] as f64 * k[(ky * 3 + kx) as usize];
                    }
                }
                lap[(y * w + x) as usize] = acc;
            }
        }
        for y in 0..h {
            for x in 0..w {
                let idx = ((y as u32 * img.width + x as u32) * 3 + c) as usize;
                let val = img.data[idx] as f64 - lap[(y * w + x) as usize];
                out.data[idx] = clamp_u8(val);
            }
        }
    }
    out
}

//! Histograma e equalização de histograma.
//! Suporta imagens cinza e coloridas (R, G, B e Intensidade).

use crate::{GrayImage, RgbImage, image_to_hsi, hsi_to_image};

/// Calcula o histograma de 256 bins (contagem de pixels por intensidade).
pub fn compute_histogram(img: &GrayImage) -> [u32; 256] {
    let mut hist = [0u32; 256];
    for &p in &img.data {
        hist[p as usize] += 1;
    }
    hist
}

/// Histogramas dos canais R, G, B e da Intensidade (média dos canais).
pub fn compute_histograms_rgb(img: &RgbImage) -> ([u32; 256], [u32; 256], [u32; 256], [u32; 256]) {
    let mut hr = [0u32; 256];
    let mut hg = [0u32; 256];
    let mut hb = [0u32; 256];
    let mut hi = [0u32; 256];

    for i in 0..(img.width * img.height) as usize {
        let r = img.data[i * 3];
        let g = img.data[i * 3 + 1];
        let b = img.data[i * 3 + 2];
        hr[r as usize] += 1;
        hg[g as usize] += 1;
        hb[b as usize] += 1;
        let intensity = ((r as u32 + g as u32 + b as u32) / 3) as u8;
        hi[intensity as usize] += 1;
    }
    (hr, hg, hb, hi)
}

/// Equalização de histograma (global) em escala de cinza.
pub fn equalize_histogram(img: &GrayImage) -> GrayImage {
    let hist = compute_histogram(img);
    let total = img.data.len() as f64;

    let mut cdf = [0u32; 256];
    cdf[0] = hist[0];
    for i in 1..256 {
        cdf[i] = cdf[i - 1] + hist[i];
    }

    let mut cdf_min = 0u32;
    for &v in &cdf {
        if v > 0 {
            cdf_min = v;
            break;
        }
    }

    let mut lut = [0u8; 256];
    let denom = (total - cdf_min as f64).max(1.0);
    for i in 0..256 {
        if cdf[i] <= cdf_min {
            lut[i] = 0;
        } else {
            let val = ((cdf[i] - cdf_min) as f64 / denom) * 255.0;
            lut[i] = val.round().clamp(0.0, 255.0) as u8;
        }
    }

    let mut out = GrayImage::new(img.width, img.height);
    for (i, &p) in img.data.iter().enumerate() {
        out.data[i] = lut[p as usize];
    }
    out
}

/// Equalização de histograma em imagens coloridas via espaço HSI.
/// Equaliza apenas o canal de Intensidade (I) e mantém H e S.
pub fn equalize_histogram_color(img: &RgbImage) -> RgbImage {
    let mut hsi = image_to_hsi(img);

    // Histograma do canal I (quantizado em 256 níveis)
    let mut hist = [0u32; 256];
    for px in &hsi {
        let idx = (px.i * 255.0).round().clamp(0.0, 255.0) as usize;
        hist[idx] += 1;
    }

    let total = hsi.len() as f64;
    let mut cdf = [0u32; 256];
    cdf[0] = hist[0];
    for i in 1..256 {
        cdf[i] = cdf[i - 1] + hist[i];
    }

    let mut cdf_min = 0u32;
    for &v in &cdf {
        if v > 0 {
            cdf_min = v;
            break;
        }
    }

    let denom = (total - cdf_min as f64).max(1.0);
    let mut lut = [0.0f64; 256];
    for i in 0..256 {
        if cdf[i] <= cdf_min {
            lut[i] = 0.0;
        } else {
            lut[i] = (cdf[i] - cdf_min) as f64 / denom;
        }
    }

    // Aplica a LUT no canal I
    for px in hsi.iter_mut() {
        let idx = (px.i * 255.0).round().clamp(0.0, 255.0) as usize;
        px.i = lut[idx];
    }

    hsi_to_image(&hsi, img.width, img.height)
}

/// Retorna o histograma normalizado (probabilidades) para plotagem.
pub fn normalized_histogram(hist: &[u32; 256]) -> [f32; 256] {
    let total: u32 = hist.iter().sum();
    let mut norm = [0.0f32; 256];
    if total > 0 {
        for i in 0..256 {
            norm[i] = hist[i] as f32 / total as f32;
        }
    }
    norm
}

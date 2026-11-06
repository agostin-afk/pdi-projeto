//! Utilitários comuns (normalização, clamp, etc.)

use crate::GrayImage;

/// Clampa um valor f64 para u8 [0, 255]
#[inline]
pub fn clamp_u8(v: f64) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}

/// Normaliza um buffer de f64 (pode conter negativos e >255) para u8 [0, 255]
/// usando min-max. Ideal para visualização de filtros (Laplaciano, Sobel, etc.).
pub fn normalize_to_u8(data: &[f64]) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for &v in data {
        if v < min { min = v; }
        if v > max { max = v; }
    }
    if (max - min).abs() < 1e-9 {
        return vec![128; data.len()]; // imagem constante → cinza médio
    }
    data.iter()
        .map(|&v| (((v - min) / (max - min)) * 255.0).round() as u8)
        .collect()
}

/// Cria uma GrayImage a partir de um buffer f64 já normalizado ou não.
pub fn gray_from_f64(width: u32, height: u32, data: &[f64], normalize: bool) -> GrayImage {
    let pixels = if normalize {
        normalize_to_u8(data)
    } else {
        data.iter().map(|&v| clamp_u8(v)).collect()
    };
    GrayImage {
        width,
        height,
        data: pixels,
    }
}

/// Garante que o kernel seja ímpar e >= 3
pub fn validate_kernel_size(size: usize) -> usize {
    let mut s = size.max(3);
    if s % 2 == 0 {
        s += 1;
    }
    s.min(31) // limite razoável
}

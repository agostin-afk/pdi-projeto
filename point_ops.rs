//! Operações ponto a ponto (ponto a ponto).
//! Funcionam tanto em GrayImage quanto em RgbImage (canal a canal quando faz sentido).

use crate::{
    GrayImage, RgbImage, image_to_hsi, hsi_to_image,
    utils::clamp_u8,
};

// ============================================================================
// Operações em escala de cinza
// ============================================================================

/// Limiarização (threshold) binária.
pub fn threshold(img: &GrayImage, threshold: u8) -> GrayImage {
    let mut out = GrayImage::new(img.width, img.height);
    for (i, &p) in img.data.iter().enumerate() {
        out.data[i] = if p >= threshold { 255 } else { 0 };
    }
    out
}

/// Negativo da imagem (255 - pixel)
pub fn negative(img: &GrayImage) -> GrayImage {
    let mut out = GrayImage::new(img.width, img.height);
    for (i, &p) in img.data.iter().enumerate() {
        out.data[i] = 255 - p;
    }
    out
}

/// Ajuste de brilho (adiciona um valor constante, com clamp)
pub fn brightness(img: &GrayImage, value: i16) -> GrayImage {
    let mut out = GrayImage::new(img.width, img.height);
    for (i, &p) in img.data.iter().enumerate() {
        out.data[i] = clamp_u8(p as f64 + value as f64);
    }
    out
}

/// Correção de gama (potência): out = 255 * (pixel/255)^gamma
pub fn gamma(img: &GrayImage, gamma: f64) -> GrayImage {
    let mut out = GrayImage::new(img.width, img.height);
    let inv_gamma = 1.0 / gamma.max(0.01);
    for (i, &p) in img.data.iter().enumerate() {
        let normalized = p as f64 / 255.0;
        out.data[i] = clamp_u8(255.0 * normalized.powf(inv_gamma));
    }
    out
}

/// Transformação linear definida por partes.
/// `points` = lista de pontos de controle (x, y) ordenados por x (0..=255).
pub fn piecewise_linear(img: &GrayImage, points: &[(u8, u8)]) -> GrayImage {
    if points.len() < 2 {
        return img.clone();
    }

    let mut pts: Vec<(u8, u8)> = points.to_vec();
    pts.sort_by_key(|p| p.0);
    if pts[0].0 != 0 {
        pts.insert(0, (0, pts[0].1));
    }
    if pts.last().unwrap().0 != 255 {
        pts.push((255, pts.last().unwrap().1));
    }

    let mut lut = [0u8; 256];
    for i in 0..pts.len() - 1 {
        let (x0, y0) = pts[i];
        let (x1, y1) = pts[i + 1];
        let dx = (x1 - x0) as f64;
        if dx < 1.0 {
            continue;
        }
        for x in x0..=x1 {
            let t = (x - x0) as f64 / dx;
            lut[x as usize] = clamp_u8(y0 as f64 + t * (y1 as f64 - y0 as f64));
        }
    }

    let mut out = GrayImage::new(img.width, img.height);
    for (i, &p) in img.data.iter().enumerate() {
        out.data[i] = lut[p as usize];
    }
    out
}

// ============================================================================
// Operações em RGB (aplicadas canal a canal ou especiais)
// ============================================================================

/// Negativo de imagem colorida (255 - cada canal)
pub fn negative_rgb(img: &RgbImage) -> RgbImage {
    let mut out = img.clone();
    for v in out.data.iter_mut() {
        *v = 255 - *v;
    }
    out
}

/// Brilho em RGB (soma o mesmo valor em R, G e B)
pub fn brightness_rgb(img: &RgbImage, value: i16) -> RgbImage {
    let mut out = RgbImage::new(img.width, img.height);
    for i in 0..img.data.len() {
        out.data[i] = clamp_u8(img.data[i] as f64 + value as f64);
    }
    out
}

/// Gama em RGB (aplicado em cada canal)
pub fn gamma_rgb(img: &RgbImage, gamma: f64) -> RgbImage {
    let mut out = RgbImage::new(img.width, img.height);
    let inv_gamma = 1.0 / gamma.max(0.01);
    for i in 0..img.data.len() {
        let normalized = img.data[i] as f64 / 255.0;
        out.data[i] = clamp_u8(255.0 * normalized.powf(inv_gamma));
    }
    out
}

/// Limiarização em RGB (aplica no canal de intensidade e mantém cor ou binário)
pub fn threshold_rgb(img: &RgbImage, threshold: u8) -> RgbImage {
    let gray = img.to_gray_weighted();
    let mut out = RgbImage::new(img.width, img.height);
    for i in 0..gray.data.len() {
        let v = if gray.data[i] >= threshold { 255 } else { 0 };
        out.data[i * 3] = v;
        out.data[i * 3 + 1] = v;
        out.data[i * 3 + 2] = v;
    }
    out
}

/// Linear por partes em RGB (canal a canal)
pub fn piecewise_linear_rgb(img: &RgbImage, points: &[(u8, u8)]) -> RgbImage {
    if points.len() < 2 {
        return img.clone();
    }

    let mut pts: Vec<(u8, u8)> = points.to_vec();
    pts.sort_by_key(|p| p.0);
    if pts[0].0 != 0 {
        pts.insert(0, (0, pts[0].1));
    }
    if pts.last().unwrap().0 != 255 {
        pts.push((255, pts.last().unwrap().1));
    }

    let mut lut = [0u8; 256];
    for i in 0..pts.len() - 1 {
        let (x0, y0) = pts[i];
        let (x1, y1) = pts[i + 1];
        let dx = (x1 - x0) as f64;
        if dx < 1.0 {
            continue;
        }
        for x in x0..=x1 {
            let t = (x - x0) as f64 / dx;
            lut[x as usize] = clamp_u8(y0 as f64 + t * (y1 as f64 - y0 as f64));
        }
    }

    let mut out = RgbImage::new(img.width, img.height);
    for i in 0..img.data.len() {
        out.data[i] = lut[img.data[i] as usize];
    }
    out
}

// ============================================================================
// Operações específicas de cor
// ============================================================================

/// Sépia (“escala de cinza amarelada”)
pub fn sepia(img: &RgbImage) -> RgbImage {
    let mut out = RgbImage::new(img.width, img.height);
    for i in 0..(img.width * img.height) as usize {
        let r = img.data[i * 3] as f64;
        let g = img.data[i * 3 + 1] as f64;
        let b = img.data[i * 3 + 2] as f64;

        // Matriz clássica de sépia
        let tr = 0.393 * r + 0.769 * g + 0.189 * b;
        let tg = 0.349 * r + 0.686 * g + 0.168 * b;
        let tb = 0.272 * r + 0.534 * g + 0.131 * b;

        out.data[i * 3] = clamp_u8(tr);
        out.data[i * 3 + 1] = clamp_u8(tg);
        out.data[i * 3 + 2] = clamp_u8(tb);
    }
    out
}

/// Ajuste de Matiz, Saturação e Intensidade (HSI).
/// delta_h em graus, delta_s e delta_i em [-1.0, 1.0] (aditivos relativos).
pub fn adjust_hsi(img: &RgbImage, delta_h: f64, delta_s: f64, delta_i: f64) -> RgbImage {
    let mut hsi = image_to_hsi(img);
    for px in hsi.iter_mut() {
        px.h = (px.h + delta_h).rem_euclid(360.0);
        px.s = (px.s + delta_s).clamp(0.0, 1.0);
        px.i = (px.i + delta_i).clamp(0.0, 1.0);
    }
    hsi_to_image(&hsi, img.width, img.height)
}

/// Ajuste de canais individuais (C/R, M/G, Y/B).
/// Os valores são offsets em [-255, 255] aplicados a cada canal.
pub fn adjust_channels(img: &RgbImage, dr: i16, dg: i16, db: i16) -> RgbImage {
    let mut out = RgbImage::new(img.width, img.height);
    for i in 0..(img.width * img.height) as usize {
        out.data[i * 3] = clamp_u8(img.data[i * 3] as f64 + dr as f64);
        out.data[i * 3 + 1] = clamp_u8(img.data[i * 3 + 1] as f64 + dg as f64);
        out.data[i * 3 + 2] = clamp_u8(img.data[i * 3 + 2] as f64 + db as f64);
    }
    out
}

/// Chroma-Key: substitui pixels próximos da cor-chave (geralmente verde)
/// por pixels da imagem de fundo.
///
/// `key` = cor a ser removida (R,G,B)
/// `distance` = distância euclidiana máxima no espaço RGB (0..=441 ≈ sqrt(3*255²))
/// `background` = imagem que substitui a cor-chave (deve ter o mesmo tamanho ou será redimensionada)
pub fn chroma_key(
    img: &RgbImage,
    key: (u8, u8, u8),
    distance: f64,
    background: &RgbImage,
) -> RgbImage {
    let mut out = img.clone();
    let (kr, kg, kb) = (key.0 as f64, key.1 as f64, key.2 as f64);
    let dist2 = distance * distance;

    // Se o fundo tiver tamanho diferente, usamos amostragem simples (vizinho)
    let same_size = background.width == img.width && background.height == img.height;

    for y in 0..img.height {
        for x in 0..img.width {
            let (r, g, b) = img.get_pixel(x, y);
            let dr = r as f64 - kr;
            let dg = g as f64 - kg;
            let db = b as f64 - kb;
            let d2 = dr * dr + dg * dg + db * db;

            if d2 <= dist2 {
                // Substitui pelo fundo
                let (br, bg, bb) = if same_size {
                    background.get_pixel(x, y)
                } else {
                    // Mapeia coordenadas
                    let bx = (x as f64 * background.width as f64 / img.width as f64) as u32;
                    let by = (y as f64 * background.height as f64 / img.height as f64) as u32;
                    background.get_pixel(bx.min(background.width - 1), by.min(background.height - 1))
                };
                out.set_pixel(x, y, br, bg, bb);
            }
        }
    }
    out
}

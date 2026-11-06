//! Representações de imagem usadas pelo processador.
//!
//! - GrayImage: tons de cinza (8 bits) — usada internamente em vários filtros.
//! - RgbImage: RGB 8 bits por canal — tipo principal da interface e operações coloridas.

use image::{DynamicImage, GrayImage as ImgGray, RgbImage as ImgRgb, Rgb};
use std::path::Path;

// ============================================================================
// GrayImage
// ============================================================================

/// Imagem em tons de cinza (8 bits).
/// Layout: row-major, data[y * width + x]
#[derive(Clone, Debug)]
pub struct GrayImage {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl GrayImage {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0; (width * height) as usize],
        }
    }

    pub fn from_vec(width: u32, height: u32, data: Vec<u8>) -> Option<Self> {
        if data.len() != (width * height) as usize {
            return None;
        }
        Some(Self { width, height, data })
    }

    #[inline]
    pub fn get(&self, x: u32, y: u32) -> u8 {
        if x >= self.width || y >= self.height {
            return 0;
        }
        self.data[(y * self.width + x) as usize]
    }

    #[inline]
    pub fn set(&mut self, x: u32, y: u32, value: u8) {
        if x < self.width && y < self.height {
            self.data[(y * self.width + x) as usize] = value;
        }
    }

    pub fn pixels(&self) -> &[u8] {
        &self.data
    }

    pub fn pixels_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Carrega qualquer formato suportado pelo crate `image` e converte para cinza.
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let img = image::open(path)?;
        Ok(Self::from_dynamic(&img))
    }

    pub fn from_dynamic(img: &DynamicImage) -> Self {
        let gray = img.to_luma8();
        Self {
            width: gray.width(),
            height: gray.height(),
            data: gray.into_raw(),
        }
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let img = ImgGray::from_raw(self.width, self.height, self.data.clone())
            .ok_or_else(|| anyhow::anyhow!("Falha ao criar GrayImage"))?;
        img.save(path)?;
        Ok(())
    }

    pub fn to_dynamic(&self) -> DynamicImage {
        let img = ImgGray::from_raw(self.width, self.height, self.data.clone())
            .expect("dados inválidos");
        DynamicImage::ImageLuma8(img)
    }

    pub fn to_rgb_dynamic(&self) -> DynamicImage {
        let mut rgb = ImgRgb::new(self.width, self.height);
        for (i, &p) in self.data.iter().enumerate() {
            let x = (i as u32) % self.width;
            let y = (i as u32) / self.width;
            rgb.put_pixel(x, y, Rgb([p, p, p]));
        }
        DynamicImage::ImageRgb8(rgb)
    }
}

// ============================================================================
// RgbImage
// ============================================================================

/// Imagem RGB (8 bits por canal).
/// Layout: data[i*3 + 0] = R, +1 = G, +2 = B
#[derive(Clone, Debug)]
pub struct RgbImage {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, // RGB interleaved
}

impl RgbImage {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0; (width * height * 3) as usize],
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let img = image::open(path)?.to_rgb8();
        Ok(Self {
            width: img.width(),
            height: img.height(),
            data: img.into_raw(),
        })
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let img = ImgRgb::from_raw(self.width, self.height, self.data.clone())
            .ok_or_else(|| anyhow::anyhow!("Falha ao criar RgbImage"))?;
        img.save(path)?;
        Ok(())
    }

    pub fn to_dynamic(&self) -> DynamicImage {
        let img = ImgRgb::from_raw(self.width, self.height, self.data.clone())
            .expect("dados inválidos");
        DynamicImage::ImageRgb8(img)
    }

    /// Retorna (r, g, b) do pixel (x, y)
    #[inline]
    pub fn get_pixel(&self, x: u32, y: u32) -> (u8, u8, u8) {
        if x >= self.width || y >= self.height {
            return (0, 0, 0);
        }
        let i = ((y * self.width + x) * 3) as usize;
        (self.data[i], self.data[i + 1], self.data[i + 2])
    }

    #[inline]
    pub fn set_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8) {
        if x < self.width && y < self.height {
            let i = ((y * self.width + x) * 3) as usize;
            self.data[i] = r;
            self.data[i + 1] = g;
            self.data[i + 2] = b;
        }
    }

    /// Conversão para cinza — média aritmética simples.
    pub fn to_gray_average(&self) -> GrayImage {
        let mut gray = GrayImage::new(self.width, self.height);
        for i in 0..(self.width * self.height) as usize {
            let r = self.data[i * 3] as u32;
            let g = self.data[i * 3 + 1] as u32;
            let b = self.data[i * 3 + 2] as u32;
            gray.data[i] = ((r + g + b) / 3) as u8;
        }
        gray
    }

    /// Conversão para cinza — média ponderada (luminância BT.601).
    pub fn to_gray_weighted(&self) -> GrayImage {
        let mut gray = GrayImage::new(self.width, self.height);
        for i in 0..(self.width * self.height) as usize {
            let r = self.data[i * 3] as u32;
            let g = self.data[i * 3 + 1] as u32;
            let b = self.data[i * 3 + 2] as u32;
            // 0.299R + 0.587G + 0.114B  ≈  (77R + 150G + 29B) >> 8
            gray.data[i] = ((77 * r + 150 * g + 29 * b) >> 8) as u8;
        }
        gray
    }

    /// Alias antigo (usa média ponderada).
    pub fn to_gray(&self) -> GrayImage {
        self.to_gray_weighted()
    }

    /// Cria RgbImage a partir de uma GrayImage (réplica nos 3 canais).
    pub fn from_gray(gray: &GrayImage) -> Self {
        let mut rgb = Self::new(gray.width, gray.height);
        for i in 0..gray.data.len() {
            let p = gray.data[i];
            rgb.data[i * 3] = p;
            rgb.data[i * 3 + 1] = p;
            rgb.data[i * 3 + 2] = p;
        }
        rgb
    }
}

// ============================================================================
// Conversão RGB ↔ HSI
// ============================================================================

/// Representa um pixel no espaço HSI.
/// H em graus [0, 360), S e I em [0.0, 1.0]
#[derive(Clone, Copy, Debug)]
pub struct Hsi {
    pub h: f64,
    pub s: f64,
    pub i: f64,
}

/// Converte um pixel RGB (0-255) para HSI.
pub fn rgb_to_hsi(r: u8, g: u8, b: u8) -> Hsi {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;

    let i = (r + g + b) / 3.0;

    let min = r.min(g).min(b);
    let s = if i < 1e-10 {
        0.0
    } else {
        1.0 - min / i
    };

    // Hue
    let mut h = 0.0;
    if s > 1e-10 {
        let num = 0.5 * ((r - g) + (r - b));
        let den = ((r - g).powi(2) + (r - b) * (g - b)).sqrt();
        let mut theta = if den.abs() < 1e-10 {
            0.0
        } else {
            (num / den).clamp(-1.0, 1.0).acos()
        };
        // acos retorna [0, π]
        if b > g {
            theta = 2.0 * std::f64::consts::PI - theta;
        }
        h = theta.to_degrees();
    }

    Hsi { h, s, i }
}

/// Converte HSI de volta para RGB (0-255).
pub fn hsi_to_rgb(hsi: Hsi) -> (u8, u8, u8) {
    let h = hsi.h.rem_euclid(360.0);
    let s = hsi.s.clamp(0.0, 1.0);
    let i = hsi.i.clamp(0.0, 1.0);

    let (r, g, b) = if s < 1e-10 {
        (i, i, i)
    } else {
        let h_sector = h / 60.0;
        let sector = h_sector.floor() as i32 % 6;
        let f = h_sector - sector as f64;

        // Fórmulas clássicas HSI → RGB
        let x = i * (1.0 - s);
        let y = i * (1.0 - s * f);
        let z = i * (1.0 - s * (1.0 - f));

        match sector {
            0 => (i, z, x),
            1 => (y, i, x),
            2 => (x, i, z),
            3 => (x, y, i),
            4 => (z, x, i),
            _ => (i, x, y), // 5
        }
    };

    (
        (r * 255.0).round().clamp(0.0, 255.0) as u8,
        (g * 255.0).round().clamp(0.0, 255.0) as u8,
        (b * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

/// Converte imagem RGB inteira para vetor de HSI (mesmo layout).
pub fn image_to_hsi(img: &RgbImage) -> Vec<Hsi> {
    let n = (img.width * img.height) as usize;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let r = img.data[i * 3];
        let g = img.data[i * 3 + 1];
        let b = img.data[i * 3 + 2];
        out.push(rgb_to_hsi(r, g, b));
    }
    out
}

/// Converte vetor de HSI de volta para RgbImage.
pub fn hsi_to_image(hsi: &[Hsi], width: u32, height: u32) -> RgbImage {
    let mut img = RgbImage::new(width, height);
    for (i, px) in hsi.iter().enumerate() {
        let (r, g, b) = hsi_to_rgb(*px);
        img.data[i * 3] = r;
        img.data[i * 3 + 1] = g;
        img.data[i * 3 + 2] = b;
    }
    img
}

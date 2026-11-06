//! Processador de imagens clássico (PDI) em Rust.
//! Suporta operações em escala de cinza e coloridas (RGB / HSI).
//!
//! Módulos:
//! - image: representações GrayImage e RgbImage + conversões RGB↔HSI
//! - point_ops: limiar, negativo, brilho, gama, linear por partes, sépia, HSI, chroma-key
//! - histogram: histograma e equalização (cinza e colorida via HSI)
//! - convolution: convolução genérica + kernels (média, gaussiano, laplaciano)
//! - filters: mediana, Sobel, gradiente, high-boost, aguçamento
//! - geometric: escala e rotação (vizinho mais próximo / bilinear)
//! - steganography: ocultação de mensagem por LSB
//! - utils: helpers (clamp, normalização, validação de kernel)

pub mod image;
pub mod point_ops;
pub mod histogram;
pub mod convolution;
pub mod filters;
pub mod geometric;
pub mod steganography;
pub mod utils;

pub use image::{GrayImage, RgbImage, Hsi, rgb_to_hsi, hsi_to_rgb, image_to_hsi, hsi_to_image};
pub use point_ops::*;
pub use histogram::*;
pub use convolution::*;
pub use filters::*;
pub use geometric::*;
pub use steganography::*;
pub use utils::*;

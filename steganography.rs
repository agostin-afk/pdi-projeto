//! Esteganografia simples por LSB (Least Significant Bit).

use crate::RgbImage;

/// Esconde uma mensagem de texto na imagem RGB usando o bit menos significativo do canal R.
/// Retorna a imagem modificada.
pub fn embed_message(img: &RgbImage, message: &str) -> RgbImage {
    let mut out = img.clone();
    let bytes = message.as_bytes();
    let len = bytes.len() as u32;

    // Guarda o tamanho da mensagem nos primeiros 32 bits (4 bytes) do canal R
    let mut bit_idx = 0usize;

    // Primeiro grava o tamanho (32 bits)
    for _i in 0..32 {
        let bit = ((len >> (31 - _i)) & 1) as u8;
        let pixel_idx = bit_idx / 8; // cada pixel tem 1 bit no R
        if pixel_idx * 3 >= out.data.len() {
            break;
        }
        let r_idx = pixel_idx * 3;
        out.data[r_idx] = (out.data[r_idx] & 0xFE) | bit;
        bit_idx += 1;
    }

    // Depois grava a mensagem
    for &byte in bytes {
        for b in 0..8 {
            let bit = (byte >> (7 - b)) & 1;
            let pixel_idx = bit_idx / 8;
            if pixel_idx * 3 >= out.data.len() {
                return out; // imagem pequena demais
            }
            let r_idx = pixel_idx * 3;
            out.data[r_idx] = (out.data[r_idx] & 0xFE) | bit;
            bit_idx += 1;
        }
    }
    out
}

/// Extrai a mensagem escondida na imagem.
pub fn extract_message(img: &RgbImage) -> String {
    let mut bit_idx = 0usize;

    // Lê o tamanho (32 bits)
    let mut len: u32 = 0;
    for _i in 0..32 {
        let pixel_idx = bit_idx / 8;
        if pixel_idx * 3 >= img.data.len() {
            return String::new();
        }
        let bit = img.data[pixel_idx * 3] & 1;
        len = (len << 1) | bit as u32;
        bit_idx += 1;
    }

    if len > 10_000 {
        // proteção contra valores absurdos
        return String::new();
    }

    let mut bytes = Vec::with_capacity(len as usize);
    for _ in 0..len {
        let mut byte = 0u8;
        for _ in 0..8 {
            let pixel_idx = bit_idx / 8;
            if pixel_idx * 3 >= img.data.len() {
                break;
            }
            let bit = img.data[pixel_idx * 3] & 1;
            byte = (byte << 1) | bit;
            bit_idx += 1;
        }
        bytes.push(byte);
    }

    String::from_utf8_lossy(&bytes).into_owned()
}

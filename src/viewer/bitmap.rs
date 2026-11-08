//! Representação em memória de uma página renderizada em buffer de pixels (`PageBitmap`).

/// Buffer de imagem bidimensional em formato RGBA8888 (4 bytes por pixel).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageBitmap {
    /// Largura da página em pixels.
    pub width: u32,
    /// Altura da página em pixels.
    pub height: u32,
    /// Vetor contíguo de pixels em ordem de linha (RGBA, tamanho `width * height * 4`).
    pub rgba_pixels: Vec<u8>,
}

impl PageBitmap {
    /// Instancia um bitmap preenchido uniformemente com uma cor de fundo RGBA.
    pub fn new_blank(width: u32, height: u32, background: [u8; 4]) -> Self {
        let total_pixels = (width as usize) * (height as usize);
        let mut rgba_pixels = Vec::with_capacity(total_pixels * 4);
        for _ in 0..total_pixels {
            rgba_pixels.extend_from_slice(&background);
        }
        Self {
            width,
            height,
            rgba_pixels,
        }
    }

    /// Altera a cor de um pixel específico nas coordenadas `(x, y)`.
    pub fn set_pixel(&mut self, x: u32, y: u32, color: [u8; 4]) {
        if x < self.width && y < self.height {
            let offset = ((y * self.width + x) as usize) * 4;
            self.rgba_pixels[offset..offset + 4].copy_from_slice(&color);
        }
    }

    /// Converte o buffer contíguo RGBA8888 para um buffer de pixels ARGB `u32` (compatível com framebuffers nativos).
    pub fn to_argb_u32(&self) -> Vec<u32> {
        let total_pixels = (self.width as usize) * (self.height as usize);
        let mut buffer = Vec::with_capacity(total_pixels);
        for chunk in self.rgba_pixels.as_chunks::<4>().0 {
            let r = chunk[0] as u32;
            let g = chunk[1] as u32;
            let b = chunk[2] as u32;
            buffer.push((r << 16) | (g << 8) | b);
        }
        buffer
    }
}

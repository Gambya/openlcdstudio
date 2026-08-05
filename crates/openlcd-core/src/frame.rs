#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameSize {
    pub width: u32,
    pub height: u32,
}

impl FrameSize {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone)]
pub struct RgbaFrame {
    size: FrameSize,
    pixels: Vec<u8>,
}

impl RgbaFrame {
    pub fn new(size: FrameSize, pixels: Vec<u8>) -> Result<Self, FrameError> {
        let expected = size.width as usize * size.height as usize * 4;

        if pixels.len() != expected {
            return Err(FrameError::InvalidBufferSize {
                expected,
                actual: pixels.len(),
            });
        }

        Ok(Self { size, pixels })
    }

    pub fn size(&self) -> FrameSize {
        self.size
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn into_pixels(self) -> Vec<u8> {
        self.pixels
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("tamanho inválido do framebuffer: esperado {expected}, recebido {actual}")]
    InvalidBufferSize { expected: usize, actual: usize },
}

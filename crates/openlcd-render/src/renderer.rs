use std::path::Path;

use ab_glyph::{FontArc, PxScale};
use image::{DynamicImage, ImageBuffer, Rgba, RgbaImage, imageops::FilterType};
use imageproc::drawing::draw_text_mut;
use openlcd_core::{FrameSize, RgbaFrame};

use crate::RenderError;

pub const KMEX_FRAME_SIZE: FrameSize = FrameSize::new(462, 1920);

pub struct Renderer {
    size: FrameSize,
    canvas: RgbaImage,
}

impl Renderer {
    pub fn new(size: FrameSize) -> Self {
        let canvas = ImageBuffer::from_pixel(size.width, size.height, Rgba([0, 0, 0, 255]));

        Self { size, canvas }
    }

    pub fn kmex() -> Self {
        Self::new(KMEX_FRAME_SIZE)
    }

    pub fn clear(&mut self, color: [u8; 4]) {
        for pixel in self.canvas.pixels_mut() {
            *pixel = Rgba(color);
        }
    }

    pub fn set_background_file(&mut self, path: impl AsRef<Path>) -> Result<(), RenderError> {
        let image = image::open(path)?;

        self.set_background(&image);

        Ok(())
    }

    pub fn set_background(&mut self, image: &DynamicImage) {
        let resized = image.resize_to_fill(self.size.width, self.size.height, FilterType::Lanczos3);

        self.canvas = resized.to_rgba8();
    }

    pub fn draw_text(
        &mut self,
        text: &str,
        x: i32,
        y: i32,
        size: f32,
        color: [u8; 4],
        font: &FontArc,
    ) {
        draw_text_mut(
            &mut self.canvas,
            Rgba(color),
            x,
            y,
            PxScale::from(size),
            font,
            text,
        );
    }

    pub fn finish(&self) -> Result<RgbaFrame, RenderError> {
        Ok(RgbaFrame::new(self.size, self.canvas.clone().into_raw())?)
    }

    pub fn canvas(&self) -> &RgbaImage {
        &self.canvas
    }
}

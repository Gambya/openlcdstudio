use ab_glyph::{FontArc, PxScale};

use image::{ImageBuffer, Rgba, RgbaImage};

use imageproc::drawing::draw_text_mut;

use openlcd_core::{FrameSize, LOGICAL_CANVAS_SIZE, RgbaFrame};

use openlcd_theme::{Layer, Scene};

use crate::Result;

pub struct SceneRenderer {
    size: FrameSize,
    font: FontArc,
}

impl SceneRenderer {
    pub fn new(size: FrameSize, font: FontArc) -> Self {
        Self { size, font }
    }

    pub fn size(&self) -> FrameSize {
        self.size
    }

    pub fn render(&self, scene: &Scene) -> Result<RgbaFrame> {
        let mut canvas =
            ImageBuffer::from_pixel(self.size.width, self.size.height, Rgba([0, 0, 0, 255]));

        for layer in &scene.layers {
            if !layer.visible() {
                continue;
            }

            match layer {
                Layer::Background(background) => {
                    render_background(&mut canvas, background.color);
                }

                Layer::Text(text) => {
                    let position = text.position.to_pixels(self.size);

                    let font_size = logical_font_size_to_pixels(text.font_size, self.size);

                    draw_text_mut(
                        &mut canvas,
                        Rgba(text.color),
                        position.x as i32,
                        position.y as i32,
                        PxScale::from(font_size),
                        &self.font,
                        &text.text,
                    );
                }
            }
        }

        Ok(RgbaFrame::new(self.size, canvas.into_raw())?)
    }
}

fn render_background(canvas: &mut RgbaImage, color: [u8; 4]) {
    let color = Rgba(color);

    for pixel in canvas.pixels_mut() {
        *pixel = color;
    }
}

fn logical_font_size_to_pixels(logical_size: f32, target: FrameSize) -> f32 {
    let scale = target.height as f32 / LOGICAL_CANVAS_SIZE;

    (logical_size * scale).max(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scales_font_for_kmex() {
        let size = FrameSize::new(462, 1920);

        let scaled = logical_font_size_to_pixels(50.0, size);

        assert_eq!(scaled, 96.0);
    }

    #[test]
    fn scales_font_for_square_display() {
        let size = FrameSize::new(480, 480);

        let scaled = logical_font_size_to_pixels(50.0, size);

        assert_eq!(scaled, 24.0);
    }
}

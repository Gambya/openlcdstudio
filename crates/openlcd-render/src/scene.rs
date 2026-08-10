use std::{cell::RefCell, collections::HashMap};

use ab_glyph::{FontArc, PxScale};

use image::{
    DynamicImage, ImageBuffer, Rgba, RgbaImage,
    imageops::{self, FilterType},
};

use imageproc::drawing::draw_text_mut;

use openlcd_core::{FrameSize, LOGICAL_CANVAS_SIZE, RgbaFrame};

use openlcd_theme::{ImageFit, ImageLayer, Layer, Scene};

use crate::Result;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ImageCacheKey {
    source: String,
    width: u32,
    height: u32,
    fit: ImageFit,
    opacity: u8,
}

pub struct SceneRenderer {
    size: FrameSize,
    font: FontArc,

    image_cache: RefCell<HashMap<ImageCacheKey, RgbaImage>>,
}

impl SceneRenderer {
    pub fn new(size: FrameSize, font: FontArc) -> Self {
        Self {
            size,
            font,
            image_cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn size(&self) -> FrameSize {
        self.size
    }

    pub fn set_size(&mut self, size: FrameSize) {
        self.size = size;
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

                Layer::Image(image) => {
                    self.render_image(&mut canvas, image)?;
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

    fn render_image(&self, canvas: &mut RgbaImage, layer: &ImageLayer) -> Result<()> {
        let rect = layer.rect.to_pixels(self.size);

        if rect.size.width == 0 || rect.size.height == 0 {
            return Ok(());
        }

        let key = ImageCacheKey {
            source: layer.source.clone(),

            width: rect.size.width,

            height: rect.size.height,

            fit: layer.fit,

            opacity: layer.opacity,
        };

        let cached = {
            let cache = self.image_cache.borrow();

            cache.get(&key).cloned()
        };

        let rendered = if let Some(image) = cached {
            image
        } else {
            let source = image::open(&layer.source)?;

            let mut rendered = prepare_image(source, rect.size.width, rect.size.height, layer.fit);

            apply_opacity(&mut rendered, layer.opacity);

            let mut cache = self.image_cache.borrow_mut();

            if cache.len() >= 16 {
                cache.clear();
            }

            cache.insert(key, rendered.clone());

            rendered
        };

        imageops::overlay(
            canvas,
            &rendered,
            rect.position.x as i64,
            rect.position.y as i64,
        );

        Ok(())
    }
}

fn prepare_image(source: DynamicImage, width: u32, height: u32, fit: ImageFit) -> RgbaImage {
    match fit {
        ImageFit::Stretch => source
            .resize_exact(width, height, FilterType::Lanczos3)
            .to_rgba8(),

        ImageFit::Cover => source
            .resize_to_fill(width, height, FilterType::Lanczos3)
            .to_rgba8(),

        ImageFit::Contain => {
            let resized = source.resize(width, height, FilterType::Lanczos3);

            let resized = resized.to_rgba8();

            let mut output = ImageBuffer::from_pixel(width, height, Rgba([0, 0, 0, 0]));

            let x = width.saturating_sub(resized.width()) / 2;

            let y = height.saturating_sub(resized.height()) / 2;

            imageops::overlay(&mut output, &resized, x as i64, y as i64);

            output
        }
    }
}

fn apply_opacity(image: &mut RgbaImage, opacity: u8) {
    if opacity == 255 {
        return;
    }

    let factor = opacity as f32 / 255.0;

    for pixel in image.pixels_mut() {
        pixel[3] = (pixel[3] as f32 * factor).round().clamp(0.0, 255.0) as u8;
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

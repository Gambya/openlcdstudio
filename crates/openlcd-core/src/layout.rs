use serde::{Deserialize, Serialize};

use crate::FrameSize;

/// Espaço lógico padrão usado pelos temas.
///
/// Todos os elementos da Scene são posicionados em um canvas virtual
/// de 1000 × 1000 unidades, independentemente da resolução física
/// do display.
pub const LOGICAL_CANVAS_SIZE: f32 = 1_000.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LogicalPosition {
    pub x: f32,
    pub y: f32,
}

impl LogicalPosition {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn to_pixels(self, target: FrameSize) -> PixelPosition {
        PixelPosition {
            x: logical_to_pixel(self.x, target.width),
            y: logical_to_pixel(self.y, target.height),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LogicalSize {
    pub width: f32,
    pub height: f32,
}

impl LogicalSize {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn to_pixels(self, target: FrameSize) -> PixelSize {
        PixelSize {
            width: logical_to_pixel(self.width, target.width),
            height: logical_to_pixel(self.height, target.height),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LogicalRect {
    pub position: LogicalPosition,
    pub size: LogicalSize,
}

impl LogicalRect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            position: LogicalPosition::new(x, y),
            size: LogicalSize::new(width, height),
        }
    }

    pub fn to_pixels(self, target: FrameSize) -> PixelRect {
        PixelRect {
            position: self.position.to_pixels(target),

            size: self.size.to_pixels(target),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelPosition {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelRect {
    pub position: PixelPosition,
    pub size: PixelSize,
}

fn logical_to_pixel(logical_value: f32, physical_size: u32) -> u32 {
    let normalized = logical_value.clamp(0.0, LOGICAL_CANVAS_SIZE) / LOGICAL_CANVAS_SIZE;

    (normalized * physical_size as f32).round() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_maps_to_center_of_kmex() {
        let target = FrameSize::new(462, 1920);

        let logical = LogicalPosition::new(500.0, 500.0);

        let pixels = logical.to_pixels(target);

        assert_eq!(pixels, PixelPosition { x: 231, y: 960 },);
    }

    #[test]
    fn rect_maps_to_square_display() {
        let target = FrameSize::new(480, 480);

        let rect = LogicalRect::new(100.0, 200.0, 500.0, 250.0);

        let pixels = rect.to_pixels(target);

        assert_eq!(pixels.position, PixelPosition { x: 48, y: 96 },);

        assert_eq!(
            pixels.size,
            PixelSize {
                width: 240,
                height: 120,
            },
        );
    }

    #[test]
    fn out_of_bounds_values_are_clamped() {
        let target = FrameSize::new(100, 100);

        let position = LogicalPosition::new(-100.0, 1_500.0);

        let pixels = position.to_pixels(target);

        assert_eq!(pixels.x, 0);
        assert_eq!(pixels.y, 100);
    }
}

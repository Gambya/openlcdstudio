use crate::FrameSize;

/// Tamanho padrão do canvas lógico usado pelos temas.
///
/// As posições e dimensões dos elementos são armazenadas neste espaço,
/// independentemente da resolução física do display.
pub const LOGICAL_CANVAS_SIZE: f32 = 1_000.0;

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, Copy, PartialEq)]
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
    fn converts_logical_position_to_kmex_pixels() {
        let target = FrameSize::new(462, 1920);

        let logical = LogicalPosition::new(500.0, 500.0);

        let pixels = logical.to_pixels(target);

        assert_eq!(pixels.x, 231);
        assert_eq!(pixels.y, 960);
    }

    #[test]
    fn converts_logical_rect_to_square_display() {
        let target = FrameSize::new(480, 480);

        let logical = LogicalRect::new(100.0, 200.0, 500.0, 250.0);

        let pixels = logical.to_pixels(target);

        assert_eq!(pixels.position.x, 48);
        assert_eq!(pixels.position.y, 96);
        assert_eq!(pixels.size.width, 240);
        assert_eq!(pixels.size.height, 120);
    }

    #[test]
    fn clamps_values_outside_logical_canvas() {
        let target = FrameSize::new(100, 100);

        let position = LogicalPosition::new(-100.0, 1_500.0);

        let pixels = position.to_pixels(target);

        assert_eq!(pixels.x, 0);
        assert_eq!(pixels.y, 100);
    }
}

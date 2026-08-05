use crate::frame::FrameSize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    Rgba8888,
    Rgb888,
    Rgb565,
    Jpeg,
    Png,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Landscape,
    Portrait,
    LandscapeFlipped,
    PortraitFlipped,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceCapabilities {
    pub model: String,
    pub native_size: FrameSize,
    pub orientation: Orientation,
    pub accepted_formats: Vec<PixelFormat>,
    pub preferred_format: PixelFormat,
    pub min_fps: f32,
    pub max_fps: f32,
    pub recommended_fps: f32,
    pub requires_continuous_frames: bool,
    pub supports_brightness: bool,
}

impl DeviceCapabilities {
    pub fn validate_fps(&self, fps: f32) -> bool {
        fps.is_finite() && fps >= self.min_fps && fps <= self.max_fps
    }
}

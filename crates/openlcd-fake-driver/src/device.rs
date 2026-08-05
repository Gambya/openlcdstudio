use openlcd_core::{DeviceCapabilities, FrameSize, Orientation, PixelFormat, RgbaFrame};
use openlcd_driver::DisplayDevice;

use crate::error::{FakeDisplayError, Result};

#[derive(Debug)]
pub struct FakeDisplay {
    capabilities: DeviceCapabilities,
    last_frame: Option<RgbaFrame>,
    frame_count: u64,
    brightness: u8,
}

impl FakeDisplay {
    pub fn new(capabilities: DeviceCapabilities) -> Self {
        Self {
            capabilities,
            last_frame: None,
            frame_count: 0,
            brightness: 100,
        }
    }

    pub fn kmex_profile() -> Self {
        Self::new(DeviceCapabilities {
            model: "Fake K-MEX HL VMAX".to_owned(),
            native_size: FrameSize::new(462, 1920),
            orientation: Orientation::Portrait,
            accepted_formats: vec![PixelFormat::Rgba8888],
            preferred_format: PixelFormat::Rgba8888,
            min_fps: 0.5,
            max_fps: 60.0,
            recommended_fps: 20.0,
            requires_continuous_frames: false,
            supports_brightness: true,
        })
    }

    pub fn square_profile(width: u32, height: u32) -> Self {
        Self::new(DeviceCapabilities {
            model: "Fake Square Display".to_owned(),
            native_size: FrameSize::new(width, height),
            orientation: Orientation::Portrait,
            accepted_formats: vec![PixelFormat::Rgba8888],
            preferred_format: PixelFormat::Rgba8888,
            min_fps: 0.1,
            max_fps: 60.0,
            recommended_fps: 20.0,
            requires_continuous_frames: false,
            supports_brightness: true,
        })
    }

    pub fn last_frame(&self) -> Option<&RgbaFrame> {
        self.last_frame.as_ref()
    }

    pub fn take_last_frame(&mut self) -> Option<RgbaFrame> {
        self.last_frame.take()
    }

    pub const fn frame_count(&self) -> u64 {
        self.frame_count
    }

    pub const fn brightness(&self) -> u8 {
        self.brightness
    }

    fn store_frame(&mut self, frame: &RgbaFrame) -> Result<()> {
        let expected = self.capabilities.native_size;

        let actual = frame.size();

        if actual != expected {
            return Err(FakeDisplayError::InvalidFrameSize { expected, actual });
        }

        self.last_frame = Some(frame.clone());

        self.frame_count += 1;

        Ok(())
    }
}

impl DisplayDevice for FakeDisplay {
    type Error = FakeDisplayError;

    fn capabilities(&self) -> &DeviceCapabilities {
        &self.capabilities
    }

    fn send_rgba(&mut self, frame: &RgbaFrame) -> Result<()> {
        self.store_frame(frame)
    }

    fn send_encoded(&mut self, _format: PixelFormat, _bytes: &[u8]) -> Result<()> {
        Err(FakeDisplayError::EncodedFormatUnsupported)
    }

    fn set_brightness(&mut self, percent: u8) -> Result<()> {
        if percent > 100 {
            return Err(FakeDisplayError::InvalidBrightness(percent));
        }

        self.brightness = percent;

        Ok(())
    }
}

use openlcd_core::{DeviceCapabilities, PixelFormat, RgbaFrame};

pub trait DisplayDevice: Send {
    type Error: std::error::Error + Send + Sync + 'static;

    fn capabilities(&self) -> &DeviceCapabilities;

    fn send_rgba(&mut self, frame: &RgbaFrame) -> Result<(), Self::Error>;

    fn send_encoded(&mut self, format: PixelFormat, bytes: &[u8]) -> Result<(), Self::Error>;

    fn set_brightness(&mut self, _percent: u8) -> Result<(), Self::Error> {
        Ok(())
    }
}

use std::{fs, path::Path, time::Duration};

use crate::{
    error::Result,
    protocol,
    transport::{SerialTransport, Transport},
};

const DEFAULT_BAUD_RATE: u32 = 115_200;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

pub struct KmexDisplay<T: Transport> {
    transport: T,
}

impl KmexDisplay<SerialTransport> {
    pub fn open(port_name: &str) -> Result<Self> {
        Self::open_with_baud_rate(port_name, DEFAULT_BAUD_RATE)
    }

    pub fn open_with_baud_rate(port_name: &str, baud_rate: u32) -> Result<Self> {
        let transport = SerialTransport::open(port_name, baud_rate, DEFAULT_TIMEOUT)?;

        Ok(Self::new(transport))
    }
}

impl<T: Transport> KmexDisplay<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn send_control(&mut self, value: u8) -> Result<()> {
        let packet = protocol::encode_control(value);

        self.transport.write_all(&packet)?;
        self.transport.flush()?;

        Ok(())
    }

    pub fn send_jpeg(&mut self, jpeg: &[u8]) -> Result<()> {
        protocol::validate_jpeg(jpeg)?;

        self.transport.write_all(jpeg)?;
        self.transport.flush()?;

        Ok(())
    }

    pub fn send_jpeg_file(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let bytes = fs::read(path)?;
        self.send_jpeg(&bytes)
    }

    pub fn send_raw(&mut self, bytes: &[u8]) -> Result<()> {
        self.transport.write_all(bytes)?;
        self.transport.flush()?;

        Ok(())
    }
}
use openlcd_core::{DeviceCapabilities, PixelFormat, RgbaFrame};
use openlcd_driver::DisplayDevice;

use crate::capabilities::kmex_vmax_capabilities;

pub struct KmexDevice {
    display: KmexDisplay<SerialTransport>,
    capabilities: DeviceCapabilities,
}

impl KmexDevice {
    pub fn open(port_name: &str) -> Result<Self> {
        Ok(Self {
            display: KmexDisplay::open(port_name)?,
            capabilities: kmex_vmax_capabilities(),
        })
    }
}

impl DisplayDevice for KmexDevice {
    type Error = crate::KmexError;

    fn capabilities(&self) -> &DeviceCapabilities {
        &self.capabilities
    }

    fn send_rgba(&mut self, _frame: &RgbaFrame) -> std::result::Result<(), Self::Error> {
        Err(crate::KmexError::UnsupportedFormat(PixelFormat::Rgba8888))
    }

    fn send_encoded(
        &mut self,
        format: PixelFormat,
        bytes: &[u8],
    ) -> std::result::Result<(), Self::Error> {
        if format != PixelFormat::Jpeg {
            return Err(crate::KmexError::UnsupportedFormat(format));
        }

        self.display.send_jpeg(bytes)
    }
}

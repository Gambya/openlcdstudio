use std::{io::Write, time::Duration};

use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};

use crate::error::Result;

pub trait Transport {
    fn write_all(&mut self, data: &[u8]) -> Result<()>;

    fn flush(&mut self) -> Result<()>;
}

pub struct SerialTransport {
    port: Box<dyn SerialPort>,
}

impl SerialTransport {
    pub fn open(port_name: &str, baud_rate: u32, timeout: Duration) -> Result<Self> {
        let port = serialport::new(port_name, baud_rate)
            .data_bits(DataBits::Eight)
            .stop_bits(StopBits::One)
            .parity(Parity::None)
            .flow_control(FlowControl::None)
            .timeout(timeout)
            .open()?;

        Ok(Self { port })
    }
}

impl Transport for SerialTransport {
    fn write_all(&mut self, data: &[u8]) -> Result<()> {
        self.port.write_all(data)?;
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        self.port.flush()?;
        Ok(())
    }
}

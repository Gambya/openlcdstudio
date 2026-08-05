use crate::error::KMexError;
use crate::transport::Transport;

pub struct KMexCG02KV<T: Transport> {
    transport: T,
}

impl<T: Transport> KMexCG02KV<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
        }
    }

    pub fn send_raw(&mut self, data: &[u8]) -> Result<(), KMexError> {
        self.transport.write(data)
    }

    pub fn recv_raw(&mut self) -> Result<Vec<u8>, KMexError> {
        self.transport.read()
    }
}
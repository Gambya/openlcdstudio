use crate::error::KMexError;

pub trait Transport {
    fn write(&mut self, data: &[u8]) -> Result<(), KMexError>;
    fn read(&mut self) -> Result<Vec<u8>, KMexError>;
}
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KMexError {
    #[error("Serial error: {0}")]
    Serial(#[from] serialport::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Protocol error")]
    Protocol,
}
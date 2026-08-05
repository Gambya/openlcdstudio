use thiserror::Error;

#[derive(Debug, Error)]
pub enum KmexError {
    #[error("erro ao acessar a porta serial: {0}")]
    Serial(#[from] serialport::Error),

    #[error("erro de entrada/saída: {0}")]
    Io(#[from] std::io::Error),

    #[error("arquivo não é um JPEG válido")]
    InvalidJpeg,

    #[error("valor de controle inválido: {0}")]
    InvalidControlValue(u8),

    #[error("a porta serial não está aberta")]
    NotConnected,

    #[error("formato não suportado pelo K-MEX: {0:?}")]
    UnsupportedFormat(openlcd_core::PixelFormat),
}

pub type Result<T> = std::result::Result<T, KmexError>;

use openlcd_core::FrameSize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FakeDisplayError {
    #[error("frame com resolução inválida: esperado {expected:?}, recebido {actual:?}")]
    InvalidFrameSize {
        expected: FrameSize,
        actual: FrameSize,
    },

    #[error("formato codificado não suportado pelo fake driver")]
    EncodedFormatUnsupported,

    #[error("brilho deve estar entre 0 e 100, recebido {0}")]
    InvalidBrightness(u8),
}

pub type Result<T> = std::result::Result<T, FakeDisplayError>;

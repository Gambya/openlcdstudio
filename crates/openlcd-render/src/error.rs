use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("erro ao processar imagem: {0}")]
    Image(#[from] image::ImageError),

    #[error("erro no framebuffer: {0}")]
    Frame(#[from] openlcd_core::FrameError),

    #[error("não foi possível carregar a fonte")]
    InvalidFont,
}

pub type Result<T> = std::result::Result<T, RenderError>;

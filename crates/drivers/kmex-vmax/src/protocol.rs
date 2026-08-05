use crate::error::{KmexError, Result};

pub const CONTROL_HEADER: [u8; 2] = [0xaa, 0xbb];
pub const CONTROL_FOOTER: [u8; 2] = [0xcc, 0xdd];

pub const JPEG_START: [u8; 2] = [0xff, 0xd8];
pub const JPEG_END: [u8; 2] = [0xff, 0xd9];

pub fn encode_control(value: u8) -> [u8; 5] {
    [
        CONTROL_HEADER[0],
        CONTROL_HEADER[1],
        value,
        CONTROL_FOOTER[0],
        CONTROL_FOOTER[1],
    ]
}

pub fn validate_jpeg(bytes: &[u8]) -> Result<()> {
    if bytes.len() < 4 {
        return Err(KmexError::InvalidJpeg);
    }

    let starts_correctly = bytes.starts_with(&JPEG_START);

    let ends_correctly = bytes.ends_with(&JPEG_END);

    if !starts_correctly || !ends_correctly {
        return Err(KmexError::InvalidJpeg);
    }

    Ok(())
}

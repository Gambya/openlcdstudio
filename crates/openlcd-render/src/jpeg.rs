use std::io::Cursor;

use image::{ColorType, codecs::jpeg::JpegEncoder};
use openlcd_core::RgbaFrame;

use crate::RenderError;

pub fn encode_jpeg(frame: &RgbaFrame, quality: u8) -> Result<Vec<u8>, RenderError> {
    let size = frame.size();

    let rgb = rgba_to_rgb(frame.pixels());

    let mut output = Cursor::new(Vec::new());

    let mut encoder = JpegEncoder::new_with_quality(&mut output, quality);

    encoder.encode(&rgb, size.width, size.height, ColorType::Rgb8.into())?;

    Ok(output.into_inner())
}

fn rgba_to_rgb(rgba: &[u8]) -> Vec<u8> {
    let mut rgb = Vec::with_capacity(rgba.len() / 4 * 3);

    for pixel in rgba.chunks_exact(4) {
        rgb.extend_from_slice(&pixel[..3]);
    }

    rgb
}

#[cfg(test)]
mod tests {
    use openlcd_core::{FrameSize, RgbaFrame};

    use super::*;

    #[test]
    fn encodes_valid_jpeg() {
        let frame = RgbaFrame::new(
            FrameSize::new(2, 2),
            vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
            ],
        )
        .unwrap();

        let jpeg = encode_jpeg(&frame, 85).unwrap();

        assert!(jpeg.starts_with(&[0xff, 0xd8],));

        assert!(jpeg.ends_with(&[0xff, 0xd9],));
    }
}

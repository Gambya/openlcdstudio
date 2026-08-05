use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use anyhow::{Context, Result};

use crate::{
    media::{MediaEntry, MediaKind},
    usbpcap::{Direction, TransferType, UsbPacket},
};

const JPEG_SOI: [u8; 2] = [0xff, 0xd8];
const JPEG_EOI: [u8; 2] = [0xff, 0xd9];

pub fn extract_jpegs(output_dir: &Path, packets: &[UsbPacket]) -> Result<Vec<MediaEntry>> {
    let mut entries = Vec::new();
    let mut sequence = 0_u64;

    for packet in packets.iter().filter(is_candidate_packet) {
        let starts = find_all(&packet.payload, &JPEG_SOI);

        for (index_in_packet, start) in starts.into_iter().enumerate() {
            sequence += 1;

            let end_marker = find_subslice(&packet.payload[start + JPEG_SOI.len()..], &JPEG_EOI)
                .map(|relative| start + JPEG_SOI.len() + relative + JPEG_EOI.len());

            let end = end_marker.unwrap_or(packet.payload.len());

            if end <= start {
                continue;
            }

            let bytes = &packet.payload[start..end];
            let complete = end_marker.is_some();

            let extension = if complete { "jpg" } else { "partial.jpg" };

            let file_name = format!(
                "image-{sequence:06}-frame-{:010}-part-{:02}.{extension}",
                packet.frame_number,
                index_in_packet + 1,
            );

            let path = output_dir.join(&file_name);

            let file = File::create(&path)
                .with_context(|| format!("unable to create {}", path.display()))?;

            let mut writer = BufWriter::new(file);

            writer
                .write_all(bytes)
                .with_context(|| format!("unable to write {}", path.display()))?;

            writer.flush()?;

            let (width, height) = jpeg_dimensions(bytes).unwrap_or((None, None));

            entries.push(MediaEntry {
                kind: MediaKind::Jpeg,
                file: file_name,

                frame_number: packet.frame_number,
                timestamp_seconds: packet.timestamp_seconds,

                bus: packet.bus,
                device: packet.device,
                endpoint: packet.endpoint_number(),

                captured_packet_payload: packet.captured_payload_length,

                declared_packet_payload: packet.declared_data_length,

                packet_truncated: packet.truncated,

                media_offset: start,
                media_size: bytes.len(),
                complete,

                width,
                height,
            });
        }
    }

    Ok(entries)
}

fn is_candidate_packet(packet: &&UsbPacket) -> bool {
    packet.direction == Direction::Out
        && packet.transfer_type == TransferType::Bulk
        && packet.payload.len() >= JPEG_SOI.len()
}

fn find_all(haystack: &[u8], needle: &[u8]) -> Vec<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return Vec::new();
    }

    haystack
        .windows(needle.len())
        .enumerate()
        .filter_map(|(offset, window)| (window == needle).then_some(offset))
        .collect()
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }

    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn jpeg_dimensions(bytes: &[u8]) -> Result<(Option<u16>, Option<u16>)> {
    if bytes.len() < 4 || bytes[0..2] != JPEG_SOI {
        return Ok((None, None));
    }

    let mut offset = 2_usize;

    while offset + 1 < bytes.len() {
        while offset < bytes.len() && bytes[offset] != 0xff {
            offset += 1;
        }

        while offset < bytes.len() && bytes[offset] == 0xff {
            offset += 1;
        }

        if offset >= bytes.len() {
            break;
        }

        let marker = bytes[offset];
        offset += 1;

        match marker {
            0x00 | 0x01 | 0xd0..=0xd9 => continue,
            _ => {}
        }

        if offset + 2 > bytes.len() {
            break;
        }

        let segment_length = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]) as usize;

        if segment_length < 2 || offset + segment_length > bytes.len() {
            break;
        }

        if is_start_of_frame(marker) && segment_length >= 7 {
            let height = u16::from_be_bytes([bytes[offset + 3], bytes[offset + 4]]);

            let width = u16::from_be_bytes([bytes[offset + 5], bytes[offset + 6]]);

            return Ok((Some(width), Some(height)));
        }

        offset += segment_length;
    }

    Ok((None, None))
}

fn is_start_of_frame(marker: u8) -> bool {
    matches!(
        marker,
        0xc0 | 0xc1 | 0xc2 | 0xc3 | 0xc5 | 0xc6 | 0xc7 | 0xc9 | 0xca | 0xcb | 0xcd | 0xce | 0xcf
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_all_markers() {
        let bytes = [0xff, 0xd8, 1, 2, 0xff, 0xd8];

        assert_eq!(find_all(&bytes, &JPEG_SOI), vec![0, 4],);
    }

    #[test]
    fn detects_complete_jpeg_end() {
        let bytes = [0xff, 0xd8, 1, 2, 3, 0xff, 0xd9, 9];

        let start = 0;

        let end =
            find_subslice(&bytes[start + 2..], &JPEG_EOI).map(|relative| start + 2 + relative + 2);

        assert_eq!(end, Some(7));
    }
}

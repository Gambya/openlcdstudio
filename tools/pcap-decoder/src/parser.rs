use std::{fs::File, io::Read, path::Path};

use anyhow::{Context, Result, bail};

use crate::usbpcap::{LINKTYPE_USBPCAP, UsbPacket, parse_usbpcap_packet};

const GLOBAL_HEADER_LENGTH: usize = 24;
const RECORD_HEADER_LENGTH: usize = 16;

#[derive(Debug, Clone, Copy)]
pub enum ByteOrder {
    Little,
    Big,
}

#[derive(Debug, Clone, Copy)]
pub enum TimestampResolution {
    Microseconds,
    Nanoseconds,
}

#[derive(Debug)]
pub struct Capture {
    pub pcap_major: u16,
    pub pcap_minor: u16,
    pub snaplen: u32,
    pub link_type: u32,
    pub byte_order: ByteOrder,
    pub timestamp_resolution: TimestampResolution,
    pub packets: Vec<UsbPacket>,
    pub skipped_records: u64,
}

pub fn parse_file(path: &Path) -> Result<Capture> {
    let mut file =
        File::open(path).with_context(|| format!("unable to open {}", path.display()))?;

    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .with_context(|| format!("unable to read {}", path.display()))?;

    if bytes.len() < GLOBAL_HEADER_LENGTH {
        bail!("file is too small to contain a classic PCAP header");
    }

    let (byte_order, timestamp_resolution) = parse_magic(&bytes[0..4])?;
    let pcap_major = read_u16(&bytes, 4, byte_order)?;
    let pcap_minor = read_u16(&bytes, 6, byte_order)?;
    let snaplen = read_u32(&bytes, 16, byte_order)?;
    let link_type = read_u32(&bytes, 20, byte_order)?;

    if link_type != LINKTYPE_USBPCAP {
        bail!(
            "unsupported PCAP link type {link_type}; expected USBPcap link type {LINKTYPE_USBPCAP}"
        );
    }

    let mut offset = GLOBAL_HEADER_LENGTH;
    let mut frame_number = 0_u64;
    let mut packets = Vec::new();
    let mut skipped_records = 0_u64;

    while offset < bytes.len() {
        if bytes.len() - offset < RECORD_HEADER_LENGTH {
            bail!("truncated PCAP record header at file offset {offset}");
        }

        let ts_seconds = read_u32(&bytes, offset, byte_order)?;
        let ts_fraction = read_u32(&bytes, offset + 4, byte_order)?;
        let captured_length = read_u32(&bytes, offset + 8, byte_order)?;
        let original_length = read_u32(&bytes, offset + 12, byte_order)?;
        offset += RECORD_HEADER_LENGTH;

        let record_end = offset
            .checked_add(captured_length as usize)
            .context("PCAP record length overflow")?;

        if record_end > bytes.len() {
            bail!(
                "truncated PCAP record at file offset {offset}: expected {captured_length} bytes, only {} remain",
                bytes.len() - offset
            );
        }

        frame_number += 1;
        let timestamp_seconds = match timestamp_resolution {
            TimestampResolution::Microseconds => {
                ts_seconds as f64 + ts_fraction as f64 / 1_000_000.0
            }
            TimestampResolution::Nanoseconds => {
                ts_seconds as f64 + ts_fraction as f64 / 1_000_000_000.0
            }
        };

        let record_bytes = &bytes[offset..record_end];
        match parse_usbpcap_packet(
            frame_number,
            timestamp_seconds,
            captured_length,
            original_length,
            record_bytes,
        ) {
            Ok(packet) => packets.push(packet),
            Err(error) => {
                skipped_records += 1;
                eprintln!("warning: {error:#}");
            }
        }

        offset = record_end;
    }

    Ok(Capture {
        pcap_major,
        pcap_minor,
        snaplen,
        link_type,
        byte_order,
        timestamp_resolution,
        packets,
        skipped_records,
    })
}

fn parse_magic(bytes: &[u8]) -> Result<(ByteOrder, TimestampResolution)> {
    match bytes {
        [0xd4, 0xc3, 0xb2, 0xa1] => Ok((ByteOrder::Little, TimestampResolution::Microseconds)),
        [0xa1, 0xb2, 0xc3, 0xd4] => Ok((ByteOrder::Big, TimestampResolution::Microseconds)),
        [0x4d, 0x3c, 0xb2, 0xa1] => Ok((ByteOrder::Little, TimestampResolution::Nanoseconds)),
        [0xa1, 0xb2, 0x3c, 0x4d] => Ok((ByteOrder::Big, TimestampResolution::Nanoseconds)),
        _ => bail!(
            "unsupported capture format or magic number: {}",
            hex::encode(bytes)
        ),
    }
}

fn read_u16(bytes: &[u8], offset: usize, order: ByteOrder) -> Result<u16> {
    let slice = bytes
        .get(offset..offset + 2)
        .with_context(|| format!("missing u16 at offset {offset}"))?;
    Ok(match order {
        ByteOrder::Little => u16::from_le_bytes([slice[0], slice[1]]),
        ByteOrder::Big => u16::from_be_bytes([slice[0], slice[1]]),
    })
}

fn read_u32(bytes: &[u8], offset: usize, order: ByteOrder) -> Result<u32> {
    let slice = bytes
        .get(offset..offset + 4)
        .with_context(|| format!("missing u32 at offset {offset}"))?;
    Ok(match order {
        ByteOrder::Little => u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]),
        ByteOrder::Big => u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]),
    })
}

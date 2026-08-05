use anyhow::{Context, Result, bail};
use serde::Serialize;

pub const LINKTYPE_USBPCAP: u32 = 249;
pub const USBPCAP_BASE_HEADER_LEN: usize = 27;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    In,
    Out,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::In => f.write_str("in"),
            Self::Out => f.write_str("out"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferType {
    Isochronous,
    Interrupt,
    Control,
    Bulk,
    Unknown(u8),
}

impl From<u8> for TransferType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Isochronous,
            1 => Self::Interrupt,
            2 => Self::Control,
            3 => Self::Bulk,
            other => Self::Unknown(other),
        }
    }
}

impl std::fmt::Display for TransferType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Isochronous => f.write_str("isochronous"),
            Self::Interrupt => f.write_str("interrupt"),
            Self::Control => f.write_str("control"),
            Self::Bulk => f.write_str("bulk"),
            Self::Unknown(value) => write!(f, "unknown-{value}"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UsbPacket {
    pub frame_number: u64,
    pub timestamp_seconds: f64,
    pub captured_length: u32,
    pub original_length: u32,

    pub header_length: u16,
    pub irp_id: u64,
    pub status: u32,
    pub function: u16,
    pub info: u8,
    pub bus: u16,
    pub device: u16,
    pub endpoint: u8,
    pub direction: Direction,
    pub transfer_type: TransferType,
    pub declared_data_length: u32,
    pub captured_payload_length: usize,
    pub truncated: bool,

    #[serde(with = "hex_bytes")]
    pub payload: Vec<u8>,
}

impl UsbPacket {
    pub fn endpoint_number(&self) -> u8 {
        self.endpoint & 0x0f
    }

    pub fn endpoint_key(&self) -> String {
        format!(
            "bus{:03}-device{:03}-ep{:02x}-{}",
            self.bus,
            self.device,
            self.endpoint_number(),
            self.direction
        )
    }
}

pub fn parse_usbpcap_packet(
    frame_number: u64,
    timestamp_seconds: f64,
    captured_length: u32,
    original_length: u32,
    bytes: &[u8],
) -> Result<UsbPacket> {
    if bytes.len() < USBPCAP_BASE_HEADER_LEN {
        bail!(
            "frame {frame_number}: USBPcap packet is too short: {} bytes",
            bytes.len()
        );
    }

    let header_length = read_u16_le(bytes, 0)?;
    let header_length_usize = usize::from(header_length);

    if header_length_usize < USBPCAP_BASE_HEADER_LEN {
        bail!("frame {frame_number}: invalid USBPcap header length {header_length}");
    }

    if header_length_usize > bytes.len() {
        bail!(
            "frame {frame_number}: USBPcap header length {header_length} exceeds captured packet length {}",
            bytes.len()
        );
    }

    let irp_id = read_u64_le(bytes, 2)?;
    let status = read_u32_le(bytes, 10)?;
    let function = read_u16_le(bytes, 14)?;
    let info = *bytes.get(16).context("missing USBPcap info field")?;
    let bus = read_u16_le(bytes, 17)?;
    let device = read_u16_le(bytes, 19)?;
    let endpoint = *bytes.get(21).context("missing USBPcap endpoint field")?;
    let transfer_raw = *bytes.get(22).context("missing USBPcap transfer field")?;
    let declared_data_length = read_u32_le(bytes, 23)?;

    let direction = if endpoint & 0x80 != 0 {
        Direction::In
    } else {
        Direction::Out
    };

    let payload = bytes[header_length_usize..].to_vec();
    let captured_payload_length = payload.len();
    let truncated = captured_payload_length < declared_data_length as usize;

    Ok(UsbPacket {
        frame_number,
        timestamp_seconds,
        captured_length,
        original_length,
        header_length,
        irp_id,
        status,
        function,
        info,
        bus,
        device,
        endpoint,
        direction,
        transfer_type: transfer_raw.into(),
        declared_data_length,
        captured_payload_length,
        truncated,
        payload,
    })
}

fn read_u16_le(bytes: &[u8], offset: usize) -> Result<u16> {
    let slice = bytes
        .get(offset..offset + 2)
        .with_context(|| format!("missing u16 at offset {offset}"))?;
    Ok(u16::from_le_bytes([slice[0], slice[1]]))
}

fn read_u32_le(bytes: &[u8], offset: usize) -> Result<u32> {
    let slice = bytes
        .get(offset..offset + 4)
        .with_context(|| format!("missing u32 at offset {offset}"))?;
    Ok(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

fn read_u64_le(bytes: &[u8], offset: usize) -> Result<u64> {
    let slice = bytes
        .get(offset..offset + 8)
        .with_context(|| format!("missing u64 at offset {offset}"))?;
    Ok(u64::from_le_bytes([
        slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7],
    ]))
}

mod hex_bytes {
    use serde::Serializer;

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_observed_bulk_header() {
        let mut packet = vec![0_u8; USBPCAP_BASE_HEADER_LEN];
        packet[0..2].copy_from_slice(&27_u16.to_le_bytes());
        packet[14..16].copy_from_slice(&9_u16.to_le_bytes());
        packet[17..19].copy_from_slice(&3_u16.to_le_bytes());
        packet[19..21].copy_from_slice(&2_u16.to_le_bytes());
        packet[21] = 0x02;
        packet[22] = 3;
        packet[23..27].copy_from_slice(&4_u32.to_le_bytes());
        packet.extend_from_slice(&[0xff, 0xd8, 0xff, 0xe0]);

        let parsed = parse_usbpcap_packet(1, 0.0, 31, 31, &packet).unwrap();
        assert_eq!(parsed.bus, 3);
        assert_eq!(parsed.device, 2);
        assert_eq!(parsed.endpoint, 0x02);
        assert_eq!(parsed.direction, Direction::Out);
        assert_eq!(parsed.transfer_type, TransferType::Bulk);
        assert_eq!(parsed.payload, [0xff, 0xd8, 0xff, 0xe0]);
        assert!(!parsed.truncated);
    }
}

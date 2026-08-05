use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
};

use anyhow::{Context, Result};

use crate::usbpcap::UsbPacket;

pub fn write_endpoint_dumps(output_dir: &Path, packets: &[UsbPacket]) -> Result<()> {
    fs::create_dir_all(output_dir)?;

    let mut grouped: BTreeMap<String, Vec<&UsbPacket>> = BTreeMap::new();
    for packet in packets.iter().filter(|packet| !packet.payload.is_empty()) {
        grouped
            .entry(packet.endpoint_key())
            .or_default()
            .push(packet);
    }

    for (key, packets) in grouped {
        let binary_path = output_dir.join(format!("{key}.bin"));
        let index_path = output_dir.join(format!("{key}.csv"));

        let binary_file = File::create(&binary_path)
            .with_context(|| format!("unable to create {}", binary_path.display()))?;
        let index_file = File::create(&index_path)
            .with_context(|| format!("unable to create {}", index_path.display()))?;

        let mut binary = BufWriter::new(binary_file);
        let mut index = BufWriter::new(index_file);

        writeln!(
            index,
            "frame,timestamp,offset,captured_payload,declared_payload,truncated,transfer,irp_id"
        )?;

        let mut offset = 0_u64;
        for packet in packets {
            binary.write_all(&packet.payload)?;
            writeln!(
                index,
                "{},{:.6},{},{},{},{},{},{}",
                packet.frame_number,
                packet.timestamp_seconds,
                offset,
                packet.captured_payload_length,
                packet.declared_data_length,
                packet.truncated,
                packet.transfer_type,
                packet.irp_id,
            )?;
            offset += packet.payload.len() as u64;
        }
    }

    Ok(())
}

pub fn write_jsonl(path: &Path, packets: &[UsbPacket]) -> Result<()> {
    let file =
        File::create(path).with_context(|| format!("unable to create {}", path.display()))?;
    let mut writer = BufWriter::new(file);

    for packet in packets {
        serde_json::to_writer(&mut writer, packet)?;
        writer.write_all(b"\n")?;
    }

    Ok(())
}

use std::{fmt::Write as _, fs::File, io::Write, path::Path};

use anyhow::{Context, Result};

use crate::{
    parser::Capture,
    statistics::{common_prefixes, endpoint_statistics},
};

pub fn print_capture_summary(capture: &Capture) {
    let payload_packets = capture
        .packets
        .iter()
        .filter(|packet| !packet.payload.is_empty())
        .count();
    let truncated_packets = capture
        .packets
        .iter()
        .filter(|packet| packet.truncated)
        .count();

    println!("Capture");
    println!("=======");
    println!(
        "PCAP version:           {}.{}",
        capture.pcap_major, capture.pcap_minor
    );
    println!("byte order:             {:?}", capture.byte_order);
    println!("timestamp resolution:  {:?}", capture.timestamp_resolution);
    println!("snaplen:                {}", capture.snaplen);
    println!("link type:              {}", capture.link_type);
    println!("parsed USB packets:     {}", capture.packets.len());
    println!("packets with payload:   {payload_packets}");
    println!("truncated payloads:     {truncated_packets}");
    println!("skipped records:        {}", capture.skipped_records);
}

pub fn write_text_report(
    path: &Path,
    capture: &Capture,
    prefix_bytes: usize,
    top_prefixes: usize,
) -> Result<()> {
    let mut output = String::new();

    writeln!(output, "Capture")?;
    writeln!(output, "=======")?;
    writeln!(
        output,
        "PCAP version: {}.{}",
        capture.pcap_major, capture.pcap_minor
    )?;
    writeln!(output, "byte order: {:?}", capture.byte_order)?;
    writeln!(
        output,
        "timestamp resolution: {:?}",
        capture.timestamp_resolution
    )?;
    writeln!(output, "snaplen: {}", capture.snaplen)?;
    writeln!(output, "link type: {}", capture.link_type)?;
    writeln!(output, "parsed USB packets: {}", capture.packets.len())?;
    writeln!(output, "skipped records: {}", capture.skipped_records)?;

    writeln!(output, "\nEndpoints")?;
    writeln!(output, "=========")?;
    for (key, stats) in endpoint_statistics(&capture.packets) {
        let average = if stats.packets_with_payload == 0 {
            0.0
        } else {
            stats.captured_payload_bytes as f64 / stats.packets_with_payload as f64
        };

        writeln!(output, "{key}")?;
        writeln!(output, "  packets: {}", stats.packet_count)?;
        writeln!(
            output,
            "  packets with payload: {}",
            stats.packets_with_payload
        )?;
        writeln!(
            output,
            "  captured payload bytes: {}",
            stats.captured_payload_bytes
        )?;
        writeln!(
            output,
            "  declared payload bytes: {}",
            stats.declared_payload_bytes
        )?;
        writeln!(output, "  minimum payload: {}", stats.min_payload_size)?;
        writeln!(output, "  maximum payload: {}", stats.max_payload_size)?;
        writeln!(output, "  average payload: {average:.2}")?;
        writeln!(output, "  truncated packets: {}", stats.truncated_packets)?;
        writeln!(output)?;
    }

    writeln!(output, "Common payload prefixes ({prefix_bytes} bytes)")?;
    writeln!(output, "==================================")?;
    for (prefix, count) in common_prefixes(&capture.packets, prefix_bytes, top_prefixes) {
        writeln!(output, "{count:>10}  {prefix}")?;
    }

    let mut file =
        File::create(path).with_context(|| format!("unable to create {}", path.display()))?;
    file.write_all(output.as_bytes())?;
    Ok(())
}

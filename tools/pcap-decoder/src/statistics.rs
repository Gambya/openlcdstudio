use std::collections::{BTreeMap, HashMap};

use crate::usbpcap::{TransferType, UsbPacket};

#[derive(Debug, Clone, Default)]
pub struct EndpointStats {
    pub packet_count: u64,
    pub packets_with_payload: u64,
    pub captured_payload_bytes: u64,
    pub declared_payload_bytes: u64,
    pub min_payload_size: usize,
    pub max_payload_size: usize,
    pub truncated_packets: u64,
    pub transfer_counts: HashMap<TransferType, u64>,
}

pub fn endpoint_statistics(packets: &[UsbPacket]) -> BTreeMap<String, EndpointStats> {
    let mut result = BTreeMap::new();

    for packet in packets {
        let stats = result
            .entry(packet.endpoint_key())
            .or_insert_with(|| EndpointStats {
                min_payload_size: usize::MAX,
                ..EndpointStats::default()
            });

        stats.packet_count += 1;
        stats.captured_payload_bytes += packet.captured_payload_length as u64;
        stats.declared_payload_bytes += packet.declared_data_length as u64;
        *stats
            .transfer_counts
            .entry(packet.transfer_type)
            .or_default() += 1;

        if !packet.payload.is_empty() {
            stats.packets_with_payload += 1;
            stats.min_payload_size = stats.min_payload_size.min(packet.captured_payload_length);
            stats.max_payload_size = stats.max_payload_size.max(packet.captured_payload_length);
        }

        if packet.truncated {
            stats.truncated_packets += 1;
        }
    }

    for stats in result.values_mut() {
        if stats.packets_with_payload == 0 {
            stats.min_payload_size = 0;
        }
    }

    result
}

pub fn common_prefixes(
    packets: &[UsbPacket],
    prefix_bytes: usize,
    limit: usize,
) -> Vec<(String, u64)> {
    if prefix_bytes == 0 || limit == 0 {
        return Vec::new();
    }

    let mut counts: HashMap<String, u64> = HashMap::new();
    for packet in packets.iter().filter(|packet| !packet.payload.is_empty()) {
        let end = packet.payload.len().min(prefix_bytes);
        let prefix = hex::encode(&packet.payload[..end]);
        *counts.entry(prefix).or_default() += 1;
    }

    let mut entries: Vec<_> = counts.into_iter().collect();
    entries.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    entries.truncate(limit);
    entries
}

pub fn print_endpoint_statistics(packets: &[UsbPacket]) {
    println!("\nEndpoints");
    println!("=========");

    for (key, stats) in endpoint_statistics(packets) {
        let average = if stats.packets_with_payload == 0 {
            0.0
        } else {
            stats.captured_payload_bytes as f64 / stats.packets_with_payload as f64
        };

        println!("{key}");
        println!("  packets:                 {}", stats.packet_count);
        println!("  packets with payload:    {}", stats.packets_with_payload);
        println!(
            "  captured payload bytes:  {}",
            stats.captured_payload_bytes
        );
        println!(
            "  declared payload bytes:  {}",
            stats.declared_payload_bytes
        );
        println!("  minimum payload:         {}", stats.min_payload_size);
        println!("  maximum payload:         {}", stats.max_payload_size);
        println!("  average payload:         {average:.2}");
        println!("  truncated packets:       {}", stats.truncated_packets);

        let mut transfers: Vec<_> = stats.transfer_counts.into_iter().collect();
        transfers.sort_by_key(|(transfer, _)| transfer.to_string());
        for (transfer, count) in transfers {
            println!("  transfer {transfer:<12} {count}");
        }
        println!();
    }
}

pub fn print_common_prefixes(packets: &[UsbPacket], prefix_bytes: usize, limit: usize) {
    println!("Common payload prefixes ({prefix_bytes} bytes)");
    println!("==================================");

    for (prefix, count) in common_prefixes(packets, prefix_bytes, limit) {
        println!("{count:>10}  {prefix}");
    }
}

use std::{
    fs::{self, File},
    io::BufWriter,
    path::Path,
};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::{jpeg, usbpcap::UsbPacket};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaKind {
    Jpeg,
}

#[derive(Debug, Clone, Serialize)]
pub struct MediaEntry {
    pub kind: MediaKind,
    pub file: String,

    pub frame_number: u64,
    pub timestamp_seconds: f64,

    pub bus: u16,
    pub device: u16,
    pub endpoint: u8,

    pub captured_packet_payload: usize,
    pub declared_packet_payload: u32,
    pub packet_truncated: bool,

    pub media_offset: usize,
    pub media_size: usize,
    pub complete: bool,

    pub width: Option<u16>,
    pub height: Option<u16>,
}

#[derive(Debug, Serialize)]
pub struct MediaManifest {
    pub format_version: u32,
    pub extracted_files: usize,
    pub complete_files: usize,
    pub partial_files: usize,
    pub entries: Vec<MediaEntry>,
}

pub fn extract_media(output_dir: &Path, packets: &[UsbPacket]) -> Result<MediaManifest> {
    fs::create_dir_all(output_dir).with_context(|| {
        format!(
            "unable to create media output directory {}",
            output_dir.display()
        )
    })?;

    let entries = jpeg::extract_jpegs(output_dir, packets)?;

    let complete_files = entries.iter().filter(|entry| entry.complete).count();

    let partial_files = entries.len().saturating_sub(complete_files);

    let manifest = MediaManifest {
        format_version: 1,
        extracted_files: entries.len(),
        complete_files,
        partial_files,
        entries,
    };

    let manifest_path = output_dir.join("manifest.json");

    let file = File::create(&manifest_path)
        .with_context(|| format!("unable to create {}", manifest_path.display()))?;

    let writer = BufWriter::new(file);

    serde_json::to_writer_pretty(writer, &manifest)
        .with_context(|| format!("unable to write {}", manifest_path.display()))?;

    Ok(manifest)
}

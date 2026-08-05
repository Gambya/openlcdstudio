mod dump;
mod jpeg;
mod media;
mod parser;
mod report;
mod statistics;
mod usbpcap;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    /// Capture produced by USBPcap.
    /// The current implementation reads
    /// classic PCAP files.
    input: PathBuf,

    /// Directory used for endpoint dumps,
    /// reports, and extracted media.
    #[arg(short, long, default_value = "pcap-decode-output")]
    output: PathBuf,

    /// Do not create concatenated endpoint
    /// payload dumps.
    #[arg(long)]
    no_dump: bool,

    /// Write one JSON object per parsed
    /// USB packet.
    #[arg(long)]
    jsonl: bool,

    /// Extract recognized media files
    /// from USB payloads.
    #[arg(long)]
    extract_media: bool,

    /// Number of bytes used when grouping
    /// payload prefixes.
    #[arg(long, default_value_t = 8)]
    prefix_bytes: usize,

    /// Number of most common payload
    /// prefixes shown in the report.
    #[arg(long, default_value_t = 20)]
    top_prefixes: usize,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let capture = parser::parse_file(&args.input)?;

    std::fs::create_dir_all(&args.output)?;

    report::print_capture_summary(&capture);

    statistics::print_endpoint_statistics(&capture.packets);

    statistics::print_common_prefixes(&capture.packets, args.prefix_bytes, args.top_prefixes);

    report::write_text_report(
        &args.output.join("report.txt"),
        &capture,
        args.prefix_bytes,
        args.top_prefixes,
    )?;

    if !args.no_dump {
        dump::write_endpoint_dumps(&args.output.join("dumps"), &capture.packets)?;
    }

    if args.jsonl {
        dump::write_jsonl(&args.output.join("packets.jsonl"), &capture.packets)?;
    }

    if args.extract_media {
        let manifest = media::extract_media(&args.output.join("media"), &capture.packets)?;

        println!(
            concat!(
                "\nMedia extraction: ",
                "{} files ",
                "({} complete, ",
                "{} partial)"
            ),
            manifest.extracted_files, manifest.complete_files, manifest.partial_files,
        );
    }

    println!("\nOutput written to {}", args.output.display(),);

    Ok(())
}

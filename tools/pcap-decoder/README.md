# pcap-decode

Small USBPcap analyzer used during reverse engineering of the K-MEX/HL VMAX display.

The supplied capture is a **classic PCAP** file with link type `249` (`LINKTYPE_USBPCAP`), even though its filename uses the `.pcapng` extension.

## Run

```bash
cargo run --release -- /path/to/vmax.pcapng --jsonl
```

Default output:

```text
pcap-decode-output/
├── report.txt
├── packets.jsonl
└── dumps/
    ├── bus003-device002-ep02-out.bin
    └── bus003-device002-ep02-out.csv
```

The CSV index records where each USB payload starts in the concatenated `.bin` file.

## Important capture limitation

The provided capture uses a snap length of 65535 bytes. Several HL VMAX bulk transfers declare payloads larger than that value, so those records are truncated. The analyzer marks them with `truncated = true`.

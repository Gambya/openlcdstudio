use std::{thread, time::Duration};

use openlcd_monitor::MonitorService;

fn main() {
    let monitor = MonitorService::start();

    loop {
        if let Some(data) = monitor.latest() {
            println!(
                concat!(
                    "CPU: {:?}% | ",
                    "Freq: {:?} MHz | ",
                    "RAM: {:?}% | ",
                    "Down: {:?} B/s | ",
                    "Up: {:?} B/s | ",
                    "{} {}"
                ),
                data.cpu_usage,
                data.cpu_frequency_mhz,
                data.memory_usage,
                data.network_download_bytes_per_second,
                data.network_upload_bytes_per_second,
                data.date.as_deref().unwrap_or("--"),
                data.clock.as_deref().unwrap_or("--"),
            );
        }

        thread::sleep(Duration::from_millis(100));
    }
}

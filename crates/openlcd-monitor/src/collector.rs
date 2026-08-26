use std::time::{Duration, Instant};

use chrono::Local;

use openlcd_runtime::RuntimeData;

use sysinfo::{Networks, System};

pub struct SystemCollector {
    system: System,
    networks: Networks,

    last_network_refresh: Instant,

    initialized: bool,
}

impl SystemCollector {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),

            networks: Networks::new_with_refreshed_list(),

            last_network_refresh: Instant::now(),

            initialized: false,
        }
    }

    pub fn collect(&mut self) -> RuntimeData {
        self.system.refresh_cpu_usage();
        self.system.refresh_cpu_frequency();
        self.system.refresh_memory();

        let now = Instant::now();

        let elapsed = now.duration_since(self.last_network_refresh);

        self.networks.refresh(true);

        self.last_network_refresh = now;

        let cpu_usage = if self.initialized {
            Some(self.system.global_cpu_usage() as f64)
        } else {
            None
        };

        let cpu_frequency_mhz = average_cpu_frequency(&self.system);

        let memory_usage = memory_usage_percent(&self.system);

        let (network_upload, network_download) = network_rates(&self.networks, elapsed);

        let local_time = Local::now();

        self.initialized = true;

        RuntimeData {
            cpu_usage,

            cpu_frequency_mhz,

            memory_usage,

            network_upload_bytes_per_second: network_upload,

            network_download_bytes_per_second: network_download,

            clock: Some(local_time.format("%H:%M:%S").to_string()),

            date: Some(local_time.format("%d/%m/%Y").to_string()),

            //
            // Ainda não implementados nesta fase.
            //
            cpu_temperature: None,
            cpu_voltage: None,

            gpu_usage: None,
            gpu_temperature: None,
            gpu_frequency_mhz: None,

            memory_frequency_mhz: None,

            disk_temperature: None,

            wifi_state: None,
            wifi_name: None,
            wifi_ssid: None,

            fps: None,
        }
    }
}

impl Default for SystemCollector {
    fn default() -> Self {
        Self::new()
    }
}

fn average_cpu_frequency(system: &System) -> Option<f64> {
    let cpus = system.cpus();

    if cpus.is_empty() {
        return None;
    }

    let total: u64 = cpus.iter().map(|cpu| cpu.frequency()).sum();

    Some(total as f64 / cpus.len() as f64)
}

fn memory_usage_percent(system: &System) -> Option<f64> {
    let total = system.total_memory();

    if total == 0 {
        return None;
    }

    Some(system.used_memory() as f64 / total as f64 * 100.0)
}

fn network_rates(networks: &Networks, elapsed: Duration) -> (Option<f64>, Option<f64>) {
    let seconds = elapsed.as_secs_f64();

    if seconds <= 0.0 {
        return (None, None);
    }

    let mut transmitted = 0_u64;

    let mut received = 0_u64;

    for (interface_name, network) in networks {
        if should_ignore_interface(interface_name) {
            continue;
        }

        transmitted = transmitted.saturating_add(network.transmitted());

        received = received.saturating_add(network.received());
    }

    (
        Some(transmitted as f64 / seconds),
        Some(received as f64 / seconds),
    )
}

fn should_ignore_interface(name: &str) -> bool {
    name == "lo"
}

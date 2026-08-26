#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeData {
    pub cpu_usage: Option<f64>,
    pub cpu_temperature: Option<f64>,
    pub cpu_frequency_mhz: Option<f64>,
    pub cpu_voltage: Option<f64>,

    pub gpu_usage: Option<f64>,
    pub gpu_temperature: Option<f64>,
    pub gpu_frequency_mhz: Option<f64>,

    pub memory_usage: Option<f64>,
    pub memory_frequency_mhz: Option<f64>,

    pub disk_temperature: Option<f64>,

    pub network_upload_bytes_per_second: Option<f64>,

    pub network_download_bytes_per_second: Option<f64>,

    pub clock: Option<String>,
    pub date: Option<String>,

    pub wifi_state: Option<String>,
    pub wifi_name: Option<String>,
    pub wifi_ssid: Option<String>,

    pub fps: Option<f64>,
}

impl Default for RuntimeData {
    fn default() -> Self {
        Self {
            cpu_usage: None,
            cpu_temperature: None,
            cpu_frequency_mhz: None,
            cpu_voltage: None,

            gpu_usage: None,
            gpu_temperature: None,
            gpu_frequency_mhz: None,

            memory_usage: None,
            memory_frequency_mhz: None,

            disk_temperature: None,

            network_upload_bytes_per_second: None,

            network_download_bytes_per_second: None,

            clock: None,
            date: None,

            wifi_state: None,
            wifi_name: None,
            wifi_ssid: None,

            fps: None,
        }
    }
}

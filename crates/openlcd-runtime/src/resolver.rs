use openlcd_theme::{DataBinding, Layer, Scene, TextBinding};

use crate::RuntimeData;

#[derive(Debug, Default)]
pub struct SceneResolver;

impl SceneResolver {
    pub const fn new() -> Self {
        Self
    }

    pub fn resolve(&self, scene: &Scene, data: &RuntimeData) -> Scene {
        let mut resolved = scene.clone();

        for layer in &mut resolved.layers {
            let Layer::Text(text) = layer else {
                continue;
            };

            let Some(binding) = &text.binding else {
                continue;
            };

            if let Some(value) = resolve_binding(binding, data) {
                text.text = value;
            }
        }

        resolved
    }
}

fn resolve_binding(binding: &TextBinding, data: &RuntimeData) -> Option<String> {
    let value = match binding.source {
        DataBinding::CpuUsage => format_number(data.cpu_usage?, binding.precision),

        DataBinding::CpuTemperature => format_number(data.cpu_temperature?, binding.precision),

        DataBinding::CpuFrequency => format_number(data.cpu_frequency_mhz?, binding.precision),

        DataBinding::CpuVoltage => format_number(data.cpu_voltage?, binding.precision),

        DataBinding::GpuUsage => format_number(data.gpu_usage?, binding.precision),

        DataBinding::GpuTemperature => format_number(data.gpu_temperature?, binding.precision),

        DataBinding::GpuFrequency => format_number(data.gpu_frequency_mhz?, binding.precision),

        DataBinding::MemoryUsage => format_number(data.memory_usage?, binding.precision),

        DataBinding::MemoryFrequency => {
            format_number(data.memory_frequency_mhz?, binding.precision)
        }

        DataBinding::DiskTemperature => format_number(data.disk_temperature?, binding.precision),

        DataBinding::NetworkUpload => {
            format_number(data.network_upload_bytes_per_second?, binding.precision)
        }

        DataBinding::NetworkDownload => {
            format_number(data.network_download_bytes_per_second?, binding.precision)
        }

        DataBinding::Clock => data.clock.clone()?,

        DataBinding::Date => data.date.clone()?,

        DataBinding::WifiState => data.wifi_state.clone()?,

        DataBinding::WifiName => data.wifi_name.clone()?,

        DataBinding::WifiSsid => data.wifi_ssid.clone()?,

        DataBinding::Fps => format_number(data.fps?, binding.precision),
    };

    Some(format!("{}{}{}", binding.prefix, value, binding.suffix,))
}

fn format_number(value: f64, precision: u8) -> String {
    format!("{value:.precision$}", precision = precision as usize,)
}

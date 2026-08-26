use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataBinding {
    CpuUsage,
    CpuTemperature,
    CpuFrequency,
    CpuVoltage,

    GpuUsage,
    GpuTemperature,
    GpuFrequency,

    MemoryUsage,
    MemoryFrequency,

    DiskTemperature,

    NetworkUpload,
    NetworkDownload,

    Clock,
    Date,

    WifiState,
    WifiName,
    WifiSsid,

    Fps,
}

impl DataBinding {
    pub const ALL: [Self; 18] = [
        Self::CpuUsage,
        Self::CpuTemperature,
        Self::CpuFrequency,
        Self::CpuVoltage,
        Self::GpuUsage,
        Self::GpuTemperature,
        Self::GpuFrequency,
        Self::MemoryUsage,
        Self::MemoryFrequency,
        Self::DiskTemperature,
        Self::NetworkUpload,
        Self::NetworkDownload,
        Self::Clock,
        Self::Date,
        Self::WifiState,
        Self::WifiName,
        Self::WifiSsid,
        Self::Fps,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::CpuUsage => "CPU Usage",
            Self::CpuTemperature => "CPU Temperature",
            Self::CpuFrequency => "CPU Frequency",
            Self::CpuVoltage => "CPU Voltage",

            Self::GpuUsage => "GPU Usage",
            Self::GpuTemperature => "GPU Temperature",
            Self::GpuFrequency => "GPU Frequency",

            Self::MemoryUsage => "Memory Usage",
            Self::MemoryFrequency => "Memory Frequency",

            Self::DiskTemperature => "Disk Temperature",

            Self::NetworkUpload => "Network Upload",
            Self::NetworkDownload => "Network Download",

            Self::Clock => "Clock",
            Self::Date => "Date",

            Self::WifiState => "Wi-Fi State",
            Self::WifiName => "Wi-Fi Name",
            Self::WifiSsid => "Wi-Fi SSID",

            Self::Fps => "FPS",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextBinding {
    pub source: DataBinding,

    #[serde(default)]
    pub prefix: String,

    #[serde(default)]
    pub suffix: String,

    #[serde(default = "default_precision")]
    pub precision: u8,
}

impl TextBinding {
    pub fn new(source: DataBinding) -> Self {
        Self {
            source,
            prefix: String::new(),
            suffix: String::new(),
            precision: default_precision(),
        }
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    pub fn with_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = suffix.into();
        self
    }

    pub const fn with_precision(mut self, precision: u8) -> Self {
        self.precision = precision;
        self
    }
}

const fn default_precision() -> u8 {
    0
}

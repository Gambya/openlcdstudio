use std::{
    fs,
    path::{Path, PathBuf},
};

const HWMON_ROOT: &str = "/sys/class/hwmon";

#[derive(Debug, Clone, PartialEq)]
pub struct TemperatureSensor {
    pub hwmon_name: String,
    pub label: Option<String>,
    pub input_path: PathBuf,
}

impl TemperatureSensor {
    pub fn read_celsius(&self) -> Option<f64> {
        read_millivalue(&self.input_path).map(|value| value / 1000.0)
    }

    pub fn display_name(&self) -> String {
        match &self.label {
            Some(label) => {
                format!("{}: {}", self.hwmon_name, label,)
            }

            None => self.hwmon_name.clone(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct HwmonSensors {
    pub temperatures: Vec<TemperatureSensor>,
}

impl HwmonSensors {
    pub fn discover() -> Self {
        Self::discover_from(Path::new(HWMON_ROOT))
    }

    fn discover_from(root: &Path) -> Self {
        let mut result = Self::default();

        let Ok(entries) = fs::read_dir(root) else {
            return result;
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if !path.is_dir() && !path.is_symlink() {
                continue;
            }

            let hwmon_name = read_trimmed(&path.join("name"))
                .unwrap_or_else(|| entry.file_name().to_string_lossy().into_owned());

            discover_temperature_channels(&path, &hwmon_name, &mut result.temperatures);
        }

        result
    }

    pub fn cpu_temperature(&self) -> Option<f64> {
        select_cpu_sensor(&self.temperatures).and_then(TemperatureSensor::read_celsius)
    }

    pub fn disk_temperature(&self) -> Option<f64> {
        select_disk_sensor(&self.temperatures).and_then(TemperatureSensor::read_celsius)
    }
}

fn discover_temperature_channels(
    directory: &Path,
    hwmon_name: &str,
    output: &mut Vec<TemperatureSensor>,
) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };

    for entry in entries.flatten() {
        let file_name = entry.file_name();

        let file_name = file_name.to_string_lossy();

        let Some(channel) = temperature_input_channel(&file_name) else {
            continue;
        };

        let input_path = entry.path();

        //
        // If we can't read the value,
        // don't expose this channel.
        //
        if read_millivalue(&input_path).is_none() {
            continue;
        }

        let label_path = directory.join(format!("temp{channel}_label"));

        let label = read_trimmed(&label_path);

        output.push(TemperatureSensor {
            hwmon_name: hwmon_name.to_owned(),

            label,

            input_path,
        });
    }
}

fn temperature_input_channel(file_name: &str) -> Option<u32> {
    let middle = file_name.strip_prefix("temp")?.strip_suffix("_input")?;

    middle.parse().ok()
}

fn select_cpu_sensor(sensors: &[TemperatureSensor]) -> Option<&TemperatureSensor> {
    sensors
        .iter()
        .filter_map(|sensor| {
            let score = cpu_sensor_score(sensor);

            (score > 0).then_some((score, sensor))
        })
        .max_by_key(|(score, _)| *score)
        .map(|(_, sensor)| sensor)
}

fn cpu_sensor_score(sensor: &TemperatureSensor) -> u32 {
    let driver = sensor.hwmon_name.to_ascii_lowercase();

    let label = sensor.label.as_deref().unwrap_or("").to_ascii_lowercase();

    let mut score = 0;

    //
    // Driver clearly associated with the CPU.
    //
    if matches!(
        driver.as_str(),
        "coretemp" | "k10temp" | "zenpower" | "peci_cputemp" | "cpu_thermal"
    ) {
        score += 100;
    }

    //
    // Package/Tctl/Tdie are generally
    // the best representatives of the CPU's
    // global temperature than an individual core.
    //
    if label.contains("package") {
        score += 80;
    }

    if label.contains("tctl") {
        score += 75;
    }

    if label.contains("tdie") {
        score += 70;
    }

    if label.contains("cpu") {
        score += 60;
    }

    if label.contains("die") {
        score += 50;
    }

    if label.contains("core") {
        score += 30;
    }

    score
}

fn select_disk_sensor(sensors: &[TemperatureSensor]) -> Option<&TemperatureSensor> {
    sensors
        .iter()
        .filter_map(|sensor| {
            let score = disk_sensor_score(sensor);

            (score > 0).then_some((score, sensor))
        })
        .max_by_key(|(score, _)| *score)
        .map(|(_, sensor)| sensor)
}

fn disk_sensor_score(sensor: &TemperatureSensor) -> u32 {
    let driver = sensor.hwmon_name.to_ascii_lowercase();

    let label = sensor.label.as_deref().unwrap_or("").to_ascii_lowercase();

    let mut score = 0;

    if driver == "drivetemp" {
        score += 100;
    }

    if driver == "nvme" {
        score += 90;
    }

    if label.contains("composite") {
        score += 40;
    }

    if label.contains("drive") || label.contains("disk") {
        score += 30;
    }

    score
}

fn read_trimmed(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn read_millivalue(path: &Path) -> Option<f64> {
    read_trimmed(path)?.parse::<f64>().ok()
}

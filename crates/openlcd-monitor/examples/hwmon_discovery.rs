use openlcd_monitor::HwmonSensors;

fn main() {
    let sensors = HwmonSensors::discover();

    println!(
        "{} sensores de temperatura encontrados",
        sensors.temperatures.len(),
    );

    for sensor in &sensors.temperatures {
        println!(
            "{} | {:?} °C | {}",
            sensor.display_name(),
            sensor.read_celsius(),
            sensor.input_path.display(),
        );
    }

    println!();

    println!("CPU selecionada: {:?} °C", sensors.cpu_temperature(),);

    println!("Disco selecionado: {:?} °C", sensors.disk_temperature(),);
}

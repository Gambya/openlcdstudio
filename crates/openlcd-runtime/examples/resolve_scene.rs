use openlcd_core::LogicalPosition;

use openlcd_runtime::{RuntimeData, SceneResolver};

use openlcd_theme::{DataBinding, Layer, Scene, TextBinding, TextLayer};

fn main() {
    let mut scene = Scene::new("M10 Binding Test");

    scene.add_text(
        TextLayer::new(
            0,
            "CPU indisponível",
            LogicalPosition::new(100.0, 100.0),
            40.0,
            [255, 255, 255, 255],
        )
        .with_binding(
            TextBinding::new(DataBinding::CpuUsage)
                .with_prefix("CPU ")
                .with_suffix("%"),
        ),
    );

    scene.add_text(
        TextLayer::new(
            0,
            "Temperatura indisponível",
            LogicalPosition::new(100.0, 250.0),
            40.0,
            [255, 255, 255, 255],
        )
        .with_binding(
            TextBinding::new(DataBinding::CpuTemperature)
                .with_prefix("CPU ")
                .with_suffix(" °C")
                .with_precision(1),
        ),
    );

    let data = RuntimeData {
        cpu_usage: Some(73.2),

        cpu_temperature: Some(61.87),

        ..Default::default()
    };

    let resolved = SceneResolver::new().resolve(&scene, &data);

    for layer in &resolved.layers {
        if let Layer::Text(text) = layer {
            println!("{}", text.text);
        }
    }
}

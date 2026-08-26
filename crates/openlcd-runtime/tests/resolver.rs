#[cfg(test)]
mod tests {
    use openlcd_core::LogicalPosition;
    use openlcd_runtime::{RuntimeData, SceneResolver};
    use openlcd_theme::{DataBinding, Layer, Scene, TextBinding, TextLayer};

    #[test]
    fn resolves_cpu_usage() {
        let mut scene = Scene::new("test");

        scene.add_text(
            TextLayer::new(
                0,
                "CPU --%",
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

        let data = RuntimeData {
            cpu_usage: Some(42.4),

            ..Default::default()
        };

        let resolved = SceneResolver::new().resolve(&scene, &data);

        let Layer::Text(text) = &resolved.layers[0] else {
            panic!("expected TextLayer");
        };

        assert_eq!(text.text, "CPU 42%",);
    }

    #[test]
    fn uses_fallback_when_data_is_missing() {
        let mut scene = Scene::new("test");

        scene.add_text(
            TextLayer::new(
                0,
                "CPU unavailable",
                LogicalPosition::new(0.0, 0.0),
                20.0,
                [255, 255, 255, 255],
            )
            .with_binding(TextBinding::new(DataBinding::CpuUsage)),
        );

        let resolved = SceneResolver::new().resolve(&scene, &RuntimeData::default());

        let Layer::Text(text) = &resolved.layers[0] else {
            panic!("expected TextLayer");
        };

        assert_eq!(text.text, "CPU unavailable",);
    }

    #[test]
    fn respects_decimal_precision() {
        let mut scene = Scene::new("test");

        scene.add_text(
            TextLayer::new(
                0,
                "--",
                LogicalPosition::new(0.0, 0.0),
                20.0,
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
            cpu_temperature: Some(54.37),

            ..Default::default()
        };

        let resolved = SceneResolver::new().resolve(&scene, &data);

        let Layer::Text(text) = &resolved.layers[0] else {
            panic!("expected TextLayer");
        };

        assert_eq!(text.text, "CPU 54.4 °C",);
    }
}

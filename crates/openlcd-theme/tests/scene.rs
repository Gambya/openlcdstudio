#[cfg(test)]
mod tests {
    use openlcd_core::LogicalPosition;
    use openlcd_theme::{BackgroundLayer, Layer, Scene, TextLayer};

    #[test]
    fn creates_incremental_layer_ids() {
        let mut scene = Scene::new("test");

        let background = scene.add_background([0, 0, 0, 255]);

        let text = scene.add_text(TextLayer::new(
            0,
            "CPU",
            LogicalPosition::new(100.0, 200.0),
            48.0,
            [255, 255, 255, 255],
        ));

        assert_eq!(background, 1);
        assert_eq!(text, 2);
    }

    #[test]
    fn removes_layer() {
        let mut scene = Scene::new("test");

        let id = scene.add_background([0, 0, 0, 255]);

        assert!(scene.remove_layer(id));

        assert!(scene.layers.is_empty());
    }

    #[test]
    fn rebuilds_next_layer_id() {
        let mut scene = Scene::new("test");

        scene
            .layers
            .push(Layer::Background(BackgroundLayer::new(10, [0, 0, 0, 255])));

        scene.rebuild_layer_ids();

        assert_eq!(scene.next_layer_id(), 11,);
    }

    #[test]
    fn moves_text_layer_up_and_down() {
        let mut scene = Scene::new("test");

        let background = scene.add_background([0, 0, 0, 255]);

        let first = scene.add_text(TextLayer::new(
            0,
            "first",
            LogicalPosition::new(0.0, 0.0),
            20.0,
            [255, 255, 255, 255],
        ));

        let second = scene.add_text(TextLayer::new(
            0,
            "second",
            LogicalPosition::new(0.0, 0.0),
            20.0,
            [255, 255, 255, 255],
        ));

        assert_eq!(scene.layers[1].id(), first,);

        assert!(scene.move_layer_up(first));

        assert_eq!(scene.layers[2].id(), first,);

        assert!(scene.move_layer_down(first));

        assert_eq!(scene.layers[1].id(), first,);

        assert!(!scene.move_layer_down(first));

        assert!(!scene.move_layer_up(background));

        assert_eq!(scene.layers[2].id(), second,);
    }
}

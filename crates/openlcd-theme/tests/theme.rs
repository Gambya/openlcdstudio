#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use openlcd_core::{LogicalPosition, LogicalRect};

    use openlcd_theme::{AssetMode, ImageLayer, Layer, Scene, TextLayer, ThemeDocument};

    fn temporary_theme_path() -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        std::env::temp_dir().join(format!("openlcd-theme-{unique}.json"))
    }

    #[test]
    fn saves_and_loads_theme() {
        let mut scene = Scene::new("Test Scene");

        scene.add_background([10, 20, 30, 255]);

        scene.add_image(ImageLayer::new(
            0,
            "/tmp/example.png",
            LogicalRect::new(100.0, 100.0, 500.0, 500.0),
        ));

        scene.add_text(TextLayer::new(
            0,
            "OpenLCD",
            LogicalPosition::new(200.0, 300.0),
            40.0,
            [255, 255, 255, 255],
        ));

        let theme = ThemeDocument::new("Test Theme", scene);

        let path = temporary_theme_path();

        theme.save(&path).unwrap();

        let loaded = ThemeDocument::load(&path).unwrap();

        assert_eq!(loaded.name, "Test Theme",);

        assert_eq!(loaded.scene.layers.len(), 3,);

        assert_eq!(loaded.asset_mode, AssetMode::External,);

        fs::remove_file(path).ok();
    }

    #[test]
    fn restores_layer_id_sequence_after_load() {
        let mut scene = Scene::new("Test");

        let first = scene.add_background([0, 0, 0, 255]);

        let second = scene.add_text(TextLayer::new(
            0,
            "A",
            LogicalPosition::new(0.0, 0.0),
            20.0,
            [255, 255, 255, 255],
        ));

        assert_eq!(first, 1);
        assert_eq!(second, 2);

        let theme = ThemeDocument::new("Test", scene);

        let path = temporary_theme_path();

        theme.save(&path).unwrap();

        let mut loaded = ThemeDocument::load(&path).unwrap();

        let third = loaded.scene.add_text(TextLayer::new(
            0,
            "B",
            LogicalPosition::new(0.0, 0.0),
            20.0,
            [255, 255, 255, 255],
        ));

        assert_eq!(third, 3);

        fs::remove_file(path).ok();
    }

    #[test]
    fn saves_managed_assets_with_relative_paths() {
        let root = std::env::temp_dir().join(format!(
            "openlcd-managed-theme-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH,)
                .unwrap()
                .as_nanos(),
        ));

        fs::create_dir_all(&root).unwrap();

        //
        // We don't need a valid image
        // to test copying/persistence.
        //
        let external_asset = root.join("external-test.png");

        fs::write(&external_asset, b"test-image-content").unwrap();

        let theme_dir = root.join("theme");

        let theme_path = theme_dir.join("theme.json");

        let mut scene = Scene::new("Managed Test");

        scene.add_background([0, 0, 0, 255]);

        let image_id = scene.add_image(ImageLayer::new(
            0,
            external_asset.to_string_lossy().into_owned(),
            LogicalRect::new(0.0, 0.0, 1000.0, 1000.0),
        ));

        let theme = ThemeDocument::new("Managed", scene);

        theme.save_managed(&theme_path).unwrap();

        let json = fs::read_to_string(&theme_path).unwrap();

        assert!(json.contains("\"asset_mode\": \"managed\"",),);

        assert!(json.contains(&format!("assets/image-layer-{image_id}.png"),),);

        assert!(!json.contains(external_asset.to_string_lossy().as_ref(),),);

        let copied_asset = theme_dir
            .join("assets")
            .join(format!("image-layer-{image_id}.png"));

        assert!(copied_asset.exists(),);

        let loaded = ThemeDocument::load(&theme_path).unwrap();

        let image = loaded
            .scene
            .layers
            .iter()
            .find_map(|layer| match layer {
                Layer::Image(image) => Some(image),

                _ => None,
            })
            .unwrap();

        assert_eq!(PathBuf::from(&image.source,), copied_asset,);

        fs::remove_dir_all(root).ok();
    }
}

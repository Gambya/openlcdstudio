use std::{env, fs, path::PathBuf};

use ab_glyph::FontArc;

use anyhow::{Context, Result};

use image::{ImageBuffer, Rgba};

use openlcd_core::{FrameSize, LogicalPosition, LogicalRect};

use openlcd_render::SceneRenderer;

use openlcd_theme::{ImageFit, ImageLayer, Scene, TextLayer};

fn main() -> Result<()> {
    let mut args = env::args_os();

    let executable = PathBuf::from(args.next().unwrap_or_default());

    let Some(image_path) = args.next().map(PathBuf::from) else {
        anyhow::bail!("Uso: {} <imagem> <font.ttf>", executable.display(),);
    };

    let Some(font_path) = args.next().map(PathBuf::from) else {
        anyhow::bail!("Uso: {} <imagem> <font.ttf>", executable.display(),);
    };

    let font_bytes = fs::read(&font_path)
        .with_context(|| format!("não foi possível ler {}", font_path.display(),))?;

    let font = FontArc::try_from_vec(font_bytes).map_err(|_| anyhow::anyhow!("fonte inválida"))?;

    let size = FrameSize::new(462, 1920);

    let mut scene = Scene::new("Image Layer Test");

    scene.add_background([0, 0, 0, 255]);

    let mut image_layer = ImageLayer::new(
        0,
        image_path.to_string_lossy().to_string(),
        LogicalRect::new(0.0, 0.0, 1000.0, 1000.0),
    );

    image_layer.fit = ImageFit::Cover;

    scene.add_image(image_layer);

    scene.add_text(TextLayer::new(
        0,
        "OpenLCD Studio",
        LogicalPosition::new(60.0, 80.0),
        32.0,
        [255, 255, 255, 255],
    ));

    scene.add_text(TextLayer::new(
        0,
        "CPU 42%",
        LogicalPosition::new(60.0, 230.0),
        46.0,
        [80, 220, 140, 255],
    ));

    let renderer = SceneRenderer::new(size, font);

    let frame = renderer.render(&scene)?;

    let image =
        ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(size.width, size.height, frame.into_pixels())
            .context("framebuffer inválido")?;

    image.save("image-scene-preview.png")?;

    println!("image-scene-preview.png gerado com sucesso");

    Ok(())
}

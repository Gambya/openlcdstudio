use std::{env, fs, path::PathBuf};

use ab_glyph::FontArc;

use anyhow::{Context, Result};

use image::{ImageBuffer, Rgba};

use openlcd_core::{FrameSize, LogicalPosition};

use openlcd_render::SceneRenderer;

use openlcd_theme::{Scene, TextLayer};

fn main() -> Result<()> {
    let mut args = env::args_os();

    let executable = PathBuf::from(args.next().unwrap_or_default());

    let Some(font_path) = args.next().map(PathBuf::from) else {
        anyhow::bail!("Uso: {} <font.ttf>", executable.display(),);
    };

    let font_bytes = fs::read(&font_path)
        .with_context(|| format!("não foi possível ler {}", font_path.display(),))?;

    let font = FontArc::try_from_vec(font_bytes).map_err(|_| anyhow::anyhow!("fonte inválida"))?;

    let size = FrameSize::new(462, 1920);

    let mut scene = Scene::new("First OpenLCD Scene");

    scene.add_background([18, 24, 42, 255]);

    scene.add_text(TextLayer::new(
        0,
        "OpenLCD Studio",
        LogicalPosition::new(80.0, 100.0),
        55.0,
        [255, 255, 255, 255],
    ));

    scene.add_text(TextLayer::new(
        0,
        "CPU 42%",
        LogicalPosition::new(80.0, 250.0),
        80.0,
        [80, 220, 140, 255],
    ));

    let renderer = SceneRenderer::new(size, font);

    let frame = renderer.render(&scene)?;

    let image =
        ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(size.width, size.height, frame.into_pixels())
            .context("framebuffer inválido")?;

    image.save("scene-preview.png")?;

    println!("scene-preview.png gerado com sucesso");

    Ok(())
}

use std::{
    env, fs,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

use ab_glyph::FontArc;
use anyhow::{Context, Result};
use kmex_vmax::KmexDisplay;
use openlcd_render::{Renderer, encode_jpeg};

fn main() -> Result<()> {
    let mut args = env::args_os();
    let executable = PathBuf::from(args.next().unwrap_or_default());

    let Some(port) = args.next().map(PathBuf::from) else {
        print_usage(&executable);
        anyhow::bail!("porta não informada");
    };

    let Some(background) = args.next().map(PathBuf::from) else {
        print_usage(&executable);
        anyhow::bail!("imagem de fundo não informada");
    };

    let Some(font_path) = args.next().map(PathBuf::from) else {
        print_usage(&executable);
        anyhow::bail!("arquivo de fonte não informado");
    };

    let port = port.to_str().context("caminho da porta inválido")?;

    let font_bytes = fs::read(&font_path)
        .with_context(|| format!("não foi possível ler {}", font_path.display(),))?;

    let font = FontArc::try_from_vec(font_bytes).map_err(|_| anyhow::anyhow!("fonte inválida"))?;

    let mut display = KmexDisplay::open(port)?;

    let started = Instant::now();
    let duration = Duration::from_secs(30);

    let frame_interval = Duration::from_millis(50);

    let mut next_frame = Instant::now();

    while started.elapsed() < duration {
        let elapsed = started.elapsed().as_secs_f32();

        let simulated_cpu = ((elapsed.sin() + 1.0) * 50.0).round();

        let mut renderer = Renderer::kmex();

        renderer.set_background_file(&background)?;

        renderer.draw_text(
            &format!("CPU {:.0}%", simulated_cpu,),
            30,
            100,
            64.0,
            [255, 255, 255, 255],
            &font,
        );

        renderer.draw_text("OpenLCD Studio", 30, 180, 42.0, [255, 255, 255, 255], &font);

        let frame = renderer.finish()?;

        let jpeg = encode_jpeg(&frame, 85)?;

        display.send_jpeg(&jpeg)?;

        next_frame += frame_interval;

        let now = Instant::now();

        if next_frame > now {
            thread::sleep(next_frame - now);
        } else {
            next_frame = now;
        }
    }

    Ok(())
}

fn print_usage(executable: &Path) {
    eprintln!(
        concat!(
            "Uso:\n",
            "  {} <porta> ",
            "<background.jpg> ",
            "<font.ttf>\n"
        ),
        executable.display(),
    );
}

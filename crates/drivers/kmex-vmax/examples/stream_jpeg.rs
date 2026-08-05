use std::{
    env, fs,
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use kmex_vmax::KmexDisplay;

fn main() -> Result<()> {
    let mut args = env::args_os();
    let executable = PathBuf::from(args.next().unwrap_or_default());

    let port = args.next().map(PathBuf::from);
    let jpeg_path = args.next().map(PathBuf::from);
    let fps = args.next();
    let duration_seconds = args.next();
    let control_value = args.next();

    let Some(port) = port else {
        print_usage(&executable);
        bail!("porta serial não informada");
    };

    let Some(jpeg_path) = jpeg_path else {
        print_usage(&executable);
        bail!("arquivo JPEG não informado");
    };

    let fps = fps
        .as_deref()
        .unwrap_or_default()
        .to_string_lossy()
        .parse::<f64>()
        .unwrap_or(2.0);

    if !fps.is_finite() || fps <= 0.0 || fps > 60.0 {
        bail!("FPS deve estar entre 0 e 60");
    }

    let duration_seconds = duration_seconds
        .as_deref()
        .unwrap_or_default()
        .to_string_lossy()
        .parse::<u64>()
        .unwrap_or(30);

    let control_value = control_value
        .map(|value| {
            value
                .to_string_lossy()
                .parse::<u8>()
                .context("controle deve estar entre 0 e 255")
        })
        .transpose()?;

    let port = port
        .to_str()
        .context("caminho da porta serial não é UTF-8 válido")?;

    let jpeg = fs::read(&jpeg_path)
        .with_context(|| format!("não foi possível ler {}", jpeg_path.display()))?;

    let frame_interval = Duration::from_secs_f64(1.0 / fps);
    let total_duration = Duration::from_secs(duration_seconds);

    println!("Abrindo dispositivo: {port}");
    println!("JPEG: {}", jpeg_path.display());
    println!("Tamanho: {} bytes", jpeg.len());
    println!("FPS solicitado: {fps:.2}");
    println!("Duração: {duration_seconds} segundos");

    let mut display =
        KmexDisplay::open(port).with_context(|| format!("não foi possível abrir {port}"))?;

    if let Some(value) = control_value {
        println!("Enviando controle: AA BB {value:02X} CC DD");
        display.send_control(value)?;
        thread::sleep(Duration::from_millis(250));
    }

    let started_at = Instant::now();
    let mut next_frame_at = started_at;
    let mut frames_sent = 0_u64;

    while started_at.elapsed() < total_duration {
        display.send_jpeg(&jpeg)?;
        frames_sent += 1;

        if frames_sent == 1 || frames_sent % 10 == 0 {
            println!(
                "Quadros enviados: {frames_sent}; tempo: {:.2}s",
                started_at.elapsed().as_secs_f64(),
            );
        }

        next_frame_at += frame_interval;

        let now = Instant::now();

        if next_frame_at > now {
            thread::sleep(next_frame_at - now);
        } else {
            next_frame_at = now;
        }
    }

    let elapsed = started_at.elapsed().as_secs_f64();
    let actual_fps = if elapsed > 0.0 {
        frames_sent as f64 / elapsed
    } else {
        0.0
    };

    println!("Transmissão encerrada.");
    println!("Quadros enviados: {frames_sent}");
    println!("FPS real: {actual_fps:.2}");

    Ok(())
}

fn print_usage(executable: &PathBuf) {
    eprintln!(
        concat!(
            "Uso:\n",
            "  {} <porta> <imagem.jpg> [fps] [segundos] [controle]\n\n",
            "Exemplo:\n",
            "  {} /dev/serial/by-id/usb-HL_VMAX_HL-VMAX-USB-Device-if00 ",
            "frame.jpg 2 30 100"
        ),
        executable.display(),
        executable.display(),
    );
}

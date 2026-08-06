use std::{
    env, fs,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use kmex_vmax::KmexDisplay;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Manifest {
    entries: Vec<ManifestEntry>,
}

#[derive(Debug, Deserialize)]
struct ManifestEntry {
    file: String,
    frame_number: u64,
    complete: bool,
    width: Option<u16>,
    height: Option<u16>,
}

struct Frame {
    file_name: String,
    frame_number: u64,
    bytes: Vec<u8>,
}

fn main() -> Result<()> {
    let mut args = env::args_os();
    let executable = PathBuf::from(args.next().unwrap_or_default());

    let port = args.next().map(PathBuf::from);
    let manifest_path = args.next().map(PathBuf::from);
    let fps = parse_arg::<f64>(args.next(), 10.0, "FPS")?;
    let duration_seconds = parse_arg::<u64>(args.next(), 30, "duração")?;

    let Some(port) = port else {
        print_usage(&executable);
        bail!("porta serial não informada");
    };

    let Some(manifest_path) = manifest_path else {
        print_usage(&executable);
        bail!("manifest.json não informado");
    };

    if !fps.is_finite() || fps <= 0.0 || fps > 60.0 {
        bail!("FPS deve ser maior que zero e no máximo 60");
    }

    let port = port
        .to_str()
        .context("o caminho da porta não é UTF-8 válido")?;

    let media_dir = manifest_path
        .parent()
        .context("manifest.json não possui diretório pai")?;

    let manifest_text = fs::read_to_string(&manifest_path)
        .with_context(|| format!("não foi possível ler {}", manifest_path.display()))?;

    let manifest: Manifest =
        serde_json::from_str(&manifest_text).context("manifest.json inválido")?;

    let mut entries: Vec<_> = manifest
        .entries
        .into_iter()
        .filter(|entry| entry.complete && entry.width == Some(462) && entry.height == Some(1920))
        .collect();

    entries.sort_by_key(|entry| entry.frame_number);

    if entries.is_empty() {
        bail!("nenhum JPEG completo de 462x1920 foi encontrado");
    }

    println!("Carregando {} quadros...", entries.len());

    let mut frames = Vec::with_capacity(entries.len());

    for entry in entries {
        let path = media_dir.join(&entry.file);

        let bytes =
            fs::read(&path).with_context(|| format!("não foi possível ler {}", path.display()))?;

        frames.push(Frame {
            file_name: entry.file,
            frame_number: entry.frame_number,
            bytes,
        });
    }

    let total_bytes: usize = frames.iter().map(|frame| frame.bytes.len()).sum();

    println!("Porta: {port}");
    println!("Quadros carregados: {}", frames.len());
    println!(
        "Dados carregados: {:.2} MiB",
        total_bytes as f64 / 1024.0 / 1024.0
    );
    println!("FPS solicitado: {fps:.2}");
    println!("Duração: {duration_seconds}s");

    let mut display =
        KmexDisplay::open(port).with_context(|| format!("não foi possível abrir {port}"))?;

    play_sequence(
        &mut display,
        &frames,
        fps,
        Duration::from_secs(duration_seconds),
    )
}

fn play_sequence(
    display: &mut KmexDisplay<kmex_vmax::SerialTransport>,
    frames: &[Frame],
    fps: f64,
    duration: Duration,
) -> Result<()> {
    let frame_interval = Duration::from_secs_f64(1.0 / fps);

    let started_at = Instant::now();
    let mut next_frame_at = started_at;
    let mut frame_index = 0_usize;
    let mut frames_sent = 0_u64;
    let mut bytes_sent = 0_u64;
    let mut total_write_time = Duration::ZERO;

    while started_at.elapsed() < duration {
        let frame = &frames[frame_index];

        let write_started_at = Instant::now();

        display
            .send_jpeg(&frame.bytes)
            .with_context(|| format!("falha ao enviar {}", frame.file_name))?;

        let write_time = write_started_at.elapsed();

        total_write_time += write_time;
        frames_sent += 1;
        bytes_sent += frame.bytes.len() as u64;

        if frames_sent == 1 || frames_sent.is_multiple_of(25) {
            println!(
                concat!(
                    "enviados={} ",
                    "origem-frame={} ",
                    "índice={} ",
                    "tamanho={} ",
                    "write={:.2}ms"
                ),
                frames_sent,
                frame.frame_number,
                frame_index,
                frame.bytes.len(),
                write_time.as_secs_f64() * 1000.0,
            );
        }

        frame_index = (frame_index + 1) % frames.len();

        next_frame_at += frame_interval;

        let now = Instant::now();

        if next_frame_at > now {
            thread::sleep(next_frame_at - now);
        } else {
            next_frame_at = now;
        }
    }

    let elapsed = started_at.elapsed();
    let elapsed_seconds = elapsed.as_secs_f64();

    let actual_fps = if elapsed_seconds > 0.0 {
        frames_sent as f64 / elapsed_seconds
    } else {
        0.0
    };

    let throughput_mib = if elapsed_seconds > 0.0 {
        bytes_sent as f64 / 1024.0 / 1024.0 / elapsed_seconds
    } else {
        0.0
    };

    let average_write_ms = if frames_sent > 0 {
        total_write_time.as_secs_f64() * 1000.0 / frames_sent as f64
    } else {
        0.0
    };

    println!();
    println!("Reprodução encerrada.");
    println!("Quadros enviados: {frames_sent}");
    println!("FPS real: {actual_fps:.2}");
    println!("Taxa de dados: {throughput_mib:.2} MiB/s");
    println!("Tempo médio de escrita: {average_write_ms:.2}ms");

    Ok(())
}

fn parse_arg<T>(value: Option<std::ffi::OsString>, default: T, description: &str) -> Result<T>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    match value {
        Some(value) => value
            .to_string_lossy()
            .parse::<T>()
            .with_context(|| format!("{description} inválido")),
        None => Ok(default),
    }
}

fn print_usage(executable: &Path) {
    eprintln!(
        concat!(
            "Uso:\n",
            "  {} <porta> <manifest.json> [fps] [segundos]\n\n",
            "Exemplo:\n",
            "  {} /dev/serial/by-id/usb-HL_VMAX_HL-VMAX-USB-Device-if00 ",
            "protocols/kmex/analysis/media/manifest.json 10 30"
        ),
        executable.display(),
        executable.display(),
    );
}

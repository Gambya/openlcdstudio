use std::{env, path::PathBuf, thread, time::Duration};

use anyhow::{Context, Result};

use kmex_vmax::KmexDisplay;

fn main() -> Result<()> {
    let mut args = env::args_os();
    let executable = args.next().unwrap_or_default();

    let port = args.next().map(PathBuf::from);
    let jpeg = args.next().map(PathBuf::from);
    let control = args.next();

    let Some(port) = port else {
        anyhow::bail!(
            "uso: {} <porta> <imagem.jpg> [valor-controle]",
            PathBuf::from(executable).display(),
        );
    };

    let Some(jpeg) = jpeg else {
        anyhow::bail!(
            "uso: {} <porta> <imagem.jpg> [valor-controle]",
            PathBuf::from(executable).display(),
        );
    };

    let port = port
        .to_str()
        .context("o caminho da porta contém caracteres inválidos")?;

    println!("Abrindo dispositivo: {port}");

    let mut display =
        KmexDisplay::open(port).with_context(|| format!("não foi possível abrir {port}"))?;

    if let Some(control) = control {
        let control = control
            .to_str()
            .context("valor de controle inválido")?
            .parse::<u8>()
            .context("o valor de controle deve estar entre 0 e 255")?;

        println!("Enviando comando: AA BB {control:02X} CC DD");

        display.send_control(control)?;

        thread::sleep(Duration::from_millis(250));
    }

    println!("Enviando JPEG: {}", jpeg.display(),);

    display
        .send_jpeg_file(&jpeg)
        .with_context(|| format!("não foi possível enviar {}", jpeg.display(),))?;

    println!("Transferência concluída.");

    Ok(())
}

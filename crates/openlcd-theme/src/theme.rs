use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{Layer, Scene};

pub const CURRENT_THEME_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetMode {
    /// Os ImageLayer ainda apontam para arquivos externos.
    ///
    /// Esta é a modalidade temporária usada durante o
    /// desenvolvimento do editor.
    External,

    /// Os assets pertencem ao diretório do tema.
    ///
    /// Será implementado na próxima etapa.
    Managed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeDocument {
    pub format_version: u32,
    pub name: String,
    pub asset_mode: AssetMode,
    pub scene: Scene,
}

impl ThemeDocument {
    pub fn new(name: impl Into<String>, scene: Scene) -> Self {
        Self {
            format_version: CURRENT_THEME_FORMAT_VERSION,

            name: name.into(),

            asset_mode: AssetMode::External,

            scene,
        }
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ThemeError> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        let json = serde_json::to_string_pretty(self)?;

        fs::write(path, json)?;

        Ok(())
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, ThemeError> {
        let path = path.as_ref();

        let json = fs::read_to_string(path)?;

        let mut theme: ThemeDocument = serde_json::from_str(&json)?;

        if theme.format_version != CURRENT_THEME_FORMAT_VERSION {
            return Err(ThemeError::UnsupportedVersion {
                expected: CURRENT_THEME_FORMAT_VERSION,

                actual: theme.format_version,
            });
        }

        // next_layer_id is runtime-only and has
        // #[serde(skip)], so we need to reconstruct it
        // after deserialization.
        theme.scene.rebuild_layer_ids();

        if theme.asset_mode == AssetMode::Managed {
            let theme_directory = path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new("."));

            for layer in &mut theme.scene.layers {
                let Layer::Image(image) = layer else {
                    continue;
                };

                let source = PathBuf::from(&image.source);

                if source.is_relative() {
                    image.source = theme_directory.join(source).to_string_lossy().into_owned();
                }
            }
        }

        Ok(theme)
    }

    pub fn save_managed(&self, path: impl AsRef<Path>) -> Result<(), ThemeError> {
        let path = path.as_ref();

        let theme_directory = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));

        fs::create_dir_all(theme_directory)?;

        let assets_directory = theme_directory.join("assets");

        fs::create_dir_all(&assets_directory)?;

        //
        // We are working on a clone.
        //
        // The scene currently open in the editor continues
        // to use its current absolute paths.
        //
        let mut persistent_theme = self.clone();

        persistent_theme.asset_mode = AssetMode::Managed;

        for layer in &mut persistent_theme.scene.layers {
            let Layer::Image(image) = layer else {
                continue;
            };

            let source_path = PathBuf::from(&image.source);

            if !source_path.exists() {
                return Err(ThemeError::AssetNotFound { path: source_path });
            }

            let extension = source_path
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or("img");

            let file_name = format!("image-layer-{}.{}", image.id, extension,);

            let destination = assets_directory.join(&file_name);

            //
            // Do not copy a file onto itself.
            //
            let source_canonical = fs::canonicalize(&source_path)?;

            let destination_canonical = fs::canonicalize(&destination).ok();

            let same_file = destination_canonical
                .as_ref()
                .is_some_and(|destination| destination == &source_canonical);

            if !same_file {
                fs::copy(&source_path, &destination)?;
            }

            //
            // IMPORTANT:
            // we store the relative path in the JSON,
            // never /home/user/...
            //
            image.source = Path::new("assets")
                .join(file_name)
                .to_string_lossy()
                .into_owned();
        }

        let json = serde_json::to_string_pretty(&persistent_theme)?;

        fs::write(path, json)?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    #[error("I/O error in theme: {0}")]
    Io(#[from] std::io::Error),

    #[error("error serializing or deserializing theme: {0}")]
    Json(#[from] serde_json::Error),

    #[error("unsupported theme version: expected {expected}, received {actual}")]
    UnsupportedVersion { expected: u32, actual: u32 },

    #[error("theme asset not found: {}", path.display())]
    AssetNotFound { path: PathBuf },
}

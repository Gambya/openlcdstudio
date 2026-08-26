use crate::TextBinding;

use openlcd_core::{LogicalPosition, LogicalRect};

use serde::{Deserialize, Serialize};

pub type LayerId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImageFit {
    Stretch,
    Contain,
    Cover,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Layer {
    Background(BackgroundLayer),
    Image(ImageLayer),
    Text(TextLayer),
}

impl Layer {
    pub const fn id(&self) -> LayerId {
        match self {
            Self::Background(layer) => layer.id,
            Self::Image(layer) => layer.id,
            Self::Text(layer) => layer.id,
        }
    }

    pub const fn visible(&self) -> bool {
        match self {
            Self::Background(layer) => layer.visible,
            Self::Image(layer) => layer.visible,
            Self::Text(layer) => layer.visible,
        }
    }

    pub fn set_visible(&mut self, visible: bool) {
        match self {
            Self::Background(layer) => {
                layer.visible = visible;
            }

            Self::Image(layer) => {
                layer.visible = visible;
            }

            Self::Text(layer) => {
                layer.visible = visible;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackgroundLayer {
    pub id: LayerId,
    pub visible: bool,
    pub color: [u8; 4],
}

impl BackgroundLayer {
    pub const fn new(id: LayerId, color: [u8; 4]) -> Self {
        Self {
            id,
            visible: true,
            color,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageLayer {
    pub id: LayerId,
    pub visible: bool,

    pub source: String,
    pub rect: LogicalRect,

    pub fit: ImageFit,

    pub opacity: u8,
}

impl ImageLayer {
    pub fn new(id: LayerId, source: impl Into<String>, rect: LogicalRect) -> Self {
        Self {
            id,
            visible: true,
            source: source.into(),
            rect,
            fit: ImageFit::Cover,
            opacity: 255,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextLayer {
    pub id: LayerId,
    pub visible: bool,

    /// Persistent layer text.
    ///
    /// When `binding` is None, this is the
    /// effective text rendered.
    ///
    /// When a binding exists, it serves as a
    /// fallback if the data is unavailable.
    pub text: String,

    /// Optional dynamic font.
    ///
    /// `serde(default)` maintains compatibility with
    /// M9 themes that do not yet have this field.
    #[serde(default)]
    pub binding: Option<TextBinding>,

    pub position: LogicalPosition,

    pub font_size: f32,

    pub color: [u8; 4],
}

impl TextLayer {
    pub fn new(
        id: LayerId,
        text: impl Into<String>,
        position: LogicalPosition,
        font_size: f32,
        color: [u8; 4],
    ) -> Self {
        Self {
            id,
            visible: true,
            text: text.into(),
            binding: None,
            position,
            font_size,
            color,
        }
    }

    pub fn with_binding(mut self, binding: TextBinding) -> Self {
        self.binding = Some(binding);
        self
    }

    pub fn clear_binding(&mut self) {
        self.binding = None;
    }

    pub fn logical_bounds(&self) -> LogicalRect {
        LogicalRect::new(self.position.x, self.position.y, 0.0, 0.0)
    }
}

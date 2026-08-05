use crate::FrameSize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DisplayShape {
    Square,
    Portrait,
    PortraitUltrawide,
    Landscape,
    LandscapeUltrawide,
}

impl DisplayShape {
    pub const fn identifier(self) -> &'static str {
        match self {
            Self::Square => "square",
            Self::Portrait => "portrait",
            Self::PortraitUltrawide => "portrait-ultrawide",
            Self::Landscape => "landscape",
            Self::LandscapeUltrawide => "landscape-ultrawide",
        }
    }
}

pub fn classify_display(size: FrameSize) -> DisplayShape {
    if size.width == 0 || size.height == 0 {
        return DisplayShape::Square;
    }

    let ratio = size.width as f32 / size.height as f32;

    if (0.90..=1.10).contains(&ratio) {
        DisplayShape::Square
    } else if ratio < 0.50 {
        DisplayShape::PortraitUltrawide
    } else if ratio < 1.0 {
        DisplayShape::Portrait
    } else if ratio > 2.0 {
        DisplayShape::LandscapeUltrawide
    } else {
        DisplayShape::Landscape
    }
}

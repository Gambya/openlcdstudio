#[cfg(test)]
mod tests {
    use openlcd_core::{DisplayShape, FrameSize, classify_display};

    #[test]
    fn classifies_kmex_as_portrait_ultrawide() {
        let shape = classify_display(FrameSize::new(462, 1920));

        assert_eq!(shape, DisplayShape::PortraitUltrawide,);
    }

    #[test]
    fn classifies_square_cooler_display() {
        let shape = classify_display(FrameSize::new(480, 480));

        assert_eq!(shape, DisplayShape::Square,);
    }

    #[test]
    fn classifies_landscape_display() {
        let shape = classify_display(FrameSize::new(800, 480));

        assert_eq!(shape, DisplayShape::Landscape,);
    }

    #[test]
    fn classifies_ultrawide_landscape_display() {
        let shape = classify_display(FrameSize::new(1920, 462));

        assert_eq!(shape, DisplayShape::LandscapeUltrawide,);
    }
}

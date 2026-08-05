#[cfg(test)]
mod tests {
    use openlcd_core::{FrameSize, RgbaFrame};
    use openlcd_driver::DisplayDevice;
    use openlcd_fake_driver::FakeDisplay;

    #[test]
    fn accepts_frame_with_correct_size() {
        let mut display = FakeDisplay::square_profile(2, 2);

        let frame = RgbaFrame::new(FrameSize::new(2, 2), vec![0; 2 * 2 * 4]).unwrap();

        display.send_rgba(&frame).unwrap();

        assert_eq!(display.frame_count(), 1,);

        assert!(display.last_frame().is_some(),);
    }

    #[test]
    fn rejects_frame_with_wrong_size() {
        let mut display = FakeDisplay::square_profile(480, 480);

        let frame = RgbaFrame::new(FrameSize::new(2, 2), vec![0; 2 * 2 * 4]).unwrap();

        assert!(display.send_rgba(&frame).is_err(),);
    }

    #[test]
    fn changes_fake_brightness() {
        let mut display = FakeDisplay::square_profile(480, 480);

        display.set_brightness(60).unwrap();

        assert_eq!(display.brightness(), 60,);
    }
}

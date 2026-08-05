use openlcd_core::{DeviceCapabilities, FrameSize, Orientation, PixelFormat};

pub fn kmex_vmax_capabilities() -> DeviceCapabilities {
    DeviceCapabilities {
        model: "K-MEX HL VMAX".to_owned(),
        native_size: FrameSize::new(462, 1920),
        orientation: Orientation::Portrait,
        accepted_formats: vec![PixelFormat::Jpeg],
        preferred_format: PixelFormat::Jpeg,
        min_fps: 0.5,
        max_fps: 20.0,
        recommended_fps: 20.0,
        requires_continuous_frames: true,
        supports_brightness: false,
    }
}

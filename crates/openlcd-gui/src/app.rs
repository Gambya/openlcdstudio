use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions, Vec2};

use openlcd_core::{
    DeviceCapabilities, DisplayShape, FrameSize, Orientation, PixelFormat, RgbaFrame,
    classify_display,
};

use openlcd_driver::DisplayDevice;
use openlcd_fake_driver::FakeDisplay;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PreviewProfile {
    Kmex,
    Square480,
    Landscape800x480,
}

impl PreviewProfile {
    const ALL: [Self; 3] = [Self::Kmex, Self::Square480, Self::Landscape800x480];

    const fn label(self) -> &'static str {
        match self {
            Self::Kmex => "K-MEX 462 × 1920",
            Self::Square480 => "Quadrado 480 × 480",
            Self::Landscape800x480 => "Landscape 800 × 480",
        }
    }

    const fn size(self) -> FrameSize {
        match self {
            Self::Kmex => FrameSize::new(462, 1920),
            Self::Square480 => FrameSize::new(480, 480),
            Self::Landscape800x480 => FrameSize::new(800, 480),
        }
    }

    fn capabilities(self) -> DeviceCapabilities {
        let size = self.size();

        let orientation = if size.height >= size.width {
            Orientation::Portrait
        } else {
            Orientation::Landscape
        };

        DeviceCapabilities {
            model: format!("Fake {}", self.label(),),
            native_size: size,
            orientation,
            accepted_formats: vec![PixelFormat::Rgba8888],
            preferred_format: PixelFormat::Rgba8888,
            min_fps: 0.1,
            max_fps: 60.0,
            recommended_fps: 20.0,
            requires_continuous_frames: false,
            supports_brightness: true,
        }
    }
}

pub struct OpenLcdApp {
    selected_profile: PreviewProfile,
    fake_display: FakeDisplay,
    texture: Option<TextureHandle>,

    brightness: u8,
    frame_count: u64,

    cpu_usage: f32,
    animate: bool,
    last_frame_time: f64,

    last_render_ms: f64,
    actual_fps: f64,
    last_render_instant: Option<std::time::Instant>,
}

impl OpenLcdApp {
    pub fn new(context: &eframe::CreationContext<'_>) -> Self {
        let selected_profile = PreviewProfile::Kmex;

        let mut app = Self {
            selected_profile,
            fake_display: FakeDisplay::new(selected_profile.capabilities()),
            texture: None,

            brightness: 100,
            frame_count: 0,

            cpu_usage: 42.0,
            animate: true,
            last_frame_time: 0.0,

            last_render_ms: 0.0,
            actual_fps: 0.0,
            last_render_instant: None,
        };

        app.render_preview(&context.egui_ctx);

        app
    }

    fn change_profile(&mut self, profile: PreviewProfile, context: &egui::Context) {
        if profile == self.selected_profile {
            return;
        }

        self.selected_profile = profile;

        self.fake_display = FakeDisplay::new(profile.capabilities());

        self.texture = None;
        self.frame_count = 0;

        self.last_render_ms = 0.0;
        self.actual_fps = 0.0;
        self.last_render_instant = None;

        self.render_preview(context);
    }

    fn update_animation(&mut self, context: &egui::Context) {
        if !self.animate {
            return;
        }

        let current_time = context.input(|input| input.time);

        let elapsed = current_time - self.last_frame_time;

        if elapsed < 1.0 / 20.0 {
            context.request_repaint();
            return;
        }

        self.last_frame_time = current_time;

        self.cpu_usage = ((current_time as f32 * 1.5).sin() * 0.5 + 0.5) * 100.0;

        self.render_preview(context);

        context.request_repaint();
    }

    fn apply_fake_brightness(frame: &RgbaFrame, brightness: u8) -> RgbaFrame {
        let factor = (brightness.min(100) as f32) / 100.0;

        let mut pixels = frame.pixels().to_vec();

        for pixel in pixels.chunks_exact_mut(4) {
            pixel[0] = (pixel[0] as f32 * factor).round().clamp(0.0, 255.0) as u8;

            pixel[1] = (pixel[1] as f32 * factor).round().clamp(0.0, 255.0) as u8;

            pixel[2] = (pixel[2] as f32 * factor).round().clamp(0.0, 255.0) as u8;
        }

        RgbaFrame::new(frame.size(), pixels)
            .expect("brightness processing preserves framebuffer size")
    }

    fn render_preview(&mut self, context: &egui::Context) {
        let render_started = std::time::Instant::now();

        let size = self.selected_profile.size();

        let frame = generate_demo_frame(size, self.cpu_usage, self.frame_count);

        if self.fake_display.send_rgba(&frame).is_err() {
            return;
        }

        self.frame_count += 1;

        let Some(frame) = self.fake_display.last_frame() else {
            return;
        };

        let preview_frame = Self::apply_fake_brightness(frame, self.fake_display.brightness());

        let color_image = ColorImage::from_rgba_unmultiplied(
            [
                preview_frame.size().width as usize,
                preview_frame.size().height as usize,
            ],
            preview_frame.pixels(),
        );

        match &mut self.texture {
            Some(texture) => {
                texture.set(color_image, TextureOptions::LINEAR);
            }

            None => {
                self.texture = Some(context.load_texture(
                    "openlcd-preview",
                    color_image,
                    TextureOptions::LINEAR,
                ));
            }
        }

        let now = std::time::Instant::now();

        if let Some(previous) = self.last_render_instant {
            let delta = now.duration_since(previous).as_secs_f64();

            if delta > 0.0 {
                self.actual_fps = 1.0 / delta;
            }
        }

        self.last_render_instant = Some(now);

        self.last_render_ms = render_started.elapsed().as_secs_f64() * 1000.0;
    }

    fn profile_panel(&mut self, ui: &mut egui::Ui, context: &egui::Context) {
        ui.heading("Dispositivo");

        egui::ComboBox::from_label("Perfil")
            .selected_text(self.selected_profile.label())
            .show_ui(ui, |ui| {
                for profile in PreviewProfile::ALL {
                    let selected = profile == self.selected_profile;

                    if ui.selectable_label(selected, profile.label()).clicked() {
                        self.change_profile(profile, context);
                    }
                }
            });

        ui.separator();

        let capabilities = self.fake_display.capabilities();

        let size = capabilities.native_size;

        let shape = classify_display(size);

        ui.label(format!("Modelo: {}", capabilities.model,));

        ui.label(format!("Resolução: {} × {}", size.width, size.height,));

        ui.label(format!("Formato: {:?}", capabilities.preferred_format,));

        ui.label(format!("Geometria: {}", shape_label(shape),));

        ui.label(format!(
            "Frames recebidos: {}",
            self.fake_display.frame_count(),
        ));

        ui.separator();

        ui.checkbox(&mut self.animate, "Animar preview");

        ui.add(egui::Slider::new(&mut self.cpu_usage, 0.0..=100.0).text("CPU simulada"));

        if ui
            .add(egui::Slider::new(&mut self.brightness, 0..=100).text("Brilho fake"))
            .changed()
        {
            let _ = self.fake_display.set_brightness(self.brightness);

            self.render_preview(context);
        }

        if ui.button("Renderizar quadro").clicked() {
            self.render_preview(context);
        }

        ui.separator();

        ui.heading("Diagnóstico");

        ui.label(format!("FPS real: {:.1}", self.actual_fps,));

        ui.label(format!("Último render: {:.2} ms", self.last_render_ms,));

        ui.label(format!("Frames renderizados: {}", self.frame_count,));

        ui.label(format!(
            "Frames recebidos pelo fake: {}",
            self.fake_display.frame_count(),
        ));

        ui.label(format!(
            "Brilho solicitado: {}%",
            self.fake_display.brightness(),
        ));

        ui.label(format!(
            "Preview: {} × {}",
            self.selected_profile.size().width,
            self.selected_profile.size().height,
        ));
    }

    fn preview_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Preview");

        let Some(texture) = &self.texture else {
            ui.label("Nenhum quadro disponível.");
            return;
        };

        let available = ui.available_size();

        let native = self.selected_profile.size();

        let preview_size = fit_size(
            Vec2::new(native.width as f32, native.height as f32),
            available,
        );

        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.add(egui::Image::new(texture).fit_to_exact_size(preview_size));
                });
            });
    }
}

impl eframe::App for OpenLcdApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let context = ui.ctx().clone();

        self.update_animation(&context);

        egui::Panel::left("device-panel")
            .default_size(280.0)
            .resizable(true)
            .show(ui, |ui| {
                self.profile_panel(ui, &context);
            });

        egui::CentralPanel::default().show(ui, |ui| {
            self.preview_panel(ui);
        });
    }
}

fn generate_demo_frame(size: FrameSize, cpu_usage: f32, frame_index: u64) -> RgbaFrame {
    let width = size.width as usize;
    let height = size.height as usize;

    let mut pixels = vec![0_u8; width * height * 4];

    let cpu_ratio = (cpu_usage / 100.0).clamp(0.0, 1.0);

    let bar_width = (width as f32 * cpu_ratio).round() as usize;

    let bar_top = height.saturating_mul(80) / 100;

    let bar_bottom = height.saturating_mul(85) / 100;

    for y in 0..height {
        for x in 0..width {
            let offset = (y * width + x) * 4;

            let horizontal = x as f32 / width.max(1) as f32;

            let vertical = y as f32 / height.max(1) as f32;

            let movement = ((frame_index as f32 * 0.03).sin() * 20.0) as i16;

            let red = (20.0 + horizontal * 50.0) as i16 + movement;

            let green = (20.0 + vertical * 70.0) as i16;

            let blue = (70.0 + horizontal * 80.0) as i16 - movement;

            pixels[offset] = red.clamp(0, 255) as u8;

            pixels[offset + 1] = green.clamp(0, 255) as u8;

            pixels[offset + 2] = blue.clamp(0, 255) as u8;

            pixels[offset + 3] = 255;

            if y >= bar_top && y < bar_bottom {
                if x < bar_width {
                    pixels[offset] = 240;
                    pixels[offset + 1] = 240;
                    pixels[offset + 2] = 240;
                } else {
                    pixels[offset] = 35;
                    pixels[offset + 1] = 35;
                    pixels[offset + 2] = 35;
                }
            }
        }
    }

    RgbaFrame::new(size, pixels).expect("demo frame always has a valid buffer")
}

fn fit_size(native: Vec2, available: Vec2) -> Vec2 {
    if native.x <= 0.0 || native.y <= 0.0 || available.x <= 0.0 || available.y <= 0.0 {
        return Vec2::ZERO;
    }

    let width_scale = available.x / native.x;

    let height_scale = available.y / native.y;

    let scale = width_scale.min(height_scale).min(1.0);

    native * scale
}

const fn shape_label(shape: DisplayShape) -> &'static str {
    match shape {
        DisplayShape::Square => "Quadrado",
        DisplayShape::Portrait => "Retrato",
        DisplayShape::PortraitUltrawide => "Retrato ultrawide",
        DisplayShape::Landscape => "Paisagem",
        DisplayShape::LandscapeUltrawide => "Paisagem ultrawide",
    }
}

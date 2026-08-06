use std::{fs, path::PathBuf, time::Instant};

use ab_glyph::FontArc;

use eframe::egui::{
    self, ColorImage, Pos2, Rect, Sense, Stroke, StrokeKind, TextureHandle, TextureOptions, Vec2,
};

use openlcd_core::{
    DeviceCapabilities, DisplayShape, FrameSize, LOGICAL_CANVAS_SIZE, LogicalPosition, Orientation,
    PixelFormat, RgbaFrame, classify_display,
};

use openlcd_driver::DisplayDevice;
use openlcd_fake_driver::FakeDisplay;

use openlcd_render::SceneRenderer;

use openlcd_theme::{Layer, LayerId, Scene, TextLayer};

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

#[derive(Debug, Clone, Copy)]
struct PreviewTransform {
    rect: Rect,
    native_size: FrameSize,
}

impl PreviewTransform {
    fn new(rect: Rect, native_size: FrameSize) -> Self {
        Self { rect, native_size }
    }

    fn screen_to_logical(&self, position: Pos2) -> Option<LogicalPosition> {
        if !self.rect.contains(position) {
            return None;
        }

        if self.rect.width() <= 0.0 || self.rect.height() <= 0.0 {
            return None;
        }

        let normalized_x = (position.x - self.rect.left()) / self.rect.width();

        let normalized_y = (position.y - self.rect.top()) / self.rect.height();

        Some(LogicalPosition::new(
            normalized_x * LOGICAL_CANVAS_SIZE,
            normalized_y * LOGICAL_CANVAS_SIZE,
        ))
    }

    fn logical_to_screen(&self, position: LogicalPosition) -> Pos2 {
        Pos2::new(
            self.rect.left() + (position.x / LOGICAL_CANVAS_SIZE) * self.rect.width(),
            self.rect.top() + (position.y / LOGICAL_CANVAS_SIZE) * self.rect.height(),
        )
    }

    fn logical_size_to_screen(&self, width: f32, height: f32) -> Vec2 {
        Vec2::new(
            width / LOGICAL_CANVAS_SIZE * self.rect.width(),
            height / LOGICAL_CANVAS_SIZE * self.rect.height(),
        )
    }
}

fn text_logical_size(text: &str, font_size: f32, target: FrameSize) -> Vec2 {
    if text.is_empty() {
        return Vec2::ZERO;
    }

    let character_count = text.chars().count() as f32;

    let pixel_font_size = font_size * target.height as f32 / LOGICAL_CANVAS_SIZE;

    let pixel_width = character_count * pixel_font_size * 0.60;

    let pixel_height = pixel_font_size * 1.20;

    let logical_width = pixel_width / target.width as f32 * LOGICAL_CANVAS_SIZE;

    let logical_height = pixel_height / target.height as f32 * LOGICAL_CANVAS_SIZE;

    Vec2::new(logical_width, logical_height)
}

fn load_default_font() -> FontArc {
    let mut candidates = Vec::new();

    if let Some(path) = std::env::var_os("OPENLCD_FONT") {
        candidates.push(PathBuf::from(path));
    }

    candidates.extend([
        PathBuf::from("/usr/share/fonts/TTF/DejaVuSans.ttf"),
        PathBuf::from("/usr/share/fonts/dejavu/DejaVuSans.ttf"),
        PathBuf::from("/usr/share/fonts/gnu-free/FreeSans.ttf"),
    ]);

    for path in candidates {
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };

        let Ok(font) = FontArc::try_from_vec(bytes) else {
            continue;
        };

        println!("Fonte carregada: {}", path.display(),);

        return font;
    }

    panic!(concat!(
        "Nenhuma fonte válida encontrada.\n",
        "Defina OPENLCD_FONT apontando para ",
        "um arquivo TTF, por exemplo:\n",
        "set -x OPENLCD_FONT ",
        "/usr/share/fonts/TTF/DejaVuSans.ttf"
    ));
}

fn create_initial_scene() -> (Scene, LayerId) {
    let mut scene = Scene::new("OpenLCD Preview");

    scene.add_background([18, 24, 42, 255]);

    scene.add_text(TextLayer::new(
        0,
        "OpenLCD Studio",
        LogicalPosition::new(60.0, 80.0),
        32.0,
        [255, 255, 255, 255],
    ));

    let cpu_layer_id = scene.add_text(TextLayer::new(
        0,
        "CPU 42%",
        LogicalPosition::new(60.0, 230.0),
        46.0,
        [80, 220, 140, 255],
    ));

    (scene, cpu_layer_id)
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

    scene: Scene,
    cpu_layer_id: LayerId,
    selected_layer_id: Option<LayerId>,
    font: FontArc,

    dragging_layer_id: Option<LayerId>,
    drag_start_logical: Option<LogicalPosition>,
    drag_start_layer_position: Option<LogicalPosition>,
}

impl OpenLcdApp {
    pub fn new(context: &eframe::CreationContext<'_>) -> Self {
        let selected_profile = PreviewProfile::Kmex;

        let font = load_default_font();

        let (scene, cpu_layer_id) = create_initial_scene();

        let mut app = Self {
            selected_profile,

            fake_display: FakeDisplay::new(selected_profile.capabilities()),

            texture: None,

            scene,
            cpu_layer_id,
            selected_layer_id: Some(cpu_layer_id),
            font,

            dragging_layer_id: None,
            drag_start_logical: None,
            drag_start_layer_position: None,

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

    fn hit_test_scene(
        &self,
        logical_position: LogicalPosition,
        target: FrameSize,
    ) -> Option<LayerId> {
        for layer in self.scene.layers.iter().rev() {
            if !layer.visible() {
                continue;
            }

            match layer {
                Layer::Text(text) => {
                    let size = text_logical_size(&text.text, text.font_size, target);

                    let left = text.position.x;

                    let top = text.position.y;

                    let right = left + size.x;

                    let bottom = top + size.y;

                    if logical_position.x >= left
                        && logical_position.x <= right
                        && logical_position.y >= top
                        && logical_position.y <= bottom
                    {
                        return Some(text.id);
                    }
                }

                Layer::Background(background) => {
                    return Some(background.id);
                }
            }
        }

        None
    }

    fn text_layer_position(&self, id: LayerId) -> Option<LogicalPosition> {
        let layer = self.scene.layer(id)?;

        match layer {
            Layer::Text(text) => Some(text.position),

            Layer::Background(_) => None,
        }
    }

    fn begin_drag(&mut self, logical_position: LogicalPosition, target: FrameSize) {
        let Some(layer_id) = self.hit_test_scene(logical_position, target) else {
            self.dragging_layer_id = None;
            return;
        };

        self.selected_layer_id = Some(layer_id);

        let Some(layer_position) = self.text_layer_position(layer_id) else {
            // Background pode ser selecionado,
            // mas não pode ser arrastado.
            self.dragging_layer_id = None;
            self.drag_start_logical = None;
            self.drag_start_layer_position = None;

            return;
        };

        self.dragging_layer_id = Some(layer_id);

        self.drag_start_logical = Some(logical_position);

        self.drag_start_layer_position = Some(layer_position);
    }

    fn update_drag(&mut self, logical_position: LogicalPosition, context: &egui::Context) {
        let Some(layer_id) = self.dragging_layer_id else {
            return;
        };

        let Some(drag_start) = self.drag_start_logical else {
            return;
        };

        let Some(layer_start) = self.drag_start_layer_position else {
            return;
        };

        let delta_x = logical_position.x - drag_start.x;

        let delta_y = logical_position.y - drag_start.y;

        let new_x = (layer_start.x + delta_x).clamp(0.0, LOGICAL_CANVAS_SIZE);

        let new_y = (layer_start.y + delta_y).clamp(0.0, LOGICAL_CANVAS_SIZE);

        let Some(layer) = self.scene.layer_mut(layer_id) else {
            return;
        };

        let Layer::Text(text) = layer else {
            return;
        };

        text.position = LogicalPosition::new(new_x, new_y);

        self.render_preview(context);
    }

    fn end_drag(&mut self) {
        self.dragging_layer_id = None;
        self.drag_start_logical = None;
        self.drag_start_layer_position = None;
    }

    fn selected_layer_screen_rect(&self, transform: PreviewTransform) -> Option<Rect> {
        let selected_id = self.selected_layer_id?;

        let layer = self.scene.layer(selected_id)?;

        match layer {
            Layer::Background(_) => Some(transform.rect),

            Layer::Text(text) => {
                let logical_size =
                    text_logical_size(&text.text, text.font_size, transform.native_size);

                let top_left = transform.logical_to_screen(text.position);

                let screen_size = transform.logical_size_to_screen(logical_size.x, logical_size.y);

                Some(Rect::from_min_size(top_left, screen_size))
            }
        }
    }

    fn properties_panel(&mut self, ui: &mut egui::Ui, context: &egui::Context) {
        ui.heading("Propriedades");

        let Some(selected_id) = self.selected_layer_id else {
            ui.label("Nenhuma layer selecionada.");

            return;
        };

        ui.label(format!("Layer ID: {}", selected_id,));

        ui.separator();

        let is_cpu_layer = selected_id == self.cpu_layer_id;

        let mut changed = false;

        {
            let Some(layer) = self.scene.layer_mut(selected_id) else {
                ui.label("Layer não encontrada.");

                return;
            };

            match layer {
                Layer::Background(background) => {
                    ui.label("Tipo: Background");

                    changed |= ui.checkbox(&mut background.visible, "Visível").changed();

                    ui.separator();

                    ui.label("Cor RGBA");

                    changed |= ui
                        .add(egui::Slider::new(&mut background.color[0], 0..=255).text("R"))
                        .changed();

                    changed |= ui
                        .add(egui::Slider::new(&mut background.color[1], 0..=255).text("G"))
                        .changed();

                    changed |= ui
                        .add(egui::Slider::new(&mut background.color[2], 0..=255).text("B"))
                        .changed();

                    changed |= ui
                        .add(egui::Slider::new(&mut background.color[3], 0..=255).text("A"))
                        .changed();
                }

                Layer::Text(text) => {
                    ui.label("Tipo: Text");

                    changed |= ui.checkbox(&mut text.visible, "Visível").changed();

                    ui.separator();

                    if is_cpu_layer {
                        ui.label("Conteúdo: dinâmico");

                        ui.label(format!("Valor atual: {}", text.text,));
                    } else {
                        ui.label("Texto");

                        changed |= ui.text_edit_singleline(&mut text.text).changed();
                    }

                    ui.separator();

                    ui.label("Posição lógica");

                    changed |= ui
                        .add(egui::Slider::new(&mut text.position.x, 0.0..=1000.0).text("X"))
                        .changed();

                    changed |= ui
                        .add(egui::Slider::new(&mut text.position.y, 0.0..=1000.0).text("Y"))
                        .changed();

                    ui.separator();

                    changed |= ui
                        .add(egui::Slider::new(&mut text.font_size, 1.0..=200.0).text("Font size"))
                        .changed();

                    ui.separator();

                    ui.label("Cor RGBA");

                    changed |= ui
                        .add(egui::Slider::new(&mut text.color[0], 0..=255).text("R"))
                        .changed();

                    changed |= ui
                        .add(egui::Slider::new(&mut text.color[1], 0..=255).text("G"))
                        .changed();

                    changed |= ui
                        .add(egui::Slider::new(&mut text.color[2], 0..=255).text("B"))
                        .changed();

                    changed |= ui
                        .add(egui::Slider::new(&mut text.color[3], 0..=255).text("A"))
                        .changed();
                }
            }
        }

        if changed {
            self.render_preview(context);
        }
    }

    fn layers_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Layers");

        ui.small(format!("{} layers", self.scene.layers.len(),));

        ui.separator();

        for layer in &self.scene.layers {
            let id = layer.id();

            let selected = self.selected_layer_id == Some(id);

            let visible_marker = if layer.visible() { "●" } else { "○" };

            let label = format!("{} {}", visible_marker, self.layer_label(layer),);

            if ui.selectable_label(selected, label).clicked() {
                self.selected_layer_id = Some(id);
            }
        }
    }

    fn layer_label(&self, layer: &Layer) -> String {
        match layer {
            Layer::Background(_) => "Background".to_owned(),

            Layer::Text(text) if text.id == self.cpu_layer_id => "CPU Usage".to_owned(),

            Layer::Text(text) => {
                format!("Text: {}", text.text,)
            }
        }
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

    fn update_scene_values(&mut self) {
        let Some(layer) = self.scene.layer_mut(self.cpu_layer_id) else {
            return;
        };

        let Layer::Text(text) = layer else {
            return;
        };

        text.text = format!("CPU {:.0}%", self.cpu_usage,);
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
        let render_started = Instant::now();

        self.update_scene_values();

        let size = self.selected_profile.size();

        let renderer = SceneRenderer::new(size, self.font.clone());

        let frame = match renderer.render(&self.scene) {
            Ok(frame) => frame,

            Err(error) => {
                eprintln!("Erro ao renderizar Scene: {error}");

                return;
            }
        };

        if let Err(error) = self.fake_display.send_rgba(&frame) {
            eprintln!("FakeDisplay rejeitou frame: {error}");

            return;
        }

        self.frame_count += 1;

        let Some(frame) = self.fake_display.last_frame() else {
            return;
        };

        let preview_frame = Self::apply_fake_brightness(frame, self.fake_display.brightness());

        let size = preview_frame.size();

        let color_image = ColorImage::from_rgba_unmultiplied(
            [size.width as usize, size.height as usize],
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

        let now = Instant::now();

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

        ui.separator();

        self.layers_panel(ui);

        ui.separator();

        self.properties_panel(ui, context);
    }

    fn preview_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Preview");

        let Some(texture) = self.texture.clone() else {
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
                let available = ui.available_size();

                let horizontal_space = (available.x - preview_size.x).max(0.0);

                let vertical_space = (available.y - preview_size.y).max(0.0);

                ui.add_space(vertical_space / 2.0);

                ui.horizontal(|ui| {
                    ui.add_space(horizontal_space / 2.0);

                    let response = ui.add(
                        egui::Image::new(&texture)
                            .fit_to_exact_size(preview_size)
                            .sense(Sense::click_and_drag()),
                    );

                    let transform = PreviewTransform::new(response.rect, native);

                    if response.drag_started() {
                        if let Some(pointer_position) = response.interact_pointer_pos() {
                            if let Some(logical_position) =
                                transform.screen_to_logical(pointer_position)
                            {
                                self.begin_drag(logical_position, native);
                            }
                        }
                    }

                    if response.dragged() {
                        if let Some(pointer_position) = response.interact_pointer_pos() {
                            if let Some(logical_position) =
                                transform.screen_to_logical(pointer_position)
                            {
                                self.update_drag(logical_position, ui.ctx());
                            }
                        }
                    }

                    if response.drag_stopped() {
                        self.end_drag();
                    }

                    if response.clicked() {
                        if let Some(pointer_position) = response.interact_pointer_pos() {
                            if let Some(logical_position) =
                                transform.screen_to_logical(pointer_position)
                            {
                                self.selected_layer_id =
                                    self.hit_test_scene(logical_position, native);
                            }
                        }
                    }

                    if let Some(selection_rect) = self.selected_layer_screen_rect(transform) {
                        let stroke_width = if self.dragging_layer_id.is_some() {
                            2.5
                        } else {
                            1.5
                        };

                        ui.painter().rect_stroke(
                            selection_rect,
                            0.0,
                            Stroke::new(stroke_width, egui::Color32::YELLOW),
                            StrokeKind::Outside,
                        );
                    }
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

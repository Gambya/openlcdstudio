use std::{fs, path::PathBuf, time::Instant};

use ab_glyph::{Font, FontArc, PxScale, ScaleFont};

use eframe::egui::{
    self, ColorImage, Pos2, Rect, Sense, Stroke, StrokeKind, TextureHandle, TextureOptions, Vec2,
};

use openlcd_core::{
    DeviceCapabilities, DisplayShape, FrameSize, LOGICAL_CANVAS_SIZE, LogicalPosition, LogicalRect,
    Orientation, PixelFormat, RgbaFrame, classify_display,
};

use openlcd_driver::DisplayDevice;
use openlcd_fake_driver::FakeDisplay;

use openlcd_render::SceneRenderer;

use openlcd_theme::{ImageFit, ImageLayer, Layer, LayerId, Scene, TextLayer, ThemeDocument};

use rfd::FileDialog;

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

fn text_logical_size(font: &FontArc, text: &str, font_size: f32, target: FrameSize) -> Vec2 {
    if text.is_empty() || target.width == 0 || target.height == 0 {
        return Vec2::ZERO;
    }

    let pixel_font_size = font_size * target.height as f32 / LOGICAL_CANVAS_SIZE;

    let scaled = font.as_scaled(PxScale::from(pixel_font_size));

    let mut width = 0.0_f32;
    let mut previous = None;

    for character in text.chars() {
        let glyph = font.glyph_id(character);

        if let Some(previous_glyph) = previous {
            width += scaled.kern(previous_glyph, glyph);
        }

        width += scaled.h_advance(glyph);

        previous = Some(glyph);
    }

    let height = scaled.height();

    let logical_width = width / target.width as f32 * LOGICAL_CANVAS_SIZE;

    let logical_height = height / target.height as f32 * LOGICAL_CANVAS_SIZE;

    Vec2::new(logical_width.max(0.0), logical_height.max(0.0))
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

const RESIZE_HANDLE_SIZE: f32 = 10.0;

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
    theme_path: Option<PathBuf>,
    cpu_layer_id: LayerId,
    selected_layer_id: Option<LayerId>,
    font: FontArc,
    renderer: SceneRenderer,

    dragging_layer_id: Option<LayerId>,
    drag_start_logical: Option<LogicalPosition>,
    drag_start_layer_position: Option<LogicalPosition>,

    resizing_layer_id: Option<LayerId>,
    resize_start_pointer: Option<Pos2>,
    resize_start_font_size: Option<f32>,
    resize_start_image_rect: Option<LogicalRect>,
}

impl OpenLcdApp {
    pub fn new(context: &eframe::CreationContext<'_>) -> Self {
        let selected_profile = PreviewProfile::Kmex;

        let font = load_default_font();
        let renderer = SceneRenderer::new(selected_profile.size(), font.clone());

        let (scene, cpu_layer_id) = create_initial_scene();

        let mut app = Self {
            selected_profile,

            fake_display: FakeDisplay::new(selected_profile.capabilities()),

            texture: None,

            scene,
            theme_path: None,
            cpu_layer_id,
            selected_layer_id: Some(cpu_layer_id),
            font,
            renderer,

            dragging_layer_id: None,
            drag_start_logical: None,
            drag_start_layer_position: None,

            resizing_layer_id: None,
            resize_start_pointer: None,
            resize_start_font_size: None,
            resize_start_image_rect: None,

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

    fn save_theme_as(&mut self) {
        let Some(path) = FileDialog::new()
            .add_filter("OpenLCD Theme", &["json"])
            .set_file_name("theme.json")
            .save_file()
        else {
            return;
        };

        let theme = ThemeDocument::new(self.scene.name.clone(), self.scene.clone());

        match theme.save_managed(&path) {
            Ok(()) => {
                self.theme_path = Some(path);
            }

            Err(error) => {
                eprintln!("Erro ao salvar tema: {error}");
            }
        }
    }

    fn save_theme(&mut self) {
        let Some(path) = self.theme_path.clone() else {
            self.save_theme_as();
            return;
        };

        let theme = ThemeDocument::new(self.scene.name.clone(), self.scene.clone());

        if let Err(error) = theme.save_managed(path) {
            eprintln!("Erro ao salvar tema: {error}");
        }
    }

    fn open_theme(&mut self, context: &egui::Context) {
        let Some(path) = FileDialog::new()
            .add_filter("OpenLCD Theme", &["json"])
            .pick_file()
        else {
            return;
        };

        let theme = match ThemeDocument::load(&path) {
            Ok(theme) => theme,

            Err(error) => {
                eprintln!("Erro ao abrir tema: {error}");

                return;
            }
        };

        self.scene = theme.scene;

        self.theme_path = Some(path);

        self.selected_layer_id = self.scene.layers.last().map(Layer::id);

        self.end_drag();
        self.end_resize();

        // O layer de CPU é especial apenas no protótipo.
        // Um tema carregado pode não possuir essa layer.
        if self.scene.layer(self.cpu_layer_id).is_none() {
            self.cpu_layer_id = 0;
        }

        self.render_preview(context);
    }

    fn file_panel(&mut self, ui: &mut egui::Ui, context: &egui::Context) {
        ui.horizontal(|ui| {
            if ui.button("Abrir").clicked() {
                self.open_theme(context);
            }

            if ui.button("Salvar").clicked() {
                self.save_theme();
            }

            if ui.button("Salvar como...").clicked() {
                self.save_theme_as();
            }

            ui.separator();

            match &self.theme_path {
                Some(path) => {
                    let name = path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("theme.json");

                    ui.label(name);
                }

                None => {
                    ui.label("Tema não salvo");
                }
            }
        });
    }

    fn pick_image_file() -> Option<PathBuf> {
        FileDialog::new()
            .add_filter("Imagens", &["png", "jpg", "jpeg", "bmp", "webp"])
            .pick_file()
    }

    fn add_text_layer(&mut self, context: &egui::Context) {
        let id = self.scene.add_text(TextLayer::new(
            0,
            "Text",
            LogicalPosition::new(100.0, 100.0),
            40.0,
            [255, 255, 255, 255],
        ));

        self.selected_layer_id = Some(id);

        self.render_preview(context);
    }

    fn add_image_layer(&mut self, context: &egui::Context) {
        let Some(path) = Self::pick_image_file() else {
            return;
        };

        let mut image = ImageLayer::new(
            0,
            path.to_string_lossy().into_owned(),
            LogicalRect::new(100.0, 100.0, 800.0, 800.0),
        );

        image.fit = ImageFit::Contain;

        let id = self.scene.add_image(image);

        self.selected_layer_id = Some(id);

        self.render_preview(context);
    }

    fn replace_selected_image(&mut self, context: &egui::Context) {
        let Some(id) = self.selected_layer_id else {
            return;
        };

        let Some(path) = Self::pick_image_file() else {
            return;
        };

        let Some(layer) = self.scene.layer_mut(id) else {
            return;
        };

        let Layer::Image(image) = layer else {
            return;
        };

        image.source = path.to_string_lossy().into_owned();

        self.render_preview(context);
    }

    fn remove_selected_layer(&mut self, context: &egui::Context) {
        let Some(id) = self.selected_layer_id else {
            return;
        };

        if self.scene.is_background(id) {
            return;
        }

        if !self.scene.remove_layer(id) {
            return;
        }

        self.end_drag();
        self.end_resize();

        self.selected_layer_id = self.scene.layers.last().map(Layer::id);

        self.render_preview(context);
    }

    fn move_selected_layer_up(&mut self, context: &egui::Context) {
        let Some(id) = self.selected_layer_id else {
            return;
        };

        if self.scene.move_layer_up(id) {
            self.render_preview(context);
        }
    }

    fn move_selected_layer_down(&mut self, context: &egui::Context) {
        let Some(id) = self.selected_layer_id else {
            return;
        };

        if self.scene.move_layer_down(id) {
            self.render_preview(context);
        }
    }

    fn selected_resize_handle_rect(&self, transform: PreviewTransform) -> Option<Rect> {
        let selected_id = self.selected_layer_id?;

        let layer = self.scene.layer(selected_id)?;

        match layer {
            Layer::Background(_) => None,

            Layer::Image(_) | Layer::Text(_) => {
                let bounds = self.selected_layer_screen_rect(transform)?;

                Some(Rect::from_center_size(
                    bounds.right_bottom(),
                    Vec2::splat(RESIZE_HANDLE_SIZE),
                ))
            }
        }
    }

    fn begin_resize(&mut self, pointer_position: Pos2) {
        let Some(layer_id) = self.selected_layer_id else {
            return;
        };

        let Some(layer) = self.scene.layer(layer_id) else {
            return;
        };

        self.resize_start_font_size = None;
        self.resize_start_image_rect = None;

        match layer {
            Layer::Background(_) => {
                return;
            }

            Layer::Image(image) => {
                self.resizing_layer_id = Some(layer_id);

                self.resize_start_pointer = Some(pointer_position);

                self.resize_start_image_rect = Some(image.rect);
            }

            Layer::Text(text) => {
                self.resizing_layer_id = Some(layer_id);

                self.resize_start_pointer = Some(pointer_position);

                self.resize_start_font_size = Some(text.font_size);
            }
        }

        // Resize e drag são mutuamente exclusivos.
        self.dragging_layer_id = None;
        self.drag_start_logical = None;
        self.drag_start_layer_position = None;
    }

    fn update_resize(
        &mut self,
        pointer_position: Pos2,
        transform: PreviewTransform,
        context: &egui::Context,
    ) {
        let Some(layer_id) = self.resizing_layer_id else {
            return;
        };

        let Some(start_pointer) = self.resize_start_pointer else {
            return;
        };

        if transform.rect.width() <= 0.0 || transform.rect.height() <= 0.0 {
            return;
        }

        let delta_screen = pointer_position - start_pointer;

        let delta_logical_x = delta_screen.x / transform.rect.width() * LOGICAL_CANVAS_SIZE;

        let delta_logical_y = delta_screen.y / transform.rect.height() * LOGICAL_CANVAS_SIZE;

        let start_font_size = self.resize_start_font_size;

        let start_image_rect = self.resize_start_image_rect;

        let Some(layer) = self.scene.layer_mut(layer_id) else {
            return;
        };

        match layer {
            Layer::Background(_) => {
                return;
            }

            Layer::Image(image) => {
                let Some(start_rect) = start_image_rect else {
                    return;
                };

                let max_width = (LOGICAL_CANVAS_SIZE - start_rect.position.x).max(1.0);

                let max_height = (LOGICAL_CANVAS_SIZE - start_rect.position.y).max(1.0);

                image.rect.size.width =
                    (start_rect.size.width + delta_logical_x).clamp(1.0, max_width);

                image.rect.size.height =
                    (start_rect.size.height + delta_logical_y).clamp(1.0, max_height);
            }

            Layer::Text(text) => {
                let Some(start_font_size) = start_font_size else {
                    return;
                };

                // Mantém o comportamento que já validamos:
                // movimento vertical altera font_size.
                let new_font_size = (start_font_size + delta_logical_y).clamp(1.0, 300.0);

                text.font_size = new_font_size;
            }
        }

        self.render_preview(context);
    }

    fn end_resize(&mut self) {
        self.resizing_layer_id = None;
        self.resize_start_pointer = None;
        self.resize_start_font_size = None;
        self.resize_start_image_rect = None;
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
                    let size = text_logical_size(&self.font, &text.text, text.font_size, target);

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

                Layer::Image(image) => {
                    let left = image.rect.position.x;

                    let top = image.rect.position.y;

                    let right = left + image.rect.size.width;

                    let bottom = top + image.rect.size.height;

                    if logical_position.x >= left
                        && logical_position.x <= right
                        && logical_position.y >= top
                        && logical_position.y <= bottom
                    {
                        return Some(image.id);
                    }
                }

                Layer::Background(background) => {
                    return Some(background.id);
                }
            }
        }

        None
    }

    fn layer_position(&self, id: LayerId) -> Option<LogicalPosition> {
        let layer = self.scene.layer(id)?;

        match layer {
            Layer::Background(_) => None,

            Layer::Image(image) => Some(image.rect.position),

            Layer::Text(text) => Some(text.position),
        }
    }

    fn begin_drag(&mut self, logical_position: LogicalPosition, target: FrameSize) {
        let Some(layer_id) = self.hit_test_scene(logical_position, target) else {
            self.dragging_layer_id = None;
            return;
        };

        self.selected_layer_id = Some(layer_id);

        let Some(layer_position) = self.layer_position(layer_id) else {
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

        let new_x = layer_start.x + delta_x;

        let new_y = layer_start.y + delta_y;

        let Some(layer) = self.scene.layer_mut(layer_id) else {
            return;
        };

        match layer {
            Layer::Background(_) => {
                return;
            }

            Layer::Image(image) => {
                let max_x = (LOGICAL_CANVAS_SIZE - image.rect.size.width).max(0.0);

                let max_y = (LOGICAL_CANVAS_SIZE - image.rect.size.height).max(0.0);

                image.rect.position.x = new_x.clamp(0.0, max_x);

                image.rect.position.y = new_y.clamp(0.0, max_y);
            }

            Layer::Text(text) => {
                text.position = LogicalPosition::new(
                    new_x.clamp(0.0, LOGICAL_CANVAS_SIZE),
                    new_y.clamp(0.0, LOGICAL_CANVAS_SIZE),
                );
            }
        }

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

            Layer::Image(image) => {
                let top_left = transform.logical_to_screen(image.rect.position);

                let size =
                    transform.logical_size_to_screen(image.rect.size.width, image.rect.size.height);

                Some(Rect::from_min_size(top_left, size))
            }

            Layer::Text(text) => {
                let logical_size = text_logical_size(
                    &self.font,
                    &text.text,
                    text.font_size,
                    transform.native_size,
                );

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

        let mut replace_image = false;

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

                Layer::Image(image) => {
                    ui.label("Arquivo");

                    let image_name = std::path::Path::new(&image.source)
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or(&image.source);

                    ui.label(image_name);

                    if ui.button("Trocar imagem...").clicked() {
                        replace_image = true;
                    }

                    ui.label("Tipo: Image");

                    changed |= ui.checkbox(&mut image.visible, "Visível").changed();

                    ui.separator();

                    ui.label("Source");

                    changed |= ui.text_edit_singleline(&mut image.source).changed();

                    ui.separator();

                    ui.label("Posição lógica");

                    changed |= ui
                        .add(egui::Slider::new(&mut image.rect.position.x, 0.0..=1000.0).text("X"))
                        .changed();

                    changed |= ui
                        .add(egui::Slider::new(&mut image.rect.position.y, 0.0..=1000.0).text("Y"))
                        .changed();

                    ui.separator();

                    ui.label("Tamanho lógico");

                    changed |= ui
                        .add(
                            egui::Slider::new(&mut image.rect.size.width, 1.0..=1000.0)
                                .text("Width"),
                        )
                        .changed();

                    changed |= ui
                        .add(
                            egui::Slider::new(&mut image.rect.size.height, 1.0..=1000.0)
                                .text("Height"),
                        )
                        .changed();

                    ui.separator();

                    ui.label("Fit");

                    egui::ComboBox::from_id_salt(format!("image-fit-{}", image.id,))
                        .selected_text(match image.fit {
                            ImageFit::Cover => "Cover",
                            ImageFit::Contain => "Contain",
                            ImageFit::Stretch => "Stretch",
                        })
                        .show_ui(ui, |ui| {
                            changed |= ui
                                .selectable_value(&mut image.fit, ImageFit::Cover, "Cover")
                                .changed();

                            changed |= ui
                                .selectable_value(&mut image.fit, ImageFit::Contain, "Contain")
                                .changed();

                            changed |= ui
                                .selectable_value(&mut image.fit, ImageFit::Stretch, "Stretch")
                                .changed();
                        });

                    ui.separator();

                    changed |= ui
                        .add(egui::Slider::new(&mut image.opacity, 0..=255).text("Opacity"))
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

        if replace_image {
            self.replace_selected_image(context);
        }
    }

    fn layers_panel(&mut self, ui: &mut egui::Ui, context: &egui::Context) {
        ui.heading("Layers");

        ui.small(format!("{} layers", self.scene.layers.len(),));

        ui.separator();

        //
        // Criate layers
        //

        ui.horizontal(|ui| {
            if ui.button("+ Texto").clicked() {
                self.add_text_layer(context);
            }

            if ui.button("+ Imagem").clicked() {
                self.add_image_layer(context);
            }
        });

        ui.separator();

        //
        // List
        //

        for layer in self.scene.layers.iter().rev() {
            let id = layer.id();

            let selected = self.selected_layer_id == Some(id);

            let visible_marker = if layer.visible() { "●" } else { "○" };

            let label = format!("{} {}", visible_marker, self.layer_label(layer),);

            if ui.selectable_label(selected, label).clicked() {
                self.selected_layer_id = Some(id);
            }
        }

        ui.separator();

        //
        // Z-order
        //

        ui.horizontal(|ui| {
            let selected = self.selected_layer_id.is_some();

            if ui
                .add_enabled(selected, egui::Button::new("Up"))
                .on_hover_text("Move layer to up")
                .clicked()
            {
                self.move_selected_layer_up(context);
            }

            if ui
                .add_enabled(selected, egui::Button::new("Down"))
                .on_hover_text("Move layer to down")
                .clicked()
            {
                self.move_selected_layer_down(context);
            }

            let removable = self
                .selected_layer_id
                .is_some_and(|id| !self.scene.is_background(id));

            if ui
                .add_enabled(removable, egui::Button::new("Remove"))
                .on_hover_text("Remove layer")
                .clicked()
            {
                self.remove_selected_layer(context);
            }
        });
    }

    fn layer_label(&self, layer: &Layer) -> String {
        match layer {
            Layer::Background(_) => "Background".to_owned(),

            Layer::Image(image) => {
                let name = std::path::Path::new(&image.source)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or(&image.source);

                format!("Image: {name}")
            }

            Layer::Text(text) if text.id == self.cpu_layer_id => "CPU Usage".to_owned(),

            Layer::Text(text) => {
                format!("Text: {}", text.text)
            }
        }
    }

    fn change_profile(&mut self, profile: PreviewProfile, context: &egui::Context) {
        self.selected_profile = profile;

        self.renderer.set_size(profile.size());

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

        let frame = match self.renderer.render(&self.scene) {
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

        self.layers_panel(ui, context);

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

                    if response.drag_started()
                        && let Some(pointer_position) = response.interact_pointer_pos()
                    {
                        let resize_handle = self.selected_resize_handle_rect(transform);

                        let resizing = resize_handle
                            .is_some_and(|rect| rect.expand(4.0).contains(pointer_position));

                        if resizing {
                            self.begin_resize(pointer_position);
                        } else if let Some(logical_position) =
                            transform.screen_to_logical(pointer_position)
                        {
                            self.begin_drag(logical_position, native);
                        }
                    }

                    if response.dragged() {
                        if let Some(pointer_position) = response.interact_pointer_pos() {
                            if self.resizing_layer_id.is_some() {
                                self.update_resize(pointer_position, transform, ui.ctx());
                            } else if let Some(logical_position) =
                                transform.screen_to_logical(pointer_position)
                            {
                                self.update_drag(logical_position, ui.ctx());
                            }
                        }
                    }

                    if response.drag_stopped() {
                        self.end_drag();
                        self.end_resize();
                    }

                    if response.clicked()
                        && let Some(pointer_position) = response.interact_pointer_pos()
                        && let Some(logical_position) =
                            transform.screen_to_logical(pointer_position)
                    {
                        self.selected_layer_id = self.hit_test_scene(logical_position, native);
                    }

                    if let Some(pointer_position) = ui.ctx().pointer_hover_pos()
                        && let Some(handle_rect) = self.selected_resize_handle_rect(transform)
                        && handle_rect.expand(4.0).contains(pointer_position)
                    {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeNwSe);
                    }

                    if let Some(selection_rect) = self.selected_layer_screen_rect(transform) {
                        let manipulating =
                            self.dragging_layer_id.is_some() || self.resizing_layer_id.is_some();

                        let stroke_width = if manipulating { 2.5 } else { 1.5 };

                        ui.painter().rect_stroke(
                            selection_rect,
                            0.0,
                            Stroke::new(stroke_width, egui::Color32::YELLOW),
                            StrokeKind::Outside,
                        );
                    }

                    if let Some(handle_rect) = self.selected_resize_handle_rect(transform) {
                        ui.painter()
                            .rect_filled(handle_rect, 1.0, egui::Color32::YELLOW);

                        ui.painter().rect_stroke(
                            handle_rect,
                            1.0,
                            Stroke::new(1.0, egui::Color32::BLACK),
                            StrokeKind::Inside,
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

        egui::Panel::top("file-panel").show(ui, |ui| {
            self.file_panel(ui, &context);
        });

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

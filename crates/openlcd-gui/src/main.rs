use eframe::egui::{self, Vec2};

use openlcd_gui::OpenLcdApp;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("OpenLCD Studio")
            .with_inner_size(Vec2::new(1280.0, 800.0))
            .with_min_inner_size(Vec2::new(900.0, 600.0)),

        ..Default::default()
    };

    eframe::run_native(
        "OpenLCD Studio",
        native_options,
        Box::new(|context| Ok(Box::new(OpenLcdApp::new(context)))),
    )
}

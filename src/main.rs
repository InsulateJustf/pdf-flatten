#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod i18n;
mod pdf;
mod ui;

use app::PdfFlattenApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 400.0])
            .with_title(i18n::title()),
        ..Default::default()
    };
    
    eframe::run_native(
        "PDF Flatten",
        options,
        Box::new(|cc| Ok(Box::new(PdfFlattenApp::new(cc)))),
    )
}

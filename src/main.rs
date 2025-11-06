#![windows_subsystem = "windows"]
use eframe::egui;
use egui::Vec2;
mod app;
mod color;
mod font_loader;
mod key_box;
mod keyboard;
mod listen;
mod press_time_map;

fn main() {
    let _qwerty_87_size = Vec2 { x: 885., y: 450. };
    let qwerty_alice_size = Vec2 { x: 1060., y: 420. };
    let size = qwerty_alice_size;
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_min_inner_size(size)
            .with_inner_size(size)
            .with_resizable(false)
            .with_active(true),
        ..Default::default()
    };
    let _ = eframe::run_native("Keyboard Heatmap", native_options, Box::new(app::setup_ui));
}

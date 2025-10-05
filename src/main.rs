#![windows_subsystem = "windows"]
use eframe::{egui, Theme};
use egui::Vec2;
mod app;
mod color;
mod key_box;
mod keyboard;
mod listen;
mod press_time_map;

fn main() {
    let qwerty_87_size = Vec2 { x: 885., y: 450. };
    let qwerty_alice_size = Vec2 { x: 1060., y: 400. };
    let size = qwerty_alice_size;
    let native_options = eframe::NativeOptions {
        min_window_size: Some(size),
        initial_window_size: Some(size),
        default_theme: Theme::Light,
        resizable: false,
        ..Default::default()
    };
    eframe::run_native("Keyboard Heatmap", native_options, Box::new(app::setup_ui))
}

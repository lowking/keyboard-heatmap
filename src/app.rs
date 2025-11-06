use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use crate::{
    color,
    keyboard::{self, KeyboardType},
    listen,
    press_time_map::PressTimesMap,
};
use chrono::prelude::DateTime;
use eframe::{
    epaint::HsvaGamma,
    glow::{self, HasContext},
    App, CreationContext,
};
use egui::Color32;
use crate::press_time_map::TOTLE_TIMES;
use std::sync::atomic::Ordering;

pub struct State {
    keyboard_type: KeyboardType,
    hue: f32,
    start_time: DateTime<chrono::Local>,
    dark_mode: bool,
    manual_theme_override: bool,
    font_family: FontFamily,
}

#[derive(Clone, Copy, PartialEq)]
pub enum FontFamily {
    Monospace,
    Proportional,
}

impl FontFamily {
    pub fn description(&self) -> &'static str {
        match self {
            FontFamily::Monospace => "Monospace",
            FontFamily::Proportional => "Proportional",
        }
    }
}

struct KeyboardHeatmap {
    state: Arc<Mutex<State>>,
    press_map: Arc<Mutex<PressTimesMap>>,
    take_screenshot: bool,
}

impl eframe::App for KeyboardHeatmap {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            state,
            press_map: _,
            take_screenshot: _,
        } = self;
        let mut state = state.lock().unwrap();

        // Get system theme preference from macOS only if not manually overridden
        if !state.manual_theme_override {
            #[cfg(target_os = "macos")]
            let system_dark_mode = {
                use std::process::Command;
                if let Ok(output) = Command::new("defaults")
                    .args(&["read", "-g", "AppleInterfaceStyle"])
                    .output()
                {
                    String::from_utf8_lossy(&output.stdout).contains("Dark")
                } else {
                    false
                }
            };
            #[cfg(not(target_os = "macos"))]
            let system_dark_mode = false;

            // Auto-switch theme based on system preference
            if system_dark_mode != state.dark_mode {
                state.dark_mode = system_dark_mode;
                // Change hue when theme changes
                if state.dark_mode {
                    state.hue = 0.0;  // Dark mode: red
                } else {
                    state.hue = 220. / 360.;  // Light mode: blue
                }
            }
        }

        // Apply theme
        if state.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        // Request continuous repaint to update colors even when in background
        ctx.request_repaint();

        let background_color = if state.dark_mode {
            Color32::from_rgb(0x3A, 0x38, 0x37)
        } else {
            Color32::WHITE
        };

        let frame = egui::Frame::none()
            .inner_margin(egui::style::Margin {
                left: 30.,
                right: 30.,
                top: 30.,
                bottom: 30.,
            })
            .fill(background_color);

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            let press_map = &mut self.press_map.lock().unwrap();
            // toolbar
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Press count: {}",
                    TOTLE_TIMES.load(Ordering::Relaxed) - 100
                ));

                ui.separator();

                ui.label(format!(
                    "Recording since {}",
                    state.start_time.format("%y-%m-%d %H:%M:%S")
                ));
                if ui.button("Clear").clicked() {
                    state.start_time = chrono::Local::now();
                    press_map.map.clear();
                    PressTimesMap::clear();
                }

                ui.separator();

                egui::ComboBox::from_id_source("keyboard_selector")
                    .selected_text(state.keyboard_type.description())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut state.keyboard_type,
                            KeyboardType::QwertyMac,
                            KeyboardType::QwertyMac.description(),
                        );
                        ui.selectable_value(
                            &mut state.keyboard_type,
                            KeyboardType::Qwerty87,
                            KeyboardType::Qwerty87.description(),
                        );
                        ui.selectable_value(
                            &mut state.keyboard_type,
                            KeyboardType::QwertyAliceWeikav,
                            KeyboardType::QwertyAliceWeikav.description(),
                        );
                    });

                ui.separator();

                color::color_slider_1d(ui, &mut state.hue, |h| {
                    HsvaGamma {
                        h,
                        s: 0.6,
                        v: 0.75,
                        a: 1.0,
                    }
                        .into()
                });

                ui.separator();

                // Font family selector (before dark mode toggle)
                egui::ComboBox::from_id_source("font_selector")
                    .selected_text(state.font_family.description())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut state.font_family,
                            FontFamily::Monospace,
                            FontFamily::Monospace.description(),
                        );
                        ui.selectable_value(
                            &mut state.font_family,
                            FontFamily::Proportional,
                            FontFamily::Proportional.description(),
                        );
                    });

                ui.separator();

                // Dark mode toggle
                let theme_text = if state.dark_mode { "☀ Light" } else { "🌙 Dark" };
                if ui.button(theme_text).clicked() {
                    state.manual_theme_override = true;
                    state.dark_mode = !state.dark_mode;
                    // Change hue when switching theme
                    if state.dark_mode {
                        // Dark mode: red (0.0)
                        state.hue = 0.0;
                    } else {
                        // Light mode: blue (220/360)
                        state.hue = 220. / 360.;
                    }
                }

                ui.separator();

                if ui.button("Save as PNG").clicked() {
                    self.take_screenshot = true;
                }
            });

            ui.separator();
            ui.add_space(30.);

            let mut keyboard = keyboard::Keyboard::new(state.keyboard_type, state.hue, state.dark_mode, state.font_family);
            keyboard.draw(press_map, ui);
        });
    }

    /// export the screenshot into a png file.
    fn post_rendering(&mut self, screen_size_px: [u32; 2], frame: &eframe::Frame) {
        if !self.take_screenshot {
            return;
        }
        self.take_screenshot = false;

        // (0, 0) is at the left bottom
        let toolbar_height = 56;
        let state = self.state.lock().unwrap();
        let Some(gl) = frame.gl() else { return };
        let [w, h] = screen_size_px;
        let w = match state.keyboard_type {
            KeyboardType::QwertyMac => w - 20 - 420,
            KeyboardType::Qwerty87 => w - 20,
            KeyboardType::QwertyAliceWeikav => w,
        };

        let h = h as i32 - toolbar_height;
        let mut buf = vec![0u8; w as usize * h as usize * 4];
        let pixels = glow::PixelPackData::Slice(&mut buf[..]);
        unsafe {
            gl.read_pixels(
                0,
                0,
                w as i32,
                h as i32,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                pixels,
            );
        }

        // Flip vertically:
        let mut rows: Vec<Vec<u8>> = buf
            .chunks(w as usize * 4)
            .into_iter()
            .map(|chunk| chunk.to_vec())
            .collect();
        rows.reverse();
        let buf: Vec<u8> = rows.into_iter().flatten().collect();

        // save as image file
        let path = native_dialog::FileDialog::new()
            .set_location("~/Desktop")
            .add_filter("PNG Image", &["png"])
            .show_save_single_file()
            .unwrap();
        if let Some(path) = path {
            if let Err(err) =
                image::save_buffer(path, &buf[..], w as u32, h as u32, image::ColorType::Rgba8)
            {
                println!("err {:?}", err);
            };
        }
    }
}

impl KeyboardHeatmap {
    fn new(state: Arc<Mutex<State>>, press_map: Arc<Mutex<PressTimesMap>>) -> Self {
        Self {
            state,
            press_map,
            take_screenshot: false,
        }
    }
}

pub fn setup_ui(_cc: &CreationContext) -> Box<dyn App> {
    let (sender, receiver) = mpsc::sync_channel(1);

    let state = Arc::new(Mutex::new(State {
        keyboard_type: KeyboardType::QwertyAliceWeikav,
        hue: 220. / 360.,
        start_time: chrono::Local::now(),
        dark_mode: false,
        manual_theme_override: false,
        font_family: FontFamily::Monospace,
    }));
    let press_map = Arc::new(Mutex::new(PressTimesMap::new()));

    // listen to keyboard press events
    {
        thread::spawn(move || {
            listen::listen_keyboard(sender.clone());
        });
    }
    // handle keyboard press events
    {
        let press_map = press_map.clone();
        thread::spawn(move || loop {
            if let Some(event_type) = receiver.recv().ok() {
                let mut press_map = press_map.lock().unwrap();
                if let rdev::EventType::KeyPress(key) = event_type {
                    press_map.key_press(key);
                }
            }
        });
    }

    Box::new(KeyboardHeatmap::new(state, press_map))
}

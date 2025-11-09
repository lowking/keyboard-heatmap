use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
    fs,
    path::PathBuf,
};

use crate::{
    color,
    font_loader,
    keyboard::{self, KeyboardType},
    listen,
    press_time_map::PressTimesMap,
};
use chrono::prelude::DateTime;
use eframe::{
    epaint::HsvaGamma,
    App, CreationContext,
};
use egui::Color32;
use crate::press_time_map::TOTLE_TIMES;
use std::sync::atomic::Ordering;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Settings {
    selected_font: String,
    font_family: FontFamily,
    font_size: f32,
    bold: bool,
    dark_mode: bool,
    hue: f32,
}

fn get_settings_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("keyboard-heatmap");
    fs::create_dir_all(&path).ok();
    path.push("settings.json");
    path
}

fn save_settings(state: &State) -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings {
        selected_font: state.selected_font.clone(),
        font_family: state.font_family,
        font_size: state.font_size,
        bold: state.bold,
        dark_mode: state.dark_mode,
        hue: state.hue,
    };
    let json = serde_json::to_string_pretty(&settings)?;
    fs::write(get_settings_path(), json)?;
    Ok(())
}

fn load_settings() -> Option<Settings> {
    let path = get_settings_path();
    if path.exists() {
        let json = fs::read_to_string(path).ok()?;
        serde_json::from_str(&json).ok()
    } else {
        None
    }
}

pub struct State {
    keyboard_type: KeyboardType,
    hue: f32,
    start_time: DateTime<chrono::Local>,
    dark_mode: bool,
    font_family: FontFamily,
    system_fonts: Vec<String>,
    font_search: String,
    selected_font: String,
    font_selector_index: usize,
    font_size: f32,
    font_size_input: String,
    bold: bool,
    font_selector_open: bool,
    font_selector_just_opened: bool,
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FontFamily {
    Monospace,
    Proportional,
}

struct KeyboardHeatmap {
    state: Arc<Mutex<State>>,
    press_map: Arc<Mutex<PressTimesMap>>,
}

impl eframe::App for KeyboardHeatmap {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            state,
            press_map: _,
        } = self;
        let mut state = state.lock().unwrap();

        // Apply theme
        if state.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        // Request repaint at a reasonable rate (5 FPS) to save CPU
        ctx.request_repaint_after(std::time::Duration::from_millis(200));

        let background_color = if state.dark_mode {
            Color32::from_rgb(0x3A, 0x38, 0x37)
        } else {
            Color32::WHITE
        };

        let frame = egui::Frame::default()
            .inner_margin(egui::Margin::same(30))
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

                egui::ComboBox::from_id_salt("keyboard_selector")
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

                // Bold font button
                let button_text = if state.bold {
                    egui::RichText::new("B").strong()
                } else {
                    egui::RichText::new("B")
                };
                if ui.button(button_text).clicked() {
                    state.bold = !state.bold;
                    let _ = save_settings(&state);
                }

                // Font size decrease button
                if ui.button("-").clicked() {
                    if state.font_size > 6.0 {
                        state.font_size -= 1.0;
                        state.font_size_input = state.font_size.to_string();
                        let _ = save_settings(&state);
                    }
                }

                // Font size label
                ui.label(format!("{:.0}", state.font_size));

                // Font size increase button
                if ui.button("+").clicked() {
                    if state.font_size < 72.0 {
                        state.font_size += 1.0;
                        state.font_size_input = state.font_size.to_string();
                        let _ = save_settings(&state);
                    }
                }

                // Custom font selector button with dropdown
                let button_text = if state.selected_font.is_empty() {
                    "Select Font".to_string()
                } else {
                    state.selected_font.clone()
                };

                let button_response = ui.button(&button_text);
                let button_rect = button_response.rect;

                if button_response.clicked() {
                    state.font_selector_open = !state.font_selector_open;
                    if state.font_selector_open {
                        // When opening, find the current font's index and mark as just opened
                        state.font_selector_just_opened = true;
                        if let Some(index) = state.system_fonts.iter().position(|f| f == &state.selected_font) {
                            state.font_selector_index = index;
                        }
                    }
                }

                // Show dropdown popup when open
                if state.font_selector_open {
                    // Position popup below the button
                    let popup_id = egui::Id::new("font_selector_popup");

                    let area_response = egui::Area::new(popup_id)
                        .fixed_pos(button_rect.left_bottom())
                        .order(egui::Order::Tooltip)  // Use higher z-index to avoid being covered by hover tooltips
                        .show(ui.ctx(), |ui| {
                            egui::Frame::popup(ui.style()).show(ui, |ui| {
                                ui.set_width(300.0);

                                // Search box - without selection highlight to avoid flickering
                                let search_response = ui.add(
                                    egui::TextEdit::singleline(&mut state.font_search)
                                        .hint_text("Search fonts...")
                                        .cursor_at_end(true)  // Keep cursor at end instead of selecting all
                                );

                                // Filter fonts based on search
                                let search_lower = state.font_search.to_lowercase();
                                let filtered_fonts: Vec<String> = if search_lower.is_empty() {
                                    state.system_fonts.clone()
                                } else {
                                    state.system_fonts
                                        .iter()
                                        .filter(|f| f.to_lowercase().contains(&search_lower))
                                        .cloned()
                                        .collect()
                                };

                                // Reset index when search changes
                                if search_response.changed() {
                                    state.font_selector_index = 0;
                                }

                                // Handle keyboard navigation - capture keys at the Area level
                                let mut index_changed = false;
                                let mut should_apply_font = false;
                                ui.input(|i| {
                                    if i.key_pressed(egui::Key::ArrowDown) {
                                        if state.font_selector_index < filtered_fonts.len().saturating_sub(1) {
                                            state.font_selector_index += 1;
                                            index_changed = true;
                                        }
                                    } else if i.key_pressed(egui::Key::ArrowUp) {
                                        if state.font_selector_index > 0 {
                                            state.font_selector_index -= 1;
                                            index_changed = true;
                                        }
                                    } else if i.key_pressed(egui::Key::Enter) {
                                        should_apply_font = true;
                                    } else if i.key_pressed(egui::Key::Escape) {
                                        state.font_selector_open = false;
                                        state.font_search.clear();
                                    }
                                });

                                // Load font preview when index changed (up/down keys)
                                if index_changed {
                                    if let Some(font) = filtered_fonts.get(state.font_selector_index) {
                                        // Load the font for preview
                                        if let Some(font_data) = font_loader::load_font_data(font) {
                                            let mut fonts = egui::FontDefinitions::default();
                                            fonts.font_data.insert(
                                                "custom_font".to_owned(),
                                                Arc::new(egui::FontData::from_owned(font_data)),
                                            );
                                            let font_family = state.font_family;
                                            fonts
                                                .families
                                                .entry(match font_family {
                                                    FontFamily::Monospace => egui::FontFamily::Monospace,
                                                    FontFamily::Proportional => egui::FontFamily::Proportional,
                                                })
                                                .or_default()
                                                .insert(0, "custom_font".to_owned());
                                            ctx.set_fonts(fonts);
                                        }
                                    }
                                }

                                // Apply font selection when Enter is pressed
                                if should_apply_font {
                                    if let Some(font) = filtered_fonts.get(state.font_selector_index) {
                                        state.selected_font = font.clone();
                                        state.font_search.clear();
                                        state.font_selector_open = false;
                                        let _ = save_settings(&state);
                                    }
                                }

                                ui.separator();

                                // Check if we need to scroll to selected item on first open
                                let should_scroll_to_selected = state.font_selector_just_opened;
                                if state.font_selector_just_opened {
                                    state.font_selector_just_opened = false;
                                }

                                // Scrollable font list
                                egui::ScrollArea::vertical()
                                    .max_height(300.0)
                                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        for (idx, font) in filtered_fonts.iter().enumerate() {
                                            let is_selected = idx == state.font_selector_index;
                                            let is_current = &state.selected_font == font;

                                            let label = if is_current {
                                                egui::RichText::new(format!("* {}", font)).strong()
                                            } else {
                                                egui::RichText::new(font.as_str())
                                            };

                                            let response = ui.selectable_label(is_selected, label);

                                            if response.clicked() {
                                                state.selected_font = font.clone();
                                                state.font_selector_index = idx;
                                                state.font_search.clear();
                                                state.font_selector_open = false;

                                                // Load the selected font
                                                if let Some(font_data) = font_loader::load_font_data(font) {
                                                    let mut fonts = egui::FontDefinitions::default();
                                                    fonts.font_data.insert(
                                                        "custom_font".to_owned(),
                                                        Arc::new(egui::FontData::from_owned(font_data)),
                                                    );
                                                    let font_family = state.font_family;
                                                    fonts
                                                        .families
                                                        .entry(match font_family {
                                                            FontFamily::Monospace => egui::FontFamily::Monospace,
                                                            FontFamily::Proportional => egui::FontFamily::Proportional,
                                                        })
                                                        .or_default()
                                                        .insert(0, "custom_font".to_owned());
                                                    ctx.set_fonts(fonts);
                                                }
                                                let _ = save_settings(&state);
                                            }

                                            // Update hover index - but don't clear keyboard selection
                                            if response.hovered() && !index_changed {
                                                state.font_selector_index = idx;
                                            }

                                            // Auto-scroll to selected item when using keyboard navigation or just opened
                                            if (index_changed || should_scroll_to_selected) && is_selected {
                                                response.scroll_to_me(Some(egui::Align::Center));
                                            }
                                        }
                                    });
                            })
                        });

                    // Close popup when clicking outside (but not on the button itself)
                    if ui.input(|i| i.pointer.any_click()) {
                        if let Some(pointer_pos) = ui.ctx().pointer_hover_pos() {
                            let popup_rect = area_response.response.rect;
                            if !popup_rect.contains(pointer_pos) && !button_rect.contains(pointer_pos) {
                                state.font_selector_open = false;
                                state.font_search.clear();
                            }
                        }
                    }
                }

                // Font family selector
                egui::ComboBox::from_id_salt("font_family_selector")
                    .selected_text(match state.font_family {
                        FontFamily::Monospace => "Monospace",
                        FontFamily::Proportional => "Proportional",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut state.font_family, FontFamily::Monospace, "Monospace");
                        ui.selectable_value(&mut state.font_family, FontFamily::Proportional, "Proportional");
                    });

                ui.separator();

                let color_response = color::color_slider_1d(ui, &mut state.hue, |h| {
                    HsvaGamma {
                        h,
                        s: 0.6,
                        v: 0.75,
                        a: 1.0,
                    }
                        .into()
                });

                // Save settings when color changes
                if color_response.changed() {
                    let _ = save_settings(&state);
                }

                // Dark mode toggle
                let theme_text = if state.dark_mode { "☀ Light" } else { "🌙 Dark" };
                if ui.button(theme_text).clicked() {
                    state.dark_mode = !state.dark_mode;
                    // Change hue when switching theme
                    if state.dark_mode {
                        // Dark mode: red (0.0)
                        state.hue = 0.0;
                    } else {
                        // Light mode: blue (220/360)
                        state.hue = 220. / 360.;
                    }
                    let _ = save_settings(&state);
                }
            });

            ui.separator();
            ui.add_space(30.);

            let mut keyboard = keyboard::Keyboard::new(state.keyboard_type, state.hue, state.dark_mode, state.font_family, state.font_size, state.bold);
            keyboard.draw(press_map, ui);
        });
    }

}

impl KeyboardHeatmap {
    fn new(state: Arc<Mutex<State>>, press_map: Arc<Mutex<PressTimesMap>>) -> Self {
        Self {
            state,
            press_map,
        }
    }
}

pub fn setup_ui(cc: &CreationContext) -> Result<Box<dyn App>, Box<dyn std::error::Error + Send + Sync>> {
    let (sender, receiver) = mpsc::sync_channel(1);

    // Get system fonts
    let system_fonts = font_loader::get_system_fonts();

    // Load saved settings or use defaults
    let saved_settings = load_settings();
    let default_font = saved_settings.as_ref()
        .map(|s| s.selected_font.clone())
        .unwrap_or_else(|| "Menlo".to_string());
    let font_family = saved_settings.as_ref()
        .map(|s| s.font_family)
        .unwrap_or(FontFamily::Monospace);
    let font_size = saved_settings.as_ref()
        .map(|s| s.font_size)
        .unwrap_or(14.0);
    let bold = saved_settings.as_ref()
        .map(|s| s.bold)
        .unwrap_or(false);
    let dark_mode = saved_settings.as_ref()
        .map(|s| s.dark_mode)
        .unwrap_or(false);
    let hue = saved_settings.as_ref()
        .map(|s| s.hue)
        .unwrap_or(220. / 360.);

    // Load font
    if let Some(font_data) = font_loader::load_font_data(&default_font) {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "custom_font".to_owned(),
            Arc::new(egui::FontData::from_owned(font_data)),
        );
        // Set as highest priority based on font family
        fonts
            .families
            .entry(match font_family {
                FontFamily::Monospace => egui::FontFamily::Monospace,
                FontFamily::Proportional => egui::FontFamily::Proportional,
            })
            .or_default()
            .insert(0, "custom_font".to_owned());
        cc.egui_ctx.set_fonts(fonts);
    }

    let state = Arc::new(Mutex::new(State {
        keyboard_type: KeyboardType::QwertyAliceWeikav,
        hue,
        start_time: chrono::Local::now(),
        dark_mode,
        font_family,
        system_fonts: system_fonts.clone(),
        font_search: String::new(),
        selected_font: default_font.clone(),
        font_selector_index: 0,
        font_size,
        font_size_input: font_size.to_string(),
        bold,
        font_selector_open: false,
        font_selector_just_opened: false,
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

    Ok(Box::new(KeyboardHeatmap::new(state, press_map)))
}

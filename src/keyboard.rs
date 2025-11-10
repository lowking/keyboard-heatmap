use std::sync::MutexGuard;
use std::collections::HashMap;

use crate::{
    key_box::{KeyBox, KeyTextsLayout},
    press_time_map::PressTimesMap,
    app::FontFamily,
};

use egui::{Color32, Sense, Ui, Vec2};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum KeyboardType {
    QwertyMac,
    Qwerty87,
    QwertyAliceWeikav,
}

impl KeyboardType {
    pub fn description(&self) -> &'static str {
        match self {
            KeyboardType::QwertyMac => "MacBook",
            KeyboardType::Qwerty87 => "87 Keys",
            KeyboardType::QwertyAliceWeikav => "Weikav Alice 68",
        }
    }
}

const FN_KEYS_PAIRS: [(&str, rdev::Key); 12] = [
    ("F1", rdev::Key::F1),
    ("F2", rdev::Key::F2),
    ("F3", rdev::Key::F3),
    ("F4", rdev::Key::F4),
    ("F5", rdev::Key::F5),
    ("F6", rdev::Key::F6),
    ("F7", rdev::Key::F7),
    ("F8", rdev::Key::F8),
    ("F9", rdev::Key::F9),
    ("F10", rdev::Key::F10),
    ("F11", rdev::Key::F11),
    ("F12", rdev::Key::F12),
];
const NUM_KEY_LINE_PAIRS: [(&str, &str, rdev::Key); 13] = [
    ("`", "~", rdev::Key::BackQuote),
    ("!", "1", rdev::Key::Num1),
    ("@", "2", rdev::Key::Num2),
    ("#", "3", rdev::Key::Num3),
    ("$", "4", rdev::Key::Num4),
    ("%", "5", rdev::Key::Num5),
    ("^", "6", rdev::Key::Num6),
    ("&", "7", rdev::Key::Num7),
    ("*", "8", rdev::Key::Num8),
    ("(", "9", rdev::Key::Num9),
    (")", "0", rdev::Key::Num0),
    ("-", "_", rdev::Key::Minus),
    ("+", "=", rdev::Key::Equal),
];

const FIRST_ALPHA_LINE_PAIRS: [(&str, rdev::Key); 10] = [
    ("Q", rdev::Key::KeyQ),
    ("W", rdev::Key::KeyW),
    ("E", rdev::Key::KeyE),
    ("R", rdev::Key::KeyR),
    ("T", rdev::Key::KeyT),
    ("Y", rdev::Key::KeyY),
    ("U", rdev::Key::KeyU),
    ("I", rdev::Key::KeyI),
    ("O", rdev::Key::KeyO),
    ("P", rdev::Key::KeyP),
];

const SECOND_ALPHA_LINE_PAIRS: [(&str, rdev::Key); 9] = [
    ("A", rdev::Key::KeyA),
    ("S", rdev::Key::KeyS),
    ("D", rdev::Key::KeyD),
    ("F", rdev::Key::KeyF),
    ("G", rdev::Key::KeyG),
    ("H", rdev::Key::KeyH),
    ("J", rdev::Key::KeyJ),
    ("K", rdev::Key::KeyK),
    ("L", rdev::Key::KeyL),
];

const THIRD_ALPHA_LINE_PAIRS: [(&str, rdev::Key); 7] = [
    ("Z", rdev::Key::KeyZ),
    ("X", rdev::Key::KeyX),
    ("C", rdev::Key::KeyC),
    ("V", rdev::Key::KeyV),
    ("B", rdev::Key::KeyB),
    ("N", rdev::Key::KeyN),
    ("M", rdev::Key::KeyM),
];

const THIRD_ALPHA_LINE_PAIRS_ALICE: [(&str, rdev::Key); 8] = [
    ("Z", rdev::Key::KeyZ),
    ("X", rdev::Key::KeyX),
    ("C", rdev::Key::KeyC),
    ("V", rdev::Key::KeyV),
    ("B1", rdev::Key::KeyB),
    ("B2", rdev::Key::KeyB),
    ("N", rdev::Key::KeyN),
    ("M", rdev::Key::KeyM),
];

const SECTION_SPACE: f32 = 15.;

/// Key rotation and offset configuration for Alice keyboard
/// Maps (row, col, key_label) to (rotation_angle, offset_x, offset_y)
/// row: 0-4, col: 0-N (left to right position in each row)
/// rotation: angle in radians, offset: (x, y) for position adjustment
fn get_alice_rotation_config() -> HashMap<(usize, usize, &'static str), (f32, f32, f32)> {
    let mut config = HashMap::new();

    // Format: config.insert((row, col, "label"), (rotation, offset_x, offset_y));
    // Positive rotation values rotate clockwise, negative values rotate counter-clockwise
    // Example: (0.14, 0.0, 0.0) means 0.14 radians (≈ 7°) with no offset

    // Row 0 - Number row with Esc (15 keys total)
    config.insert((0, 0, "Esc"), (0.0, 0.0, 0.0));
    config.insert((0, 1, "~`"), (0.0, 0.0, 0.0));
    config.insert((0, 2, "1!"), (0.0, 0.0, 0.0));
    config.insert((0, 3, "2@"), (0.0, 0.0, -10.0));
    config.insert((0, 4, "3#"), (0.14, -10.0, 0.0));
    config.insert((0, 5, "4$"), (0.14, -10.0, 8.0));
    config.insert((0, 6, "5%"), (0.14, -10.0, 16.0));
    config.insert((0, 7, "6^"), (0.14, -10.0, 24.0));

    config.insert((0, 8, "7&"), (-0.14, 10.0, 19.0));
    config.insert((0, 9, "8*"), (-0.14, 10.0, 11.0));
    config.insert((0, 10, "9("), (-0.14, 10.0, 3.0));
    config.insert((0, 11, "0)"), (-0.14, 10.0, -5.0));
    config.insert((0, 12, "_-"), (0.0, 0.0, -10.0));
    config.insert((0, 13, "=+"), (0.0, 0.0, 0.0));
    config.insert((0, 14, "Back"), (0.0, 0.0, 0.0));

    // Row 1 - QWERTY row (15 keys total)
    config.insert((1, 0, "Esc"), (0.0, 0.0, 0.0));
    config.insert((1, 1, "Tab"), (0.0, 0.0, 0.0));
    config.insert((1, 2, "Q"), (0.0, 0.0, 0.0));

    config.insert((1, 3, "W"), (0.14, 0.0, -5.0));
    config.insert((1, 4, "E"), (0.14, 0.0, 3.0));
    config.insert((1, 5, "R"), (0.14, 0.0, 11.0));
    config.insert((1, 6, "T"), (0.14, 0.0, 19.0));

    config.insert((1, 7, "Y"), (-0.14, 7.0, 22.0));
    config.insert((1, 8, "U"), (-0.14, 7.0, 14.0));
    config.insert((1, 9, "I"), (-0.14, 7.0, 6.0));
    config.insert((1, 10, "O"), (-0.14, 7.0, -2.0));

    config.insert((1, 11, "P"), (0.0, 0.0, -8.0));
    config.insert((1, 12, "{["), (0.0, 0.0, 0.0));
    config.insert((1, 13, "}]"), (0.0, 0.0, 0.0));
    config.insert((1, 14, "|\\"), (0.0, 0.0, 0.0));

    // Row 2 - ASDF row (14 keys total)
    config.insert((2, 0, "Esc"), (0.0, 0.0, 0.0));
    config.insert((2, 1, "Caps\nLock"), (0.0, 0.0, 0.0));
    config.insert((2, 2, "A"), (0.0, 0.0, 0.0));

    config.insert((2, 3, "S"), (0.14, 0.0, -3.0));
    config.insert((2, 4, "D"), (0.14, 0.0, 5.0));
    config.insert((2, 5, "F"), (0.14, 0.0, 13.0));
    config.insert((2, 6, "G"), (0.14, 0.0, 21.0));

    config.insert((2, 7, "H"), (-0.14, 6.0, 21.0));
    config.insert((2, 8, "J"), (-0.14, 6.0, 13.0));
    config.insert((2, 9, "K"), (-0.14, 6.0, 5.0));
    config.insert((2, 10, "L"), (-0.14, 6.0, -3.0));

    config.insert((2, 11, ";:"), (0.0, 0.0, 0.0));
    config.insert((2, 12, "'\""), (0.0, 0.0, 0.0));
    config.insert((2, 13, "Enter"), (0.0, 0.0, 0.0));

    // Row 3 - ZXCV row (14 keys total)
    config.insert((3, 0, "Shift"), (0.0, 0.0, 0.0));
    config.insert((3, 1, "Z"), (0.0, 0.0, 0.0));
    config.insert((3, 2, "X"), (0.14, 0.0, 1.0));
    config.insert((3, 3, "C"), (0.14, 0.0, 9.0));
    config.insert((3, 4, "V"), (0.14, 0.0, 17.0));
    config.insert((3, 5, "B1"), (0.14, 0.0, 25.0));

    config.insert((3, 6, "B2"), (-0.14, 0.0, 25.0));
    config.insert((3, 7, "N"), (-0.14, 0.0, 17.0));
    config.insert((3, 8, "M"), (-0.14, 0.0, 9.0));
    config.insert((3, 9, ",<"), (-0.14, 0.0, 1.0));

    config.insert((3, 10, ".>"), (0.0, 0.0, 0.0));
    config.insert((3, 11, "/?"), (0.0, 0.0, 0.0));
    config.insert((3, 12, "↑"), (0.0, 0.0, 0.0));
    config.insert((3, 13, "Shift"), (0.0, 0.0, 0.0));

    // Row 4 - Bottom row (9 keys total)
    config.insert((4, 0, "Ctrl"), (0.0, 0.0, 0.0));
    config.insert((4, 1, "Fn"), (0.0, 0.0, 0.0));

    config.insert((4, 2, "Cmd"), (0.14, 9.0, 7.0));
    config.insert((4, 3, "Option"), (0.14, 9.0, 21.0));

    config.insert((4, 4, " "), (-0.14, -15.0, 18.0));
    config.insert((4, 5, "Win"), (-0.14, -15.0, 1.0));
    config.insert((4, 6, "←"), (0.0, 0.0, 0.0));
    config.insert((4, 7, "↓"), (0.0, 0.0, 0.0));
    config.insert((4, 8, "→"), (0.0, 0.0, 0.0));

    config
}

pub struct Keyboard {
    // different from key numbers (and OSs)
    keyboard_type: KeyboardType,
    // [0, 1], the hue of the color
    hue: f32,
    dark_mode: bool,
    font_family: FontFamily,
    font_size: f32,
    bold: bool,
}

impl Keyboard {
    pub fn new(keyboard_type: KeyboardType, hue: f32, dark_mode: bool, font_family: FontFamily, font_size: f32, bold: bool) -> Self {
        Self { keyboard_type, hue, dark_mode, font_family, font_size, bold }
    }
}
impl Keyboard {
    pub fn draw(&mut self, map: &MutexGuard<PressTimesMap>, ui: &mut Ui) {
        // Read AVERAGE_TIMES once for all keys
        use std::sync::atomic::Ordering;
        use crate::press_time_map::AVERAGE_TIMES;
        let average_times = AVERAGE_TIMES.load(Ordering::Relaxed) as u32;

        match self.keyboard_type {
            KeyboardType::QwertyMac => self.draw_mac_keyboard(map, ui, average_times),
            KeyboardType::Qwerty87 => self.draw_87_keyboard(map, ui, average_times),
            KeyboardType::QwertyAliceWeikav => self.draw_alice_keyboard(map, ui, average_times),
        }
    }

    fn draw_mac_keyboard(&mut self, map: &MutexGuard<PressTimesMap>, ui: &mut Ui, average_times: u32) {
        let basic_size = Vec2 { x: 50., y: 50. };
        // 1st line
        ui.horizontal(|ui| {
            self.draw_single_label_key(map, Vec2 { x: 70., y: 50. }, rdev::Key::Escape, "Esc", ui, average_times);

            for key_pair in FN_KEYS_PAIRS.iter() {
                self.draw_single_label_key(map, basic_size, key_pair.1, key_pair.0, ui, average_times)
            }

            self.draw_single_label_key(map, basic_size, rdev::Key::Unknown(0), "Power", ui, average_times);
        });
        ui.add_space(3.);

        // 2nd line
        ui.horizontal(|ui| {
            for key_pair in NUM_KEY_LINE_PAIRS.iter() {
                self.draw_double_labels_key(
                    map, basic_size, key_pair.2, key_pair.0, key_pair.1, ui, average_times,
                );
            }

            self.draw_single_label_key(
                map,
                Vec2 { x: 70., y: 50. },
                rdev::Key::Backspace,
                "Back", ui, average_times);
        });
        ui.add_space(3.);

        // tab line
        ui.horizontal(|ui| {
            self.draw_single_label_key(map, Vec2 { x: 70., y: 50. }, rdev::Key::Tab, "Tab", ui, average_times);

            for key_pair in FIRST_ALPHA_LINE_PAIRS.iter() {
                self.draw_single_label_key(map, basic_size, key_pair.1, key_pair.0, ui, average_times);
            }

            for key_pair in [
                ("[", "{", rdev::Key::LeftBracket),
                ("]", "}", rdev::Key::RightBracket),
                ("\\", "|", rdev::Key::BackSlash),
            ] {
                self.draw_double_labels_key(
                    map, basic_size, key_pair.2, key_pair.0, key_pair.1, ui, average_times,
                );
            }
        });
        ui.add_space(3.);

        // caps lock line
        ui.horizontal(|ui| {
            self.draw_single_label_key(
                map,
                Vec2 { x: 85., y: 50. },
                rdev::Key::CapsLock,
                "Caps\nLock", ui, average_times);

            for key_pair in SECOND_ALPHA_LINE_PAIRS.iter() {
                self.draw_single_label_key(map, basic_size, key_pair.1, key_pair.0, ui, average_times);
            }

            for key_pair in [
                (":", ";", rdev::Key::SemiColon),
                ("\"", "'", rdev::Key::Quote),
            ] {
                self.draw_double_labels_key(
                    map, basic_size, key_pair.2, key_pair.0, key_pair.1, ui, average_times,
                );
            }

            // enter
            self.draw_single_label_key(
                map,
                Vec2 { x: 93., y: 50. },
                rdev::Key::Return,
                "Enter", ui, average_times);
        });
        ui.add_space(3.);

        // shift line
        ui.horizontal(|ui| {
            self.draw_single_label_key(
                map,
                Vec2 { x: 118., y: 50. },
                rdev::Key::ShiftLeft,
                "Shift", ui, average_times);

            for key_pair in THIRD_ALPHA_LINE_PAIRS.iter() {
                self.draw_single_label_key(map, basic_size, key_pair.1, key_pair.0, ui, average_times)
            }

            for key_pair in [
                ("<", ",", rdev::Key::Comma),
                (">", ".", rdev::Key::Dot),
                ("?", "/", rdev::Key::Slash),
            ] {
                self.draw_double_labels_key(
                    map, basic_size, key_pair.2, key_pair.0, key_pair.1, ui, average_times,
                );
            }

            // right shift
            self.draw_single_label_key(
                map,
                Vec2 { x: 118., y: 50. },
                rdev::Key::ShiftRight,
                "Shift", ui, average_times);
        });
        ui.add_space(3.);

        // last line
        ui.horizontal(|ui| {
            self.draw_single_label_key(map, basic_size, rdev::Key::Function, "Fn", ui, average_times);
            self.draw_single_label_key(map, basic_size, rdev::Key::ControlLeft, "Ctrl", ui, average_times);
            self.draw_single_label_key(map, basic_size, rdev::Key::Alt, "Opt", ui, average_times);
            self.draw_single_label_key(
                map,
                Vec2 { x: 61., y: 50. },
                rdev::Key::MetaLeft,
                "Cmd", ui, average_times);
            self.draw_single_label_key(map, Vec2 { x: 280., y: 50. }, rdev::Key::Space, " ", ui, average_times);
            self.draw_single_label_key(
                map,
                Vec2 { x: 61., y: 50. },
                rdev::Key::MetaRight,
                "Cmd", ui, average_times);
            self.draw_single_label_key(map, basic_size, rdev::Key::AltGr, "Opt", ui, average_times);

            let left_times = map.get_key_times(rdev::Key::LeftArrow);
            let mut left_key = KeyBox::new(
                Vec2 { x: 50., y: 24. },
                KeyTextsLayout::Center1("←".to_string()),
                rdev::Key::LeftArrow,
                left_times,
                self.hue,
                self.dark_mode,
                self.font_family,
                self.font_size,
                self.bold,
            );
            ui.vertical(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2 { x: 50., y: 24. }, Sense::hover());
                ui.painter().rect_filled(rect, 0., Color32::TRANSPARENT);
                left_key.ui(ui, average_times);
            });

            let up_times = map.get_key_times(rdev::Key::UpArrow);
            let mut up_key = KeyBox::new(
                Vec2 { x: 50., y: 24. },
                KeyTextsLayout::Center1("↑".to_string()),
                rdev::Key::UpArrow,
                up_times,
                self.hue,
                self.dark_mode,
                self.font_family,
                self.font_size,
                self.bold,
            );
            let down_times = map.get_key_times(rdev::Key::DownArrow);
            let mut down_key = KeyBox::new(
                Vec2 { x: 50., y: 24. },
                KeyTextsLayout::Center1("↓".to_string()),
                rdev::Key::DownArrow,
                down_times,
                self.hue,
                self.dark_mode,
                self.font_family,
                self.font_size,
                self.bold,
            );
            ui.vertical(|ui| {
                up_key.ui(ui, average_times);
                down_key.ui(ui, average_times);
            });

            // ->
            let right_times = map.get_key_times(rdev::Key::RightArrow);
            let mut right_key = KeyBox::new(
                Vec2 { x: 50., y: 24. },
                KeyTextsLayout::Center1("→".to_string()),
                rdev::Key::RightArrow,
                right_times,
                self.hue,
                self.dark_mode,
                self.font_family,
                self.font_size,
                self.bold,
            );
            ui.vertical(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2 { x: 50., y: 25. }, Sense::hover());
                ui.painter().rect_filled(rect, 0., Color32::TRANSPARENT);
                right_key.ui(ui, average_times);
            });
        });
    }

    fn draw_87_keyboard(&mut self, map: &MutexGuard<PressTimesMap>, ui: &mut Ui, average_times: u32) {
        let basic_size = Vec2 { x: 50., y: 50. };
        // 1st line
        ui.horizontal(|ui| {
            self.draw_single_label_key(map, basic_size, rdev::Key::Escape, "Esc", ui, average_times);

            self.draw_empty_key(basic_size, ui);

            for key_pair in FN_KEYS_PAIRS.iter() {
                self.draw_single_label_key(map, basic_size, key_pair.1, key_pair.0, ui, average_times);
                if key_pair.1 == rdev::Key::F4 || key_pair.1 == rdev::Key::F8 {
                    ui.add_space(25.);
                }
            }

            ui.add_space(SECTION_SPACE);

            for key_pair in [
                ("PrtSc", rdev::Key::PrintScreen),
                ("ScrLk", rdev::Key::ScrollLock),
                ("Pause", rdev::Key::Pause),
            ] {
                self.draw_single_label_key(map, basic_size, key_pair.1, key_pair.0, ui, average_times);
            }
        });
        ui.add_space(3.);

        // 2nd line
        ui.horizontal(|ui| {
            for key_pair in NUM_KEY_LINE_PAIRS.iter() {
                self.draw_double_labels_key(
                    map, basic_size, key_pair.2, key_pair.0, key_pair.1, ui, average_times,
                );
            }

            self.draw_single_label_key(
                map,
                Vec2 { x: 100., y: 50. },
                rdev::Key::Backspace,
                "Back", ui, average_times);

            ui.add_space(SECTION_SPACE);

            for key_pair in [
                ("Ins", rdev::Key::Insert),
                ("Home", rdev::Key::Home),
                ("PgUp", rdev::Key::PageUp),
            ] {
                self.draw_single_label_key(map, basic_size, key_pair.1, key_pair.0, ui, average_times);
            }
        });
        ui.add_space(3.);

        // tab line
        ui.horizontal(|ui| {
            self.draw_single_label_key(map, Vec2 { x: 70., y: 50. }, rdev::Key::Tab, "Tab", ui, average_times);

            for key_pair in FIRST_ALPHA_LINE_PAIRS.iter() {
                self.draw_single_label_key(map, basic_size, key_pair.1, key_pair.0, ui, average_times);
            }

            for key_pair in [
                ("[", "{", rdev::Key::LeftBracket),
                ("]", "}", rdev::Key::RightBracket),
            ] {
                self.draw_double_labels_key(
                    map, basic_size, key_pair.2, key_pair.0, key_pair.1, ui, average_times,
                );
            }

            let key_pair = ("\\", "|", rdev::Key::BackSlash);
            self.draw_double_labels_key(
                map,
                Vec2 { x: 80., y: 50. },
                key_pair.2,
                key_pair.0,
                key_pair.1, ui, average_times);

            ui.add_space(SECTION_SPACE);

            for key_pair in [
                ("Del", rdev::Key::Delete),
                ("End", rdev::Key::End),
                ("PgDn", rdev::Key::PageDown),
            ] {
                self.draw_single_label_key(map, basic_size, key_pair.1, key_pair.0, ui, average_times);
            }
        });
        ui.add_space(3.);

        // caps lock line
        ui.horizontal(|ui| {
            self.draw_single_label_key(
                map,
                Vec2 { x: 85., y: 50. },
                rdev::Key::CapsLock,
                "Caps\nLock", ui, average_times);

            for key_pair in SECOND_ALPHA_LINE_PAIRS.iter() {
                self.draw_single_label_key(map, basic_size, key_pair.1, key_pair.0, ui, average_times);
            }

            for key_pair in [
                (":", ";", rdev::Key::SemiColon),
                ("\"", "'", rdev::Key::Quote),
            ] {
                self.draw_double_labels_key(
                    map, basic_size, key_pair.2, key_pair.0, key_pair.1, ui, average_times,
                );
            }

            // enter
            self.draw_single_label_key(
                map,
                Vec2 { x: 123., y: 50. },
                rdev::Key::Return,
                "Enter", ui, average_times);

            ui.add_space(SECTION_SPACE);
            for _ in 0..3 {
                self.draw_empty_key(basic_size, ui);
            }
        });
        ui.add_space(3.);

        // shift line
        ui.horizontal(|ui| {
            self.draw_single_label_key(
                map,
                Vec2 { x: 118., y: 50. },
                rdev::Key::ShiftLeft,
                "Shift", ui, average_times);

            for key_pair in THIRD_ALPHA_LINE_PAIRS.iter() {
                self.draw_single_label_key(map, basic_size, key_pair.1, key_pair.0, ui, average_times)
            }

            for key_pair in [
                ("<", ",", rdev::Key::Comma),
                (">", ".", rdev::Key::Dot),
                ("?", "/", rdev::Key::Slash),
            ] {
                self.draw_double_labels_key(
                    map, basic_size, key_pair.2, key_pair.0, key_pair.1, ui, average_times,
                );
            }

            // right shift
            self.draw_single_label_key(
                map,
                Vec2 { x: 148., y: 50. },
                rdev::Key::ShiftRight,
                "Shift", ui, average_times);

            ui.add_space(SECTION_SPACE);
            self.draw_empty_key(basic_size, ui);
            self.draw_single_label_key(map, basic_size, rdev::Key::UpArrow, "↑", ui, average_times);
            self.draw_empty_key(basic_size, ui);
        });
        ui.add_space(3.);

        // last line
        ui.horizontal(|ui| {
            let ctrl_size = Vec2 { x: 60., y: 50. };
            self.draw_single_label_key(map, ctrl_size, rdev::Key::ControlLeft, "Ctrl", ui, average_times);

            self.draw_single_label_key(map, ctrl_size, rdev::Key::MetaLeft, "Win", ui, average_times);
            self.draw_single_label_key(map, ctrl_size, rdev::Key::Alt, "Alt", ui, average_times);

            self.draw_single_label_key(map, Vec2 { x: 378., y: 50. }, rdev::Key::Space, " ", ui, average_times);

            self.draw_single_label_key(map, ctrl_size, rdev::Key::AltGr, "Alt", ui, average_times);
            self.draw_single_label_key(map, ctrl_size, rdev::Key::Function, "Fn", ui, average_times);
            // no menu in rdev::Key
            self.draw_single_label_key(map, ctrl_size, rdev::Key::Unknown(110), "Menu", ui, average_times);
            self.draw_single_label_key(map, ctrl_size, rdev::Key::ControlRight, "Ctrl", ui, average_times);

            ui.add_space(SECTION_SPACE);
            self.draw_single_label_key(map, basic_size, rdev::Key::LeftArrow, "←", ui, average_times);
            self.draw_single_label_key(map, basic_size, rdev::Key::UpArrow, "↑", ui, average_times);
            self.draw_single_label_key(map, basic_size, rdev::Key::RightArrow, "→", ui, average_times);
        });
    }

    fn draw_alice_keyboard(&mut self, map: &MutexGuard<PressTimesMap>, ui: &mut Ui, average_times: u32) {
        let basic_size = Vec2 { x: 50., y: 50. };
        let left_divide_size = 10.;
        let middle_divide_size = 8.;
        let rotation_config = get_alice_rotation_config();

        // Helper closure to get rotation and offset for a key by row, column, and label
        let get_config = |row: usize, col: usize, label: &str| -> (f32, Vec2) {
            rotation_config
                .get(&(row, col, label))
                .map(|&(rotation, offset_x, offset_y)| (rotation, Vec2::new(offset_x, offset_y)))
                .unwrap_or((0.0, Vec2::ZERO))
        };

        // Row 0 - Number row with Esc
        ui.horizontal(|ui| {
            ui.add_space(left_divide_size * 2.);
            let (rotation, offset) = get_config(0, 0, "Esc");
            self.draw_single_label_key_rotated(map, basic_size, rdev::Key::Escape, "Esc", rotation, offset, ui, average_times);

            ui.add_space(left_divide_size);

            for (idx, key_pair) in NUM_KEY_LINE_PAIRS.iter().enumerate() {
                let label = format!("{}{}", key_pair.1, key_pair.0);
                let (rotation, offset) = get_config(0, idx + 1, &label);
                self.draw_double_labels_key_rotated(
                    map, basic_size, key_pair.2, key_pair.0, key_pair.1, rotation, offset, ui, average_times,
                );
                if key_pair.2 == rdev::Key::Num2 || key_pair.2 == rdev::Key::Num6 || key_pair.2 == rdev::Key::Num0 {
                    ui.add_space(middle_divide_size);
                    continue;
                }
            }

            let (rotation, offset) = get_config(0, 14, "Back");
            self.draw_single_label_key_rotated(
                map,
                Vec2 { x: 110., y: 50. },
                rdev::Key::Backspace,
                "Back",
                rotation,
                offset, ui, average_times);
        });
        ui.add_space(3.);

        // Row 1 - QWERTY row
        ui.horizontal(|ui| {
            ui.add_space(left_divide_size);
            let (rotation, offset) = get_config(1, 0, "Esc");
            self.draw_single_label_key_rotated(map, basic_size, rdev::Key::Escape, "Esc", rotation, offset, ui, average_times);

            ui.add_space(left_divide_size);

            let (rotation, offset) = get_config(1, 1, "Tab");
            self.draw_single_label_key_rotated(map, Vec2 { x: 70., y: 50. }, rdev::Key::Tab, "Tab", rotation, offset, ui, average_times);

            for (idx, key_pair) in FIRST_ALPHA_LINE_PAIRS.iter().enumerate() {
                let (rotation, offset) = get_config(1, idx + 2, key_pair.0);
                self.draw_single_label_key_rotated(map, basic_size, key_pair.1, key_pair.0, rotation, offset, ui, average_times);
                if key_pair.1 == rdev::Key::KeyQ {
                    ui.add_space(middle_divide_size);
                    continue;
                }
                if key_pair.1 == rdev::Key::KeyT {
                    ui.add_space(35.);
                    continue;
                }
                if key_pair.1 == rdev::Key::KeyO {
                    ui.add_space(middle_divide_size);
                    continue;
                }
            }

            for (idx, key_pair) in [
                ("[", "{", rdev::Key::LeftBracket),
                ("]", "}", rdev::Key::RightBracket),
            ]
            .iter()
            .enumerate()
            {
                let label = format!("{}{}", key_pair.1, key_pair.0);
                let (rotation, offset) = get_config(1, idx + 12, &label);
                self.draw_double_labels_key_rotated(
                    map, basic_size, key_pair.2, key_pair.0, key_pair.1, rotation, offset, ui, average_times,
                );
            }

            let key_pair = ("\\", "|", rdev::Key::BackSlash);
            let label = format!("{}{}", key_pair.1, key_pair.0);
            let (rotation, offset) = get_config(1, 14, &label);
            self.draw_double_labels_key_rotated(
                map,
                Vec2 { x: 80., y: 50. },
                key_pair.2,
                key_pair.0,
                key_pair.1,
                rotation,
                offset, ui, average_times);
        });
        ui.add_space(3.);

        // Row 2 - ASDF row
        ui.horizontal(|ui| {
            let (rotation, offset) = get_config(2, 0, "Esc");
            self.draw_single_label_key_rotated(map, basic_size, rdev::Key::Escape, "Esc", rotation, offset, ui, average_times);

            ui.add_space(left_divide_size);

            let (rotation, offset) = get_config(2, 1, "Caps\nLock");
            self.draw_single_label_key_rotated(
                map,
                Vec2 { x: 85., y: 50. },
                rdev::Key::CapsLock,
                "Caps\nLock",
                rotation,
                offset, ui, average_times);

            for (idx, key_pair) in SECOND_ALPHA_LINE_PAIRS.iter().enumerate() {
                let (rotation, offset) = get_config(2, idx + 2, key_pair.0);
                self.draw_single_label_key_rotated(map, basic_size, key_pair.1, key_pair.0, rotation, offset, ui, average_times);
                if key_pair.1 == rdev::Key::KeyA {
                    ui.add_space(middle_divide_size);
                    continue;
                }
                if key_pair.1 == rdev::Key::KeyG {
                    ui.add_space(50.);
                    continue;
                }
                if key_pair.1 == rdev::Key::KeyL {
                    ui.add_space(middle_divide_size);
                    continue;
                }
            }

            for (idx, key_pair) in [
                (":", ";", rdev::Key::SemiColon),
                ("\"", "'", rdev::Key::Quote),
            ]
            .iter()
            .enumerate()
            {
                let label = format!("{}{}", key_pair.1, key_pair.0);
                let (rotation, offset) = get_config(2, idx + 11, &label);
                self.draw_double_labels_key_rotated(
                    map, basic_size, key_pair.2, key_pair.0, key_pair.1, rotation, offset, ui, average_times,
                );
            }

            // enter
            let (rotation, offset) = get_config(2, 13, "Enter");
            self.draw_single_label_key_rotated(
                map,
                Vec2 { x: 125., y: 50. },
                rdev::Key::Return,
                "Enter",
                rotation,
                offset, ui, average_times);
        });
        ui.add_space(3.);

        // Row 3 - ZXCV row
        ui.horizontal(|ui| {
            self.draw_empty_key(basic_size, ui);

            let (rotation, offset) = get_config(3, 0, "Shift");
            self.draw_single_label_key_rotated(
                map,
                Vec2 { x: 118., y: 50. },
                rdev::Key::ShiftLeft,
                "Shift",
                rotation,
                offset, ui, average_times);

            for (idx, key_pair) in THIRD_ALPHA_LINE_PAIRS_ALICE.iter().enumerate() {
                let (rotation, offset) = get_config(3, idx + 1, key_pair.0);
                self.draw_single_label_key_rotated(map, basic_size, key_pair.1, key_pair.0, rotation, offset, ui, average_times);
                if key_pair.1 == rdev::Key::KeyZ {
                    ui.add_space(middle_divide_size);
                    continue;
                }
                if key_pair.0 == "B1" {
                    ui.add_space(10.);
                    continue;
                }
            }

            for (idx, key_pair) in [
                ("<", ",", rdev::Key::Comma),
                (">", ".", rdev::Key::Dot),
                ("?", "/", rdev::Key::Slash),
            ]
            .iter()
            .enumerate()
            {
                let label = format!("{}{}", key_pair.1, key_pair.0);
                let (rotation, offset) = get_config(3, idx + 9, &label);
                self.draw_double_labels_key_rotated(
                    map, basic_size, key_pair.2, key_pair.0, key_pair.1, rotation, offset, ui, average_times,
                );
                if key_pair.2 == rdev::Key::Comma {
                    ui.add_space(middle_divide_size);
                }
            }

            let (rotation, offset) = get_config(3, 12, "↑");
            self.draw_single_label_key_rotated(map, basic_size, rdev::Key::UpArrow, "↑", rotation, offset, ui, average_times);
            // right shift
            let (rotation, offset) = get_config(3, 13, "Shift");
            self.draw_single_label_key_rotated(map, Vec2 { x: 100., y: 50. }, rdev::Key::ShiftRight, "Shift", rotation, offset, ui, average_times);
        });
        ui.add_space(3.);

        // Row 4 - Bottom row
        ui.horizontal(|ui| {
            self.draw_empty_key(basic_size, ui);

            let ctrl_size = Vec2 { x: 65., y: 50. };
            let (rotation, offset) = get_config(4, 0, "Ctrl");
            self.draw_single_label_key_rotated(map, ctrl_size, rdev::Key::ControlLeft, "Ctrl", rotation, offset, ui, average_times);

            let (rotation, offset) = get_config(4, 1, "Fn");
            self.draw_single_label_key_rotated(map, ctrl_size, rdev::Key::Function, "Fn", rotation, offset, ui, average_times);
            ui.add_space(65.);
            let (rotation, offset) = get_config(4, 2, "Cmd");
            self.draw_single_label_key_rotated(map, Vec2 { x: 61., y: 50. }, rdev::Key::MetaLeft, "Cmd", rotation, offset, ui, average_times);
            let (rotation, offset) = get_config(4, 3, "Option");
            self.draw_single_label_key_rotated(map, Vec2 { x: 120., y: 50. }, rdev::Key::Alt, "Option", rotation, offset, ui, average_times);
            ui.add_space(50.);
            let (rotation, offset) = get_config(4, 4, " ");
            self.draw_single_label_key_rotated(map, Vec2 { x: 158., y: 50. }, rdev::Key::Space, " ", rotation, offset, ui, average_times);
            let (rotation, offset) = get_config(4, 5, "Win");
            self.draw_single_label_key_rotated(map, ctrl_size, rdev::Key::MetaLeft, "Win", rotation, offset, ui, average_times);
            ui.add_space(35.);
            let (rotation, offset) = get_config(4, 6, "←");
            self.draw_single_label_key_rotated(map, basic_size, rdev::Key::LeftArrow, "←", rotation, offset, ui, average_times);
            let (rotation, offset) = get_config(4, 7, "↓");
            self.draw_single_label_key_rotated(map, basic_size, rdev::Key::DownArrow, "↓", rotation, offset, ui, average_times);
            let (rotation, offset) = get_config(4, 8, "→");
            self.draw_single_label_key_rotated(map, basic_size, rdev::Key::RightArrow, "→", rotation, offset, ui, average_times);
        });
    }

    fn draw_double_labels_key(
        &mut self,
        map: &MutexGuard<PressTimesMap>,
        size: Vec2,
        key: rdev::Key,
        top_name: &str,
        bottom_name: &str,
        ui: &mut Ui,
        average_times: u32,
    ) {
        let times = map.get_key_times(key);
        let mut key = KeyBox::new(
            size,
            KeyTextsLayout::TopBottom((top_name.to_string(), bottom_name.to_string())),
            key,
            times,
            self.hue,
            self.dark_mode,
                self.font_family,
                self.font_size,
                self.bold,
        );
        key.ui(ui, average_times);
    }

    fn draw_double_labels_key_rotated(
        &mut self,
        map: &MutexGuard<PressTimesMap>,
        size: Vec2,
        key: rdev::Key,
        top_name: &str,
        bottom_name: &str,
        rotation: f32,
        offset: Vec2,
        ui: &mut Ui,
        average_times: u32,
    ) {
        let times = map.get_key_times(key);
        let mut key = KeyBox::new(
            size,
            KeyTextsLayout::TopBottom((top_name.to_string(), bottom_name.to_string())),
            key,
            times,
            self.hue,
            self.dark_mode,
                self.font_family,
                self.font_size,
                self.bold,
        )
        .with_rotation_and_offset(rotation, offset);
        key.ui(ui, average_times);
    }

    fn draw_single_label_key(
        &mut self,
        map: &MutexGuard<PressTimesMap>,
        size: Vec2,
        key: rdev::Key,
        name: &str,
        ui: &mut Ui,
        average_times: u32,
    ) {
        let times = map.get_key_times(key);
        let mut key = KeyBox::new(
            size,
            KeyTextsLayout::Center1(name.to_string()),
            key,
            times,
            self.hue,
            self.dark_mode,
                self.font_family,
                self.font_size,
                self.bold,
        );
        key.ui(ui, average_times);
    }

    fn draw_single_label_key_rotated(
        &mut self,
        map: &MutexGuard<PressTimesMap>,
        size: Vec2,
        key: rdev::Key,
        name: &str,
        rotation: f32,
        offset: Vec2,
        ui: &mut Ui,
        average_times: u32,
    ) {
        let times = map.get_key_times(key);
        let mut key = KeyBox::new(
            size,
            KeyTextsLayout::Center1(name.to_string()),
            key,
            times,
            self.hue,
            self.dark_mode,
                self.font_family,
                self.font_size,
                self.bold,
        )
        .with_rotation_and_offset(rotation, offset);
        key.ui(ui, average_times);
    }

    fn draw_empty_key(&mut self, size: Vec2, ui: &mut Ui) {
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
        ui.painter().rect_filled(rect, 0., Color32::TRANSPARENT);
    }
}

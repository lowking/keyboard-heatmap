use egui::{Align2, Color32, RichText, Sense, Stroke, Ui, Vec2};

use crate::color::{get_color, get_strike_color};
use crate::app::FontFamily;
use eframe::egui::emath::Rot2;

/// Layout in a key box, shows how to display the key contents
#[derive(Clone)]
pub enum KeyTextsLayout {
    TopBottom((String, String)),
    Center1(String),
}
/// component of a key on a keyboard
#[allow(dead_code)]
pub struct KeyBox {
    size: Vec2,
    rounding: f32,
    stroke_width: f32,
    layout: KeyTextsLayout,
    key: rdev::Key,
    press_times: u32,
    hue: f32,
    rotation: f32, // rotation angle in radians
    offset: Vec2,  // position offset for alignment after rotation
    dark_mode: bool,
    font_family: FontFamily,
    font_size: f32,
    bold: bool,
    // Color cache
    cached_color: Option<Color32>,
    cached_stroke_color: Option<Color32>,
    cached_press_times: u32,
    cached_average_times: u32,
    // Font cache
    cached_font_id: Option<egui::FontId>,
}

impl KeyBox {
    pub fn new(
        size: Vec2,
        texts: KeyTextsLayout,
        key: rdev::Key,
        press_times: u32,
        hue: f32,
        dark_mode: bool,
        font_family: FontFamily,
        font_size: f32,
        bold: bool,
    ) -> KeyBox {
        Self {
            size,
            rounding: 5.0,
            stroke_width: 2.0,
            layout: texts,
            key,
            press_times,
            hue,
            rotation: 0.0,
            offset: Vec2::ZERO,
            dark_mode,
            font_family,
            font_size,
            bold,
            cached_color: None,
            cached_stroke_color: None,
            cached_press_times: 0,
            cached_average_times: 0,
            cached_font_id: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_rotation_and_offset(mut self, rotation: f32, offset: Vec2) -> Self {
        self.rotation = rotation;
        self.offset = offset;
        self
    }

    #[allow(dead_code)]
    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }
}
impl KeyBox {
    pub fn ui(&mut self, ui: &mut Ui, average_times: u32) {
        let (rect, _resp) = ui.allocate_exact_size(self.size, Sense::hover());

        // Only recalculate colors if press_times or average_times changed
        let needs_color_update = self.cached_color.is_none()
            || self.cached_press_times != self.press_times
            || self.cached_average_times != average_times;

        let (filled_color, stroke_color) = if needs_color_update {
            let normalized_times = self.press_times * 32 / average_times;
            let color = get_color(self.hue, normalized_times, self.dark_mode);
            let stroke = get_strike_color(color, self.dark_mode);

            // Update cache
            self.cached_color = Some(color);
            self.cached_stroke_color = Some(stroke);
            self.cached_press_times = self.press_times;
            self.cached_average_times = average_times;

            (color, stroke)
        } else {
            // Use cached colors
            (self.cached_color.unwrap(), self.cached_stroke_color.unwrap())
        };

        // Text color based on theme
        let text_color = if self.dark_mode {
            Color32::from_rgb(100, 100, 100)  // Darker gray for dark mode
        } else {
            Color32::from_rgb(32, 5, 64)  // Dark purple for light mode
        };

        // Get or create cached font ID
        let font_id = if let Some(ref cached) = self.cached_font_id {
            cached.clone()
        } else {
            let font_id = if self.bold {
                // For bold text, increase the font size slightly
                match self.font_family {
                    FontFamily::Monospace => egui::FontId::monospace(self.font_size * 1.1),
                    FontFamily::Proportional => egui::FontId::proportional(self.font_size * 1.1),
                }
            } else {
                match self.font_family {
                    FontFamily::Monospace => egui::FontId::monospace(self.font_size),
                    FontFamily::Proportional => egui::FontId::proportional(self.font_size),
                }
            };
            self.cached_font_id = Some(font_id.clone());
            font_id
        };

        if self.rotation != 0.0 {
            // Draw rotated key using transform
            let center = rect.center() + self.offset;
            let rotation = Rot2::from_angle(self.rotation);
            let half_size = self.size * 0.5;

            // Create rounded rectangle mesh with proper rounding
            let mut mesh = egui::Mesh::default();

            // Number of segments per corner for rounded corners
            let corner_segments = 8;
            let rounding = self.rounding.min(half_size.x.min(half_size.y));

            // Build rounded rectangle vertices in order
            let mut vertices = Vec::new();

            // Corner centers (inset by rounding radius)
            let corner_centers = [
                egui::Vec2::new(-half_size.x + rounding, -half_size.y + rounding), // top-left
                egui::Vec2::new(half_size.x - rounding, -half_size.y + rounding),  // top-right
                egui::Vec2::new(half_size.x - rounding, half_size.y - rounding),   // bottom-right
                egui::Vec2::new(-half_size.x + rounding, half_size.y - rounding),  // bottom-left
            ];

            // Generate vertices for each corner arc (clockwise from each corner's start angle)
            // Top-left corner: from 180° to 270°
            for j in 0..=corner_segments {
                let t = j as f32 / corner_segments as f32;
                let angle = std::f32::consts::PI + t * std::f32::consts::PI * 0.5;
                let offset = egui::Vec2::new(angle.cos() * rounding, angle.sin() * rounding);
                let local_pos = corner_centers[0] + offset;
                let rotated_pos = center + rotation * local_pos;
                vertices.push(rotated_pos);
            }

            // Top-right corner: from 270° to 0°
            for j in 0..=corner_segments {
                let t = j as f32 / corner_segments as f32;
                let angle = std::f32::consts::PI * 1.5 + t * std::f32::consts::PI * 0.5;
                let offset = egui::Vec2::new(angle.cos() * rounding, angle.sin() * rounding);
                let local_pos = corner_centers[1] + offset;
                let rotated_pos = center + rotation * local_pos;
                vertices.push(rotated_pos);
            }

            // Bottom-right corner: from 0° to 90°
            for j in 0..=corner_segments {
                let t = j as f32 / corner_segments as f32;
                let angle = t * std::f32::consts::PI * 0.5;
                let offset = egui::Vec2::new(angle.cos() * rounding, angle.sin() * rounding);
                let local_pos = corner_centers[2] + offset;
                let rotated_pos = center + rotation * local_pos;
                vertices.push(rotated_pos);
            }

            // Bottom-left corner: from 90° to 180°
            for j in 0..=corner_segments {
                let t = j as f32 / corner_segments as f32;
                let angle = std::f32::consts::PI * 0.5 + t * std::f32::consts::PI * 0.5;
                let offset = egui::Vec2::new(angle.cos() * rounding, angle.sin() * rounding);
                let local_pos = corner_centers[3] + offset;
                let rotated_pos = center + rotation * local_pos;
                vertices.push(rotated_pos);
            }

            // Create mesh from vertices using triangle fan from center
            let num_vertices = vertices.len();
            for v in &vertices {
                mesh.colored_vertex(*v, filled_color);
            }
            mesh.colored_vertex(center, filled_color); // center vertex
            let center_idx = num_vertices as u32;

            // Create triangles
            for i in 0..num_vertices {
                let next_i = (i + 1) % num_vertices;
                mesh.add_triangle(i as u32, next_i as u32, center_idx);
            }

            ui.painter().add(egui::Shape::mesh(mesh));

            // Draw stroke for rotated rectangle with rounded corners
            for i in 0..vertices.len() {
                let next_i = (i + 1) % vertices.len();
                ui.painter().line_segment(
                    [vertices[i], vertices[next_i]],
                    Stroke {
                        width: self.stroke_width,
                        color: stroke_color,
                    },
                );
            }

            // Draw text with rotation using TextShape
            match &self.layout {
                KeyTextsLayout::TopBottom(top_bottom) => {
                    // Create galleys for text
                    let top_galley = ui.painter().layout_no_wrap(
                        top_bottom.0.clone(),
                        font_id.clone(),
                        text_color,
                    );
                    let bottom_galley = ui.painter().layout_no_wrap(
                        top_bottom.1.clone(),
                        font_id.clone(),
                        text_color,
                    );

                    // Calculate text positions in local space
                    let top_offset = egui::Vec2::new(0.0, -7.5);
                    let bottom_offset = egui::Vec2::new(0.0, 7.5);

                    // Rotate the offset vectors
                    let rotated_top_offset = rotation * top_offset;
                    let rotated_bottom_offset = rotation * bottom_offset;

                    // Calculate final positions (center of each text)
                    let top_center = center + rotated_top_offset;
                    let bottom_center = center + rotated_bottom_offset;

                    // Draw rotated text using TextShape
                    let top_pos = egui::Pos2::new(
                        top_center.x - top_galley.size().x * 0.5,
                        top_center.y - top_galley.size().y * 0.5
                    );
                    ui.painter().add(egui::epaint::TextShape::new(
                        top_pos,
                        top_galley,
                        text_color,
                    ).with_angle(self.rotation));

                    let bottom_pos = egui::Pos2::new(
                        bottom_center.x - bottom_galley.size().x * 0.5,
                        bottom_center.y - bottom_galley.size().y * 0.5
                    );
                    ui.painter().add(egui::epaint::TextShape::new(
                        bottom_pos,
                        bottom_galley,
                        text_color,
                    ).with_angle(self.rotation));
                }
                KeyTextsLayout::Center1(text) => {
                    let galley = ui.painter().layout_no_wrap(
                        text.clone(),
                        font_id.clone(),
                        text_color,
                    );

                    // Draw rotated text using TextShape
                    let text_pos = egui::Pos2::new(
                        center.x - galley.size().x * 0.5,
                        center.y - galley.size().y * 0.5
                    );
                    ui.painter().add(egui::epaint::TextShape::new(
                        text_pos,
                        galley,
                        text_color,
                    ).with_angle(self.rotation));
                }
            }
        } else {
            // Draw normal non-rotated key
            let draw_rect = rect.translate(self.offset);
            ui.painter().rect_filled(draw_rect, self.rounding, filled_color);
            match &self.layout {
                KeyTextsLayout::TopBottom(top_bottom) => {
                    // Use two separate text for better alignment
                    ui.painter().text(
                        draw_rect.center() + egui::Vec2::new(0.0, -7.5),
                        Align2::CENTER_CENTER,
                        &top_bottom.0,
                        font_id.clone(),
                        text_color,
                    );
                    ui.painter().text(
                        draw_rect.center() + egui::Vec2::new(0.0, 7.5),
                        Align2::CENTER_CENTER,
                        &top_bottom.1,
                        font_id.clone(),
                        text_color,
                    );
                }
                KeyTextsLayout::Center1(text) => {
                    ui.painter().text(
                        draw_rect.center(),
                        Align2::CENTER_CENTER,
                        text,
                        font_id.clone(),
                        text_color,
                    );
                }
            }

            ui.painter().rect_stroke(
                draw_rect,
                self.rounding,
                Stroke {
                    width: self.stroke_width,
                    color: stroke_color,
                },
                egui::epaint::StrokeKind::Middle,
            );
        }

        // Hover tooltip - all keys use pointer-following tooltips
        if let Some(pointer_pos) = ui.ctx().pointer_hover_pos() {
            let should_show_tooltip = if self.rotation != 0.0 {
                // For rotated keys, check if pointer is inside the rotated rectangle
                let center = rect.center() + self.offset;
                let half_size = self.size * 0.5;
                let rotation = Rot2::from_angle(self.rotation);

                // Transform pointer position to local space (inverse rotation)
                let local_pos = rotation.inverse() * (pointer_pos - center);

                // Check if local position is inside the rectangle
                local_pos.x.abs() <= half_size.x && local_pos.y.abs() <= half_size.y
            } else if self.offset != Vec2::ZERO {
                // For non-rotated keys with offset, check if pointer is inside the translated rectangle
                let draw_rect = rect.translate(self.offset);
                draw_rect.contains(pointer_pos)
            } else {
                // For keys with no rotation or offset, check the original rect
                rect.contains(pointer_pos)
            };

            if should_show_tooltip {
                // Position tooltip above the cursor (5 pixels above)
                let tooltip_pos = egui::Pos2::new(pointer_pos.x, pointer_pos.y - 5.0);

                // Use the new Tooltip API for egui 0.33
                egui::Area::new(egui::Id::new(format!("key_hover_{:?}", self.key)))
                    .pivot(egui::Align2::CENTER_BOTTOM)
                    .fixed_pos(tooltip_pos)
                    .order(egui::Order::Foreground)  // Use Foreground order so dropdowns (Tooltip order) appear above
                    .show(ui.ctx(), |ui| {
                        egui::Frame::popup(ui.style()).show(ui, |ui| {
                            // Display number with appropriate color for visibility
                            let tooltip_text_color = if self.dark_mode {
                                Color32::from_rgb(220, 220, 220)  // 暗黑模式：亮灰色
                            } else {
                                Color32::from_rgb(50, 50, 50)  // 浅色模式：深灰色
                            };
                            // Use wrap_mode to prevent text wrapping
                            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                            ui.label(RichText::new(format!("{}", self.press_times)).color(tooltip_text_color));
                        });
                    });
            }
        }
    }
}

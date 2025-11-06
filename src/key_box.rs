use egui::{Align2, Color32, RichText, Sense, Stroke, Ui, Vec2};

use crate::color::{get_color, get_strike_color};
use eframe::egui::emath::Rot2;
use std::sync::atomic::Ordering;
use crate::press_time_map::AVERAGE_TIMES;

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
}

impl KeyBox {
    pub fn new(
        size: Vec2,
        texts: KeyTextsLayout,
        key: rdev::Key,
        press_times: u32,
        hue: f32,
        dark_mode: bool,
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
        }
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_rotation_and_offset(mut self, rotation: f32, offset: Vec2) -> Self {
        self.rotation = rotation;
        self.offset = offset;
        self
    }

    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }
}
impl KeyBox {
    pub fn ui(&mut self, ui: &mut Ui) {
        let (rect, _resp) = ui.allocate_exact_size(self.size, Sense::hover());
        let filled_color = get_color(self.hue, self.press_times * 32 / (AVERAGE_TIMES.load(Ordering::Relaxed) as u32), self.dark_mode);

        // Text color based on theme
        let text_color = if self.dark_mode {
            Color32::from_rgb(100, 100, 100)  // Darker gray for dark mode
        } else {
            Color32::from_rgb(32, 5, 64)  // Dark purple for light mode
        };

        if self.rotation != 0.0 {
            // Draw rotated key with rounded corners using mesh
            let center = rect.center() + self.offset;
            let rotation = Rot2::from_angle(self.rotation);
            let half_size = self.size * 0.5;
            let corner_radius = self.rounding;
            let segments_per_corner = 8;

            // Generate all vertices for the rounded rectangle in local coordinates,
            // then rotate them around the center
            let mut vertices = Vec::new();

            // Helper to add corner arc vertices in local space
            let mut add_corner_vertices = |corner_local: egui::Vec2, start_angle: f32| {
                for i in 0..=segments_per_corner {
                    let angle = start_angle + (i as f32 / segments_per_corner as f32) * std::f32::consts::PI * 0.5;
                    let arc_offset = egui::Vec2::new(angle.cos(), angle.sin()) * corner_radius;
                    let local_point = corner_local + arc_offset;
                    // Rotate the local point and translate to world space
                    let world_point = center + rotation * local_point;
                    vertices.push(world_point);
                }
            };

            // Top-left corner (local coordinates)
            add_corner_vertices(
                egui::Vec2::new(-half_size.x + corner_radius, -half_size.y + corner_radius),
                std::f32::consts::PI
            );

            // Top-right corner
            add_corner_vertices(
                egui::Vec2::new(half_size.x - corner_radius, -half_size.y + corner_radius),
                -std::f32::consts::PI * 0.5
            );

            // Bottom-right corner
            add_corner_vertices(
                egui::Vec2::new(half_size.x - corner_radius, half_size.y - corner_radius),
                0.0
            );

            // Bottom-left corner
            add_corner_vertices(
                egui::Vec2::new(-half_size.x + corner_radius, half_size.y - corner_radius),
                std::f32::consts::PI * 0.5
            );

            // Create mesh with center vertex for fan triangulation
            let mut mesh = egui::Mesh::default();
            mesh.colored_vertex(center, filled_color);

            for v in &vertices {
                mesh.colored_vertex(*v, filled_color);
            }

            // Triangulate from center
            let vertex_count = vertices.len();
            for i in 0..vertex_count {
                let next_i = (i + 1) % vertex_count;
                mesh.add_triangle(0, (i + 1) as u32, (next_i + 1) as u32);
            }

            ui.painter().add(egui::Shape::mesh(mesh));

            // Draw stroke (outline) - use the same vertices for consistent shape
            let stroke_color = get_strike_color(filled_color, self.dark_mode);
            for i in 0..vertices.len() {
                let next_i = (i + 1) % vertices.len();
                ui.painter().line_segment(
                    [vertices[i], vertices[next_i]],
                    egui::Stroke::new(self.stroke_width, stroke_color)
                );
            }

            // Draw text at center
            match &self.layout {
                KeyTextsLayout::TopBottom(top_bottom) => {
                    ui.painter().text(
                        center,
                        Align2::CENTER_BOTTOM,
                        top_bottom.0.clone(),
                        egui::FontId::monospace(13.),
                        text_color,
                    );
                    ui.painter().text(
                        center,
                        Align2::CENTER_TOP,
                        top_bottom.1.clone(),
                        egui::FontId::monospace(13.),
                        text_color,
                    );
                }
                KeyTextsLayout::Center1(text) => {
                    ui.painter().text(
                        center,
                        Align2::CENTER_CENTER,
                        text,
                        egui::FontId::monospace(13.),
                        text_color,
                    );
                }
            }
        } else {
            // Draw normal non-rotated key
            let draw_rect = rect.translate(self.offset);
            ui.painter().rect_filled(draw_rect, self.rounding, filled_color);
            match &self.layout {
                KeyTextsLayout::TopBottom(top_bottom) => {
                    ui.painter().text(
                        draw_rect.center(),
                        Align2::CENTER_BOTTOM,
                        top_bottom.0.clone(),
                        egui::FontId::monospace(13.),
                        text_color,
                    );
                    ui.painter().text(
                        draw_rect.center(),
                        Align2::CENTER_TOP,
                        top_bottom.1.clone(),
                        egui::FontId::monospace(13.),
                        text_color,
                    );
                }
                KeyTextsLayout::Center1(text) => {
                    ui.painter().text(
                        draw_rect.center(),
                        Align2::CENTER_CENTER,
                        text,
                        egui::FontId::monospace(13.),
                        text_color,
                    );
                }
            }

            ui.painter().rect_stroke(
                draw_rect,
                self.rounding,
                Stroke {
                    width: self.stroke_width,
                    color: get_strike_color(filled_color, self.dark_mode),
                },
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
                egui::show_tooltip_at_pointer(
                    ui.ctx(),
                    egui::Id::new(format!("key_hover_{:?}", self.key)),
                    |ui| {
                        // Remove background and border
                        ui.visuals_mut().window_fill = Color32::TRANSPARENT;
                        ui.visuals_mut().window_stroke = egui::Stroke::NONE;

                        // Different shadow for dark mode and light mode
                        ui.style_mut().visuals.window_shadow = if self.dark_mode {
                            // Dark mode shadow
                            egui::epaint::Shadow {
                                extrusion: 8.0,
                                color: Color32::from_black_alpha(100),
                            }
                        } else {
                            // Light mode shadow
                            egui::epaint::Shadow {
                                extrusion: 8.0,
                                color: Color32::from_black_alpha(40),
                            }
                        };

                        // Display number with appropriate color for visibility
                        let tooltip_text_color = if self.dark_mode {
                            Color32::from_rgb(220, 220, 220)  // 暗黑模式：亮灰色
                        } else {
                            Color32::from_rgb(50, 50, 50)  // 浅色模式：深灰色
                        };
                        ui.label(RichText::new(format!("{}", self.press_times)).color(tooltip_text_color));
                    }
                );
            }
        }
    }
}

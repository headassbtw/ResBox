use egui::{vec2, Align2, Color32, Pos2, Rect, Rounding, Stroke};

use crate::{widgets::page_header::page_header, TemplateApp, SESSION_CACHE};

impl TemplateApp {
    pub fn sessions_page(&mut self, ui: &mut egui::Ui) {
        page_header(ui, "Sessions", &self.username());

        let sessions = SESSION_CACHE.lock();

        egui::containers::ScrollArea::vertical().scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden).show(ui, |ui| {
            for (id, session) in sessions.iter() {
                
                let (response, painter) = ui.allocate_painter(vec2(ui.available_width(), 104.0), egui::Sense::click());

                if !ui.is_rect_visible(response.rect) { continue; }

                if response.is_pointer_button_down_on() {
                    painter.rect_filled(response.rect, Rounding::same(0.0), ui.style().visuals.widgets.active.bg_fill);
                } else if response.hovered() {
                    painter.rect_filled(response.rect, Rounding::same(0.0), ui.style().visuals.widgets.hovered.bg_fill);
                }

                // 72 diameter image

                let img_rect = Rect {
                    min: Pos2 { x: ui.spacing().window_margin.left,        y: response.rect.min.y + 16.0 },
                    max: Pos2 { x: ui.spacing().window_margin.left + 72.0, y: response.rect.max.y - 16.0 }
                };

                let text_pos = img_rect.max + vec2(16.0, -36.0);

                painter.text(text_pos - vec2(0.0, 5.0), Align2::LEFT_BOTTOM, &session.name, egui::FontId::proportional(24.0), Color32::WHITE);
                painter.text(text_pos + vec2(0.0, 5.0), Align2::LEFT_TOP, format!("{}/{} users", &session.joined_users, &session.max_users), egui::FontId::proportional(18.0), Color32::GRAY);

                painter.circle(img_rect.center(), img_rect.width() / 2.0, Color32::RED, Stroke::NONE);
            }
        });
    }
}
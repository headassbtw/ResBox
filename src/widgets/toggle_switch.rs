use egui::{vec2, Color32, Layout, Rounding, Sense, Stroke, Vec2};

use crate::{ACCENT, HOVER_COL};

pub fn toggle_ui(ui: &mut egui::Ui, label: &str, on: &mut bool) -> egui::Response {
    let mut big_rect = ui.available_rect_before_wrap();
    big_rect.max.y = big_rect.min.y + ui.style().spacing.interact_size.y;
    let (mut rect, mut big_response) = ui.allocate_exact_size(big_rect.size(), egui::Sense::click());
    rect.min.x = ui.style().spacing.window_margin.left;
    let desired_size = 24.0 * egui::vec2(2.0, 1.0);
    
    if big_response.is_pointer_button_down_on() {
        ui.painter().rect_filled(big_rect, Rounding::same(0.0), ui.style().visuals.widgets.active.bg_fill);
    } else if big_response.hovered() {
        ui.painter().rect_filled(big_rect, Rounding::same(0.0), ui.style().visuals.widgets.hovered.bg_fill);
    }

    ui.allocate_ui_at_rect(rect, |ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| { // segoe is a bit problematic.
                ui.allocate_space(vec2(0.0, 14.0));
                ui.add(egui::Label::new(egui::RichText::new(label).size(24.0).color(Color32::WHITE).line_height(None)).selectable(false));

            });
            
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                ui.allocate_space(vec2(ui.style().spacing.window_margin.right, 0.0));
                
                let (_, rect) = ui.allocate_space(desired_size);
                
                if big_response.clicked() {
                    *on = !*on;
                    big_response.mark_changed();
                }

                big_response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, ""));

                if ui.is_rect_visible(rect) {
                    let how_on = ui.ctx().animate_bool(big_response.id, *on);
                    let radius = 0.5 * rect.height();
                    ui.painter()
                    .rect(rect, radius, Color32::TRANSPARENT, Stroke::new(2.0, Color32::WHITE));
                    // Paint the circle, animating it from left to right with `how_on`:
                    let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
                    let center = egui::pos2(circle_x, rect.center().y);
                    ui.painter()
                        .circle(center, 0.5 * radius, Color32::WHITE, Stroke::NONE);
            }
        });
        
        // All done! Return the interaction response so the user can check what happened
        // (hovered, clicked, ...) and maybe show a tooltip:
        });
    }).response
}
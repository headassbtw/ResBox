use egui::{pos2, vec2, Align2, Color32, FontId, Layout, Pos2, Rect, Rounding, Sense, Stroke, Vec2};

use crate::{ACCENT, HOVER_COL, TEXT_COL};

pub fn toggle_ui(ui: &mut egui::Ui, label: &str, on: &mut bool) -> egui::Response {
    let (mut response, painter) = ui.allocate_painter(vec2(ui.available_width(),ui.style().spacing.interact_size.y), egui::Sense::click());
    
    if !ui.is_rect_visible(response.rect) { return response; }
    
    //TODO: selected (idk it's a new egui thing)
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, false, label));

    if response.is_pointer_button_down_on() {
        painter.rect_filled(response.rect, Rounding::same(0.0), ui.style().visuals.widgets.active.bg_fill);
    } else if response.hovered() {
        painter.rect_filled(response.rect, Rounding::same(0.0), ui.style().visuals.widgets.hovered.bg_fill);
    }

    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }

    let radius = (response.rect.height() / 2.0) - 18.0;

    let switch_rect = Rect {
        min: Pos2 { x: response.rect.max.x - (24.0 + (radius * 4.0)), y: response.rect.min.y + 18.0 },
        max: Pos2 { x: response.rect.max.x - 24.0, y: response.rect.max.y - 18.0 }
    };

    let how_on = ui.ctx().animate_bool(response.id, *on);
    
    painter.rect(switch_rect, radius, Color32::TRANSPARENT, Stroke::new(2.0, TEXT_COL));
    let circle_x = egui::lerp((switch_rect.left() + radius)..=(switch_rect.right() - radius), how_on);
    let center = egui::pos2(circle_x, switch_rect.center().y);
    painter.circle(center, 0.5 * radius, TEXT_COL, Stroke::NONE);

    painter.text(pos2(response.rect.left() + ui.style().spacing.window_margin.left, response.rect.center().y - 5.0), Align2::LEFT_CENTER, label, FontId::proportional(24.0), TEXT_COL);

    response
}
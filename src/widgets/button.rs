use egui::{pos2, vec2, Align2, Color32, FontId, Rounding, Stroke, Widget};

const ACCENT: Color32 = Color32::from_rgb(220, 53, 60);
const HOVER_COL: Color32 = Color32::from_gray(51);

pub fn metro_button(ui: &mut egui::Ui, label: &str, icon: Option<(&str, f32)>) -> egui::Response {
    let size = vec2(ui.available_size_before_wrap().x, ui.style().spacing.interact_size.y);
    //let guh = ui.allocate_response(size, egui::Sense::click());
    let (res, painter) = ui.allocate_painter(size, egui::Sense::click());
    if res.is_pointer_button_down_on() {
        ui.painter().rect_filled(res.rect, Rounding::same(0.0), ui.style().visuals.widgets.active.bg_fill);
    } else if res.hovered() {
        ui.painter().rect_filled(res.rect, Rounding::same(0.0), ui.style().visuals.widgets.hovered.bg_fill);
    }
    let mut place_rect = res.rect.clone();
    place_rect.min.x += ui.style().spacing.window_margin.left;
    place_rect.min.y += 14.0;
    place_rect.max.y -= 14.0;
    place_rect.max.x -= ui.style().spacing.window_margin.right;


    if let Some((icon, icon_width)) = icon {
        let mut icon_rect = place_rect.clone();
        place_rect.min.x += icon_width + 16.0;
        icon_rect.min.y += 8.0;
        icon_rect.max.y += 2.0;
        icon_rect.max.x = icon_rect.min.x + icon_width;
        //ui.painter().rect_filled(icon_rect, Rounding::same(0.0), Color32::RED);
        
        ui.painter().text(icon_rect.center(), Align2::CENTER_CENTER, icon, FontId::proportional(24.0), Color32::WHITE);
    }

    let galley = ui.painter().layout(label.to_owned(), FontId::proportional(24.0), Color32::WHITE, place_rect.width());
    let fuck = pos2(place_rect.min.x, ((place_rect.height() - 24.0) / 2.0) + place_rect.min.y);
    //ui.painter().rect_filled(galley.rect.translate(fuck.to_vec2()), Rounding::same(0.0), Color32::from_white_alpha(128));
    //ui.painter().rect_filled(galley.rect.translate(fuck.to_vec2()).translate(vec2(0.0, -3.0)), Rounding::same(0.0), Color32::from_white_alpha(128));
    ui.painter().galley(fuck - vec2(0.0, 3.0), galley, Color32::WHITE);

    res
}
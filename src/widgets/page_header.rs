use egui::{pos2, vec2, Align2, Color32, FontId, Response};

use crate::{SUBHEADER_COL, TEXT_COL};

pub fn page_header(ui: &mut egui::Ui, header: &str, subheader: &str) -> Response{
    ui.style_mut().spacing.item_spacing.y = 10.0;
        
    let (response, painter) = ui.allocate_painter(vec2(ui.available_width(), 90.0), egui::Sense::focusable_noninteractive());

    let header_pos = pos2(response.rect.min.x + ui.style().spacing.window_margin.left + 10.0, response.rect.min.y);
    let subheader_pos = pos2(response.rect.min.x + ui.style().spacing.window_margin.left + 10.0, response.rect.min.y + 40.0);

    painter.text(header_pos, Align2::LEFT_TOP, header, FontId::proportional(30.0), TEXT_COL);
    painter.text(subheader_pos, Align2::LEFT_TOP, subheader, FontId::proportional(20.0), SUBHEADER_COL);
    response
}
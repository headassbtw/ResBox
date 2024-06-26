use egui::epaint::{emath::lerp, vec2, Color32, Pos2, Rect, Shape, Stroke};

use egui::load::SizedTexture;
use egui::{pos2, show_tooltip, Align2, FontFamily, FontId, ImageSource, Response, Rounding, Sense, Ui, Widget, WidgetInfo, WidgetType};

use crate::image::LoadableImage;

use super::segoe_boot_spinner::SegoeBootSpinner;

pub fn loadable_image(ui: &mut egui::Ui, img: &LoadableImage, rect: egui::Rect, unloaded_text: &str, fill_color: Color32, radius: f32, allocate: bool) {
    if allocate {
        let _resp = ui.allocate_rect(rect, egui::Sense::click());
    }
    match img {
        LoadableImage::Unloaded => {
            ui.painter().rect_filled(rect, Rounding::same(radius), fill_color);
            ui.painter().text(rect.center(), Align2::CENTER_CENTER, unloaded_text, FontId::proportional(rect.width() / 2.0), Color32::WHITE);
        },
        LoadableImage::Loading => {
            ui.painter().rect_filled(rect, Rounding::same(radius), fill_color);
            SegoeBootSpinner::new().size(radius).paint_at(ui, rect);
        },
        LoadableImage::Loaded(img) =>  {
            let img = egui::Image::new(ImageSource::Texture(SizedTexture { id: *img, size: rect.size() }));
            img.fit_to_exact_size(rect.size()).rounding(Rounding::same(radius)).paint_at(ui, rect);
        },
    }
}
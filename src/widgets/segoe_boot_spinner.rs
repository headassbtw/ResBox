use egui::epaint::{emath::lerp, vec2, Color32, Pos2, Rect, Shape, Stroke};

use egui::{Align2, FontFamily, FontId, Response, Sense, Ui, Widget, WidgetInfo, WidgetType};

/// A spinner widget used to indicate loading.
///
/// See also: [`crate::ProgressBar`].
#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
#[derive(Default)]
pub struct SegoeBootSpinner {
    /// Uses the style's `interact_size` if `None`.
    size: Option<f32>,
    color: Option<Color32>,
}

impl SegoeBootSpinner {
    /// Create a new spinner that uses the style's `interact_size` unless changed.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the spinner's size. The size sets both the height and width, as the spinner is always
    /// square. If the size isn't set explicitly, the active style's `interact_size` is used.
    #[inline]
    pub fn size(mut self, size: f32) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets the spinner's color.
    #[inline]
    pub fn color(mut self, color: impl Into<Color32>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Paint the spinner in the given rectangle.
    pub fn paint_at(&self, ui: &Ui, rect: Rect) {
        // first frame: 0x297 U-E052
        // last frame: 0x310 U-E0C6
        if ui.is_rect_visible(rect) {
            ui.ctx().request_repaint(); // because it is animated

            let color = self
                .color
                .unwrap_or_else(|| ui.visuals().strong_text_color());
            let time = (ui.input(|i| i.time) + 4.0) / 4.0;
            let time = time - time.floor();
            let interp: u32 = (time * 120.0).floor() as u32;
            let fuck =  char::from_u32((0xE052 + interp).into()).unwrap();
            let size = self.size.unwrap_or_else(|| rect.height() * 0.8);
            ui.painter().text(rect.center(), Align2::CENTER_CENTER, fuck, FontId { size, family: FontFamily::Name("Segoe Boot".into()) }, color);
        }
    }
}

impl Widget for SegoeBootSpinner {
    fn ui(self, ui: &mut Ui) -> Response {
        let size = self
            .size
            .unwrap_or_else(|| ui.style().spacing.interact_size.y);
        let (rect, response) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
        response.widget_info(|| WidgetInfo::new(WidgetType::ProgressIndicator));
        self.paint_at(ui, rect);

        response
    }
}

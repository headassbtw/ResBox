use egui::{pos2, vec2, Align2, Color32, FontId, RichText, Rounding};

use crate::{widgets::{button::metro_button, loadable_image::loadable_image, page_header::page_header}, FrontendNotificationIcon, TemplateApp, ACCENT, SUBHEADER_COL};

impl TemplateApp {
    pub fn notifications_page(&mut self, ui: &mut egui::Ui) {
        
        page_header(ui, "Notifications", &self.username());

        ui.style_mut().spacing.item_spacing.y = 4.0;
        
        ui.style_mut().spacing.interact_size.y = 60.0;
        if metro_button(ui, "Clear all", None).clicked() {
            self.notifications.clear();
        }
        ui.style_mut().spacing.interact_size.y = 94.0;

        egui::containers::ScrollArea::vertical().scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden).show(ui, |ui| {
        

            for i in (0)..(self.notifications.len() as usize) {
                let assets = self.notifications.get((self.notifications.len() as usize-1) - i).unwrap();
                let mut rect = ui.available_rect_before_wrap();
                rect.max.y = rect.min.y + ui.style().spacing.interact_size.y;
                
                ui.painter().rect_filled(rect, Rounding::same(0.0), ui.style().visuals.widgets.hovered.bg_fill);

                ui.horizontal(|notif: &mut egui::Ui| {
                    notif.allocate_space(vec2(72.0 - notif.cursor().left(),0.0));
                    
                    let mut icon_rect = notif.available_rect_before_wrap().clone();
                    let mut header_rect = icon_rect.clone();
                    let mut subtext_rect = header_rect.clone();

                    
                    icon_rect.max.x = icon_rect.min.x + 74.0;
                    icon_rect.min.y += 10.0;
                    icon_rect.max.y -= 10.0;
                    match &assets.icon {
                        FrontendNotificationIcon::SegoeIcon(text) => {
                            notif.painter().rect_filled(icon_rect, Rounding::same(0.0), ACCENT);
                            notif.put(icon_rect, egui::Label::new(egui::RichText::new(text).color(Color32::WHITE).size(60.0)).selectable(false));
                        },
                        FrontendNotificationIcon::LoadableImage(img) => {
                            loadable_image(notif, img, icon_rect, "", ACCENT, 0.0, true);
                        },
                    }
                    
                    
                    notif.allocate_space(vec2(8.0, 0.0));

                    
                    notif.vertical(|texts| {
                        texts.allocate_space(vec2(0.0, 12.0));
                        texts.style_mut().spacing.item_spacing.y = 8.0;
                        texts.add(egui::Label::new(egui::RichText::new(&assets.text).color(Color32::WHITE).size(24.0)).selectable(false));
                        texts.add(egui::Label::new(egui::RichText::new(&assets.sub).color(SUBHEADER_COL).size(24.0)).selectable(false));
                    });

                });
            }    
        });
    }

}
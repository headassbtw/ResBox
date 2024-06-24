use std::cmp::Ordering;

use chrono::{DateTime, Datelike};
use egui::{pos2, text::{LayoutJob, LayoutSection, TextWrapping}, vec2, Align2, Color32, FontId, RichText, Rounding, TextFormat};

use crate::{api::client::{Message, MessageType, ResDateTime}, widgets::{button::metro_button, user_info::{draw_user_pic_at, UserInfoVariant}}, FrontendPage, TemplateApp, CONTACTS_LIST, HOVER_COL, MESSAGE_CACHE};

impl TemplateApp {
    pub fn messages_page(&mut self, ui: &mut egui::Ui) {
        ui.style_mut().spacing.item_spacing.y = 10.0;
        ui.horizontal(|ui| {
            ui.allocate_space(vec2(ui.style().spacing.window_margin.left, 0.0));
            ui.vertical(|ui| {
                ui.label(RichText::new("Messages").size(30.0).color(Color32::WHITE));
                ui.label(RichText::new(if let Some(you) = &self.you { you.username.clone() } else { "".to_string() }).size(20.0));
                ui.allocate_space(vec2(0.0,20.0));
            });
        });
        ui.style_mut().spacing.item_spacing.y = 4.0;
        ui.style_mut().spacing.interact_size.y = 60.0;  
        if metro_button(ui, "New message", Some(("", 72.0))).clicked() {

        }

        egui::containers::ScrollArea::vertical().scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden).show(ui, |ui| {
            let guh = MESSAGE_CACHE.lock();
            let contacts = CONTACTS_LIST.lock();
            let mut hash_vec: Vec<(&String, &Vec<Message>)> = guh.iter().collect();

            ui.style_mut().spacing.item_spacing.y = 4.0;

            hash_vec.sort_by(|a, b| {
                let (c,d) = *a;
                let (e,f) = *b;

                if let Some(first) = d.last() {
                    if let Some(first_cmp) = f.last() { 
                        first_cmp.last_update_time.0.cmp(&first.last_update_time.0)
                    } else {
                        Ordering::Less
                    }
                } else {
                    Ordering::Less
                }
            });

            for (id, vec) in hash_vec {
                if let Some(last) = vec.last() {
                    let mut rect = ui.cursor().clone();
                    rect.max.y = rect.min.y + 104.0;

                    let resp = ui.allocate_rect(rect, egui::Sense::click());
                    if resp.is_pointer_button_down_on() {
                        ui.painter().rect_filled(rect, Rounding::same(0.0), ui.style().visuals.widgets.active.bg_fill);
                    } else if resp.hovered() {
                        ui.painter().rect_filled(rect, Rounding::same(0.0), ui.style().visuals.widgets.hovered.bg_fill);
                    }

                    if resp.clicked() {
                        self.current_page = FrontendPage::ConversationPage(id.clone());
                    }

                    let mut img_rect = rect.clone();
                    img_rect.min.x = 72.0;
                    img_rect.min.y += 16.0;
                    img_rect.max.y -= 16.0;
                    img_rect.max.x = img_rect.min.x + 72.0;

                    let mut bound_rect = img_rect.clone();
                    bound_rect.min.x  = img_rect.max.x + 16.0;
                    bound_rect.max.x  = rect.max.x - ui.style().spacing.window_margin.right;

                    let pfp_draw_variant = {
                        if let Some(profile) = contacts.get(id) {
                            UserInfoVariant::Contact(profile)
                        } else {
                            UserInfoVariant::Uncached(id)
                        }
                    };

                    let msg = match last.message_type {
                        MessageType::Text => &last.content,
                        MessageType::Object => "Shared content",
                        MessageType::Sound => "Shared content",
                        MessageType::SessionInvite => "[Session Invite]",
                    };

                    draw_user_pic_at(ui, img_rect, &mut self.image_cache, pfp_draw_variant);

                    let left_center = img_rect.center() + vec2(52.0, 0.0);

                    
                    

                    let u_galley = ui.painter().layout(if let Some(name) = contacts.get(id) { name.contact_username.clone() } else { id.to_string() }, FontId::proportional(24.0), Color32::WHITE, bound_rect.width());
                    let u_rect = Align2::LEFT_BOTTOM.anchor_size(left_center, u_galley.size());
                    ui.painter().galley(u_rect.min - vec2(0.0, 4.0), u_galley, Color32::WHITE);

                    let date_pos = pos2(bound_rect.max.x, u_rect.max.y - 4.0);

                    let date = &last.send_time.0;
                    ui.painter().text(date_pos, Align2::RIGHT_BOTTOM, format!("{}/{}/{}", date.month(), date.day(), date.year()), FontId::proportional(18.0), Color32::GRAY);

                    let mut message_job = LayoutJob::simple_singleline(msg.to_owned(), FontId::proportional(20.0), Color32::GRAY);
                    message_job.wrap = TextWrapping::truncate_at_width(bound_rect.width());

                    let m_galley = ui.painter().layout_job(message_job);
                    let m_rect = Align2::LEFT_TOP.anchor_size(left_center, m_galley.size());
                    ui.painter().galley(m_rect.min + vec2(0.0, 4.0), m_galley, Color32::GRAY);

                    //ui.painter().text(left_center, Align2::LEFT_BOTTOM, "USERNAME", FontId::proportional(24.0), Color32::WHITE);
                    //ui.painter().text(left_center, Align2::LEFT_TOP, "MESSAGE", FontId::proportional(20.0), Color32::GRAY);
                }
            }

        });
    }
}
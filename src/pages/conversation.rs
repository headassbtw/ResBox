use chrono::{DateTime, Datelike, Utc};
use egui::{text::{LayoutJob, TextWrapping}, vec2, Align, Align2, Color32, FontId, Layout, Margin, Mesh, RichText, Rounding, Shape, Stroke, TextEdit};

use crate::{api::client::{Contact, MessageType, ResDateTime}, backend::thread::UiToReso, disgusting_bullshit, widgets::{button::metro_button, user_info::{user_info_widget, UserInfoVariant}}, TemplateApp, CONTACTS_LIST, MESSAGE_CACHE};

impl TemplateApp {
    pub fn conversation_page(&mut self, ui: &mut egui::Ui, id: String) {
        {
            let contacts = CONTACTS_LIST.lock();
            if let Some(contact) = contacts.get(&id) {
                user_info_widget(ui, &mut self.image_cache, UserInfoVariant::Contact(contact));
            } else {
                ui.style_mut().spacing.item_spacing.y = 10.0;
                ui.horizontal(|ui| {
                    ui.allocate_space(vec2(ui.style().spacing.window_margin.left, 0.0));
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Message Page").size(30.0).color(Color32::WHITE));
                        ui.label(RichText::new("Oh fuck").size(20.0));
                        ui.allocate_space(vec2(0.0,20.0));
                    });
                });
            }
        }

        ui.with_layout(Layout::bottom_up(egui::Align::Min), |ui| {
            let (bottom_rect, bottom_resp) = ui.allocate_exact_size(vec2(ui.available_width(), 116.0), egui::Sense::click());
            ui.painter().rect_filled(bottom_rect, Rounding::same(0.0), Color32::from_gray(11));
            
            
            ui.with_layout(Layout::top_down(egui::Align::Min), |messages| {
                egui::containers::ScrollArea::vertical().scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden).show(messages, |ui| {
                    ui.style_mut().spacing.item_spacing.y = 24.0;
                    let max_text_width = ui.available_width() - 182.0; // 72 left, 40 right, 74 because microsoft felt like it
                    ui.allocate_space(vec2(ui.available_width(), 0.0));

                    // 72 left
                    // 40 right
                    // 330 text max width (width - 74?)
                    // boxes expand horizontally, no apparent minimum?
                    // messages should abide by this themselves, other text is fully centered
                    {
                        let messages = MESSAGE_CACHE.lock();
                        if let Some(msgs) = messages.get(&id) {
                            let mut date: Option<&ResDateTime> = None;
                            for message in msgs {
                                let cur_date = &message.send_time;
                                let should_draw_date: bool = if let Some(d) = date {
                                    (d.0.timestamp() + 86400) < cur_date.0.timestamp() // more than a day later
                                } else {
                                    true
                                };
                                date = Some(&cur_date);
                                
                                if should_draw_date {
                                    ui.with_layout(Layout::top_down(egui::Align::Center), |new_date| {
                                        let date = &date.unwrap();
                                        
                                        new_date.label(format!("{}/{}/{}", date.0.month(), date.0.day(), date.0.year()));
                                    });
                                }
                                let text = match message.message_type {
                                    MessageType::Text => &message.content,
                                    MessageType::Object => "[Object]",
                                    MessageType::Sound => "[Audio]",
                                    MessageType::SessionInvite => "[Session Invite]",
                                };

                                let galley = ui.painter().layout(text.to_string(), FontId::proportional(24.0), Color32::WHITE, max_text_width);



                                // 16px all sides padding
                                let (mut paint_rect, msg_resp) = ui.allocate_exact_size(vec2(galley.rect.max.x + 32.0, galley.rect.max.y + 56.0), egui::Sense::click());
                                paint_rect.max.y -= 14.0;
                                let paint_rect = if self.is_you(&message.sender_id) {
                                    paint_rect.translate(vec2(ui.available_width() - (40.0 +  paint_rect.width()), 0.0))
                                } else {
                                    paint_rect.translate(vec2(72.0, 0.0))
                                };
                                if self.is_you(&message.sender_id) {
                                    ui.painter().rect_filled(paint_rect, Rounding::same(0.0), ui.style().visuals.widgets.active.bg_fill);
                                    let mut mesh = Mesh::default();
                                    mesh.colored_vertex(paint_rect.right_bottom(), ui.style().visuals.widgets.active.bg_fill);
                                    mesh.colored_vertex(paint_rect.right_bottom() + vec2(0.0, 14.0), ui.style().visuals.widgets.active.bg_fill);
                                    mesh.colored_vertex(paint_rect.right_bottom() + vec2(-14.0, 0.0), ui.style().visuals.widgets.active.bg_fill);
                                    mesh.add_triangle(0, 1, 2);
                                    ui.painter().add(Shape::mesh(mesh));
                                } else {
                                    //ui.painter().rect(paint_rect, Rounding::same(0.0), Color32::TRANSPARENT, Stroke::new(2.0, Color32::RED));
                                    ui.painter().line_segment([paint_rect.min, paint_rect.min + vec2(paint_rect.width(), 0.0)], Stroke::new(2.0, Color32::GRAY)); // top
                                    ui.painter().line_segment([paint_rect.max, paint_rect.min + vec2(paint_rect.width(), 0.0)], Stroke::new(2.0, Color32::GRAY)); // right
                                    ui.painter().line_segment([paint_rect.min, paint_rect.min + vec2(0.0, paint_rect.height() + 14.0)], Stroke::new(2.0, Color32::GRAY)); // left
                                    ui.painter().line_segment([paint_rect.max, paint_rect.max - vec2(paint_rect.width() - 14.0, 0.0)], Stroke::new(2.0, Color32::GRAY)); // bottom
                                    ui.painter().line_segment([paint_rect.min + vec2(0.0, paint_rect.height() + 14.0), paint_rect.max - vec2(paint_rect.width() - 14.0, 0.0)], Stroke::new(2.0, Color32::GRAY)); // diagonal
                                }

                                ui.painter().galley(paint_rect.min + vec2(16.0, 16.0), galley, Color32::GREEN);
                            }
                        } else {
                            let header = "Go ahead, say hi";
                            let mut job = LayoutJob::simple_singleline(header.to_string(), FontId::proportional(32.0), Color32::WHITE);
                            job.halign = Align::Center;
                            let galley = ui.painter().layout_job(job);
                            ui.painter().galley(ui.available_rect_before_wrap().center() - vec2(0.0, 38.0), galley, Color32::WHITE);
                            
                            let mut job = LayoutJob::simple_singleline(format!("There is no current conversation between you and {}, try sending them a message.", ""), FontId::proportional(24.0), Color32::GRAY);
                            job.wrap = TextWrapping::default();
                            job.wrap.max_width = ui.available_width() - 60.0;

                            let galley = ui.painter().layout_job(job);
                            let rect = Align2::CENTER_TOP.anchor_size(ui.available_rect_before_wrap().center(), galley.size());
                            ui.painter().galley(rect.min, galley, Color32::GRAY);

                            ui.painter().text(ui.available_rect_before_wrap().center() - vec2(0.0, 68.0), Align2::CENTER_CENTER, "", FontId::monospace(96.0), Color32::WHITE);
                        }
                    }
                });
            });

            ui.allocate_ui_at_rect(bottom_rect, |bar| {
                bar.with_layout(Layout::top_down(egui::Align::Min), |bar| {
                    bar.style_mut().spacing.interact_size.y = 68.0;
                    bar.style_mut().spacing.window_margin.left = 74.0;

                    let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                        let mut layout_job: egui::text::LayoutJob = LayoutJob::simple_singleline(string.to_string(), egui::FontId::new(24.0, eframe::epaint::FontFamily::Proportional), Color32::WHITE);
                        layout_job.wrap.max_width = wrap_width;
                        ui.fonts(|f| f.layout_job(layout_job))
                    };

                    disgusting_bullshit(bar, false);
                    let marge = Margin { left: bar.style().spacing.window_margin.left, right: bar.style().spacing.window_margin.right, top: 12.0, bottom: 22.0 };
                    let res = bar.add_sized(vec2(bottom_rect.width(), 68.0), TextEdit::singleline(&mut self.entry_fields.message_buffer)
                        .desired_width(bottom_rect.width())
                        .vertical_align(egui::Align::Min)
                        .text_color(Color32::WHITE)
                        .hint_text("Reply")
                        .margin(marge)
                        .font(egui::FontId::new(24.0, eframe::epaint::FontFamily::Proportional))
                        .frame(false)
                    );

                    if res.lost_focus()  && res.ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.entry_fields.user_info_query_results.clear();
                        self.backend.tx.send(UiToReso::SignalSendMessage(id.clone(), self.entry_fields.message_buffer.clone())).unwrap();
                        self.entry_fields.message_buffer = String::new();
                    }
                    //bar.painter().rect_filled(res.rect, Rounding::same(0.0), Color32::from_white_alpha(64));
                });
            });

        });


        
        
    }
}

                
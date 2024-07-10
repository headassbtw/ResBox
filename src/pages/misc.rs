use egui::{vec2, Margin, TextEdit};

use crate::{backend::{self, thread::UiToReso}, disgusting_bullshit, icon_notification, widgets::{button::metro_button, page_header::page_header, segoe_boot_spinner::SegoeBootSpinner, toggle_switch::toggle_ui, user_info::{user_info_widget, UserInfoVariant}}, FrontendPage, TemplateApp, CONTENT_LEFT_PAD, CONTENT_RIGHT_PAD, KEYRING_SERVICE, KEYRING_USER, TEXT_COL};

impl TemplateApp {
    pub fn signin_page(&mut self, ui: &mut egui::Ui) {
        ui.style_mut().spacing.interact_size.y = 60.0;
        
        page_header(ui, "Sign In", "0 Signed in");

        ui.style_mut().spacing.item_spacing.y = 0.0;
        let mut test_rect = ui.available_rect_before_wrap();
        test_rect.max.y = test_rect.min.y + 60.0;
        let marge = Margin { left: CONTENT_LEFT_PAD, right: CONTENT_RIGHT_PAD, top: 12.0, bottom: 12.0 };

        if !self.logged_in && self.can_attempt_login{
            disgusting_bullshit(ui, false);
            ui.add_sized(test_rect.size(), TextEdit::singleline(&mut self.entry_fields.login_details.username)
                .desired_width(test_rect.width())
                .vertical_align(egui::Align::Center)
                .text_color(TEXT_COL)
                .hint_text("Username")
                .margin(marge)
                .font(egui::FontId::new(24.0, eframe::epaint::FontFamily::Proportional))
                .frame(false)
            );
            
            disgusting_bullshit(ui, false);
            ui.add_sized(test_rect.size(), TextEdit::singleline(&mut self.entry_fields.login_details.password)
                .desired_width(test_rect.width())
                .vertical_align(egui::Align::Center)
                .text_color(TEXT_COL)
                .hint_text("Password")
                .margin(marge)
                .font(egui::FontId::new(24.0, eframe::epaint::FontFamily::Proportional))
                .password(true)
                .frame(false)
            );

            
            toggle_ui(ui, "Remember Me", &mut self.entry_fields.login_details.remember_me);
            
            if metro_button(ui, "Log in", Some(("", 24.0))).clicked() {
                self.backend.tx.send(backend::thread::UiToReso::TokenRequestCredentials(self.entry_fields.login_details.username.clone(), self.entry_fields.login_details.password.clone(), self.entry_fields.login_details.remember_me)).unwrap();
                self.can_attempt_login = false;      
                self.set_page(FrontendPage::LoadingPage);
            }
        }
    }
    
    pub fn user_search_page(&mut self, ui: &mut egui::Ui) {
        page_header(ui, "Query Users", &self.username());
        let marge = Margin { left: CONTENT_LEFT_PAD, right: CONTENT_RIGHT_PAD, top: 12.0, bottom: 12.0 };
        let i_size = vec2(ui.available_width(), 60.0);
        ui.style_mut().spacing.interact_size.y = 60.0;

        disgusting_bullshit(ui, false);
        let text_re = ui.add_sized(i_size, TextEdit::singleline(&mut self.entry_fields.user_info_query)
            .desired_width(i_size.x)
            .vertical_align(egui::Align::Center)
            .text_color(TEXT_COL)
            .hint_text("User ID")
            .margin(marge)
            .font(egui::FontId::new(24.0, eframe::epaint::FontFamily::Proportional))
            .frame(false)
        );

        

        if (text_re.lost_focus()  && text_re.ctx.input(|i| i.key_pressed(egui::Key::Enter))) || metro_button(ui, "Search", Some(("", 24.0))).clicked() {
            self.entry_fields.user_info_query_results.clear();
            self.backend.tx.send(UiToReso::UserInfoRequest(self.entry_fields.user_info_query.clone())).unwrap();
        }

        ui.style_mut().spacing.interact_size.y = 80.0;

        egui::containers::ScrollArea::vertical().scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden).show(ui, |ui| {
            ui.style_mut().spacing.interact_size.y = 104.0;
            ui.style_mut().spacing.item_spacing.y = 4.0;
            for user in &self.entry_fields.user_info_query_results {
                if let Some(userinfo) = self.cached_user_infos.get(user) {
                    let id = userinfo.id.clone();
                    
                    if user_info_widget(ui, &mut self.image_cache, UserInfoVariant::Cached(&userinfo)).clicked() {
                        ui.label("i hate rust mutability");
                        //self.set_page(FrontendPage::ProfilePage(id));
                    }
                } else {
                    user_info_widget(ui, &mut self.image_cache, UserInfoVariant::Uncached(&user));
                }
            }
        });
    }

    pub fn unknown_page(&mut self, ui: &mut egui::Ui) {
        page_header(ui, "Unknown Page", "Not Implemented");
    }

    pub fn settings_page(&mut self, ui: &mut egui::Ui) {
        page_header(ui, "Settings", "[UNFINISHED/DEBUG]");
        ui.style_mut().spacing.interact_size.y = 60.0;

        if metro_button(ui, "Clear persistent credentials", Some(("", 24.0))).clicked() {
            self.entry_fields.login_details.remember_me = false;
            self.entry_fields.login_details.username = "".to_owned();
            self.entry_fields.login_details.password = "".to_owned();
            if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER) {
                if let Err(err) = entry.delete_password() {
                    self.notifications.push(icon_notification("", "Keyring deletion failed", format!("{}", err).as_str()));
                }
            }
        }
        if metro_button(ui, "Request Status", None).clicked() {
            self.backend.tx.send(UiToReso::SignalRequestStatus(None, false)).unwrap();
        }
    }

    pub fn loading_page(&mut self, ui: &mut egui::Ui) {
        ui.put(ui.available_rect_before_wrap(), SegoeBootSpinner::new().size(150.0));
    }
}
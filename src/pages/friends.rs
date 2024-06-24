
use std::{collections::{BTreeMap, HashMap}, future::IntoFuture, sync::Arc};

use keyring::{Entry, Result};

use eframe::{glow, Frame};
use egui::{epaint::{text::cursor::PCursor, Shadow}, load::SizedTexture, mutex::Mutex, output::OutputEvent, vec2, Align2, Color32, FontData, FontDefinitions, FontId, ImageSource, Layout, Margin, Pos2, Rect, RichText, Rounding, Stroke, TextEdit, TextureId, Vec2, Widget};
use log::{debug, error};
use tokio;

use lazy_static::lazy_static;


use crate::{widgets::{button::metro_button, page_header::page_header, user_info::{user_info_widget, UserInfoVariant}}, FrontendPage, TemplateApp, CONTACTS_LIST, CONTENT_LEFT_PAD, CONTENT_RIGHT_PAD};

impl TemplateApp {
    pub fn friends_page(&mut self, ui: &mut egui::Ui) {
        page_header(ui, "Friends", &self.username());

        ui.style_mut().spacing.interact_size.y = 60.0;
        

        if metro_button(ui, "Find someone", None).clicked() {
            self.current_page = FrontendPage::UserSearchPage;
        }

        

        egui::containers::ScrollArea::vertical().scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden).show(ui, |ui| {
            ui.style_mut().spacing.interact_size.y = 104.0;
            ui.style_mut().spacing.item_spacing.y = 4.0;
            for (id, user) in CONTACTS_LIST.lock().iter() {
                if !user.is_accepted { continue; }
                if user_info_widget(ui, &mut self.image_cache, UserInfoVariant::Contact(&user)).clicked() {
                    self.current_page = FrontendPage::ProfilePage(id.clone());
                }
            }
        });
    }
}
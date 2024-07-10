use std::{cmp::Ordering, collections::{BTreeMap, HashMap}, future::IntoFuture, sync::Arc};

use keyring::{Entry, Result};

use eframe::{glow, Frame};
use egui::{epaint::{text::cursor::PCursor, Shadow}, load::SizedTexture, mutex::Mutex, output::OutputEvent, vec2, Align2, Color32, FontData, FontDefinitions, FontId, ImageSource, Layout, Margin, Order, Pos2, Rect, RichText, Rounding, Stroke, TextEdit, TextureId, Vec2, Widget};
use log::{debug, error};
use tokio;

use lazy_static::lazy_static;


use crate::{api::client::Contact, backend::thread::OnlineStatus, widgets::{button::metro_button, page_header::page_header, user_info::{user_info_widget, UserInfoVariant}}, FrontendPage, TemplateApp, CONTACTS_LIST, CONTENT_LEFT_PAD, CONTENT_RIGHT_PAD, USER_STATUSES};

impl TemplateApp {
    pub fn friends_page(&mut self, ui: &mut egui::Ui) {
        page_header(ui, "Friends", &self.username());

        ui.style_mut().spacing.interact_size.y = 60.0;
        

        if metro_button(ui, "Find someone", None).clicked() {
            self.set_page(FrontendPage::UserSearchPage);
        }

        

        egui::containers::ScrollArea::vertical().scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden).show(ui, |ui| {
            ui.style_mut().spacing.interact_size.y = 104.0;
            ui.style_mut().spacing.item_spacing.y = 4.0;
            let list = CONTACTS_LIST.lock(); // needs to be on a seperate line to lock, ffs rust
            let mut ctx_list: Vec<(&String, &Contact)> = list.iter().collect();
            { // we lock the user statuses in the user info widget, locking it in the scope above hardlocks the UI (ask me how i know)
                let stat_list = USER_STATUSES.lock();
                //TODO: this sorting doesn't work
                ctx_list.sort_by(|a, b| {
                    let (c,d) = *a;
                    let (e,f) = *b;
            
                    if let Some(first) = stat_list.get(c) {
                        if let Some(first_cmp) = stat_list.get(e) { 
                            if first.online_status.is_none() && first_cmp.online_status.is_none() { return Ordering::Equal }

                            if let Some(first) = &first.online_status {
                                // we already checked both, if one is ok then the other is too
                                return first_cmp.online_status.as_ref().unwrap().cmp(first)
                            } return Ordering::Less
                        } return Ordering::Less
                    } return Ordering::Less
                });
            }
        
            for (id, user) in ctx_list {
                if !user.is_accepted { continue; }
                if user_info_widget(ui, &mut self.image_cache, UserInfoVariant::Contact(&user)).clicked() {
                    self.set_page(FrontendPage::ProfilePage(id.clone()));
                }
            }
        });
    }
}
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

                // Last online
                ctx_list.sort_by(|a, b| {
                    let (_,d) = *a;
                    let (_,f) = *b;
            
                    return f.latest_message_time.0.cmp(&d.latest_message_time.0);
                });

                // Current online status
                ctx_list.sort_by(|a, b| {
                    let (c,_) = *a;
                    let (e,_) = *b;
            
                    if !stat_list.contains_key(c) && !stat_list.contains_key(e) { return Ordering::Equal; }
                    //we've ruled out both not existing, we can check individually
                    if !stat_list.contains_key(c) { return Ordering::Greater; } // b exists, a doesn't, b > a
                    if !stat_list.contains_key(e) { return Ordering::Less; }    // a exists, b doesn't, a > b
                    
                    return stat_list.get(c).unwrap().online_status.cmp(&stat_list.get(e).unwrap().online_status);
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
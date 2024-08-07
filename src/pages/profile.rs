use egui::{vec2, Align2, Color32, FontId};

use crate::{
    widgets::{
        button::metro_button, loadable_image::loadable_image, user_info::{uid_to_color, user_color_and_subtext}
    }, FrontendPage, TemplateApp, CONTACTS_LIST, HOVER_COL, SESSION_CACHE, THIS_FUCKING_SUCKS, USER_STATUSES
};


impl TemplateApp {
    pub fn profile_page(&mut self, ui: &mut egui::Ui, id: String) {
        let is_you: bool = if let Some(youid) = &self.user_id { youid.eq(&id) } else { false };
        let is_contact: bool = { CONTACTS_LIST.lock().get(&id).is_some() };
        let mut pfp_rect = ui.cursor().clone();
        pfp_rect.max.y = pfp_rect.min.y + 284.0;
        let shrink_factor = (pfp_rect.width() - 284.0) / 2.0;
        pfp_rect.max.x -= shrink_factor;
        pfp_rect.min.x += shrink_factor;
        
        let mut pfp_path: Option<String> = None;

        {
            let contacts = CONTACTS_LIST.lock();
            let contact = contacts.get(&id).clone();

            if let Some(contact) = contact {
                if let Some(profile) = &contact.profile {
                    pfp_path = Some(profile.icon_url.clone())
                }
            }
        }

        if pfp_path.is_none() {
            if let Some(user) = self.cached_user_infos.get(&id) {
                if let Some(profile) = &user.profile {
                    pfp_path = Some(profile.icon_url.clone());
                }
            }
        }

        if let Some(path) = pfp_path{
            let loadable = self.image_cache.get_image(&path);
            loadable_image(ui, &loadable, pfp_rect, "", HOVER_COL, 142.0, false);
        } else {
            ui.painter().circle_filled(pfp_rect.center(), 142.0, uid_to_color(&id));
            ui.painter().text(pfp_rect.center(), Align2::CENTER_CENTER, "", FontId::proportional(142.0), Color32::WHITE);
        }
        
        
        
        let mut avail_rect = ui.cursor().clone();
        avail_rect.min.y = pfp_rect.max.y + 20.0;

        let name_pos = avail_rect.min + vec2(avail_rect.width() / 2.0, 0.0);



        let name = if is_you {
            if let Some(name) = &self.you {
                name.username.clone()
            } else {
                "You".to_owned()
            }
        } else if is_contact {
            if let Some(contact) = CONTACTS_LIST.lock().get(&id) {
                contact.contact_username.clone()
            } else {
                "Unknown Contact".to_owned()
            }
        } else {
            if let Some(user) = self.cached_user_infos.get(&id) {
                user.username.clone()
            } else {
                "Unknown User".to_owned()
            }
        };

        let (col, subtext) = user_color_and_subtext(&id);
        //TODO: add subtext

        let sub_basis = ui.painter().text(name_pos, Align2::CENTER_TOP, name, FontId::proportional(24.0), Color32::WHITE);
        let id_pos = sub_basis.min + vec2(sub_basis.width() / 2.0, 38.0);
        ui.painter().text(id_pos, Align2::CENTER_TOP, &id, FontId::proportional(24.0), Color32::GRAY);
        let stat_pos = sub_basis.min + vec2(-18.0, 16.0);
        if let Some(col) = col {
            ui.painter().circle_filled(stat_pos, 6.0, col);
        }

        avail_rect.min.y += 80.0;
        ui.allocate_space(vec2(0.0, avail_rect.min.y - ui.cursor().min.y));

        ui.style_mut().spacing.interact_size.y = 60.0;
        ui.style_mut().spacing.item_spacing.y = 0.0;

        if metro_button(ui, "Request Status", None).clicked() {
            self.backend.tx.send(crate::backend::thread::UiToReso::SignalRequestStatus(Some(id.clone()), false)).unwrap();
        }

        if self.is_you(&id) { return ; }
        
        if metro_button(ui, "Send message", None).clicked() {
            self.set_page(FrontendPage::ConversationPage(id.clone()));
        }
        
        ui.set_enabled(false);
        if !is_contact {
            metro_button(ui, "Friend Request", Some(("", 24.0)));
        }
        metro_button(ui, "Block", None);


        // this is here for debug purposes, it's useful so i'm going to leave it here
        // i made this commit on a laptop forgive me
        let jank0 = USER_STATUSES.lock();
        
        if jank0.get(&id).is_none() { return; }
        let status = jank0.get(&id).unwrap();

        for (idx, session) in &mut status.sessions.iter().enumerate() {
            let active = if status.current_session_index < 0 { false } else { (status.current_session_index as usize).eq(&idx) };
            let jank1 = THIS_FUCKING_SUCKS.lock();
            let jank2 = SESSION_CACHE.lock();
            if let Some(session_id) = jank1.get(&session.session_hash) {
                if let Some(session) = jank2.get(session_id) {
                    ui.label(format!("{}{}", if active {"Active - "} else {""}, session.name));
                } else {
                    ui.label(format!("{}hash in table but session not", if active {"Active - "} else {""}));
                }
            } else {
                ui.label(format!("{}hash not in table", if active {"Active - "} else {""}));
            }
        }
        
    }
}
use std::cmp::max;

use egui::{epaint::{emath::lerp, Rect, Shape}, pos2, vec2, Align2, Color32, FontId, Pos2, Rounding, Stroke, FontFamily, Response, Sense, Ui, Widget, WidgetInfo, WidgetType};

use crate::{api::client::{Contact, UserInfo}, image::ResDbImageCache, main, TemplateApp, SUBHEADER_COL};
use super::loadable_image::loadable_image;

pub enum UserInfoVariant<'a> {
    Cached(&'a UserInfo),
    Contact(&'a Contact),
    Uncached(&'a String),
}
/// Parses user ID as a color, Deterministic
pub fn uid_to_color(uid: &String) -> Color32 {
    
    let split_pos: usize = uid.len() / 3;

    let (r, gb) = uid.split_at(split_pos);
    let (g, b) = gb.split_at(split_pos);

    let r: u32 = r.chars().map(|c| c.to_digit(10).or_else(|| Some(32)).unwrap()).sum::<u32>();
    let g: u32 = g.chars().map(|c| c.to_digit(10).or_else(|| Some(32)).unwrap()).sum::<u32>();
    let b: u32 = b.chars().map(|c| c.to_digit(10).or_else(|| Some(32)).unwrap()).sum::<u32>();

    let r = r as u8;
    let g = g as u8;
    let b = b as u8;

    Color32::from_rgb(r, g, b)
}

pub fn draw_user_pic_at(ui: &mut egui::Ui, rect: egui::Rect, cache: &mut ResDbImageCache, info: UserInfoVariant) {
    let radius = rect.width() / 2.0;
    if match info {
        UserInfoVariant::Uncached(_) => true,
        UserInfoVariant::Contact(contact) => {
            if let Some(prof) = &contact.profile {
                let loadable = cache.get_image(&prof.icon_url);
                loadable_image(ui, &loadable, rect, "", uid_to_color(&contact.id), radius, false);
                false
            } else { true }
        },
        UserInfoVariant::Cached(user) => {
            if let Some(prof) = &user.profile {
                let loadable = cache.get_image(&prof.icon_url);
                loadable_image(ui, &loadable, rect, "", uid_to_color(&user.id), radius, false);
                false
            } else { true }
        },
    } {
        ui.painter().circle_filled(rect.center(), radius, uid_to_color(
            match info {
                UserInfoVariant::Cached(inf) => &inf.username,
                UserInfoVariant::Contact(inf) => &inf.contact_username,
                UserInfoVariant::Uncached(id) => id,
            }
         ));
        ui.painter().text(rect.center(), Align2::CENTER_CENTER, "", FontId::proportional(radius), Color32::WHITE);
    }
}

pub fn user_info_widget(ui: &mut egui::Ui, cache: &mut ResDbImageCache, info: UserInfoVariant) -> egui::Response {
    let height = ui.style().spacing.interact_size.y;
    let mut rect = ui.cursor().clone();
    rect.max.y = height + rect.min.y;
    let response = ui.allocate_rect(rect, egui::Sense::click());

    if !ui.is_rect_visible(rect) { return response; }

    let pfp_radius = (height - 32.0 ) / 2.0;

    // usually 104  
    // pfps usually 72

    // 16px padding

    let bound_rect = Rect {
        min: Pos2 { x: rect.min.x + ui.style().spacing.window_margin.left + (pfp_radius * 2.0) + 16.0,  y: rect.min.y + 16.0 },
        max: Pos2 { x: rect.max.x - ui.style().spacing.window_margin.right,                             y: rect.max.y - 16.0 }
    };

    if response.is_pointer_button_down_on() {
        ui.painter().rect_filled(rect, Rounding::same(0.0), ui.style().visuals.widgets.active.bg_fill);
    } else if response.hovered() {
        ui.painter().rect_filled(rect, Rounding::same(0.0), ui.style().visuals.widgets.hovered.bg_fill);
    }

    let circle_pos = rect.min + vec2(ui.style().spacing.window_margin.left + pfp_radius, pfp_radius + 16.0);

    let cirlcle_rect = egui::Rect::from_center_size(circle_pos, vec2(pfp_radius * 2.0, pfp_radius * 2.0));

    let blank_ref = &"".to_owned();

    let (main, sub, stat, needs_draw) = {
        match info {
            UserInfoVariant::Uncached(uid) => {
                (
                    uid,
                    blank_ref,
                    blank_ref,
                    true
                )
            },
            UserInfoVariant::Contact(contact) => {
                (
                    &contact.contact_username,
                    &contact.id,
                    &contact.contact_status,
                    if let Some(prof) = &contact.profile {
                        let loadable = cache.get_image(&prof.icon_url);
                        loadable_image(ui, &loadable, cirlcle_rect, "", uid_to_color(&contact.id), 32.0, false);
                        false
                    } else { true }
            )
            },
            UserInfoVariant::Cached(user) => {
                (
                    &user.username,
                    &user.id,
                    blank_ref,
                    if let Some(prof) = &user.profile {
                        let loadable = cache.get_image(&prof.icon_url);
                        loadable_image(ui, &loadable, cirlcle_rect, "", uid_to_color(&user.id), 32.0, false);
                        false
                    } else { true }
                )
            },
        }
    };
    
    if needs_draw {
        ui.painter().circle_filled(circle_pos, pfp_radius, uid_to_color(if sub.is_empty() { main } else {sub} ));
        ui.painter().text(circle_pos, Align2::CENTER_CENTER, "", FontId::proportional(pfp_radius), Color32::WHITE);
    }

    let mut fullname_rect = ui.painter().text(bound_rect.min, Align2::LEFT_TOP, main, FontId::proportional(24.0), Color32::WHITE);

    fullname_rect.min.x = fullname_rect.max.x;
    fullname_rect.max.x = bound_rect.max.x;
    fullname_rect.min.x += 12.0;

    ui.painter().text(pos2(fullname_rect.min.x, fullname_rect.max.y), Align2::LEFT_BOTTOM, sub, FontId::proportional(18.0), Color32::GRAY);

    ui.painter().text(bound_rect.min + vec2(0.0, 34.0), Align2::LEFT_TOP, stat, FontId::proportional(24.0), Color32::GRAY);

    response
}
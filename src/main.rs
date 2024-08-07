#![warn(clippy::all, rust_2018_idioms)]
#![allow(unused_imports)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{collections::{BTreeMap, HashMap}, future::IntoFuture, sync::Arc};

use keyring::{Entry, Result};

use backend::thread::{BackendThread, BroadcastTarget, InitialLoginType, UiToReso, UserStatus, SessionUpdate};
use eframe::{glow, Frame};
use egui::{epaint::{text::cursor::PCursor, Shadow}, load::SizedTexture, mutex::Mutex, output::OutputEvent, pos2, vec2, Align2, Color32, FontData, FontDefinitions, FontId, ImageSource, Key, Layout, Margin, PointerButton, Pos2, Rect, RichText, Rounding, Stroke, TextEdit, TextureId, UiStackInfo, Vec2, Widget};
use humansize::{SizeFormatter, DECIMAL};
use image::{LoadableImage, ResDbImageCache};
use log::{debug, error};
use tokio;

use lazy_static::lazy_static;

mod api;
mod widgets;
mod backend;
mod pages;
mod self_helpers;
mod bridge;

pub mod image;

use api::{client::{Contact, Message, UserInfo}, login};
use widgets::{button::metro_button, loadable_image::loadable_image, page_header::page_header, segoe_boot_spinner::{self, SegoeBootSpinner}, toggle_switch::{self, toggle_ui}, user_info::{uid_to_color, user_info_widget, UserInfoVariant}};

const KEYRING_SERVICE: &str = "com.headassbtw";
const KEYRING_USER: &str = "resbox";

#[tokio::main]
async fn main() -> eframe::Result<()> {

    //let fmt_subscriber = tracing_subscriber::FmtSubscriber::builder()
    //.with_max_level(tracing::Level::TRACE)
    //.finish();
    //tracing::subscriber::set_global_default(fmt_subscriber)
    //.expect("setting tracing default failed");

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 1080.0])
            .with_app_id(format!("{}.{}", KEYRING_SERVICE, KEYRING_USER))
            // the quintessential "traffic lights in content" all osx apps have
            .with_fullsize_content_view(true)
            .with_title_shown(false)
            .with_titlebar_shown(false)

            .with_min_inner_size([600.0, 900.0]),
            
        ..Default::default()
    };
    eframe::run_native(
        "this will be resonite graphics in 2015",
        native_options,
        Box::new(|cc| Ok(Box::new(TemplateApp::new(cc)))),
    )
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct LoginSettings {
    username: String,
}

struct LoginDetails {
    username: String,
    password: String,
    remember_me: bool
}

struct TemporaryEntryFields {
    user_info_query: String,
    user_info_query_results: Vec<String>,
    login_details: LoginDetails,
    message_buffer: String,
}

pub struct TemplateApp {
    backend : BackendThread,
    /// prevents user from spamming the login endpoint
    can_attempt_login: bool,
    logged_in: bool,
    you: Option<UserInfo>,
    user_id: Option<String>,
    token: String,
    notifications: Vec<FrontendNotification>,
    /// past pages viewed
    page_stack: Vec<FrontendPage>,
    /// used to index page_stack, to support both going back and forward
    current_page: usize,
    entry_fields: TemporaryEntryFields,
    cached_user_infos: HashMap<String, UserInfo>,
    image_cache: ResDbImageCache,
}

enum FrontendNotificationIcon {
    SegoeIcon(String),
    LoadableImage(LoadableImage)
}

pub struct FrontendNotification {
    icon: FrontendNotificationIcon,
    text: String,
    sub: String,
}

#[derive(PartialEq)]
enum FrontendPage {
    SignInPage,
    ProfilePage(String),
    ConversationPage(String),
    FriendsPage,
    UserSearchPage,
    SessionsPage,
    MessagesPage,
    NotificationsPage,
    LoadingPage,
    SettingsPage,
    UnknownPage
}
pub const SIDEBAR_ITEM_SPACING: f32 = 20.0; 
pub const SIDEBAR_ITEM_SIZE: f32 = 104.0;

pub const CONTENT_LEFT_PAD: f32 = 70.0;
pub const CONTENT_RIGHT_PAD: f32 = 24.0;

pub const SUBHEADER_COL: Color32 = Color32::from_gray(170);

/// Used in place of white on disabled elements
pub const DISABLED_COL: Color32 = Color32::from_gray(121);
pub const TEXT_COL: Color32 = Color32::from_gray(250);

pub const ACCENT: Color32 = Color32::from_rgb(220, 53, 60);
pub const HOVER_COL: Color32 = Color32::from_gray(51);

lazy_static! { // sue me.
    pub static ref CONTACTS_LIST: Mutex<HashMap<String, Contact>> = Mutex::new(HashMap::new());
    pub static ref USER_STATUSES: Mutex<HashMap<String, UserStatus>> = Mutex::new(HashMap::new());
    pub static ref MESSAGE_CACHE: Mutex<HashMap<String, Vec<Message>>> = Mutex::new(HashMap::new());
    pub static ref SESSION_CACHE: Mutex<HashMap<String, SessionUpdate>> = Mutex::new(HashMap::new());
    /// THIS FUCKING SUCKS!
    pub static ref REFRESH_UI: Mutex<bool> = Mutex::new(false);

    pub static ref THIS_FUCKING_SUCKS: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

fn icon_notification(icon: &str, header: &str, details: &str) -> FrontendNotification {
    FrontendNotification {
        icon: FrontendNotificationIcon::SegoeIcon(icon.to_owned()),
        text: header.to_owned(),
        sub: details.to_owned()
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        cc.egui_ctx.style_mut(|style| {
            #[cfg(debug_assertions)] {
                style.debug.show_resize = true;
                style.debug.debug_on_hover_with_all_modifiers = true;
                style.debug.hover_shows_next = true;
            }

            style.visuals.widgets.active.bg_fill = ACCENT;
            style.visuals.widgets.hovered.bg_fill = HOVER_COL;
        });
        
        

        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert("Segoe UI".to_owned(), egui::FontData::from_static(include_bytes!("../segoeui.ttf")));
        fonts.font_data.insert("MDL2 Icons".to_owned(), egui::FontData::from_static(include_bytes!("../segmdl2.ttf")));
        fonts.font_data.insert("Segoe Boot".to_owned(), egui::FontData::from_static(include_bytes!("../segoe_slboot.ttf")));

        fonts.families.insert(egui::FontFamily::Name("MDL2 Icons".into()), vec!["MDL2 Icons".to_owned()]);
        fonts.families.insert(egui::FontFamily::Name("Segoe Boot".into()), vec!["Segoe Boot".to_owned()]);
        
        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "Segoe UI".to_owned());
        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "MDL2 Icons".to_owned());


        fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().insert(0, "MDL2 Icons".to_owned());
        fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().insert(0, "Segoe Boot".to_owned());

        cc.egui_ctx.set_fonts(fonts);


        let mut r = Vec::new();
        //r.push(icon_notification("", "Key", "Figma Balls"));
        if cfg!(debug_assertions) {
            r.push(icon_notification("","Windows Phone","Wake up babe it's 2015"));
        }

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        let name: String = if let Some(storage) = cc.storage {
            eframe::get_value(storage, "username").unwrap_or("".to_string())
        } else { "".to_string() };

        let creds = if !name.is_empty() {
            let entry = Entry::new(KEYRING_SERVICE, KEYRING_USER);
            if let Ok(entry) = entry {
                if let Ok(pass) = entry.get_password() {
                    InitialLoginType::PreviousToken { username: name.clone(), session_token: pass }
                } else {
                    println!("no password");
                    InitialLoginType::Fresh
                }
            } else {
                r.push(icon_notification("","No Keyring","Entry retrival failed"));
                InitialLoginType::Fresh
            }
        } else {
            println!("no name");
            InitialLoginType::Fresh
        };

        Self { 
            can_attempt_login: false, // check for a cached token
            logged_in: false,
            user_id: None,
            token: String::new(),
            you: None,
            page_stack: {
                let mut vec = Vec::new();
                vec.push(FrontendPage::LoadingPage);
                vec
            },
            current_page: 0,
            notifications: r,
            cached_user_infos: HashMap::new(),
            backend: BackendThread::new(&cc.egui_ctx, creds),
            entry_fields: TemporaryEntryFields {
                user_info_query: String::new(),
                user_info_query_results: Vec::new(),
                login_details: LoginDetails {
                    remember_me: { !name.is_empty() },
                    username: name,
                    password: String::new(),
                },
                message_buffer: String::new(),
            },
            image_cache: ResDbImageCache::new(cc.egui_ctx.clone()),
        }
    }
}

fn disgusting_bullshit(ui: &mut egui::Ui, click_test: bool) {
    let mut test_rect = ui.available_rect_before_wrap();
    test_rect.max.y = test_rect.min.y + 60.0;
    test_rect.min.x = 0.0;
    let hov = if let Some(pos) = ui.ctx().input(|i| i.pointer.interact_pos()) {
        test_rect.contains(pos)
    } else {
        false
    };
    let m1down = ui.ctx().input(|i| i.pointer.primary_down()) && hov;


    if m1down && click_test{
        ui.painter().rect_filled(test_rect, Rounding::same(0.0), ACCENT);
    } else if hov {
        ui.painter().rect_filled(test_rect, Rounding::same(0.0), HOVER_COL);
    }
}

fn sidebar_button(text: &str, ui: &mut egui::Ui) -> bool {
    ui.add_sized(vec2(SIDEBAR_ITEM_SIZE,SIDEBAR_ITEM_SIZE), egui::Button::new(egui::RichText::new(text).size(32.0).color(Color32::WHITE)).frame(false)).clicked()
}

fn sidebar_top_pos(idx: u8) -> f32 {
    SIDEBAR_ITEM_SPACING + if idx == 0 { 0.0 } else { (20.0 + SIDEBAR_ITEM_SIZE) + ((SIDEBAR_ITEM_SIZE+4.0) * (idx-1) as f32) }
}

impl eframe::App for TemplateApp {

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "username", &self.entry_fields.login_details.username);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(result) = self.backend.rx.try_recv() {
            bridge::to_ui::process_to_ui(self, result);
        }

        let panel_frame = egui::Frame {
            inner_margin: Margin::same(0.0),
            outer_margin: Margin::same(0.0),
            rounding: Rounding::ZERO,
            shadow: Shadow::NONE,
            fill: Color32::from_gray(31),
            stroke: Stroke::NONE,
        };
        egui::CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
            ui.input_mut(|state| {
                for ev in &state.events { // this shit sucks
                    match ev {
                        egui::Event::Key { key, physical_key, pressed, repeat, modifiers } => {
                            if key == &Key::Escape && *pressed && !*repeat{
                                self.page_back();
                            }
                        },
                        egui::Event::PointerButton { pos, button, pressed, modifiers } => {
                            if button == &PointerButton::Extra1 && *pressed{
                                self.page_back();
                            } else if button == &PointerButton::Extra2 && *pressed{
                                self.page_forward();
                            }
                        },
                        _ => {}
                    }
                }
            });
            let mut avail_rect = ui.available_rect_before_wrap();
            avail_rect.min.x = avail_rect.max.x - 104.0;
            let mut sidebar = ui.child_ui(avail_rect, Layout::top_down(egui::Align::Center), None); {

                sidebar.style_mut().spacing.item_spacing.y = 20.0;

                sidebar.painter().rect_filled(avail_rect, Rounding::same(0.0), Color32::from_gray(6));
                sidebar.allocate_space(vec2(0.0, 0.0));
                
                let tab = match self.current_page() {
                    FrontendPage::SignInPage => 0,
                    FrontendPage::ProfilePage(id) => {if let Some(you) = &self.user_id { if you.eq(id) { 0 } else { 255 }} else { 255 }},
                    FrontendPage::ConversationPage(_) => 255,
                    FrontendPage::FriendsPage => 1,
                    FrontendPage::UserSearchPage => 255,
                    FrontendPage::SessionsPage => 2,
                    FrontendPage::MessagesPage => 3,
                    FrontendPage::NotificationsPage => 4,
                    FrontendPage::SettingsPage => 5,
                    FrontendPage::LoadingPage => 255,
                    FrontendPage::UnknownPage => 255,
                };
                let paint_offset_from_top = ctx.animate_value_with_time("sidebar_item_highlight_abs_y".into(), sidebar_top_pos(tab), 0.125);
                let mut paint_rect = Rect::clone(&avail_rect);
                paint_rect.min.y = paint_offset_from_top;
                paint_rect.max.y = paint_rect.min.y + SIDEBAR_ITEM_SIZE;
                sidebar.painter().rect_filled(paint_rect, Rounding::same(0.0), ACCENT);

                if self.logged_in {
                    let (rect, response) = sidebar.allocate_exact_size(vec2(SIDEBAR_ITEM_SIZE,SIDEBAR_ITEM_SIZE), egui::Sense::click());
                    if response.clicked() { self.set_page(FrontendPage::ProfilePage(self.user_id.clone().unwrap())); }

                    let needs_placeholder: bool = if let Some(profile) = &self.you {
                        if let Some(profile) = &profile.profile {
                            let loadable = self.image_cache.get_image(&profile.icon_url);
                            let shrink_factor = 0.0 - (64.0 - rect.width()) / 2.0;
                            loadable_image(ui, &loadable, rect.clone().shrink(shrink_factor), "", HOVER_COL, 32.0, false);
                            false
                        } else {
                            true
                        }
                    } else {
                        true
                    };
                    if needs_placeholder {   
                        sidebar.painter().circle_filled(rect.center(), 32.0, HOVER_COL);
                        sidebar.painter().text(rect.center(), Align2::CENTER_CENTER, "", FontId::proportional(32.0), Color32::WHITE);
                    }
                    
                } else if !self.can_attempt_login {
                    if sidebar.add_sized(vec2(SIDEBAR_ITEM_SIZE,SIDEBAR_ITEM_SIZE), segoe_boot_spinner::SegoeBootSpinner::new().size(32.0)).clicked() {
                        self.set_page(FrontendPage::LoadingPage);
                    }
                } else {
                    if sidebar.add_sized(vec2(SIDEBAR_ITEM_SIZE,SIDEBAR_ITEM_SIZE), egui::Button::new(egui::RichText::new("").size(32.0).color(Color32::WHITE)).frame(false)).clicked() {
                        self.set_page(FrontendPage::SignInPage);
                    }
                }
                
                sidebar.style_mut().spacing.item_spacing.y = 4.0;

                if sidebar_button("", &mut sidebar) { self.set_page(FrontendPage::FriendsPage); }  // friends

                if sidebar_button("", &mut sidebar) { self.set_page(FrontendPage::SessionsPage); } // parties (sessions)

                if sidebar_button("", &mut sidebar) { self.set_page(FrontendPage::MessagesPage); } // messages

                if {
                    let (rect, response) = sidebar.allocate_exact_size(vec2(SIDEBAR_ITEM_SIZE, SIDEBAR_ITEM_SIZE), egui::Sense::click());

                    // 10px between icon and text
                    // icon is 32px, text is 18px

                    if self.notifications.len() <= 0 {
                        sidebar.painter_at(rect).text(rect.center(), Align2::CENTER_CENTER, "", FontId::monospace(32.0), Color32::WHITE);
                    } else {
                        let count_galley = ui.painter().layout(self.notifications.len().to_string(), FontId::proportional(18.0), Color32::WHITE, SIDEBAR_ITEM_SIZE);
                        let width = 32.0 + 10.0 + count_galley.rect.width();
                        let icon_center = pos2((rect.center().x - width / 2.0) + 16.0, rect.center().y + 3.0);
                        let label_pos = pos2(icon_center.x + 26.0, (rect.center().y - count_galley.rect.height() / 2.0) - 3.0);

                        sidebar.painter_at(rect).text(icon_center, Align2::CENTER_CENTER, "", FontId::monospace(32.0), Color32::WHITE);
                        sidebar.painter().galley(label_pos, count_galley, Color32::WHITE)
                    }

                    response.clicked()
                } {
                    self.set_page(FrontendPage::NotificationsPage);
                }

                if sidebar_button("", &mut sidebar) { self.set_page(FrontendPage::SettingsPage); } // settings

                sidebar.with_layout(Layout::bottom_up(egui::Align::Center), |dbg_warn| {
                    dbg_warn.allocate_space(vec2(0.0, 4.0));
                    egui::warn_if_debug_build(dbg_warn);
                });
            }

            let mut content_rect = ui.available_rect_before_wrap();
            content_rect.max.x -= SIDEBAR_ITEM_SIZE;
            
            //ui.painter().rect_filled(content_rect, Rounding::same(0.0), Color32::from_black_alpha(20));
            ui.allocate_ui_at_rect(content_rect, |page| {
                page.allocate_space(vec2(page.available_rect_before_wrap().width(), 32.0));
                
                page.style_mut().spacing.interact_size = vec2(page.available_size_before_wrap().x, 80.0);
                page.style_mut().spacing.window_margin = Margin::symmetric(CONTENT_LEFT_PAD, 0.0);
                page.style_mut().spacing.window_margin.right = CONTENT_RIGHT_PAD;
                
                match self.current_page() {
                    FrontendPage::SignInPage => self.signin_page(page),
                    FrontendPage::ProfilePage(id) => self.profile_page(page, id.to_string()),
                    FrontendPage::FriendsPage => self.friends_page(page),
                    FrontendPage::SessionsPage => self.sessions_page(page),
                    FrontendPage::NotificationsPage => self.notifications_page(page),
                    FrontendPage::LoadingPage => self.loading_page(page),
                    FrontendPage::UserSearchPage => self.user_search_page(page),
                    FrontendPage::SettingsPage => self.settings_page(page),
                    FrontendPage::MessagesPage => self.messages_page(page),
                    FrontendPage::ConversationPage(id) => self.conversation_page(page, id.to_string()),
                    _ =>self.unknown_page(page)
                }
            });
            
        });
    }

    fn on_exit(&mut self, _gl: Option<&glow::Context>) {
        self.backend
            .tx
            .send(backend::thread::UiToReso::ShutdownRequest)
            .unwrap();
        self.image_cache.shutdown();
    }
}

#![warn(clippy::all, rust_2018_idioms)]
#![allow(unused_imports)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{collections::{BTreeMap, HashMap}, future::IntoFuture, sync::Arc};

use keyring::{Entry, Result};

use backend::thread::{BackendThread, BroadcastTarget, InitialLoginType, UiToReso, UserStatus};
use eframe::{glow, Frame};
use egui::{epaint::{text::cursor::PCursor, Shadow}, load::SizedTexture, mutex::Mutex, output::OutputEvent, vec2, Align2, Color32, FontData, FontDefinitions, FontId, ImageSource, Layout, Margin, Pos2, Rect, RichText, Rounding, Stroke, TextEdit, TextureId, Vec2, Widget};
use humansize::{SizeFormatter, DECIMAL};
use image::{LoadableImage, ResDbImageCache};
use log::{debug, error};
use tokio;

use lazy_static::lazy_static;

mod api;
mod widgets;
mod backend;
mod pages;

pub mod image;

use api::{client::{Contact, Message, UserInfo}, login};
use widgets::{button::metro_button, loadable_image::loadable_image, segoe_boot_spinner::{self, SegoeBootSpinner}, toggle_switch::{self, toggle_ui}, user_info::{uid_to_color, user_info_widget, UserInfoVariant}};

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
            .with_min_inner_size([600.0, 1080.0]),
            
        ..Default::default()
    };
    eframe::run_native(
        "this will be resonite graphics in 2015",
        native_options,
        Box::new(|cc| Box::new(TemplateApp::new(cc))),
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
    current_page: FrontendPage,
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

const BUTTON_HEIGHT: f32 = 60.0;

pub const SUBHEADER_COL: Color32 = Color32::from_gray(174);

pub const ACCENT: Color32 = Color32::from_rgb(220, 53, 60);
pub const HOVER_COL: Color32 = Color32::from_gray(51);

const KEYRING_SERVICE: &str = "com.headassbtw";
const KEYRING_USER: &str = "resbox";

lazy_static! {
    pub static ref CONTACTS_LIST: Mutex<HashMap<String, Contact>> = Mutex::new(HashMap::new());
    pub static ref MESSAGE_CACHE: Mutex<HashMap<String, Vec<Message>>> = Mutex::new(HashMap::new());
}

fn icon_notification(icon: &str, header: &str, details: &str) -> FrontendNotification {
    FrontendNotification {
        icon: FrontendNotificationIcon::SegoeIcon(icon.to_owned()),
        text: header.to_owned(),
        sub: details.to_owned()
    }
}

impl TemplateApp {

    fn username(&mut self) -> String {
        if let Some(you) = &self.you {
            you.username.clone()
        } else {
            String::from("you")
        }
    }

    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        egui_extras::install_image_loaders(&cc.egui_ctx);
        cc.egui_ctx.style_mut(|style| {
            if cfg!(debug_assertions) {
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
            current_page: FrontendPage::LoadingPage,
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

impl TemplateApp {    

    pub fn is_you(&self, id: &String) -> bool {
        if let Some(you_id) = &self.user_id {
            you_id.eq(id)
        } else {
            false
        }
    }

    pub fn signin_page(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.style_mut().spacing.item_spacing.y = 10.0;
        ui.style_mut().spacing.interact_size.y = 60.0;
        
        ui.horizontal(|ui| {
            ui.allocate_space(vec2(ui.style().spacing.window_margin.left, 0.0));
            ui.vertical(|ui| {
                ui.label(RichText::new("Sign In").size(30.0).color(Color32::WHITE));
                ui.label(RichText::new(format!("0 Signed in")).size(20.0));
                ui.allocate_space(vec2(0.0,20.0));
            });
        });
        ui.style_mut().spacing.item_spacing.y = 0.0;
        let mut test_rect = ui.available_rect_before_wrap();
        test_rect.max.y = test_rect.min.y + 60.0;
        let mut marge = Margin { left: CONTENT_LEFT_PAD, right: CONTENT_RIGHT_PAD, top: 12.0, bottom: 12.0 };

        if !self.logged_in && self.can_attempt_login{
            disgusting_bullshit(ui, false);
            ui.add_sized(test_rect.size(), TextEdit::singleline(&mut self.entry_fields.login_details.username)
                .desired_width(test_rect.width())
                .vertical_align(egui::Align::Center)
                .text_color(Color32::WHITE)
                .hint_text("Username")
                .margin(marge)
                .font(egui::FontId::new(24.0, eframe::epaint::FontFamily::Proportional))
                .frame(false)
            );
            
            disgusting_bullshit(ui, false);
            ui.add_sized(test_rect.size(), TextEdit::singleline(&mut self.entry_fields.login_details.password)
                .desired_width(test_rect.width())
                .vertical_align(egui::Align::Center)
                .text_color(Color32::WHITE)
                .hint_text("Password")
                .margin(marge)
                .font(egui::FontId::new(24.0, eframe::epaint::FontFamily::Proportional))
                .password(true)
                .frame(false)
            );

            
            toggle_ui(ui, "Remember Me", &mut self.entry_fields.login_details.remember_me);
            
            disgusting_bullshit(ui, true);
            
            if metro_button(ui, "Log in", Some(("", 24.0))).clicked() {
                self.backend.tx.send(backend::thread::UiToReso::TokenRequestCredentials(self.entry_fields.login_details.username.clone(), self.entry_fields.login_details.password.clone(), self.entry_fields.login_details.remember_me)).unwrap();
                self.can_attempt_login = false;      
                self.current_page = FrontendPage::LoadingPage;          
            }
        }
    }

    pub fn notifications_page(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.style_mut().spacing.item_spacing.y = 10.0;
        ui.horizontal(|ui| {
            ui.allocate_space(vec2(ui.style().spacing.window_margin.left, 0.0));
            ui.vertical(|ui| {
                ui.label(RichText::new("Notifications").size(30.0).color(Color32::WHITE));
                ui.label(RichText::new(self.username()).size(20.0));
                ui.allocate_space(vec2(0.0,20.0));
            });
        });
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
                ui.painter().rect_filled(rect, Rounding::same(0.0), HOVER_COL);

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

    pub fn user_search_page(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.style_mut().spacing.item_spacing.y = 10.0;
        ui.horizontal(|ui| {
            ui.allocate_space(vec2(ui.style().spacing.window_margin.left, 0.0));
            ui.vertical(|ui| {
                ui.label(RichText::new("Query Users").size(30.0).color(Color32::WHITE));
                ui.label(RichText::new(self.username()).size(20.0));
                ui.allocate_space(vec2(0.0,20.0));
            });
        });
        let marge = Margin { left: CONTENT_LEFT_PAD, right: CONTENT_RIGHT_PAD, top: 12.0, bottom: 12.0 };
        let i_size = vec2(ui.available_width(), 60.0);
        ui.style_mut().spacing.interact_size.y = 60.0;

        disgusting_bullshit(ui, false);
        let text_re = ui.add_sized(i_size, TextEdit::singleline(&mut self.entry_fields.user_info_query)
            .desired_width(i_size.x)
            .vertical_align(egui::Align::Center)
            .text_color(Color32::WHITE)
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
                        self.current_page = FrontendPage::ProfilePage(id);
                    }
                } else {
                    user_info_widget(ui, &mut self.image_cache, UserInfoVariant::Uncached(user));
                }
            }
        });
    }

    pub fn unknown_page(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.style_mut().spacing.item_spacing.y = 10.0;
        ui.horizontal(|ui| {
            ui.allocate_space(vec2(ui.style().spacing.window_margin.left, 0.0));
            ui.vertical(|ui| {
                ui.label(RichText::new("Unknown Page").size(30.0).color(Color32::WHITE));
                ui.label(RichText::new("Not Implemented").size(20.0));
                ui.allocate_space(vec2(0.0,20.0));
            });
        });
    }

    pub fn settings_page(&mut self, ui: &mut egui::Ui) {
        ui.style_mut().spacing.item_spacing.y = 10.0;
        ui.horizontal(|ui| {
            ui.allocate_space(vec2(ui.style().spacing.window_margin.left, 0.0));
            ui.vertical(|ui| {
                ui.label(RichText::new("Settings").size(30.0).color(Color32::WHITE));
                ui.label(RichText::new("[UNFINISHED/DEBUG]").size(20.0));
                ui.allocate_space(vec2(0.0,20.0));
            });
        });
        ui.style_mut().spacing.interact_size.y = 60.0;

        if metro_button(ui, "Clear persistent credentials", Some(("", 24.0))).clicked() {
            self.entry_fields.login_details.remember_me = false;
            self.entry_fields.login_details.username = "".to_owned();
            self.entry_fields.login_details.password = "".to_owned();
            if let Ok(entry) = Entry::new(KEYRING_SERVICE, KEYRING_USER) {
                entry.delete_password();
            }
        }
    }

    pub fn loading_page(&mut self, ui: &mut egui::Ui) {
        ui.put(ui.available_rect_before_wrap(), SegoeBootSpinner::new().size(150.0));
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
            match result {
                backend::thread::ResoToUi::LoggedInResponse(token, uid) => {
                    self.token = token.clone();
                    self.user_id = Some(uid.clone());
                    self.logged_in = true;
                    self.backend.tx.send(backend::thread::UiToReso::SignalConnectRequest(self.token.clone())).unwrap();
                    //self.notifications.push(icon_notification("", "SignalR Status Disabled", "SignalInitializeStatus not sent"));
                    self.backend.tx.send(backend::thread::UiToReso::SignalInitializeStatus).unwrap();
                    self.backend.tx.send(backend::thread::UiToReso::SignalRequestStatus(String::new(), false)).unwrap();
                    self.backend.tx.send(backend::thread::UiToReso::SignalListenOnKey(String::new())).unwrap();
                    self.backend.tx.send(backend::thread::UiToReso::SignalBroadcastStatus(UserStatus::new().id(uid.clone()), BroadcastTarget::new())).unwrap();
                    if self.current_page == FrontendPage::LoadingPage {
                        self.current_page = FrontendPage::ProfilePage(uid.clone());
                    }
                    if !self.entry_fields.login_details.remember_me {
                        self.entry_fields.login_details.username = "".to_owned();
                        self.entry_fields.login_details.password = "".to_owned();
                        if let Ok(entry) = Entry::new(KEYRING_SERVICE, KEYRING_USER) {
                            if let Err(err) = entry.delete_password() {
                                match err {
                                    keyring::Error::NoEntry => {
                                        //ignore it.
                                    },
                                    _ => {
                                        self.notifications.push(icon_notification("", "Keyring deletion failed", format!("{}", err).as_str()
                                        ));
                                    },
                                }
                            }
                            
                        }
                    }

                    if let Ok(entry) = Entry::new(KEYRING_SERVICE, KEYRING_USER) {
                        if self.entry_fields.login_details.remember_me {
                            if let Err(err) = entry.set_password(&token) {
                                let err_str = 
                                match err {
                                    keyring::Error::PlatformFailure(err) => format!("Platform failure"),
                                    keyring::Error::NoStorageAccess(err) => format!("No storage access"),
                                    keyring::Error::NoEntry => todo!(),
                                    keyring::Error::BadEncoding(_) => "Bad encoding".to_string(),
                                    keyring::Error::TooLong(attr, max_len) => format!("attribute \"{}\" too long (max {})", attr, max_len),
                                    keyring::Error::Invalid(attr, reason) => format!("attribute \"{}\": {}", attr, reason),
                                    keyring::Error::Ambiguous(amb) => {
                                        "Ambiguous"
                                    }.to_string(),
                                    _ => "Unknown Error".to_string(),
                                };
                                println!("Keyring error: {}", err_str);
                                self.notifications.push(FrontendNotification {
                                    icon: FrontendNotificationIcon::SegoeIcon("".to_owned()),
                                    text: "Keyring Failed".to_owned(),
                                    sub: err_str});
                            }
                        } else {
                            let _ = entry.delete_password();
                        }
                    }
                },
                backend::thread::ResoToUi::LoginFailedResponse(reason) => {
                    self.notifications.push(FrontendNotification {
                        icon: FrontendNotificationIcon::SegoeIcon("".to_owned()),
                        text: "Login failed".to_owned(),
                        sub: format!("{}", match reason {
                            api::client::LoginError::InvalidCredentials => {
                                "Invalid Credentials"
                            },
                            api::client::LoginError::JsonParseFailed => {
                                "JSON Parse Failed"
                            },
                            api::client::LoginError::RequestFailed => {
                                "HTTP Request Failed"
                            },
                            api::client::LoginError::ReachedTheEnd => {
                                "Reached the end without returning"
                            },
                        })
                    });
                    self.can_attempt_login = true;
                    
                },
                backend::thread::ResoToUi::UserInfoResponse(id, user) => {
                    if let Some(you_id) = &self.user_id.clone() {
                        if you_id.eq(&id) {
                            self.you = Some(user.clone());
                            if let Some(profile) = &user.profile {
                                self.notifications.push(FrontendNotification { icon: FrontendNotificationIcon::LoadableImage(self.image_cache.get_image(&profile.icon_url)), text: format!("Hi {}!", &user.username), sub: "You're signed in".to_owned() });
                            } else {
                                self.notifications.push(FrontendNotification { icon: FrontendNotificationIcon::SegoeIcon("".to_owned()), text: format!("Hi {}!", &user.username), sub: "You're signed in".to_owned() });
                            }
                        }
                    }

                    self.cached_user_infos.insert(id.clone(), user);

                    if self.current_page == FrontendPage::UserSearchPage {
                        self.entry_fields.user_info_query_results.push(id);
                    }
                },
                backend::thread::ResoToUi::SignalConnectFailedResponse(err) => {
                    self.notifications.push(FrontendNotification {
                        icon: FrontendNotificationIcon::SegoeIcon("".to_owned()),
                        text: "SignalR Connect failed".to_owned(),
                        sub: format!("{}", match err {
                            signalrs_client::builder::BuilderError::Negotiate { source } => {
                                match source {
                                    signalrs_client::builder::NegotiateError::Request { source } => "Negotiation Request error",
                                    signalrs_client::builder::NegotiateError::Deserialization { source } => "Negotiation Deserialization error",
                                    signalrs_client::builder::NegotiateError::Unsupported => "Negotiation Unsupported",
                                }
                            },
                            signalrs_client::builder::BuilderError::Url(_) => "Url",
                            signalrs_client::builder::BuilderError::Transport { source } => {
                                "Transport error"
                            },
                        })
                    });
                },
                backend::thread::ResoToUi::SignalRequestFailedResponse(stat) => {
                    self.notifications.push(FrontendNotification {
                        icon: FrontendNotificationIcon::SegoeIcon("".to_owned()),
                        text: "SignalR Request failed".to_owned(),
                        sub: format!("{}", match stat {
                                signalrs_client::error::ClientError::Malformed { direction, source } => { "Malformed request" },
                                signalrs_client::error::ClientError::Hub { source } => { match source {
                                    signalrs_client::hub::error::HubError::Generic { message } => "Generic hub error",
                                    signalrs_client::hub::error::HubError::Extraction { source } => "Message extraction failed",
                                    signalrs_client::hub::error::HubError::Unsupported { message } =>{"Hub feature unsupported"},
                                    signalrs_client::hub::error::HubError::Unprocessable { message } => {"Message could not be processed"},
                                    signalrs_client::hub::error::HubError::Incomprehensible { source } => {"Message could not be understood"},
                                } },
                                signalrs_client::error::ClientError::ProtocolError { message } => { "Protocol violated" },
                                signalrs_client::error::ClientError::NoResponse { message } => { "No response" },
                                signalrs_client::error::ClientError::Result { message } => { println!("{}", message); "Server error" },
                                signalrs_client::error::ClientError::TransportInavailable { message } => { "Cannot reach transport" },
                                signalrs_client::error::ClientError::Handshake { message } => { println!("{}", message); "Handshake" },
                            })
                        });
                },
                backend::thread::ResoToUi::SignalConnectedResponse => {
                    self.notifications.push(FrontendNotification { icon: FrontendNotificationIcon::SegoeIcon("".to_owned()), text: "SignalR Connected!".to_owned(), sub: format!("") });
                }
                backend::thread::ResoToUi::SignalUninitialized => {
                    self.notifications.push(FrontendNotification { icon: FrontendNotificationIcon::SegoeIcon("".to_owned()), text: "SignalR not initialized".to_owned(), sub: format!("yet tried to make a call") });
                }
                backend::thread::ResoToUi::ThreadCrashedResponse(err) => {
                    //  exclamation mark
                    self.notifications.push(FrontendNotification { icon: FrontendNotificationIcon::SegoeIcon("".to_owned()), text: "Backend Crashed".to_owned(), sub: format!("{}", err) });
                },
                backend::thread::ResoToUi::PreviousTokenInvalidResponse => {
                    if self.current_page == FrontendPage::LoadingPage {
                        self.current_page = FrontendPage::SignInPage;
                    }
                    self.can_attempt_login = true;
                },
            }
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
            let mut avail_rect = ui.available_rect_before_wrap();
            avail_rect.min.x = avail_rect.max.x - 104.0;
            let mut sidebar = ui.child_ui(avail_rect, Layout::top_down(egui::Align::Center)); {

                sidebar.style_mut().spacing.item_spacing.y = 20.0;

                sidebar.painter().rect_filled(avail_rect, Rounding::same(0.0), Color32::from_gray(6));
                sidebar.allocate_space(vec2(0.0, 0.0));
                
                // base = 20.0
                // 1-* = 40.0+SIDEBAR_ITEM_SIZE + (4.0+SIDEBAR_ITEM_SIZE) * i
                let tab = match &self.current_page {
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
                    if response.clicked() { self.current_page = FrontendPage::ProfilePage(self.user_id.clone().unwrap()); }

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
                        self.current_page = FrontendPage::LoadingPage;
                    }
                } else {
                    if sidebar.add_sized(vec2(SIDEBAR_ITEM_SIZE,SIDEBAR_ITEM_SIZE), egui::Button::new(egui::RichText::new("").size(32.0).color(Color32::WHITE)).frame(false)).clicked() {
                        self.current_page = FrontendPage::SignInPage;
                    }
                }
                
                sidebar.style_mut().spacing.item_spacing.y = 4.0;

                if sidebar_button("", &mut sidebar) { self.current_page = FrontendPage::FriendsPage; } // friends

                if sidebar_button("", &mut sidebar) { self.current_page = FrontendPage::SessionsPage; } // parties

                if sidebar_button("", &mut sidebar) { self.current_page = FrontendPage::MessagesPage; } // messages

                if sidebar_button("", &mut sidebar) { self.current_page = FrontendPage::NotificationsPage; } // notifications?

                if sidebar_button("", &mut sidebar) { self.current_page = FrontendPage::SettingsPage; } // settings

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
                
                match &self.current_page {
                    FrontendPage::SignInPage => self.signin_page(ctx, page),
                    FrontendPage::ProfilePage(id) => self.profile_page(ctx, page, id.to_string()),
                    FrontendPage::FriendsPage => self.friends_page(ctx, page),
                    FrontendPage::NotificationsPage => self.notifications_page(ctx, page),
                    FrontendPage::LoadingPage => self.loading_page(page),
                    FrontendPage::UserSearchPage => self.user_search_page(ctx, page),
                    FrontendPage::SettingsPage => self.settings_page(page),
                    FrontendPage::MessagesPage => self.messages_page(page),
                    FrontendPage::ConversationPage(id) => self.conversation_page(page, id.to_string()),
                    _ =>self.unknown_page(ctx, page)
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

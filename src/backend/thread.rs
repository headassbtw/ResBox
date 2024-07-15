use std::{collections::HashSet, future::{Future, IntoFuture}, ops::{Add, DerefMut}, str::FromStr, sync::mpsc::{Receiver, Sender}, time::{Duration, SystemTime}};
use chrono::{Date, DateTime, Timelike, Utc};
use egui::ahash::HashMap;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_json::{json, Map};
use signalrs_client::{error::ClientError, hub::{arguments::HubArgument, Hub}, SignalRClient};
use signalrs_derive::HubArgument;
use log::info;
use anyhow::Error;
use uuid::Uuid;
use lazy_static::lazy_static;

use crate::{api::{self, client::{Contact, LoginError, Message, ResDateTime}}, CONTACTS_LIST, MESSAGE_CACHE, REFRESH_UI, SESSION_CACHE, THIS_FUCKING_SUCKS, USER_STATUSES};

#[derive(Debug, Serialize_repr, Deserialize_repr, Clone, PartialEq)]
#[repr(u8)]
pub enum BroadcastGroup {
    Public,
    AllContacts,
    SpecificContacts,
    BroadcastKey,
    ConnectionIds
}

#[derive(Debug, Serialize, Clone, Deserialize, PartialEq)]
pub enum UserSessionType {
    Unknown,
    GraphicalClient,
    ChatClient,
    Headless,
    Bot,
}

#[derive(Debug, Serialize, Clone, Deserialize, PartialEq)]
pub enum OutputDevice {
    Unknown,
    Screen,
    VR,
    Camera,
}

#[derive(Debug, Serialize, Clone, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum OnlineStatus {
    Offline,
    Invisible,
    Away,
    Busy,
    Online,
    Sociable,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, HubArgument)]
#[serde(rename_all = "camelCase")]
pub struct BroadcastTarget {
    pub group : BroadcastGroup,
    pub target_ids: Vec<String>,
}

impl BroadcastTarget {
    pub fn new() -> Self {
        Self { group: BroadcastGroup::Public, target_ids: Vec::new() }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct RSAParametersData {
    exponent: String,
    modulus: String,
    p: Option<serde_json::Value>,
    q: Option<serde_json::Value>,
    #[serde(rename = "DP")]
    dp: Option<serde_json::Value>,
    #[serde(rename = "DQ")]
    dq: Option<serde_json::Value>,
    inverse_q: Option<serde_json::Value>,
    d: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, HubArgument)]
pub enum SessionAccessLevel {
    Private,
    LAN,
    Contacts,
    ContactsPlus,
    RegisteredUsers,
    Anyone,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserSessionMetadata {
    pub session_hash: String,
    pub access_level: SessionAccessLevel, // probably an enum
    pub session_hidden: bool,
    pub is_host: bool,
    pub broadcast_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, HubArgument)]
#[serde(rename_all = "camelCase")]
pub struct UserStatus {
    pub user_id: String,
    pub user_session_id: String,
    pub session_type: UserSessionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_device: Option<OutputDevice>,
    pub is_mobile: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub online_status: Option<OnlineStatus>,
    pub is_present: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_presence_timestamp: Option<ResDateTime>, // DateTime
    pub last_status_change: ResDateTime, // DateTime
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash_salt: Option<String>,
    pub app_version: String,
    pub compatibility_hash: Option<String>, 
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_r_s_a_key: Option<RSAParametersData>,
    pub sessions: Vec<UserSessionMetadata>,
    pub current_session_index: i64,
}

impl UserStatus {
    pub fn new() -> Self {
        let now = SystemTime::now();
        let now: ResDateTime = ResDateTime(now.into());


        Self {
            user_id: String::new(),
            user_session_id: Uuid::new_v4().to_string(),
            session_type: UserSessionType::ChatClient,
            output_device: Some(OutputDevice::Unknown),
            is_mobile: false,
            online_status: Some(OnlineStatus::Online),
            is_present: true,
            last_presence_timestamp: Some(now.clone()),
            last_status_change: now.clone(),
            hash_salt: None,
            app_version: "0.0.0 of null".into(),
            compatibility_hash: None,
            public_r_s_a_key: None,
            sessions: Vec::new(),
            current_session_index: -1,
        }
    }
    pub fn id(mut self, id: String) -> Self {
        self.user_id = id;
        self
    }
}

pub enum UiToReso {
    TokenRequestCredentials(String, String, bool),

    UserInfoRequest(String),
    UserStatusRequest(String),

    SignalConnectRequest(String, String),
    SignalInitializeStatus,
    SignalListenOnKey(String),
    SignalRequestStatus(Option<String>, bool),
    SignalBroadcastStatus(UserStatus, BroadcastTarget),
    SignalSendMessage(String, String),

    ShutdownRequest,
}
pub enum ResoToUi {
    LoggedInResponse(String, String),
    LoginFailedResponse(LoginError),
    PreviousTokenInvalidResponse,

    SignalConnectedResponse,
    SignalConnectFailedResponse(signalrs_client::builder::BuilderError),
    SignalRequestFailedResponse(ClientError),
    SignalUninitialized,

    UserInfoResponse(String, api::client::UserInfo),

    ThreadCrashedResponse(anyhow::Error)
}

pub struct BackendThread {
    pub rx: Receiver<ResoToUi>,
    pub tx: Sender<UiToReso>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, HubArgument)]
#[serde(rename_all = "camelCase")]
struct RecordId {
    record_id: String,
    owner_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, HubArgument)]
#[serde(rename_all = "camelCase")]
pub struct SessionUpdate {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    //#[serde(skip_serializing_if = "Option::is_none")]
    //corresponding_world_id: Option<RecordId>,
    pub tags: HashSet<String>,
    pub session_id: String,
    pub normalized_session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_user_session_id: Option<String>,
    pub host_machine_id: String,
    pub host_username: String,
    pub compatibility_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub universe_id: Option<String>,
    pub app_version: String,
    pub headless_host: bool, // todo: rename is_headless_host?
    #[serde(rename = "sessionURLs")]
    pub session_urls: Vec<String>,
    pub parent_session_ids: Vec<String>,
    pub nested_session_ids: Vec<String>,
    //session_users: Vec<SessionUser>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    pub joined_users: u32,
    pub active_users: u32,
    pub total_joined_users: u32,
    pub total_active_users: u32,
    pub max_users: u32,
    pub mobile_friendly: bool,
    pub session_begin_time: ResDateTime,
    pub last_update: String,
    //access_level: SessionAccessLevel,
    pub hide_from_listing: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub broadcast_key: Option<String>,
    pub has_ended: bool,
    pub is_valid: bool,
}


pub enum InitialLoginType {
    Fresh,
    PreviousToken {
        username: String,
        session_token: String,
    }
}

#[derive(Debug, Serialize, Deserialize, HubArgument)]
struct HubArgumentValue(serde_json::Value);

async fn status_update(message: UserStatus) {
    println!("Recieved status update for {}", message.user_id);
    
    if message.hash_salt.is_some() {
        for (id, session) in SESSION_CACHE.lock().iter() {

            let to_digest = format!("{}{}", id, message.hash_salt.clone().unwrap());
            let digested = sha256::digest(to_digest).to_ascii_uppercase();
            THIS_FUCKING_SUCKS.lock().insert(digested, session.session_id.clone());
        }
    }

    let mut statuses = USER_STATUSES.lock();
    statuses.insert(message.user_id.clone(), message);
    *REFRESH_UI.lock().deref_mut() = true;
}

async fn server_log(message: String) {
    println!("Reso server: {}", message);
}

async fn session_update(message: SessionUpdate) {
    let mut list = SESSION_CACHE.lock();
    let key = message.session_id.clone();
    list.insert(key, message);
    *REFRESH_UI.lock().deref_mut() = true;
}

async fn message_receive(message: String) {
    println!("message received: {}", message);
    *REFRESH_UI.lock().deref_mut() = true;
}

async fn message_sent(message: String) {
    println!("message sent: {}", message);
    *REFRESH_UI.lock().deref_mut() = true;
}



impl BackendThread {
    pub fn new(ctx: &egui::Context, creds: InitialLoginType) -> Self {
        let (tx0, rx1) = std::sync::mpsc::channel();
        let (tx1, rx0) = std::sync::mpsc::channel();
        let context = ctx.clone();
        let tx00 = tx0.clone();
        tokio::task::spawn(async move {
            let tx11 = tx1.clone();
            let result = BackendThread::run(rx1, tx00, tx1, &context, creds).await;
            if let Err(res) = result {
                tx11.send(ResoToUi::ThreadCrashedResponse(res)).unwrap();
            }
        });

        Self { rx: rx0, tx: tx0}
    }

    async fn run(
        rx1: Receiver<UiToReso>,
        tx0: Sender<UiToReso>,
        tx1: Sender<ResoToUi>,
        ctx: &egui::Context,
        creds: InitialLoginType
    ) -> anyhow::Result<()> {
        let mut client: Option<SignalRClient> = None;
        let mut api_client = api::client::Client::new();

        match creds {
            InitialLoginType::PreviousToken { username, session_token } => {
                let api_login = api_client.login(&username, api::client::UserSessionsAuthReq::Token {
                    _type: "sessionToken".to_owned(), session_token
                }, true).await; // assume remember me, because why not tbh
                if let core::result::Result::Ok(token) = api_login {
                    let your_id = api_client.user_id.clone().unwrap();
                    tx1.send(ResoToUi::LoggedInResponse(token, your_id.clone())).unwrap();
                    if let Ok(you) = api_client.get_user(&your_id.clone()).await {
                        tx1.send(ResoToUi::UserInfoResponse(your_id.clone(), you)).unwrap();
                        
                        api_client.get_contacts(&your_id.clone()).await;
                        api_client.get_messages(&your_id.clone()).await;
                        api_client.get_sessions().await;
                    } else {
                        println!("uh? whoops?");
                    }
                } else {
                    println!("previous tokens invalid, boowomp {:?}", api_login.err().unwrap());
                    tx1.send(ResoToUi::PreviousTokenInvalidResponse).unwrap();
                }
            },
            InitialLoginType::Fresh => {
                tx1.send(ResoToUi::PreviousTokenInvalidResponse).unwrap();
            },
        }
        ctx.request_repaint();
       
        let mut future  = SystemTime::now();
        future = future.checked_add(Duration::from_secs(10)).unwrap();

        'outer: loop {
            let now = SystemTime::now();
            if now >= future {
                future = now.checked_add(Duration::from_secs(10)).unwrap();
                tx0.send(UiToReso::SignalRequestStatus(None, false)).unwrap(); // this sucks!
            }
            {
                let mut boolin = REFRESH_UI.lock();
                if *boolin {
                    ctx.request_repaint();
                    *boolin = false;
                }
            }
            let request = rx1.try_recv();
            if request.is_err() {
                continue;
            }

            match request? {
                UiToReso::TokenRequestCredentials(username, pass, remember) => {
                    let api_login = api_client.login(&username, api::client::UserSessionsAuthReq::Credentials {
                        _type: "password".to_owned(), password: pass
                    }, remember).await;
                    if let core::result::Result::Ok(token) = api_login {
                        let your_id = api_client.user_id.clone().unwrap();
                        tx1.send(ResoToUi::LoggedInResponse(token, your_id.clone())).unwrap();
                        if let Ok(you) = api_client.get_user(&your_id.clone()).await {
                            tx1.send(ResoToUi::UserInfoResponse(your_id.clone(), you)).unwrap();
                            api_client.get_contacts(&your_id.clone()).await;
                            api_client.get_messages(&your_id.clone()).await;
                        } else {
                            println!("uh? whoops?");
                        }
                    } else {
                        tx1.send(ResoToUi::LoginFailedResponse(api_login.err().unwrap())).unwrap();
                    }
                    ctx.request_repaint();
                },
                UiToReso::SignalConnectRequest(id, token) => {
                    let hub = Hub::default()
                    .method("ReceiveStatusUpdate", status_update)
                    .method("Debug", server_log)    
                    .method("ReceiveMessage", message_receive)
                    .method("MessageSent", message_sent)
                    .method("ReceiveSessionUpdate", session_update)
                    ;

                    let result = SignalRClient::builder("api.resonite.com")
                    .use_hub("hub")
                    .with_client_hub(hub)
                    .use_authentication(signalrs_client::builder::Auth::Resonite { uid: api_client.hwid.clone(), id, token })
                    //.use_unencrypted_connection()
                    .build().await;
                    if let core::result::Result::Ok(r_client) = result {
                        client = Some(r_client);
                        tx1.send(ResoToUi::SignalConnectedResponse).unwrap();
                    } else {
                        tx1.send(ResoToUi::SignalConnectFailedResponse(result.err().unwrap())).unwrap();
                    }
                    ctx.request_repaint();
                },
                UiToReso::SignalInitializeStatus => {
                    if let Some(client) = &client {
                        let func_res = client
                        .method("InitializeStatus")
                        .invoke::<HashMap<String, Vec<Contact>>>();
                        let fut = func_res.await;
                        if let Err(res) = fut {
                            println!("signal request failed: {:?}", res);
                            tx1.send(ResoToUi::SignalRequestFailedResponse(res)).unwrap();
                        }
                    } else { tx1.send(ResoToUi::SignalUninitialized).unwrap(); }
                },
                UiToReso::SignalRequestStatus(id, invis) => {
                    if let Some(client) = &client {
                        let func = client.method("RequestStatus").arg(id);
                        let func_result = if let Ok(build) = func {
                            let build = build.arg(invis);
                            if let Ok(build) = build {
                                build.invoke_unit().await
                            } else { println!("SignalR invocation arg failed: {:?}", build.err().unwrap()); continue; }
                        } else { println!("SignalR invocation build failed: {:?}", func.err().unwrap()); continue; };
                        if let Err(msg) = func_result {
                            println!("SignalR invocation failed: {:?}", msg);
                            tx1.send(ResoToUi::SignalRequestFailedResponse(msg)).unwrap();
                        } else {
                            //println!("guh");
                        }
                    } else { tx1.send(ResoToUi::SignalUninitialized).unwrap(); }
                },
                UiToReso::SignalBroadcastStatus(a, b) => {
                    // cache it for ourselfs first
                    { USER_STATUSES.lock().insert(a.user_id.clone(), a.clone()); }
                    if let Some(client) = &client {
                        let res = client.method("BroadcastStatus").arg(a)
                        .and_then(|build| build.arg(b));
                        if let Ok(res) = res {
                            if let Err(msg) = res.invoke_unit().await {
                                tx1.send(ResoToUi::SignalRequestFailedResponse(msg)).unwrap();
                            }
                        } else if let Err(msg) = res {
                            tx1.send(ResoToUi::SignalRequestFailedResponse(msg)).unwrap();
                        }
                    } else { tx1.send(ResoToUi::SignalUninitialized).unwrap(); }
                },
                UiToReso::UserInfoRequest(uid) => {
                    let uinfo = api_client.get_users(&uid.clone()).await;
                    if let Ok(user) = uinfo {
                        for user in user {
                            tx1.send(ResoToUi::UserInfoResponse(user.id.clone(), user)).unwrap();
                        }
                        ctx.request_repaint();
                    } else {
                        println!("{:?}",uinfo.err().unwrap());
                    }
                },
                UiToReso::SignalSendMessage(uid, content) => {
                    let now = SystemTime::now();
                    let now: DateTime<Utc> = now.into();
                    let now = ResDateTime(now);
                    let send = Message {
                        id: String::new(),
                        sender_id: api_client.user_id.clone().unwrap(),
                        recipient_id: uid.clone(),
                        other_id: uid.clone(),
                        message_type: api::client::MessageType::Text,
                        content,
                        send_time: now.clone(),
                        last_update_time: now,
                        read_time: None,
                        is_migrated: true,
                        owner_id: api_client.user_id.clone().unwrap(),
                    };

                    if let Some(client) = &client {
                        let res = client.method("SendMessage").arg(send);
                        if let Ok(res) = res {
                            if let Err(msg) = res.invoke_unit().await {
                                tx1.send(ResoToUi::SignalRequestFailedResponse(msg)).unwrap();
                            }
                        } else if let Err(msg) = res {
                            tx1.send(ResoToUi::SignalRequestFailedResponse(msg)).unwrap();
                        }
                    } else { tx1.send(ResoToUi::SignalUninitialized).unwrap(); }
                },
                UiToReso::SignalListenOnKey(key) => {
                    if let Some(client) = &client {
                        let res = client.method("ListenOnKey").arg(key);
                        if let Ok(res) = res {
                            if let Err(msg) = res.invoke_unit().await {
                                tx1.send(ResoToUi::SignalRequestFailedResponse(msg)).unwrap();
                            }
                        } else if let Err(msg) = res {
                            tx1.send(ResoToUi::SignalRequestFailedResponse(msg)).unwrap();
                        }
                    } else { tx1.send(ResoToUi::SignalUninitialized).unwrap(); }
                },
                UiToReso::UserStatusRequest(id) => {
                    api_client.get_status(&id).await;
                },
                UiToReso::ShutdownRequest => break 'outer Ok(()),
            }
        }
    }
}
use std::{fmt, future::Future, process::Output, sync::Arc};

use chrono::{DateTime, Utc};
use log::{debug, error};
use reqwest::{self, header, Method, Request};
use hardware_id;
use uuid::Uuid;
use serde::{self, de::{MapAccess, Visitor}, ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};
use sha256::digest;

use crate::{CONTACTS_LIST, MESSAGE_CACHE};

pub struct Client {
    req: Option<reqwest::Client>,
    uuid: Uuid,
    pub hwid: String,
    pub logged_in: bool,
    pub user_id: Option<String>,
    token: Option<String>
}

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug)]
#[serde(untagged)]
pub enum UserSessionsAuthReq {
    Credentials {
        #[serde(rename = "$type")]
        _type: String,
        password: String,
    },
    Token {
        #[serde(rename = "$type")]
        _type: String,
        #[serde(rename = "sessionToken")]
        session_token: String,
    }
}

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug)]
#[serde(rename_all = "camelCase")]
struct UserSessionsReq {
    username: String,
    authentication: UserSessionsAuthReq,
    secret_machine_id: String,
    remember_me: bool,
}

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug)]
#[serde(rename_all = "camelCase")]
struct Entity {
    user_id: String,
    token: String,
    created: String,
    expire: String,
    remember_me: bool,
    secret_machine_id_hash: String,
    secret_machine_id_salt: String,
    uid_hash: String,
    uid_salt: String,
    original_login_type: String,
    original_login_id: String,
    logout_url_client_side: bool,
    session_login_counter: u64,
    #[serde(rename = "sourceIP")]
    source_ip: String,
    user_agent: String,
    is_machine_bound: bool,
    partition_key: String,
    row_key: String,
}

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug)]
struct ConfigFile {
    path: String,
    content: String,
}

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug)]
struct LoginResponse {
    entity: Entity,
    #[serde(rename = "configFiles")]
    #[serde(skip_serializing_if = "Option::is_none")]
    config_files: Option<Vec<ConfigFile>>
}

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub icon_url: String,
    pub display_badges: Vec<()>
}

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug)]
struct QuotaBytesSources {
    base: u64
}

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug)]
#[serde(rename_all = "camelCase")]
struct MigratedData {
    username: String,
    user_id: String,
    quota_bytes: String,
    quota_bytes_sources: QuotaBytesSources,
    registration_date: String,
}

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub normalized_username: String,
    pub registration_date: String,
    pub is_verified: bool,
    pub is_locked: bool,
    pub supress_ban_evasion: bool,
    #[serde(rename = "2fa_login")]
    pub twofactor_login: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<UserProfile>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    pub id: String,
    pub contact_username: String,
    pub contact_status: String,
    pub is_accepted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<UserProfile>,
    pub latest_message_time: ResDateTime,
    pub is_migrated: bool,
    pub is_counterpart_migrated: bool,
    pub owner_id: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub enum MessageType {
    Text,
    Object,
    Sound,
    SessionInvite,
}

#[derive(Debug, Clone)]
pub struct ResDateTime(pub DateTime<Utc>);
struct ResDateTimeVisitor;

impl<'de> Visitor<'de> for ResDateTimeVisitor {
    type Value = ResDateTime;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an RFC-3339 string")
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where E: serde::de::Error, {
        let string = if !v.contains("Z") && !v.contains("+"){ 
            // so you don't have to google it, Z is effectively "+00:00", aka UTC
            v.to_owned() + "Z"
        } else { v.to_owned() };

        let res = DateTime::parse_from_rfc3339(&string);
        if let Ok(dt) = res {
            Ok(ResDateTime(DateTime::<Utc>::from(dt)))
        } else {    
            Err(serde::de::Error::custom(format!("&str: \"{}\" {}", string, res.err().unwrap())))
        }
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where E: serde::de::Error, {
        self.visit_borrowed_str(&v)
    }
}

impl Serialize for ResDateTime {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_rfc3339())
    }
}

impl<'de> Deserialize<'de> for ResDateTime {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<ResDateTime, D::Error> {
        deserializer.deserialize_string(ResDateTimeVisitor)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub sender_id: String,
    pub recipient_id: String,
    pub other_id: String,
    pub message_type: MessageType,
    pub content: String,
    pub send_time: ResDateTime,
    pub last_update_time: ResDateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_time: Option<ResDateTime>,
    pub is_migrated: bool,
    pub owner_id: String,
}

#[derive(std::fmt::Debug)]
pub enum LoginError {
    InvalidCredentials,
    JsonParseFailed,
    RequestFailed,
    ReachedTheEnd
}

#[derive(std::fmt::Debug)]
pub enum UserInfoError {
    /// Profile JSON parsing failed
    JsonParseFailed,
    /// Request failed to send
    RequestFailed,
    /// Either no profile for UID, or no matches for username
    NoResults,
    /// No API connector is present
    NoClient
}

impl Client {
    pub fn new() -> Self {
        let hwid = hardware_id::get_id().expect("couldn't get HWID");
        let client = reqwest::Client::builder().user_agent("some fuckass rust app that looks like the 2015 xbox one guide").build();

        let mut array_tmp: [u8; 16] = Default::default();
        //hwid.clone().split_off(16).as_bytes().try_into().unwrap();

        let mut a = hwid.clone();
        let _ = a.split_off(16);

        array_tmp.copy_from_slice(a.as_bytes());

        let hwid: String = digest(hwid);

        let client = if let Ok(client) = client {
            Some(client)
        } else {
            None
        };
        Self {
            req: client,
            uuid: Uuid::from_bytes(array_tmp),
            hwid,
            logged_in: false,
            user_id: None,
            token: None,
        }
    }

    /// Returns your token if sucessfull
    pub async fn login(&mut self, username: &str, auth_variant: UserSessionsAuthReq, remember_me: bool) -> Result<String, LoginError> {
        if let Some(client) = &self.req {
            let mut headers = header::HeaderMap::new();
            headers.insert("UID", header::HeaderValue::from_str(&self.hwid).expect("man"));
            headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());

            let body: UserSessionsReq = UserSessionsReq {
                username: username.to_owned(),
                authentication: auth_variant,
                secret_machine_id: self.uuid.to_owned().into(),
                remember_me
            };
            let request = client.post("https://api.resonite.com/userSessions")
            .body(serde_json::to_string(&body).unwrap()).headers(headers);
        

            let response = if let Ok(r) = request.send().await { r } else { return Err(LoginError::RequestFailed) };
            let response = &response.bytes().await;
            let jason_bytes = if let Ok(r) = response { r } else { return Err(LoginError::RequestFailed) };
            
            let jason: &str = if let Ok(string) = std::str::from_utf8(&jason_bytes) { string } else { return Err(LoginError::JsonParseFailed) };
            if jason.eq("Login.InvalidCredentials") { return Err(LoginError::InvalidCredentials) }
            let resp: LoginResponse = if let Ok(resp) = serde_json::from_str(jason) { resp } else { println!("{:?}", jason); return Err(LoginError::JsonParseFailed) };
            
            self.user_id = Some(resp.entity.user_id.clone());
            self.logged_in = true;
            self.token = Some(resp.entity.token.clone());

            

            return Ok(resp.entity.token)
        }
        Err(LoginError::ReachedTheEnd)
    }

    pub async fn get_users(&mut self, id: &str) -> Result<Vec<UserInfo>, UserInfoError>{
        let is_by_username = !id.to_lowercase().get(..2).eq(&Some("u-"));

        let jason = if let Some(guh) = self.get_json(&format!("users{}", if is_by_username { format!("?name={}", id) } else { format!("/{}", id) })).await {
            guh
        } else { return Err(UserInfoError::RequestFailed); };

        if jason.eq("Invalid User ID") { return Err(UserInfoError::NoResults); }

        let user: Result<Vec<UserInfo>, serde_json::Error> = serde_json::from_str(&jason);
        if let Ok(user) = user {
            return Ok(user);
        } else {
            println!("couldn't deserialize data: {}", user.err().unwrap());
            println!("{}", jason);
            Err(UserInfoError::JsonParseFailed)
        }
    }

    pub async fn get_user(&mut self, id: &str) -> Result<UserInfo, UserInfoError>{
        let jason = if let Some(guh) = self.get_json(&format!("users/{}", id)).await {
            guh
        } else { return Err(UserInfoError::RequestFailed); };

        if jason.eq("Invalid User ID") { return Err(UserInfoError::NoResults); }

        let user: Result<UserInfo, serde_json::Error> = serde_json::from_str(&jason);
        if let Ok(user) = user {
            return Ok(user);
        } else {
            println!("couldn't deserialize data: {}", user.err().unwrap());
            println!("{}", jason);
            Err(UserInfoError::JsonParseFailed)
        }
    }

    pub async fn get_contacts(&mut self, id: &str) {
        let jason = if let Some(guh) = self.get_json(&format!("users/{}/contacts", id)).await {
            guh
        } else { return; };

        let user_parse_res = serde_json::from_str(&jason);

        let users: Vec<Contact> = if let Ok(res) = user_parse_res { res } else { println!("{}", user_parse_res.err().unwrap()); println!("{}", jason); return; };

        {
            let mut list = CONTACTS_LIST.lock();
            for user in users {
                list.insert(user.id.clone(), user);
            }   
        }
    }

    pub async fn get_messages(&mut self, id: &str) {
        let jason = if let Some(guh) = self.get_json(&format!("users/{}/messages", id)).await {
            guh
        } else { return; };

        let messages_parse_res = serde_json::from_str(&jason);
        let messages: Vec<Message> = if let Ok(res) = messages_parse_res { res } else { println!("{}", messages_parse_res.err().unwrap()); println!("{}", jason); return; };

        {
            let mut cache = MESSAGE_CACHE.lock();
            for message in messages {
                let vec = if let Some(vec) = cache.get_mut(&message.other_id) {
                    vec
                } else {
                    cache.insert(message.other_id.clone(), Vec::new());
                    cache.get_mut(&message.other_id).expect("bro")
                };

                vec.push(message);
            }

            for (_, msg) in cache.iter_mut() {
                msg.sort_by(|a, b| a.last_update_time.0.cmp(&b.last_update_time.0));
            }
        }
    }

    pub async fn get_status(&mut self, id: &str) {
        if let Some(guh) = self.get_json(&format!("users/{}/status", id)).await {
            println!("user: {}", guh);
        }
    }

    async fn get_json(&mut self, endpoint: &str) -> Option<String> {
        let client = if let Some(client) = &self.req { client } else { return None };

        let mut headers = header::HeaderMap::new();
        headers.insert("UID", header::HeaderValue::from_str(&self.hwid).expect("man"));
        headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
        headers.insert("Authorization", if let Ok(res) = header::HeaderValue::from_str(&format!("res {}:{}", 
            if let Some(res) = &self.user_id.clone() { res } else {return None },
            if let Some(res) = &self.token.clone() { res } else {return None })) { res } else { return None }
        );

        let request = client
        .get(format!("https://api.resonite.com/{}", endpoint))
        .headers(headers);

        let response = if let Ok(res) = request.send().await { res } else { println!("boowomp"); return None };
        if response.status().is_client_error() { println!("/{} errored! {:?}", endpoint, response.error_for_status()); return None }
        
        
        let jason_bytes = if let Ok(res) = response.bytes().await { res } else { return None };
        let jason: &str = if let Ok(res) = std::str::from_utf8(&jason_bytes) { res } else { return None };
        Some(jason.to_owned())
    }
}
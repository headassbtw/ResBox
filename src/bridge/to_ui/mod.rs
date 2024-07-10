use crate::{backend::{self, thread::ResoToUi}, icon_notification, FrontendNotification, FrontendNotificationIcon, FrontendPage, TemplateApp};

// i don't like doing one function per file, but some of these functions are really long, it's somewhat justified

mod logged_in;
use logged_in::logged_in;

pub fn process_to_ui(app: &mut TemplateApp, variant: ResoToUi) {
    match variant {
        backend::thread::ResoToUi::LoggedInResponse(token, uid) => logged_in(app, token, uid),
        backend::thread::ResoToUi::LoginFailedResponse(reason) => {
            app.notifications.push(icon_notification("", "Login failed", &format!("{}", reason)));
            app.can_attempt_login = true;
        }
        backend::thread::ResoToUi::UserInfoResponse(id, user) => {
            if app.is_you(&id) {
                app.you = Some(user.clone());
                if let Some(profile) = &user.profile {
                    app.notifications.push(FrontendNotification { icon: FrontendNotificationIcon::LoadableImage(app.image_cache.get_image(&profile.icon_url)), text: format!("Hi {}!", &user.username), sub: "You're signed in".to_owned() });
                } else {
                    app.notifications.push(icon_notification("", &format!("Hi {}!", &user.username), "You're signed in"));
                }
            }

            app.cached_user_infos.insert(id.clone(), user);
            if app.current_page() == &FrontendPage::UserSearchPage {
                app.entry_fields.user_info_query_results.push(id);
            }
        }
        backend::thread::ResoToUi::SignalConnectFailedResponse(err) => {
            app.notifications.push(FrontendNotification {
                icon: FrontendNotificationIcon::SegoeIcon("".to_owned()),
                text: "SignalR Connect failed".to_owned(),
                sub: format!("{}", match err {
                    signalrs_client::builder::BuilderError::Negotiate { source } => {
                        match source {
                            signalrs_client::builder::NegotiateError::Request { source } => {
                                println!("Negotiation error: {}", source);
                                format!("Negotiation Request error")
                            },
                            signalrs_client::builder::NegotiateError::Deserialization { source } => format!("Negotiation Deserialization error"),
                            signalrs_client::builder::NegotiateError::Unsupported => format!("Negotiation Unsupported"),
                        }
                    },
                    signalrs_client::builder::BuilderError::Url(_) => format!("Url"),
                    signalrs_client::builder::BuilderError::Transport { source } => {
                        format!("Transport error: {}", source)
                    },
                })
            });
        }
        backend::thread::ResoToUi::SignalRequestFailedResponse(stat) => {
            app.notifications.push(FrontendNotification {
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
                        signalrs_client::error::ClientError::Result { message } => { println!("server error: {}", message); "Server error" },
                        signalrs_client::error::ClientError::TransportInavailable { message } => { "Cannot reach transport" },
                        signalrs_client::error::ClientError::Handshake { message } => { println!("{}", message); "Handshake" },
                    })
                });
        }
        backend::thread::ResoToUi::SignalConnectedResponse => app.notifications.push(icon_notification("", "SignalR Connected!", "")),
        backend::thread::ResoToUi::SignalUninitialized => app.notifications.push(icon_notification("", "SignalR not initialized", "yet tried to make a call")),
        backend::thread::ResoToUi::ThreadCrashedResponse(err) => {
            //  exclamation mark
            app.notifications.push(icon_notification("", "Backend Crashed", &format!("{}", err)));
        }
        backend::thread::ResoToUi::PreviousTokenInvalidResponse => {
            if app.current_page() == &FrontendPage::LoadingPage {
                app.set_page(FrontendPage::SignInPage);
            }
            app.can_attempt_login = true;
        }
    }

}
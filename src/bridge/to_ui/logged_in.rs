use crate::{backend::{self, thread::{BroadcastTarget, UserStatus}}, icon_notification, FrontendNotification, FrontendPage, TemplateApp, KEYRING_SERVICE, KEYRING_USER};

pub fn logged_in(app: &mut TemplateApp, token: String, user_id: String) {
    app.token = token.clone();
    app.user_id = Some(user_id.clone());
    app.logged_in = true;
    app.backend.tx.send(backend::thread::UiToReso::SignalConnectRequest(user_id.clone(), app.token.clone())).unwrap();
    //app.notifications.push(icon_notification("", "SignalR Status Disabled", "SignalInitializeStatus not sent"));
    app.backend.tx.send(backend::thread::UiToReso::SignalRequestStatus(None, false)).unwrap(); // might be polling? idk?
    app.backend.tx.send(backend::thread::UiToReso::SignalInitializeStatus).unwrap();
    //app.backend.tx.send(backend::thread::UiToReso::SignalListenOnKey(String::new())).unwrap();
    app.backend.tx.send(backend::thread::UiToReso::SignalBroadcastStatus(UserStatus::new().id(user_id.clone()), BroadcastTarget::new())).unwrap();
    if app.current_page() == &FrontendPage::LoadingPage {
        app.page_stack.remove(app.current_page);
        app.page_stack.insert(app.current_page, FrontendPage::ProfilePage(user_id.clone()));
    }
    if !app.entry_fields.login_details.remember_me {
        app.entry_fields.login_details.username = "".to_owned();
        app.entry_fields.login_details.password = "".to_owned();
        if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER) {
            if let Err(err) = entry.delete_password() {
                match err {
                    keyring::Error::NoEntry => {
                        // we don't have an entry to clear, we already accomplished what we wanted
                    },
                    _ => {
                        app.notifications.push(icon_notification("", "Keyring deletion failed", format!("{}", err).as_str()
                        ));
                    },
                }
            }
            
        }
    }

    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER) {
        if app.entry_fields.login_details.remember_me {
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
                app.notifications.push(icon_notification("","Keyring Failed",&err_str));
            }
        } else {
            let _ = entry.delete_password();
        }
    }
}
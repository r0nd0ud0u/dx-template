use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::{
    common::{CtxConnectedUsers, DISCONNECTED_USER},
    pages::login_page::LoginPage,
};

/// Home page — a signed-out visitor sees the login/register cards; a signed-in user
/// sees a welcome message and the live connected-users list (this skeleton's demo of
/// the websocket -> shared server state -> broadcast pipeline, see
/// `websocket_handler::event::broadcast_connected_users`). Replace this body with your
/// own app/game's home content once you build on top of the template.
#[component]
pub fn Home() -> Element {
    // contexts
    let local_login_name_session = use_context::<Signal<String>>();
    let connected_users = use_context::<CtxConnectedUsers>().0;
    // Snapshot for this render
    let user_name = local_login_name_session();

    if user_name == *DISCONNECTED_USER {
        rsx! {
            LoginPage {}
        }
    } else {
        rsx! {
            div { class: "home-container",
                div { class: "rotate-scale-up",
                    h1 { class: "rpg-title", {t!("home-title")} }
                }
                p { class: "rpg-subtitle", {t!("home-welcome", user_name : user_name.clone())} }
                div { class: "rpg-card auth-card",
                    p { class: "auth-section-title", {t!("home-connected-users-title")} }
                    if connected_users().is_empty() {
                        p { class: "rpg-answer", {t!("home-connected-users-empty")} }
                    } else {
                        ul { class: "connected-users-list",
                            for name in connected_users() {
                                li { key: "{name}",
                                    span { class: "connected-users-dot" }
                                    "{name}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

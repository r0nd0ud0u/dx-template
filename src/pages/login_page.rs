use dioxus::fullstack::CborEncoding;
use dioxus::prelude::*;
use dioxus::{fullstack::UseWebsocket, logger::tracing};
use dioxus_i18n::t;

use crate::auth_manager::server_fn::get_user_id;
use crate::websocket_handler::NO_CLIENT_ID;
use crate::websocket_handler::event::{ClientEvent, ServerEvent};
use crate::{
    auth_manager::server_fn::{get_use_password, login, register},
    common::{CtxDeviceToken, Route},
    components::{
        button::{Button, ButtonVariant},
        input::Input,
    },
};

#[component]
pub fn LoginPage() -> Element {
    // contexts
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let mut local_login_name_session = use_context::<Signal<String>>();
    let mut local_login_id_session = use_context::<Signal<i64>>();
    let mut device_token = use_context::<CtxDeviceToken>().0;
    // nav
    let navigator = use_navigator();
    // logon
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut logon_answer = use_signal(String::new);
    let set_username = move |e: FormEvent| username.set(e.value());
    let set_password = move |e: FormEvent| password.set(e.value());
    // register
    let mut register_name = use_signal(String::new);
    let mut register_password = use_signal(String::new);
    let mut register_answer = use_signal(String::new);
    let set_register = move |e: FormEvent| register_name.set(e.value());
    let set_register_pw = move |e: FormEvent| register_password.set(e.value());

    // Fetch the USE_PASSWORD flag from the server. Deliberately a client-only
    // use_effect + spawn (not use_resource): use_resource's value gets resolved during
    // SSR and embedded in the page for hydration, which hits a known Dioxus hydration
    // bug (https://github.com/DioxusLabs/dioxus/issues/3583) that crashes the client
    // with "Error deserializing data: Semantic(Some(0), \"expected bool\")" and leaves
    // the whole page unresponsive (can't type, clicks land on stale handlers).
    let mut use_pw = use_signal(|| false);
    use_effect(move || {
        spawn(async move {
            if let Ok(v) = get_use_password().await {
                use_pw.set(v);
            }
        });
    });

    rsx! {
        div { class: "home-container",
            div { class: "rotate-scale-up",
                h1 { class: "rpg-title", {t!("home-title")} }
            }
            div { class: "auth-grid",
                // --- Sign in card ---
                div { class: "rpg-card auth-card",
                    p { class: "auth-section-title", {t!("login-sign-in-title")} }
                    Input {
                        placeholder: t!("login-username-placeholder"),
                        r#type: "text",
                        value: "{username}",
                        oninput: set_username,
                    }
                    if use_pw() {
                        Input {
                            placeholder: t!("login-password-placeholder"),
                            r#type: "password",
                            value: "{password}",
                            oninput: set_password,
                        }
                    }
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| async move {
                            if username().trim().is_empty() || (use_pw() && password().trim().is_empty()) {
                                logon_answer
                                    .set(
                                        if use_pw() {
                                            t!("login-empty-fields")
                                        } else {
                                            t!("login-empty-username")
                                        },
                                    );
                                return;
                            }
                            tracing::info!("Attempting to log in with username: {}", username());
                            match login(username(), password(), use_pw()).await {
                                Ok(proof) => {
                                    logon_answer.set(t!("login-success", username : username()));
                                    // The server only just handed us this proof because a real
                                    // login succeeded — persist it as our device_token so
                                    // AddPlayer/LoginAllSessions can prove to the websocket that
                                    // we're actually allowed to claim this username.
                                    device_token.set(proof.clone());
                                    match get_user_id().await {
                                        Ok(sql_id) => {
                                            *local_login_id_session.write() = sql_id;
                                            *local_login_name_session.write() = username();
                                            let _ = socket
                                                .clone()
                                                .send(
                                                    ClientEvent::LoginAllSessions(
                                                        username(),
                                                        sql_id,
                                                        proof.clone(),
                                                    ),
                                                )
                                                .await;
                                            navigator.push(Route::Home {});
                                            sql_id
                                        }
                                        Err(e) => {
                                            tracing::info!("{}", e.to_owned());
                                            -1
                                        }
                                    };
                                }
                                Err(e) => {
                                    tracing::info!("{}", e.to_owned());
                                    logon_answer.set(format!("{}", e.to_owned()));
                                }
                            }
                        },
                        {t!("login-sign-in-button")}
                    }
                    if !logon_answer().is_empty() {
                        p { class: "rpg-answer", "{logon_answer}" }
                    }
                }
                // --- Sign up card ---
                div { class: "rpg-card auth-card",
                    p { class: "auth-section-title", {t!("login-create-account-title")} }
                    Input {
                        placeholder: t!("login-choose-username-placeholder"),
                        r#type: "text",
                        value: "{register_name}",
                        oninput: set_register,
                    }
                    if use_pw() {
                        Input {
                            placeholder: t!("login-choose-password-placeholder"),
                            r#type: "password",
                            value: "{register_password}",
                            oninput: set_register_pw,
                        }
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| async move {
                            if register_name().trim().is_empty()
                                || (use_pw() && register_password().trim().is_empty())
                            {
                                register_answer
                                    .set(
                                        if use_pw() {
                                            t!("login-empty-fields")
                                        } else {
                                            t!("login-empty-username")
                                        },
                                    );
                                return;
                            }
                            match register(register_name(), register_password(), use_pw()).await {
                                Ok(()) => {
                                    match login(register_name(), register_password(), use_pw()).await {
                                        Ok(proof) => {
                                            device_token.set(proof.clone());
                                            *local_login_name_session.write() = register_name();
                                            let sql_id = (get_user_id().await)
                                                .unwrap_or(NO_CLIENT_ID);
                                            *local_login_id_session.write() = sql_id;
                                            let _ = socket
                                                .clone()
                                                .send(
                                                    ClientEvent::LoginAllSessions(
                                                        register_name(),
                                                        sql_id,
                                                        proof,
                                                    ),
                                                )
                                                .await;
                                            navigator.push(Route::Home {});
                                        }
                                        Err(e) => {
                                            tracing::info!("{}", e.to_owned());
                                            register_answer.set(t!("login-invalid-login"));
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::info!("{}", e.to_owned());
                                    register_answer.set(t!("login-name-taken"));
                                }
                            }
                        },
                        {t!("login-sign-up-button")}
                    }
                    if !register_answer().is_empty() {
                        p { class: "rpg-answer-error", "{register_answer}" }
                    }
                }
            }
        }
    }
}

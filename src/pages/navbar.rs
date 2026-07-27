use dioxus::{
    fullstack::{CborEncoding, UseWebsocket},
    logger::tracing,
    prelude::*,
};
use dioxus_i18n::t;

use crate::{
    auth_manager::server_fn::{change_password, get_use_password, logout},
    common::{CtxAppLang, CtxSyncedInsecureCerts, CtxSyncedServerUrl, DISCONNECTED_USER, Route},
    components::{
        alert_dialog::{
            AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
            AlertDialogDescription, AlertDialogRoot, AlertDialogTitle,
        },
        button::{Button, ButtonVariant},
        input::Input,
        sidebar::{Sidebar, SidebarTrigger},
    },
    websocket_handler::{
        NO_CLIENT_ID,
        event::{ClientEvent, ServerEvent},
    },
};

/// Whether the current username represents a signed-in user (vs. the
/// disconnected placeholder), i.e. whether the sign-out label/state applies.
fn is_signed_in(username: &str) -> bool {
    username != DISCONNECTED_USER.as_str()
}

/// Shared navbar component.
#[component]
pub fn Navbar() -> Element {
    // contexts
    let socket = use_context::<UseWebsocket<ClientEvent, ServerEvent, CborEncoding>>();
    let mut local_login_name_session = use_context::<Signal<String>>();
    let mut local_login_id_session = use_context::<Signal<i64>>();
    let mut app_lang = use_context::<CtxAppLang>().0;
    // Native clients only — see the doc comment on CtxSyncedServerUrl in common.rs for why
    // these are declared in App() and consumed here via context rather than called
    // directly with use_synced_storage in Navbar (a #[layout(...)] component).
    let mut synced_server_url = use_context::<CtxSyncedServerUrl>().0;
    let mut synced_insecure_certs = use_context::<CtxSyncedInsecureCerts>().0;

    // nav
    let navigator = use_navigator();

    // dialog open states — lifted here so the roots can live outside the navbar div
    let mut help_open = use_signal(|| false);

    // Server connection settings dialog — native only (gated at render time below via
    // `cfg!(target_arch = "wasm32")`, since #[cfg] attributes aren't supported inside
    // rsx!;).
    let mut server_settings_open = use_signal(|| false);
    // Mobile-only nav drawer (Sidebar) — the desktop controls group is duplicated
    // into it (CSS-gated visibility, see .navbar-desktop-group/.navbar-mobile-trigger
    // in main.css) so narrow screens get a proper drawer instead of a cramped row.
    let mut mobile_nav_open = use_signal(|| false);
    // Draft state so typing doesn't write to storage on every keystroke; populated from
    // the synced values each time the dialog is opened (see the trigger button below).
    let mut server_url_draft = use_signal(String::new);
    let mut insecure_certs_draft = use_signal(|| false);
    let mut server_saved_message = use_signal(|| false);

    // Change-password dialog — only relevant while USE_PASSWORD is on (see login_page.rs
    // for why this is fetched client-side via use_effect + spawn rather than use_resource).
    let mut use_pw = use_signal(|| false);
    use_effect(move || {
        spawn(async move {
            if let Ok(v) = get_use_password().await {
                use_pw.set(v);
            }
        });
    });
    let mut change_password_open = use_signal(|| false);
    let mut old_password_draft = use_signal(String::new);
    let mut new_password_draft = use_signal(String::new);
    let mut confirm_password_draft = use_signal(String::new);
    let mut change_password_answer = use_signal(String::new);

    // snapshot
    let snap_local_login_name_session = local_login_name_session();

    rsx! {
        div { class: "page-layout",
            // ── Navbar bar ────────────────────────────────────────────────────────
            div { class: "navbar",
                // Left: brand
                div { style: "display: flex; align-items: center; gap: 1rem;",
                    Link { class: "navbar-brand", to: Route::Home {}, "⚡ dx-template" }
                }
                // Mobile-only hamburger trigger — opens the Sidebar drawer below.
                // Hidden on desktop, shown ≤768px (see .navbar-mobile-trigger in main.css).
                div { class: "navbar-mobile-trigger",
                    SidebarTrigger { open: mobile_nav_open, label: t!("navbar-menu-open") }
                }
                // Right: trigger buttons only (no dialog roots here). Hidden on mobile —
                // see .navbar-desktop-group in main.css — and duplicated into the Sidebar
                // drawer below for narrow screens.
                div { class: "navbar-desktop-group",
                    // Language dropdown — current language is the selected option.
                    select {
                        class: "navbar-lang-select",
                        "aria-label": t!("lang-select-label"),
                        value: "{app_lang()}",
                        onchange: move |e| app_lang.set(e.value()),
                        option { value: "en", "🇬🇧 English" }
                        option { value: "fr", "🇫🇷 Français" }
                    }
                    // Help trigger
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| help_open.set(true),
                        "?"
                    }
                    // Server settings trigger (native only — excluded from web-server SSR
                    // so the hydration stream matches the wasm32 client's render)
                    if cfg!(all(not(target_arch = "wasm32"), not(feature = "server"))) {
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| {
                                server_url_draft.set(synced_server_url());
                                insecure_certs_draft.set(synced_insecure_certs());
                                server_saved_message.set(false);
                                server_settings_open.set(true);
                            },
                            {t!("navbar-server-settings")}
                        }
                    }
                    // Change-password trigger (signed-in users, only while USE_PASSWORD is on)
                    if is_signed_in(&snap_local_login_name_session) && use_pw() {
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| {
                                old_password_draft.set(String::new());
                                new_password_draft.set(String::new());
                                confirm_password_draft.set(String::new());
                                change_password_answer.set(String::new());
                                change_password_open.set(true);
                            },
                            {t!("navbar-change-password")}
                        }
                    }
                    if is_signed_in(&snap_local_login_name_session) {
                        span { class: "navbar-user", "👤 {snap_local_login_name_session}" }
                    }
                    Button {
                        style: "width: 160px; box-sizing: border-box; text-align: center; white-space: nowrap;",
                        variant: if is_signed_in(&snap_local_login_name_session) { ButtonVariant::Destructive } else { ButtonVariant::Secondary },
                        onclick: move |_| async move {
                            if local_login_name_session() != *DISCONNECTED_USER {
                                match logout().await {
                                    Ok(_) => {
                                        tracing::info!("{} is logged out", local_login_name_session());
                                        let _ = socket
                                            .clone()
                                            .send(ClientEvent::RequestLogOut(local_login_name_session()))
                                            .await;
                                        *local_login_name_session.write() = (*DISCONNECTED_USER).to_string();
                                        *local_login_id_session.write() = NO_CLIENT_ID;
                                    }
                                    Err(_) => {
                                        tracing::info!("Error on {} logout", local_login_name_session())
                                    }
                                }
                            }
                            navigator.push(Route::Home {});
                        },
                        if is_signed_in(&snap_local_login_name_session) {
                            {t!("navbar-sign-out")}
                        } else {
                            {t!("navbar-sign-in")}
                        }
                    }
                }
            }

            // ── Dialog roots — rendered at layout level, NOT inside the navbar div ──

            // Help dialog
            AlertDialogRoot { open: help_open(), on_open_change: move |v| help_open.set(v),
                AlertDialogContent {
                    AlertDialogTitle { {t!("help-title")} }
                    AlertDialogDescription {
                        div { style: "text-align:left; line-height:1.8;",
                            p { {t!("help-body-1")} }
                            p { {t!("help-body-2")} }
                        }
                    }
                    AlertDialogAction {
                        AlertDialogCancel { {t!("common-close")} }
                    }
                }
            }

            // Server settings dialog (native only — excluded from web-server SSR)
            if cfg!(all(not(target_arch = "wasm32"), not(feature = "server"))) {
                AlertDialogRoot {
                    open: server_settings_open(),
                    on_open_change: move |v| server_settings_open.set(v),
                    AlertDialogContent {
                        AlertDialogTitle { {t!("server-settings-title")} }
                        AlertDialogDescription {
                            div { style: "text-align:left; display:flex; flex-direction:column; gap:0.75rem;",
                                p {
                                    {
                                        t!(
                                            "server-settings-current", url : dioxus::fullstack::get_server_url()
                                            .to_owned()
                                        )
                                    }
                                }
                                Input {
                                    placeholder: t!("server-settings-placeholder"),
                                    r#type: "text",
                                    value: "{server_url_draft}",
                                    oninput: move |e: FormEvent| server_url_draft.set(e.value()),
                                }
                                label { style: "display:flex; align-items:center; gap:0.5rem; cursor:pointer;",
                                    input {
                                        r#type: "checkbox",
                                        checked: insecure_certs_draft(),
                                        onchange: move |e: FormEvent| insecure_certs_draft.set(e.checked()),
                                    }
                                    span { {t!("server-settings-insecure-label")} }
                                }
                                p { style: "font-size:0.85em; color:var(--rpg-text-muted);",
                                    {t!("server-settings-insecure-warning")}
                                }
                                if server_saved_message() {
                                    p { style: "color:var(--rpg-gold); font-weight:600;",
                                        {t!("server-settings-saved")}
                                    }
                                }
                            }
                        }
                        AlertDialogActions {
                            AlertDialogCancel { {t!("common-cancel")} }
                            Button {
                                variant: ButtonVariant::Primary,
                                onclick: move |_| {
                                    synced_server_url.set(server_url_draft());
                                    synced_insecure_certs.set(insecure_certs_draft());
                                    server_saved_message.set(true);
                                },
                                {t!("server-settings-save")}
                            }
                        }
                    }
                }
            }

            // Change-password dialog
            if is_signed_in(&snap_local_login_name_session) && use_pw() {
                AlertDialogRoot {
                    open: change_password_open(),
                    on_open_change: move |v| change_password_open.set(v),
                    AlertDialogContent {
                        AlertDialogTitle { {t!("change-password-title")} }
                        AlertDialogDescription {
                            div { style: "text-align:left; display:flex; flex-direction:column; gap:0.75rem;",
                                Input {
                                    placeholder: t!("change-password-current-placeholder"),
                                    r#type: "password",
                                    value: "{old_password_draft}",
                                    oninput: move |e: FormEvent| old_password_draft.set(e.value()),
                                }
                                Input {
                                    placeholder: t!("change-password-new-placeholder"),
                                    r#type: "password",
                                    value: "{new_password_draft}",
                                    oninput: move |e: FormEvent| new_password_draft.set(e.value()),
                                }
                                Input {
                                    placeholder: t!("change-password-confirm-placeholder"),
                                    r#type: "password",
                                    value: "{confirm_password_draft}",
                                    oninput: move |e: FormEvent| confirm_password_draft.set(e.value()),
                                }
                                if !change_password_answer().is_empty() {
                                    p { class: "rpg-answer", "{change_password_answer}" }
                                }
                            }
                        }
                        AlertDialogActions {
                            AlertDialogCancel { {t!("common-cancel")} }
                            Button {
                                variant: ButtonVariant::Primary,
                                onclick: move |_| async move {
                                    if new_password_draft().trim().is_empty() {
                                        change_password_answer.set(t!("change-password-empty"));
                                        return;
                                    }
                                    if new_password_draft() != confirm_password_draft() {
                                        change_password_answer.set(t!("change-password-mismatch"));
                                        return;
                                    }
                                    match change_password(
                                            local_login_name_session(),
                                            old_password_draft(),
                                            new_password_draft(),
                                            use_pw(),
                                        )
                                        .await
                                    {
                                        Ok(()) => {
                                            change_password_answer.set(t!("change-password-saved"));
                                            old_password_draft.set(String::new());
                                            new_password_draft.set(String::new());
                                            confirm_password_draft.set(String::new());
                                        }
                                        Err(e) => {
                                            change_password_answer.set(format!("{}", e.to_owned()));
                                        }
                                    }
                                },
                                {t!("change-password-save")}
                            }
                        }
                    }
                }
            }

            // ── Mobile navigation drawer ─────────────────────────────────────────
            // Duplicates the desktop controls group above for narrow screens (see
            // .navbar-desktop-group / .navbar-mobile-trigger in main.css) — every
            // action also closes the drawer after firing.
            Sidebar { open: mobile_nav_open, title: Some(t!("navbar-menu-title")),
                select {
                    class: "navbar-lang-select",
                    "aria-label": t!("lang-select-label"),
                    value: "{app_lang()}",
                    onchange: move |e| app_lang.set(e.value()),
                    option { value: "en", "🇬🇧 English" }
                    option { value: "fr", "🇫🇷 Français" }
                }
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        help_open.set(true);
                        mobile_nav_open.set(false);
                    },
                    {t!("help-title")}
                }
                if cfg!(all(not(target_arch = "wasm32"), not(feature = "server"))) {
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            server_url_draft.set(synced_server_url());
                            insecure_certs_draft.set(synced_insecure_certs());
                            server_saved_message.set(false);
                            server_settings_open.set(true);
                            mobile_nav_open.set(false);
                        },
                        {t!("navbar-server-settings")}
                    }
                }
                if is_signed_in(&snap_local_login_name_session) && use_pw() {
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            old_password_draft.set(String::new());
                            new_password_draft.set(String::new());
                            confirm_password_draft.set(String::new());
                            change_password_answer.set(String::new());
                            change_password_open.set(true);
                            mobile_nav_open.set(false);
                        },
                        {t!("navbar-change-password")}
                    }
                }
                if is_signed_in(&snap_local_login_name_session) {
                    span { class: "navbar-user", "👤 {snap_local_login_name_session}" }
                }
                Button {
                    variant: if is_signed_in(&snap_local_login_name_session) { ButtonVariant::Destructive } else { ButtonVariant::Secondary },
                    onclick: move |_| async move {
                        if local_login_name_session() != *DISCONNECTED_USER {
                            match logout().await {
                                Ok(_) => {
                                    tracing::info!("{} is logged out", local_login_name_session());
                                    let _ = socket
                                        .clone()
                                        .send(ClientEvent::RequestLogOut(local_login_name_session()))
                                        .await;
                                    *local_login_name_session.write() = (*DISCONNECTED_USER).to_string();
                                    *local_login_id_session.write() = NO_CLIENT_ID;
                                }
                                Err(_) => {
                                    tracing::info!("Error on {} logout", local_login_name_session())
                                }
                            }
                        }
                        mobile_nav_open.set(false);
                        let navigator = use_navigator();
                        navigator.push(Route::Home {});
                    },
                    if is_signed_in(&snap_local_login_name_session) {
                        {t!("navbar-sign-out")}
                    } else {
                        {t!("navbar-sign-in")}
                    }
                }
            }

            Outlet::<Route> {}

            // ── Footer ────────────────────────────────────────────────────────────
            footer { class: "app-footer",
                div { class: "app-footer-inner",
                    div { class: "app-footer-brand",
                        span { class: "app-footer-icon", "⚡" }
                        span { class: "app-footer-name", "dx-template" }
                        span { class: "app-footer-version", {concat!("v", env!("CARGO_PKG_VERSION"))} }
                    }
                    div { class: "app-footer-section",
                        span { class: "app-footer-section-title", {t!("footer-about")} }
                        a {
                            href: "https://dioxuslabs.com",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            {t!("footer-built-with-dioxus")}
                        }
                        span { "⚡ Rust + WASM" }
                    }
                }
            }
        } // end page-layout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_in_is_the_inverse_of_disconnected_placeholder() {
        assert!(is_signed_in("some-user"));
        assert!(!is_signed_in(DISCONNECTED_USER.as_str()));
    }
}

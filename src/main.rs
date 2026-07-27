use dioxus::{
    fullstack::{WebSocketOptions, use_websocket},
    logger::tracing::{self, Level},
    prelude::*,
};
use dioxus_i18n::prelude::*;
use dioxus_sdk_storage::{LocalStorage, use_synced_storage};
#[cfg(all(not(feature = "server"), not(target_arch = "wasm32")))]
use dioxus_sdk_storage::{StorageBacking, set_dir};
#[cfg(not(target_arch = "wasm32"))]
use dotenv::dotenv;
use dx_template::{
    common::{
        CtxAppLang, CtxConnectedUsers, CtxDeviceToken, CtxSyncedInsecureCerts, CtxSyncedServerUrl,
        DISCONNECTED_USER, DX_COMP_CSS, Route, SYNCED_DEVICE_TOKEN_KEY,
    },
    components::{
        alert_dialog, button, input, label, popover, select, separator, sheet, sidebar, tabs,
        tooltip,
    },
    websocket_handler::{
        NO_CLIENT_ID,
        event::{ClientEvent, ServerEvent, on_rcv_client_event},
    },
};
// These constants are only used in the native (non-web, non-server) build path where
// CtxSyncedServerUrl / CtxSyncedInsecureCerts are backed by use_synced_storage.
#[cfg(all(not(feature = "server"), not(target_arch = "wasm32")))]
use dx_template::common::{SYNCED_INSECURE_CERTS_KEY, SYNCED_SERVER_URL_KEY};
use unic_langid::langid;
// StorageBacking is needed on wasm32 to call LocalStorage::get / LocalStorage::set
// (trait methods) inside the login-restore effect.  On native it is already
// imported above via the #[cfg(all(not(feature="server"),not(target_arch="wasm32")))]
// block; on the server binary the effect block doesn't compile at all.
#[cfg(target_arch = "wasm32")]
use dioxus_sdk_storage::StorageBacking;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    // Reads the .env file. Native builds (server, and native clients below) can have one;
    // the browser (wasm32) has no filesystem so it never reads one.
    #[cfg(not(target_arch = "wasm32"))]
    dotenv().ok();

    // Init logger
    let _ = dioxus::logger::init(
        std::env::var("LOG_LEVEL")
            .unwrap_or_else(|_| "info".to_owned())
            .parse::<Level>()
            .unwrap_or(Level::INFO),
    );
    tracing::info!("Rendering app!");

    // Native clients (desktop, mobile) connect to a remote server over the same
    // websocket/server-fn protocol the web client uses, but — unlike a browser page, which
    // infers the server from same-origin — they have no origin to infer it from, so the
    // server URL must be set explicitly before launch.
    #[cfg(all(not(feature = "server"), not(target_arch = "wasm32")))]
    {
        // dioxus-sdk-storage's LocalStorage falls back to a filesystem backend on native
        // targets and panics if this isn't called before first use — the browser build never
        // hits this path since it uses the browser's actual localStorage instead.
        //
        // On Android, `directories::BaseDirs::new()` (used internally by set_dir!()) returns
        // None because the `directories` crate doesn't support Android — unwrapping it panics
        // and leaves the screen white. Use the app's known internal data path instead.
        #[cfg(target_os = "android")]
        set_dir!("/data/data/io.github.r0ndoudou.dxtemplate/files/dx-template");
        #[cfg(not(target_os = "android"))]
        set_dir!();

        // Resolution order: a runtime env var (e.g. `dx serve`) always wins, for dev
        // convenience; then whatever the user last saved via the in-app Server settings
        // dialog (pages/navbar.rs) — read straight off disk here since main() runs before
        // any component/hook exists; then the value baked in at *compile* time (via
        // `option_env!`, see build.rs) — the only option available to an installed Android
        // APK, which has no shell to read env vars from and no prior launch to have saved
        // anything from; then a hardcoded fallback.
        let persisted_server_url =
            LocalStorage::get::<String>(&dx_template::common::SYNCED_SERVER_URL_KEY.to_owned())
                .filter(|s: &String| !s.is_empty());
        let server_url = std::env::var("SERVER_URL")
            .ok()
            .or(persisted_server_url)
            .or_else(|| option_env!("SERVER_URL").map(str::to_owned))
            .unwrap_or_else(|| "http://127.0.0.1:8080".to_owned());
        tracing::info!("Native client connecting to server at {server_url}");
        dioxus::fullstack::set_server_url(Box::leak(server_url.into_boxed_str()));

        // Opt-in escape hatch for a server behind a self-signed/untrusted TLS certificate
        // (e.g. a dev or home-lab deployment with no real domain for Let's Encrypt). Off by
        // default: this disables certificate validation for every request the client makes
        // (server-fn calls *and* the websocket handshake both go through the same underlying
        // reqwest client), so it must only be used against a server you trust on a network you
        // trust — it removes protection against a MITM impersonating the server. Same
        // runtime-env, then-persisted, then-compile-time-baked fallback as SERVER_URL above,
        // for the same reasons.
        let persisted_insecure_certs =
            LocalStorage::get::<bool>(&dx_template::common::SYNCED_INSECURE_CERTS_KEY.to_owned());
        let insecure_accept_invalid_certs = std::env::var("INSECURE_ACCEPT_INVALID_CERTS")
            .ok()
            .or_else(|| persisted_insecure_certs.map(|b| b.to_string()))
            .or_else(|| option_env!("INSECURE_ACCEPT_INVALID_CERTS").map(str::to_owned));
        if insecure_accept_invalid_certs.as_deref() == Some("true") {
            tracing::warn!(
                "INSECURE_ACCEPT_INVALID_CERTS=true — TLS certificate validation is DISABLED for all requests to {}. Do not use this against an untrusted network or server.",
                dioxus::fullstack::get_server_url()
            );
            let client = dioxus::fullstack::reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .cookie_store(true)
                .build()
                .expect("failed to build insecure reqwest client");
            // Best-effort: if something already initialized the default client (shouldn't
            // happen this early), fall back to the validated default rather than panicking.
            let _ = dioxus::fullstack::GLOBAL_REQUEST_CLIENT.set(client);
        }
    }

    // On the client, we simply launch the app as normal, taking over the main thread
    #[cfg(not(feature = "server"))]
    dioxus::launch(App);

    // On the server, we can use `dioxus::serve` to create a server that serves our app.
    //
    // The `serve` function takes a closure that returns a `Future` which resolves to an `axum::Router`.
    //
    // We return a `Router` such that dioxus sets up logging, hot-reloading, devtools, and wires up the
    // IP and PORT environment variables to our server.
    #[cfg(feature = "server")]
    dioxus::serve(|| async {
        use axum_session::{SessionConfig, SessionLayer, SessionStore};
        use axum_session_auth::AuthConfig;
        use axum_session_sqlx::SessionSqlitePool;
        use dx_template::{
            auth_manager::{
                auth::AuthLayer,
                db::get_db,
                server_fn::{auth_rate_limit, update_all_connection_status},
            },
            websocket_handler::STARTING_CLIENT_ID,
        };

        let bind_ip = std::env::var("IP").unwrap_or_else(|_| "0.0.0.0".to_owned());
        let bind_port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_owned());
        let is_main_server = std::env::var("IS_MAIN_SERVER")
            .unwrap_or_else(|_| "false".to_owned())
            .trim()
            .to_lowercase()
            == "true";
        tracing::info!(
            "dx-template server starting on {}:{} (main server: {})",
            bind_ip,
            bind_port,
            is_main_server
        );

        // Only the designated main/local server instance resets connection status for
        // every user at start (to avoid stale is_connected from a previous run). A second
        // server process pointed at the same database (e.g. a secondary/staging instance)
        // must NOT do this — it would wipe out is_connected for users genuinely connected
        // to the main instance right now.
        if is_main_server {
            update_all_connection_status(false).await.unwrap();
        }
        // create db pool for session store
        let pool = get_db().await;

        // Create an axum router that dioxus will attach the app to
        Ok(dioxus::server::router(App)
            .layer(axum::middleware::from_fn(auth_rate_limit))
            .layer(AuthLayer::new(Some(pool.clone())).with_config(
                AuthConfig::<i64>::default().with_anonymous_user_id(Some(STARTING_CLIENT_ID)),
            ))
            .layer(SessionLayer::new(
                SessionStore::<SessionSqlitePool>::new(
                    Some(pool.clone().into()),
                    SessionConfig::default().with_table_name("test_table"),
                )
                .await?,
            )))
    });
}

#[component]
fn App() -> Element {
    // i18n — English default, French available; toggle lives in the Navbar.
    use_init_i18n(|| {
        I18nConfig::new(langid!("en-US"))
            .with_locale((langid!("en-US"), include_str!("./i18n/en-US.ftl")))
            .with_locale((langid!("fr-FR"), include_str!("./i18n/fr-FR.ftl")))
    });

    // Local UI state
    let mut connected_users: Signal<Vec<String>> = use_signal(Vec::new);

    // Log which server URL this client is about to talk to (server-fn calls + websocket) —
    // same-origin implicit on web/server, explicit remote target on native — to make
    // connectivity issues visible without needing native-only debugging.
    use_effect(|| {
        tracing::info!(
            "[client] connecting to server at {}",
            dioxus::fullstack::get_server_url()
        );
    });

    let socket = use_websocket(|| on_rcv_client_event(WebSocketOptions::new()));

    // synced storage
    // login_name drives which page is rendered (LoginPage vs home content). Starting with
    // the server default (DISCONNECTED_USER) on both the server binary and the WASM client
    // ensures the hydration render produces the same component tree on both sides — avoiding
    // a known Dioxus SSR hydration bug (https://github.com/DioxusLabs/dioxus/issues/3583)
    // that mis-aligns the SSR data stream when the client renders different components than
    // the server.  On native (desktop/mobile), use_synced_storage handles everything.
    #[cfg(any(target_arch = "wasm32", feature = "server"))]
    let mut login_name_session_local_sync = use_signal(|| DISCONNECTED_USER.clone());
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "server")))]
    let mut login_name_session_local_sync =
        use_synced_storage::<LocalStorage, String>("synced_user_sql_name".to_owned(), || {
            DISCONNECTED_USER.clone()
        });
    let mut login_id_session_local_sync =
        use_synced_storage::<LocalStorage, i64>("synced_user_sql_id".to_owned(), || NO_CLIENT_ID); // from db, integer primary key not null and from 1 upwards
    // Persistent per-device/browser random token — generated once, lazily, on first read.
    // See CtxDeviceToken / SYNCED_DEVICE_TOKEN_KEY doc comments in common.rs.
    let device_token_local_sync =
        use_synced_storage::<LocalStorage, String>(SYNCED_DEVICE_TOKEN_KEY.to_owned(), || {
            format!("{:032x}", rand::random::<u128>())
        });
    let app_lang_local_sync =
        use_synced_storage::<LocalStorage, String>("synced_app_lang".to_owned(), || {
            "en".to_owned()
        });
    // Native-only server URL/TLS-validation override, editable from the Navbar's Server
    // settings dialog; declared here (not in Navbar) since use_synced_storage there
    // stack-overflows the app (Navbar is a #[layout] component, not the route root).
    // Hooks must run in the same order on every platform, so these are declared
    // unconditionally but fall back to an inert signal off native — a real
    // use_synced_storage there hits a Dioxus SSR hydration bug
    // (https://github.com/DioxusLabs/dioxus/issues/3583).
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "server")))]
    let synced_server_url =
        use_synced_storage::<LocalStorage, String>(SYNCED_SERVER_URL_KEY.to_owned(), || {
            dioxus::fullstack::get_server_url().to_owned()
        });
    #[cfg(any(target_arch = "wasm32", feature = "server"))]
    let synced_server_url = use_signal(|| dioxus::fullstack::get_server_url().to_owned());

    #[cfg(all(not(target_arch = "wasm32"), not(feature = "server")))]
    let synced_insecure_certs =
        use_synced_storage::<LocalStorage, bool>(SYNCED_INSECURE_CERTS_KEY.to_owned(), || false);
    #[cfg(any(target_arch = "wasm32", feature = "server"))]
    let synced_insecure_certs = use_signal(|| false);

    // Keep dioxus-i18n's active locale synced to the persisted "en"/"fr" value —
    // covers both the initial load from localStorage and every toggle click.
    use_effect(move || {
        let mut i18n = i18n();
        let target = if app_lang_local_sync() == "fr" {
            langid!("fr-FR")
        } else {
            langid!("en-US")
        };
        i18n.set_language(target);
    });

    // Set the theme to dark on app load.
    // `document::eval` (not raw web_sys) so this also works on desktop/mobile clients,
    // which don't compile web_sys (it's a wasm-bindgen crate, native targets don't have it).
    use_effect(|| {
        document::eval("document.documentElement.setAttribute('data-theme', 'dark');");
    });

    // Android's WebView (used by the native mobile client) never enables "wide viewport"
    // mode, so it ignores <meta name="viewport"> and evaluates every `@media (max-width:
    // ...)` rule in main.css against a fake ~980px layout — the mobile nav hamburger
    // breakpoint (see .navbar-desktop-group/.navbar-mobile-trigger in main.css) never
    // matches there no matter how narrow the phone screen actually is. window.screen.width
    // reports the real device width regardless of that bug, so toggle a class on <html>
    // from it as a fallback the CSS in main.css (the `html.force-mobile-nav` rules) also
    // hooks into. Harmless no-op on platforms where the media query already works
    // correctly (screen.width there reports the real viewport too, so this just
    // redundantly confirms the same state).
    use_effect(|| {
        document::eval(
            r#"
            function applyForceMobileNav() {
                var w = (window.screen && window.screen.width) ? window.screen.width : window.innerWidth;
                document.documentElement.classList.toggle('force-mobile-nav', w <= 768);
            }
            applyForceMobileNav();
            window.addEventListener('resize', applyForceMobileNav);
            window.addEventListener('orientationchange', applyForceMobileNav);
            "#,
        );
    });

    // On web: restore login_name from localStorage after the initial hydration render, then
    // persist any future changes.  The first call of the effect (immediately after hydration)
    // reads localStorage and updates the signal if a saved session exists — triggering a
    // re-render that shows the correct page.  Subsequent calls (on signal changes) persist
    // the new value so it survives page reloads.  An Rc<Cell<bool>> flag guards the
    // first-vs-subsequent distinction within the same browser session.
    #[cfg(target_arch = "wasm32")]
    {
        use std::{cell::Cell, rc::Rc};
        let restored = use_hook(|| Rc::new(Cell::new(false)));
        use_effect(move || {
            let name = login_name_session_local_sync(); // subscribe so effect re-runs on changes
            if restored.get() {
                // After initial restoration: persist any changes to localStorage
                LocalStorage::set("synced_user_sql_name".to_owned(), &name);
            } else {
                // First call after hydration: restore the saved session (do NOT save yet,
                // to avoid overwriting localStorage with the default "not connected" value)
                restored.set(true);
                if let Some(saved) = LocalStorage::get::<String>(&"synced_user_sql_name".to_owned())
                {
                    if !saved.is_empty() && saved != *DISCONNECTED_USER {
                        login_name_session_local_sync.set(saved);
                    }
                }
            }
        });
    }

    // Receive events from the websocket and update local signals.
    use_future(move || {
        let mut socket = socket;
        async move {
            tracing::info!("[client] ws-loop starting");
            while let Ok(event) = socket.recv().await {
                tracing::debug!("[client] ws-loop: received an event");
                match event {
                    ServerEvent::NewClientOnExistingPlayer(_msg, client_id) => {
                        let login_name_session_local_sync = login_name_session_local_sync();
                        let login_id_session_local_sync = login_id_session_local_sync();
                        // re-send AddPlayer to server (e.g. after a hot-reload/resume)
                        if login_name_session_local_sync != *DISCONNECTED_USER
                            && login_id_session_local_sync != NO_CLIENT_ID
                        {
                            let _ = socket
                                .clone()
                                .send(ClientEvent::AddPlayer(
                                    login_name_session_local_sync.clone(),
                                    device_token_local_sync(),
                                ))
                                .await;
                            tracing::info!(
                                "Client {} sent AddPlayer for player {} (id {})",
                                client_id,
                                login_name_session_local_sync,
                                login_id_session_local_sync
                            );
                        }
                    }
                    ServerEvent::InitClient(id) => {
                        tracing::info!("Client initialized with id {}", id);
                    }
                    ServerEvent::ReconnectAllSessions(username, sql_id) => {
                        let login_name_session_local_sync = login_name_session_local_sync();
                        let login_id_session_local_sync = login_id_session_local_sync();
                        if login_name_session_local_sync == username
                            && login_id_session_local_sync == sql_id
                        {
                            tracing::info!(
                                "ReconnectAllSessions for player {}",
                                login_name_session_local_sync
                            );
                            let _ = socket
                                .clone()
                                .send(ClientEvent::AddPlayer(
                                    login_name_session_local_sync.clone(),
                                    device_token_local_sync(),
                                ))
                                .await;
                        } else {
                            tracing::info!(
                                "Skipping ReconnectAllSessions for player {} (username: {}, sql_id: {})",
                                login_name_session_local_sync,
                                username,
                                sql_id
                            );
                        }
                    }
                    ServerEvent::LogOut => {
                        tracing::info!("Received LogOut event, resetting client data");
                        login_name_session_local_sync.set(DISCONNECTED_USER.clone());
                        login_id_session_local_sync.set(NO_CLIENT_ID);
                    }
                    ServerEvent::ConnectedUsers(usernames) => {
                        connected_users.set(usernames);
                    }
                }
            }
            tracing::warn!("[client] ws-loop EXITED — deserialization error or socket closed");
        }
    });

    use_context_provider(|| socket);
    use_context_provider(|| login_name_session_local_sync);
    use_context_provider(|| login_id_session_local_sync);
    use_context_provider(|| CtxDeviceToken(device_token_local_sync));
    use_context_provider(|| CtxConnectedUsers(connected_users));
    // UI language ("en"/"fr") — localStorage-backed so it works pre-login
    use_context_provider(|| CtxAppLang(app_lang_local_sync));
    // Native clients only: server address / TLS-validation override, editable from the
    // Server settings dialog in Navbar (see the doc comment on CtxSyncedServerUrl above).
    use_context_provider(|| CtxSyncedServerUrl(synced_server_url));
    use_context_provider(|| CtxSyncedInsecureCerts(synced_insecure_certs));

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: DX_COMP_CSS }
        // Shared dx-components-library stylesheets: loaded here (app root) rather than via
        // a document::Link nested inside each component's own render — dioxus-desktop only
        // injects document::Link stylesheets declared at the App() root into <head>, not
        // ones declared inside a child component (see each component's STYLE_CSS comment).
        document::Link { rel: "stylesheet", href: alert_dialog::STYLE_CSS }
        document::Link { rel: "stylesheet", href: button::STYLE_CSS }
        document::Link { rel: "stylesheet", href: input::STYLE_CSS }
        document::Link { rel: "stylesheet", href: label::STYLE_CSS }
        document::Link { rel: "stylesheet", href: popover::STYLE_CSS }
        document::Link { rel: "stylesheet", href: select::STYLE_CSS }
        document::Link { rel: "stylesheet", href: separator::STYLE_CSS }
        document::Link { rel: "stylesheet", href: sheet::STYLE_CSS }
        document::Link { rel: "stylesheet", href: sidebar::STYLE_CSS }
        document::Link { rel: "stylesheet", href: tabs::STYLE_CSS }
        document::Link { rel: "stylesheet", href: tooltip::STYLE_CSS }

        Router::<Route> {}
    }
}

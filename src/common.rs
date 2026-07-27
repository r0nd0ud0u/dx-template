use dioxus::prelude::*;
use once_cell::sync::Lazy;

use crate::pages::home_page::Home;
use crate::pages::navbar::Navbar;

// Lazy
pub static DISCONNECTED_USER: Lazy<String> = Lazy::new(|| "not connected".to_owned());

#[derive(Debug, Clone, Routable, PartialEq, serde::Serialize, serde::Deserialize)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
}

pub const DX_COMP_CSS: Asset = asset!("/assets/dx-components-theme.css");

/// Native clients only: `dioxus_sdk_storage` keys for the user-chosen server address
/// and TLS-validation override, shared between `main.rs` (reads them once at startup,
/// before launch) and `pages/navbar.rs` (the settings UI that writes them). Kept as
/// constants so the two call sites can't drift apart.
pub const SYNCED_SERVER_URL_KEY: &str = "synced_server_url";
pub const SYNCED_INSECURE_CERTS_KEY: &str = "synced_insecure_certs";

/// Per-device/browser random token, generated once and persisted locally (both web and
/// native). Sent alongside `LoginAllSessions`/`AddPlayer` so the server can tell a genuine
/// reconnect of the same device apart from a different device that still has a stale cached
/// session for the same username (see `websocket_handler::event::add_player`).
pub const SYNCED_DEVICE_TOKEN_KEY: &str = "synced_device_token";

// ── Per-setting context newtypes ─────────────────────────────────────────────
// Each wraps a `Signal<T>` in a distinct type so that Dioxus context lookup
// (which is keyed by TypeId) stores and retrieves them independently.

/// Current UI language ("en" or "fr"), synced to browser localStorage (works
/// pre-login, unlike a SQLite-backed per-user setting would). Drives dioxus-i18n's
/// t!() chrome strings via the sync effect in main.rs.
#[derive(Clone, Copy)]
pub struct CtxAppLang(pub Signal<String>);

/// Native clients only: the user-chosen server address, persisted via
/// `SYNCED_SERVER_URL_KEY` and read with priority over the compile-time-baked default
/// on the *next* launch (see `main.rs`). Declared in `App()` and provided via context
/// rather than called directly in `pages/navbar.rs` — calling `use_synced_storage`
/// inside a `#[layout(...)]` component instead of the route root stack-overflows the
/// app at startup.
#[derive(Clone, Copy)]
pub struct CtxSyncedServerUrl(pub Signal<String>);

/// Native clients only: whether to accept invalid/self-signed TLS certificates for the
/// server above. Same persistence/priority and same App()-not-Navbar constraint as
/// `CtxSyncedServerUrl`.
#[derive(Clone, Copy)]
pub struct CtxSyncedInsecureCerts(pub Signal<bool>);

/// This device/browser's persistent random token — see `SYNCED_DEVICE_TOKEN_KEY`. Provided
/// via context (declared in `App()`) for the same reasons as `CtxSyncedServerUrl`.
#[derive(Clone, Copy)]
pub struct CtxDeviceToken(pub Signal<String>);

/// Sorted usernames of everyone currently logged in, kept in sync with
/// `ServerEvent::ConnectedUsers` broadcasts (see `main.rs`'s websocket receive loop).
/// This is the skeleton's demo feature proving the websocket/shared-server-state/
/// broadcast pipeline works end to end — replace it with your own app's live state.
#[derive(Clone, Copy)]
pub struct CtxConnectedUsers(pub Signal<Vec<String>>);

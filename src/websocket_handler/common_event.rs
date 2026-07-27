use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// server only: live connection bookkeeping — which client ids currently belong to
/// which logged-in username (`players`), and which device currently "owns" that
/// username's session (`device_tokens`). See `add_player`/`login_all_sessions` in
/// `event.rs` for how the device-token lock is claimed/verified.
#[derive(Default)]
pub struct ConnectionManager {
    pub players: HashMap<String, Vec<u32>>,
    pub device_tokens: HashMap<String, String>,
}

pub static CONNECTION_MANAGER: Lazy<Arc<Mutex<ConnectionManager>>> =
    Lazy::new(|| Arc::new(Mutex::new(ConnectionManager::default())));

#[cfg(feature = "server")]
use crate::auth_manager::server_fn::update_connection_status;
#[cfg(feature = "server")]
use crate::websocket_handler::common_event::CONNECTION_MANAGER;
#[cfg(feature = "server")]
use anyhow::Result;
use dioxus::fullstack::{CborEncoding, WebSocketOptions, Websocket};
#[cfg(feature = "server")]
use dioxus::logger::tracing;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use once_cell::sync::Lazy;
#[cfg(feature = "server")]
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};
#[cfg(feature = "server")]
use tokio::sync::mpsc;

#[cfg(feature = "server")]
static NEXT_CLIENT_ID: AtomicUsize = AtomicUsize::new(1);

/// server only: map of client id -> sender to that client's outgoing queue
#[cfg(feature = "server")]
type ClientId = usize;
#[cfg(feature = "server")]
type ClientTx = mpsc::UnboundedSender<ServerEvent>;
#[cfg(feature = "server")]
type ClientsMap = HashMap<ClientId, ClientTx>;
#[cfg(feature = "server")]
type SharedClients = Arc<Mutex<ClientsMap>>;
#[cfg(feature = "server")]
static CLIENTS: Lazy<SharedClients> = Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientEvent {
    LoginAllSessions(String, i64, String), // username, sql-id, device_token
    AddPlayer(String, String),             // username, device_token
    RequestLogOut(String),                 // username
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerEvent {
    InitClient(u32),                        // this client's id
    NewClientOnExistingPlayer(String, u32), // welcome message, client id
    ReconnectAllSessions(String, i64),      // username, sql-id
    LogOut,
    /// Sorted usernames of everyone currently logged in — the skeleton's
    /// end-to-end demo of session -> websocket -> shared server state -> broadcast.
    ConnectedUsers(Vec<String>),
}

#[get("/api/new-event")]
pub async fn on_rcv_client_event(
    options: WebSocketOptions,
) -> Result<Websocket<ClientEvent, ServerEvent, CborEncoding>> {
    Ok(options.on_upgrade(move |mut socket| async move {
        #[cfg(feature = "server")]
        {
            // Assign id
            let client_id = NEXT_CLIENT_ID.fetch_add(1, Ordering::SeqCst) as u32;

            // Channel for outgoing messages to this client
            let (tx, mut rx) = mpsc::unbounded_channel::<ServerEvent>();

            // Register client
            {
                let mut clients = CLIENTS.lock().unwrap();
                clients.insert(client_id as usize, tx);
                tracing::info!("Client {} connected (total: {})", client_id, clients.len());
            }

            let _ = socket.send(ServerEvent::InitClient(client_id)).await;
            let _ = socket
                .send(ServerEvent::NewClientOnExistingPlayer(
                    format!("Welcome! (id={})", client_id),
                    client_id,
                ))
                .await;

            // Main loop: handle incoming socket messages and outgoing queued messages
            loop {
                tokio::select! {
                    // Outgoing messages destined for this client
                    maybe_msg = rx.recv() => {
                        if let Some(msg) = maybe_msg {
                            if socket.send(msg).await.is_err() {
                                tracing::error!("[server] socket.send to client {} FAILED", client_id);
                            }
                        } else {
                            break;
                        }
                    },

                    // Incoming message from client
                    res = socket.recv() => {
                        tracing::info!("Receiving message from client {}, message: {:?}", client_id, res);
                        match res {
                            Ok(ClientEvent::LoginAllSessions(username, sql_id, device_token)) => {
                                tracing::info!("Received LoginAllSessions from client {}: {:?}", client_id, username);
                                login_all_sessions(username, sql_id, device_token);
                            }
                            Ok(ClientEvent::AddPlayer(username, device_token)) => {
                                tracing::info!("Adding new player from client {}: {:?}", client_id, username);
                                add_player(username, client_id, device_token);
                            }
                            Ok(ClientEvent::RequestLogOut(user_name)) => {
                                tracing::info!("{} is logged out", user_name);
                                send_logout_to_server(user_name, client_id);
                            }
                            Err(_) => {
                                // socket closed / deserialization error
                                tracing::info!("Client {} disconnected", client_id);
                                handle_disconnect(client_id).await;
                                break;
                            }
                        }
                    }
                }
            }

            // cleanup on disconnect
            {
                let mut clients = CLIENTS.lock().unwrap();
                clients.remove(&(client_id as usize));
                tracing::info!("Client {} removed. Remaining: {}", client_id, clients.len());
            }
        }
    }))
}

/// Whether `name` currently has at least one live, registered websocket connection —
/// the real-time source of truth `login()` checks in addition to the DB's `is_connected`
/// flag, since that flag can lag behind (or outlive) the actual live connection state
/// (a crash that skipped clean disconnect, a reconnect racing the grace-period timer, etc.).
#[cfg(feature = "server")]
pub fn is_username_connected(name: &str) -> bool {
    CONNECTION_MANAGER
        .lock()
        .unwrap()
        .players
        .get(name)
        .is_some_and(|ids| !ids.is_empty())
}

/// Registers `id` under `name` in the connection manager, but only if `device_token`
/// matches the device currently recognized as owning the login for `name` — or no
/// device is recognized yet, in which case this call claims the lock. Rejects (no-op)
/// a mismatched device_token, which is how a stale client with a cached session for
/// `name` on a different device gets blocked from silently rejoining once another
/// device has genuinely logged in as `name` (see `login_all_sessions`, which resets
/// the lock on every fresh login).
///
/// `device_token` must also match `name`'s current entry in `LOGIN_PROOFS` — a secret
/// the server only ever hands out from a successful `login()` call — otherwise this is
/// a raw websocket connection (or a tampered/stale client) trying to claim a username it
/// never actually authenticated as, and the call is rejected outright.
#[cfg(feature = "server")]
pub fn add_player(name: String, id: u32, device_token: String) {
    use crate::auth_manager::server_fn::auth::LOGIN_PROOFS;
    let proof_ok = LOGIN_PROOFS
        .lock()
        .unwrap()
        .get(&name)
        .is_some_and(|proof| proof == &device_token);
    if !proof_ok {
        tracing::warn!(
            "Rejected AddPlayer for {} from client {}: device_token does not match a live login proof for this user",
            name,
            id
        );
        // Tell this specific client its reconnect was refused so it drops the stale local
        // session and falls back to the login page, instead of silently sitting there
        // showing a "connected" UI that the server never actually accepted.
        let clients = CLIENTS.lock().unwrap();
        if let Some(sender) = clients.get(&(id as usize)) {
            let _ = sender.send(ServerEvent::LogOut);
        }
        return;
    }
    {
        let mut cm = CONNECTION_MANAGER.lock().unwrap();
        match cm.device_tokens.get(&name) {
            Some(owner) if owner != &device_token => {
                tracing::warn!(
                    "Rejected AddPlayer for {} from client {}: owned by a different device",
                    name,
                    id
                );
                return;
            }
            _ => {
                cm.device_tokens.insert(name.clone(), device_token);
                cm.players.entry(name.clone()).or_default().push(id);
                tracing::info!("All connected players: {:?}", cm.players);
            }
        }
    }
    broadcast_connected_users();
}

/// Requires `device_token` to match `username`'s current `LOGIN_PROOFS` entry — i.e. the caller
/// must have actually just completed a real `login()` call for this username — otherwise a raw
/// websocket connection could broadcast a fake reconnect for (and seize the device-token lock
/// on) any username with no authentication at all.
#[cfg(feature = "server")]
pub fn login_all_sessions(username: String, sql_id: i64, device_token: String) {
    use crate::auth_manager::server_fn::auth::LOGIN_PROOFS;
    let proof_ok = LOGIN_PROOFS
        .lock()
        .unwrap()
        .get(&username)
        .is_some_and(|proof| proof == &device_token);
    if !proof_ok {
        tracing::warn!(
            "Rejected LoginAllSessions for {}: device_token does not match a live login proof for this user",
            username
        );
        return;
    }
    {
        // A fresh login only reaches here once the DB `is_connected` gate in auth.rs's
        // login() has passed, which only happens when no other session was already live for
        // this username — so any players/device_tokens entries still present are stale
        // leftovers (e.g. a client that crashed or never called logout). Reset them so this
        // login's device becomes the sole recognized owner going forward.
        let mut cm = CONNECTION_MANAGER.lock().unwrap();
        cm.players.remove(&username);
        cm.device_tokens.insert(username.clone(), device_token);
    }
    let clients = CLIENTS.lock().unwrap();
    for (&_other_id, sender) in clients.iter() {
        let _ = sender.send(ServerEvent::ReconnectAllSessions(username.clone(), sql_id));
    }
}

#[cfg(feature = "server")]
pub fn send_logout_to_server(user_name: String, client_id: u32) {
    {
        let mut cm = CONNECTION_MANAGER.lock().unwrap();
        cm.players
            .retain(|player_name, _| player_name != &user_name);
        tracing::info!("All connected players after logout: {:?}", cm.players);
    }
    {
        let clients = CLIENTS.lock().unwrap();
        if let Some(sender) = clients.get(&(client_id as usize)) {
            let _ = sender.send(ServerEvent::LogOut);
        }
    }
    broadcast_connected_users();
}

/// Admin-triggered account deletion: immediately force-logs-out every live session
/// `username` currently has and drops it from the connection manager's bookkeeping
/// so a stale client can't silently rejoin via `AddPlayer`/`LoginAllSessions` afterward.
#[cfg(feature = "server")]
pub fn force_logout_user(username: &str) {
    use crate::auth_manager::server_fn::auth::LOGIN_PROOFS;

    let client_ids = {
        let mut cm = CONNECTION_MANAGER.lock().unwrap();
        let ids = cm.players.remove(username).unwrap_or_default();
        cm.device_tokens.remove(username);
        ids
    };

    {
        let clients = CLIENTS.lock().unwrap();
        for id in &client_ids {
            if let Some(sender) = clients.get(&(*id as usize)) {
                let _ = sender.send(ServerEvent::LogOut);
            }
        }
    }

    LOGIN_PROOFS.lock().unwrap().remove(username);
    broadcast_connected_users();
}

/// Handles a closed websocket: drops this client id from whichever username it
/// belonged to, and — only once that was the user's *last* live connection — clears
/// the DB `is_connected` flag and invalidates the login proof.
///
/// Unlike a stateful game server, there's no in-progress session state here worth
/// protecting across a quick reload, so (unlike dx-rpg, which this template is based
/// on) this intentionally has no reconnect grace period: a page reload will simply
/// show the user as briefly disconnected before its new socket reconnects.
#[cfg(feature = "server")]
async fn handle_disconnect(client_id: u32) {
    let (username, was_last_connection) =
        {
            let mut cm = CONNECTION_MANAGER.lock().unwrap();

            let Some(username) = cm.players.iter().find_map(|(player_name, ids)| {
                ids.contains(&client_id).then(|| player_name.clone())
            }) else {
                // Never logged in (still on the login page) — nothing to clean up.
                return;
            };

            let mut was_last = false;
            cm.players.retain(|player_name, ids| {
                if player_name == &username {
                    ids.retain(|&id| id != client_id);
                    was_last = ids.is_empty();
                    !was_last
                } else {
                    true
                }
            });

            tracing::info!("Player {} with id {} is disconnecting", username, client_id);
            (username, was_last)
        };

    if !was_last_connection {
        tracing::info!(
            "{} still has other active connections, not closing anything",
            username
        );
        return;
    }

    match update_connection_status(username.clone(), false).await {
        Ok(_) => tracing::info!("{} disconnected from db", username),
        Err(e) => tracing::error!("DB error: {:?}", e),
    }
    {
        use crate::auth_manager::server_fn::auth::LOGIN_PROOFS;
        LOGIN_PROOFS.lock().unwrap().remove(&username);
    }
    broadcast_connected_users();
}

/// Pure helper (unit-tested below): the sorted list of usernames currently connected.
#[cfg(feature = "server")]
fn sorted_usernames(players: &HashMap<String, Vec<u32>>) -> Vec<String> {
    let mut usernames: Vec<String> = players.keys().cloned().collect();
    usernames.sort();
    usernames
}

/// Broadcasts the current connected-users list to every live client (logged in or not
/// — seeing who else is online is part of the demo, even before you sign in yourself).
#[cfg(feature = "server")]
fn broadcast_connected_users() {
    let usernames = sorted_usernames(&CONNECTION_MANAGER.lock().unwrap().players);
    let clients = CLIENTS.lock().unwrap();
    for sender in clients.values() {
        let _ = sender.send(ServerEvent::ConnectedUsers(usernames.clone()));
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;

    #[test]
    fn sorted_usernames_orders_alphabetically() {
        let mut players = HashMap::new();
        players.insert("Zoe".to_owned(), vec![3]);
        players.insert("Amir".to_owned(), vec![1]);
        players.insert("Bao".to_owned(), vec![2]);
        assert_eq!(sorted_usernames(&players), vec!["Amir", "Bao", "Zoe"]);
    }

    #[test]
    fn sorted_usernames_empty_when_no_players() {
        assert!(sorted_usernames(&HashMap::new()).is_empty());
    }

    #[test]
    fn sorted_usernames_ignores_client_id_lists() {
        // A username with multiple connected client ids (e.g. two browser tabs)
        // still appears exactly once.
        let mut players = HashMap::new();
        players.insert("Amir".to_owned(), vec![1, 2, 3]);
        assert_eq!(sorted_usernames(&players), vec!["Amir"]);
    }
}

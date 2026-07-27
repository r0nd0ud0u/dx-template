pub mod event;
// Not public outside of this module
#[cfg(feature = "server")]
mod common_event;

pub const NO_CLIENT_ID: i64 = -1;
pub const STARTING_CLIENT_ID: i64 = 1;

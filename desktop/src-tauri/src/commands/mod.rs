pub mod account;
pub mod session;
pub mod message;
pub mod contact;
pub mod browser;
pub mod translation;
pub mod sync;

use rusqlite::Connection;
use std::sync::{Mutex, Arc, RwLock};
use crate::cloak::pool::InstancePool;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub pool: Arc<RwLock<InstancePool>>,
}

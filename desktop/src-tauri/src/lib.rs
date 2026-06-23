mod config;
mod error;
pub mod commands;
pub mod cloak;
pub mod protocol;
pub mod translation;
pub mod storage;
pub mod mqtt;
pub mod sync;
pub mod audio;

use config::AppConfig;
use commands::AppState;
use cloak::pool::InstancePool;
use cloak::config::PoolConfig;
use std::sync::{Mutex, Arc, RwLock};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = AppConfig::default();
    if let Err(e) = config.ensure_dirs() {
        eprintln!("Failed to create app directories: {}", e);
    }
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(&config.log_level)
    ).init();

    // 初始化 Job Object 进程保护 — 主进程退出时自动杀死所有 CloakBrowser
    if let Err(e) = cloak::job::init() {
        log::warn!("Job Object 初始化失败: {}", e);
    }

    let conn = storage::open(&config.db_path).expect("Failed to open database");
    let pool = Arc::new(RwLock::new(InstancePool::new(PoolConfig::default())));
    let state = AppState { db: Mutex::new(conn), pool };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::account::list_accounts,
            commands::account::get_account,
            commands::account::add_account,
            commands::account::update_account,
            commands::account::delete_account,
            commands::session::list_sessions,
            commands::session::get_session,
            commands::session::update_session,
            commands::session::delete_session,
            commands::message::list_messages,
            commands::message::search_messages,
            commands::message::delete_message,
            commands::contact::list_contacts,
            commands::contact::search_contacts,
            commands::contact::update_contact,
            commands::contact::delete_contact,
            commands::browser::open_session_browser,
            commands::browser::close_browser,
            commands::browser::switch_browser_session,
            
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

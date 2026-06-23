use tauri::State;
use crate::commands::AppState;
use crate::storage::{message, models::Message};

#[tauri::command]
pub fn list_messages(state: State<AppState>, session_id: String, limit: i64, offset: i64) -> Result<Vec<Message>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    message::list(&conn, &session_id, limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_messages(state: State<AppState>, account_id: String, query: String, limit: i64) -> Result<Vec<Message>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    message::search(&conn, &account_id, &query, limit).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_message(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    message::delete(&conn, &id).map_err(|e| e.to_string())?;
    Ok(())
}

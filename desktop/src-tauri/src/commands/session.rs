use tauri::State;
use crate::commands::AppState;
use crate::storage::{session, models::Session};

#[tauri::command]
pub fn list_sessions(state: State<AppState>, account_id: String, limit: i64, offset: i64) -> Result<Vec<Session>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    session::list(&conn, &account_id, limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_session(state: State<AppState>, id: String) -> Result<Option<Session>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    session::get(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_session(state: State<AppState>, s: Session) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    session::update(&conn, &s).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_session(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    session::delete(&conn, &id).map_err(|e| e.to_string())?;
    Ok(())
}

use tauri::State;
use crate::commands::AppState;
use crate::storage::{contact, models::Contact};

#[tauri::command]
pub fn list_contacts(state: State<AppState>, account_id: String) -> Result<Vec<Contact>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    contact::list(&conn, &account_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_contacts(state: State<AppState>, account_id: String, query: String) -> Result<Vec<Contact>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    contact::search(&conn, &account_id, &query).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_contact(state: State<AppState>, c: Contact) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    contact::update(&conn, &c).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_contact(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    contact::delete(&conn, &id).map_err(|e| e.to_string())?;
    Ok(())
}

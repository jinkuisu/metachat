use tauri::State;
use crate::commands::AppState;
use crate::storage::{account, models::Account};

#[tauri::command]
pub fn list_accounts(state: State<AppState>) -> Result<Vec<Account>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    account::list(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_account(state: State<AppState>, id: String) -> Result<Option<Account>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    account::get(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_account(state: State<AppState>, account: Account) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    account::insert(&conn, &account).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn update_account(state: State<AppState>, account: Account) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    account::update(&conn, &account).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_account(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    account::delete(&conn, &id).map_err(|e| e.to_string())?;
    Ok(())
}

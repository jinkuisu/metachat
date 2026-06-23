#[tauri::command]
pub fn query_messages(_session_id: String, _limit: u32) -> Result<String, String> { Ok(String::new()) }
#[tauri::command]
pub fn migrate_database() -> Result<String, String> { Ok(String::new()) }

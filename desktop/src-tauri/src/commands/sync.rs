#[tauri::command]
pub fn upload_contacts() -> Result<bool, String> { Ok(true) }
#[tauri::command]
pub fn full_upload() -> Result<bool, String> { Ok(true) }

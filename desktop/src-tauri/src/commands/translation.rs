#[tauri::command]
pub fn translate(_text: String, _source: String, _target: String) -> Result<String, String> { Ok(String::new()) }
#[tauri::command]
pub fn batch_translate(_texts: Vec<String>, _target: String) -> Result<Vec<String>, String> { Ok(vec![]) }

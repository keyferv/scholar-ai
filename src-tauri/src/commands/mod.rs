#[tauri::command]
pub fn health_check() -> String {
    "ScholarAI is running".to_string()
}

#[tauri::command]
pub fn get_app_info() -> serde_json::Value {
    serde_json::json!({
        "name": "ScholarAI",
        "version": "0.1.0",
    })
}
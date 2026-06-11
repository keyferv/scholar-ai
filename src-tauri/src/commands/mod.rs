use serde_json::json;

#[tauri::command]
pub fn health_check() -> serde_json::Value {
    json!({
        "name": "ScholarAI",
        "version": "0.1.0",
        "status": "running",
    })
}

#[tauri::command]
pub fn get_app_info() -> serde_json::Value {
    json!({
        "name": "ScholarAI",
        "version": "0.1.0",
    })
}
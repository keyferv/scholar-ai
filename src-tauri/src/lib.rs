mod ai_bridge;
mod commands;
mod config;
mod db;

use ai_bridge::AiBridge;
use commands::{
    add_provider, delete_provider, get_app_info, health_check, list_providers,
    set_active_provider, update_provider,
};
use config::ConfigManager;
use db::Database;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let db = Database::new(app.path().app_data_dir()?.join("scholar-ai.db"))?;
            db.run_migrations()?;
            app.manage(db);

            let config = ConfigManager::new(app.path().app_data_dir()?.join("config.json"));
            app.manage(config);

            let ai_bridge = AiBridge::new();
            // Graceful degradation: if sidecar fails to spawn, log and continue.
            // Commands that need the sidecar will return errors individually.
            if let Err(e) = ai_bridge.spawn_sidecar() {
                log::error!("Failed to spawn sidecar (will degrade gracefully): {}", e);
            }
            app.manage(ai_bridge);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            health_check,
            get_app_info,
            ai_bridge::start_sidecar,
            ai_bridge::sidecar_health_command,
            ai_bridge::test_provider,
            ai_bridge::send_chat_message,
            list_providers,
            add_provider,
            update_provider,
            delete_provider,
            set_active_provider,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    // Hook sidecar cleanup to app exit.
    app.run(|_app_handle, event| {
        if let tauri::RunEvent::Exit = event {
            // Sidecar cleanup happens automatically when AiBridge is dropped.
            // The Drop impl calls stop_sidecar().
            log::info!("App exiting, cleaning up sidecar...");
        }
    });
}
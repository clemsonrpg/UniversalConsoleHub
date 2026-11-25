pub mod adapters;
pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod ipc;

use tauri::Manager;

use crate::adapters::registry::AdapterRegistry;
use crate::application::services::AppServices;
use crate::infrastructure::db::manager::DatabaseManager;
use crate::infrastructure::system::logging::init_logging;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_local_data_dir()
                .map_err(|e| std::io::Error::other(format!("app_local_data_dir failed: {e}")))?;
            let db_path = app_data_dir.join("universal_console_hub.db");
            let backup_dir = app_data_dir.join("backups");

            let db = DatabaseManager::open(&db_path)
                .map_err(|e| std::io::Error::other(format!("sqlite init failed: {e}")))?;
            let services =
                AppServices::new(db, AdapterRegistry::with_phase_one_adapters(), backup_dir);

            app.manage(services);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::commands::ping_hub,
            ipc::commands::get_hub_bootstrap,
            ipc::commands::list_consoles,
            ipc::commands::get_console_details,
            ipc::commands::list_emulators,
            ipc::commands::configure_emulator_path,
            ipc::commands::inspect_rpcs3,
            ipc::commands::register_firmware_file,
            ipc::commands::get_active_profile,
            ipc::commands::update_active_profile,
            ipc::commands::list_games,
            ipc::commands::register_game,
            ipc::commands::toggle_game_favorite,
            ipc::commands::list_achievement_providers,
            ipc::commands::list_trophies,
            ipc::commands::get_app_settings,
            ipc::commands::update_app_setting,
            ipc::commands::create_database_backup,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Universal Console Hub");
}

#[cfg(test)]
mod tests {
    use crate::ipc::commands::ping_hub;

    #[test]
    fn ping_hub_returns_phase_one_health() {
        let health = ping_hub();
        assert_eq!(health.app_name, "Universal Console Hub");
        assert_eq!(health.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(health.phase, 1);
    }
}

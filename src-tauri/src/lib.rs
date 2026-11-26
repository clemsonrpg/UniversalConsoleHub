pub mod adapters;
pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod ipc;

use std::sync::Arc;

use tauri::Manager;
use tracing::{info, warn};

use crate::adapters::registry::AdapterRegistry;
use crate::application::live_session_orchestrator::TauriSessionEventSink;
use crate::application::services::AppServices;
use crate::infrastructure::db::manager::DatabaseManager;
use crate::infrastructure::system::logging::init_logging;
use crate::infrastructure::system::window_manager::TauriWindowManager;

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
            let cover_cache_dir = app_data_dir.join("cache").join("covers");

            let db = DatabaseManager::open(&db_path)
                .map_err(|e| std::io::Error::other(format!("sqlite init failed: {e}")))?;
            let services = AppServices::with_dirs(
                db,
                AdapterRegistry::with_phase_one_adapters(),
                backup_dir,
                cover_cache_dir,
            );
            services.live_session_orchestrator().attach_runtime_hooks(
                Arc::new(TauriWindowManager::new(app.handle().clone())),
                Arc::new(TauriSessionEventSink::new(app.handle().clone())),
            );
            services
                .gamepad_monitor()
                .start_with_tauri(app.handle().clone());

            match services.session_service().reconcile_orphaned_sessions() {
                Ok(report) if report.recovered_count > 0 => {
                    info!(
                        recovered = report.recovered_count,
                        seconds = report.total_recovered_seconds,
                        "reconciled orphaned game sessions on startup using last_heartbeat_at"
                    );
                }
                Ok(_) => {}
                Err(err) => {
                    warn!(error = %err, "failed to reconcile orphaned sessions on startup");
                }
            }

            app.manage(services);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Phase 1 commands (18 — all preserved)
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
            // Phase 2 commands (13 — total 31)
            ipc::commands::get_console_environment,
            ipc::commands::add_console_scan_path,
            ipc::commands::remove_console_scan_path,
            ipc::commands::scan_console_library,
            ipc::commands::update_console_setting,
            ipc::commands::get_game_details,
            ipc::commands::update_game_metadata,
            ipc::commands::set_game_cover,
            ipc::commands::get_statistics_overview,
            ipc::commands::list_activity_history,
            ipc::commands::list_database_backups,
            ipc::commands::inspect_database_backup,
            ipc::commands::restore_database_backup,
            // Phase 3 commands (7 — total 38)
            ipc::commands::get_console_profile,
            ipc::commands::sync_console_profile,
            ipc::commands::launch_console_system,
            ipc::commands::launch_game,
            ipc::commands::get_active_session,
            ipc::commands::stop_active_session,
            ipc::commands::record_runtime_verification,
            // Phase 4 commands (3 — total 41)
            ipc::commands::sync_console_trophies,
            ipc::commands::get_trophy_center_overview,
            ipc::commands::list_game_trophies,
            // Theme Packages & Windows Gamepad commands (4 — total 45)
            ipc::commands::list_installed_themes,
            ipc::commands::import_theme_package,
            ipc::commands::remove_installed_theme,
            ipc::commands::list_connected_gamepads,
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

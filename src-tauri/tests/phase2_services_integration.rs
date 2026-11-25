use std::fs;

use tempfile::TempDir;
use universal_console_hub_lib::adapters::discovery::param_sfo::build_synthetic_param_sfo;
use universal_console_hub_lib::adapters::registry::AdapterRegistry;
use universal_console_hub_lib::application::services::AppServices;
use universal_console_hub_lib::domain::environment::ConsoleEnvironmentMode;
use universal_console_hub_lib::domain::scanner::ScanProgressEvent;
use universal_console_hub_lib::domain::session::EXIT_STATUS_RECOVERED_AFTER_CRASH;
use universal_console_hub_lib::infrastructure::db::console_env_repo::UpdateGameMetadataInput;
use universal_console_hub_lib::infrastructure::db::manager::DatabaseManager;

const MINIMAL_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00,
];

#[test]
fn end_to_end_phase2_services_scan_sessions_recovery_covers_stats_and_live_restore() {
    let tmp = TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("phase2_app.db");
    let backup_dir = tmp.path().join("backups");
    let cover_cache_dir = tmp.path().join("cache").join("covers");
    let db = DatabaseManager::open(&db_path).expect("open db");
    let services = AppServices::with_dirs(
        db,
        AdapterRegistry::with_phase_one_adapters(),
        backup_dir,
        cover_cache_dir.clone(),
    );

    // 1. Verify ConsoleEnvironment modes (PS3 authentic-capable vs PS4 integrated alternative)
    let ps3_env = services
        .get_console_environment("ps3")
        .expect("ps3 environment");
    assert_eq!(
        ps3_env.environment_mode,
        ConsoleEnvironmentMode::AuthenticSystemCapable
    );
    let ps4_env = services
        .get_console_environment("ps4")
        .expect("ps4 environment");
    assert_eq!(
        ps4_env.environment_mode,
        ConsoleEnvironmentMode::IntegratedAlternativeOnly
    );

    // 2. Add PS3 scan directory with a synthetic PARAM.SFO + EBOOT.BIN and scan twice (idempotent)
    let ps3_dir = tmp.path().join("Área de Trabalho").join("PS3_Games");
    let game_dir = ps3_dir.join("Demons_Souls").join("PS3_GAME");
    fs::create_dir_all(game_dir.join("USRDIR")).unwrap();
    fs::write(
        game_dir.join("PARAM.SFO"),
        build_synthetic_param_sfo(&[
            ("APP_VER", "01.00"),
            ("CATEGORY", "DG"),
            ("TITLE", "Demon's Souls"),
            ("TITLE_ID", "BLUS30443"),
        ]),
    )
    .unwrap();
    fs::write(
        game_dir.join("USRDIR").join("EBOOT.BIN"),
        b"SYNTHETIC_EBOOT",
    )
    .unwrap();

    services
        .environment_service()
        .add_scan_path("ps3", &ps3_dir.to_string_lossy(), true)
        .expect("add ps3 scan path");

    let mut progress_events = Vec::new();
    let scan1 = services
        .scanner_service()
        .scan_console("ps3", |evt| progress_events.push(evt))
        .expect("first ps3 scan");
    assert_eq!(scan1.games_added, 1);
    assert_eq!(scan1.versions_added, 1);
    assert_eq!(scan1.skipped_existing, 0);
    assert!(progress_events
        .iter()
        .any(|e| matches!(e, ScanProgressEvent::Started { .. })));
    assert!(progress_events
        .iter()
        .any(|e| matches!(e, ScanProgressEvent::Completed { .. })));

    let scan2 = services
        .scanner_service()
        .scan_console("ps3", |_| {})
        .expect("second idempotent ps3 scan");
    assert_eq!(scan2.games_added, 0);
    assert_eq!(scan2.versions_added, 0);
    assert_eq!(scan2.skipped_existing, 1);

    // 3. Cover art import + metadata update via EnvironmentService
    let ps3_env_after = services.get_console_environment("ps3").unwrap();
    assert_eq!(ps3_env_after.games.len(), 1);
    let game_id = ps3_env_after.games[0].id.clone();

    let src_cover = tmp.path().join("demons_cover.png");
    fs::write(&src_cover, MINIMAL_PNG).unwrap();
    let with_cover = services
        .environment_service()
        .set_game_cover(&game_id, &src_cover.to_string_lossy())
        .expect("set cover");
    let cover_str = with_cover.cover_path.expect("cover saved");
    assert!(cover_str.contains("covers"));

    services
        .environment_service()
        .update_game_metadata(UpdateGameMetadataInput {
            game_id: game_id.clone(),
            description: "Clássico PS3".to_string(),
            developer: Some("FromSoftware".to_string()),
            publisher: Some("Atlus".to_string()),
            release_year: Some(2009),
            genre: Some("Action RPG".to_string()),
            completion_percent: 100,
        })
        .expect("update metadata");

    // 4. Session lifecycle + crash recovery strictly from last_heartbeat_at + idempotency
    let started = services
        .session_service()
        .start_recorded_session(&game_id, None, "emu-rpcs3", Some("2026-11-15T18:00:00Z"))
        .expect("start session");
    services
        .session_service()
        .heartbeat_session(&started.id, Some("2026-11-15T18:40:00Z"))
        .expect("heartbeat at +2400s");

    let rec1 = services
        .session_service()
        .reconcile_orphaned_sessions()
        .expect("reconcile orphan");
    assert_eq!(rec1.recovered_count, 1);
    assert_eq!(rec1.total_recovered_seconds, 2400);

    let rec2 = services
        .session_service()
        .reconcile_orphaned_sessions()
        .expect("idempotent second reconcile");
    assert_eq!(rec2.recovered_count, 0);

    let details = services
        .environment_service()
        .get_game_details(&game_id)
        .expect("game details");
    assert_eq!(details.sessions.len(), 1);
    assert_eq!(
        details.sessions[0].exit_status,
        EXIT_STATUS_RECOVERED_AFTER_CRASH
    );
    assert_eq!(details.sessions[0].duration_seconds, 2400);

    let stats = services
        .statistics_service()
        .get_statistics_overview()
        .expect("stats overview");
    assert_eq!(stats.total_playtime_seconds, 2400);
    assert_eq!(stats.games_completed, 1);
    assert_eq!(stats.completion_rate_percent, 100);
    assert_eq!(stats.top_games.len(), 1);

    let activity = services
        .statistics_service()
        .list_activity_history(Some("library_scanned"), 20)
        .expect("filtered activity");
    assert!(!activity.is_empty());

    // 5. Backup list, inspect, mutate, and live restore through BackupService
    let backup = services
        .backup_service()
        .create_backup(Some("phase2_snapshot.db".to_string()))
        .expect("create backup");
    let listed = services
        .backup_service()
        .list_backups()
        .expect("list backups");
    assert_eq!(listed.len(), 1);

    let inspected = services
        .backup_service()
        .inspect_backup(&backup.backup_path)
        .expect("inspect backup");
    assert_eq!(inspected.games_count, 1);

    // Mutate DB then restore live
    services
        .repos
        .db()
        .with_connection(|conn| {
            conn.execute("DELETE FROM games", []).unwrap();
            Ok(())
        })
        .unwrap();
    assert!(services
        .get_console_environment("ps3")
        .unwrap()
        .games
        .is_empty());

    let restored = services
        .backup_service()
        .restore_backup(&backup.backup_path)
        .expect("restore backup live");
    assert!(restored.integrity_ok);
    assert_eq!(
        services.get_console_environment("ps3").unwrap().games.len(),
        1
    );
}

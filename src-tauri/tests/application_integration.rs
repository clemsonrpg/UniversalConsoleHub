use std::fs;

use tempfile::TempDir;
use universal_console_hub_lib::adapters::registry::AdapterRegistry;
use universal_console_hub_lib::application::services::{
    AppServices, RegisterGameRequest, UpdateProfileRequest,
};
use universal_console_hub_lib::infrastructure::db::manager::DatabaseManager;

#[test]
fn bootstrap_returns_real_sqlite_state_and_honest_unconfigured_rpcs3() {
    let tmp = TempDir::new().expect("tempdir");
    let db = DatabaseManager::open(&tmp.path().join("app.db")).expect("open db");
    let services = AppServices::new(
        db,
        AdapterRegistry::with_phase_one_adapters(),
        tmp.path().join("backups"),
    );

    let boot = services.get_bootstrap().expect("bootstrap");
    assert_eq!(boot.app_name, "Universal Console Hub");
    assert_eq!(boot.phase, 1);
    assert_eq!(boot.consoles.len(), 6);
    assert_eq!(boot.emulators.len(), 6);
    assert_eq!(boot.achievement_providers.len(), 5);
    assert_eq!(boot.games.len(), 0);
    assert_eq!(boot.statistics.total_playtime_seconds, 0);
    assert!(!boot.rpcs3_inspection.configured);
}

#[test]
fn configures_rpcs3_validates_firmware_and_keeps_xmb_runtime_unconfirmed_in_phase_one() {
    let tmp = TempDir::new().expect("tempdir");
    let db = DatabaseManager::open(&tmp.path().join("app.db")).expect("open db");
    let services = AppServices::new(
        db,
        AdapterRegistry::with_phase_one_adapters(),
        tmp.path().join("backups"),
    );

    let rpcs3_dir = tmp.path().join("RPCS3");
    let vsh_dir = rpcs3_dir.join("dev_flash").join("vsh").join("module");
    fs::create_dir_all(&vsh_dir).expect("mkdir vsh");
    fs::create_dir_all(rpcs3_dir.join("dev_hdd0")).expect("mkdir hdd");
    let exe_path = rpcs3_dir.join("rpcs3.exe");
    fs::write(&exe_path, b"synthetic-exe").expect("exe");
    fs::write(vsh_dir.join("vsh.self"), b"synthetic-vsh").expect("vsh");

    let synthetic_pup = tmp.path().join("PS3UPDAT.PUP");
    fs::write(&synthetic_pup, b"SCEUF\x00\x00\x00test").expect("pup");

    let configured = services
        .configure_emulator_path("rpcs3", &exe_path.to_string_lossy())
        .expect("configure rpcs3");
    assert!(configured.executable_path.is_some());

    let fw = services
        .register_firmware_file("ps3", "rpcs3", &synthetic_pup.to_string_lossy())
        .expect("register firmware");
    assert_eq!(fw.validation_status, "package_header_and_hash_validated");

    let details = services.get_console_details("ps3").expect("ps3 details");
    let inspection = details.rpcs3_inspection.expect("rpcs3 inspection");
    assert!(inspection.configured);
    let readiness = inspection.readiness.expect("readiness");
    assert!(readiness.firmware_package_validated);
    assert!(readiness.system_files_present);
    assert!(readiness.command_buildable);
    assert!(!readiness.xmb_runtime_confirmed);
}

#[test]
fn persists_profile_game_favorite_setting_and_safe_backup_end_to_end() {
    let tmp = TempDir::new().expect("tempdir");
    let db = DatabaseManager::open(&tmp.path().join("app.db")).expect("open db");
    let services = AppServices::new(
        db,
        AdapterRegistry::with_phase_one_adapters(),
        tmp.path().join("backups"),
    );

    let profile = services
        .update_active_profile(UpdateProfileRequest {
            username: "clemson".to_string(),
            display_name: "Clemson".to_string(),
            bio: "PS3 & Retro enthusiast".to_string(),
            privacy_level: "friends".to_string(),
        })
        .expect("update profile");
    assert_eq!(profile.display_name, "Clemson");

    let (game, _) = services
        .register_game(RegisterGameRequest {
            console_slug: "ps3".to_string(),
            title: "God of War III".to_string(),
            version_label: "v1.03".to_string(),
            region: "USA".to_string(),
            serial_code: Some("BCUS-98111".to_string()),
            file_path: r"D:\PS3\GOW3\PS3_GAME\USRDIR\EBOOT.BIN".to_string(),
            metadata_source: "manual_user_entry".to_string(),
        })
        .expect("register game");
    assert!(services.toggle_game_favorite(&game.id).expect("favorite"));

    let settings = services
        .update_setting("ui.fullscreen_default", "true")
        .expect("update setting");
    assert_eq!(
        settings.get("ui.fullscreen_default").map(String::as_str),
        Some("true")
    );

    // Path traversal backup filename must be rejected
    assert!(services
        .create_database_backup(Some("../escape.db".to_string()))
        .is_err());

    let backup = services
        .create_database_backup(Some("manual_test_backup.db".to_string()))
        .expect("create backup");
    assert!(backup.integrity_ok);
    assert!(backup.backup_path.is_file());
}

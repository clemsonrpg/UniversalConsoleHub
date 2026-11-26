use std::fs;

use tempfile::TempDir;
use universal_console_hub_lib::adapters::registry::AdapterRegistry;
use universal_console_hub_lib::application::services::AppServices;
use universal_console_hub_lib::domain::console_profile::{
    DataOrigin, EmulatorStatus, SyncTrigger, TrophySyncReadiness,
};
use universal_console_hub_lib::infrastructure::db::console_profile_repo::ConsoleProfileRepository;
use universal_console_hub_lib::infrastructure::db::manager::DatabaseManager;

const MINIMAL_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

fn build_synthetic_param_sfo(title: &str, title_id: &str, app_ver: &str) -> Vec<u8> {
    let entries: Vec<(&str, &str)> = vec![
        ("APP_VER", app_ver),
        ("CATEGORY", "DG"),
        ("TITLE", title),
        ("TITLE_ID", title_id),
    ];

    let mut key_table = Vec::new();
    let mut key_offsets = Vec::new();
    for (k, _) in &entries {
        key_offsets.push(key_table.len() as u16);
        key_table.extend_from_slice(k.as_bytes());
        key_table.push(0);
    }
    while key_table.len() % 4 != 0 {
        key_table.push(0);
    }

    let mut data_table = Vec::new();
    let mut data_meta = Vec::new();
    for (_, v) in &entries {
        let offset = data_table.len() as u32;
        let mut bytes = v.as_bytes().to_vec();
        bytes.push(0);
        let len = bytes.len() as u32;
        let mut max_len = len;
        while max_len % 4 != 0 {
            max_len += 1;
        }
        bytes.resize(max_len as usize, 0);
        data_table.extend_from_slice(&bytes);
        data_meta.push((len, max_len, offset));
    }

    let header_len = 20u32;
    let index_len = (entries.len() * 16) as u32;
    let key_table_start = header_len + index_len;
    let data_table_start = key_table_start + key_table.len() as u32;

    let mut out = Vec::new();
    out.extend_from_slice(b"\0PSF");
    out.extend_from_slice(&0x0000_0101u32.to_le_bytes());
    out.extend_from_slice(&key_table_start.to_le_bytes());
    out.extend_from_slice(&data_table_start.to_le_bytes());
    out.extend_from_slice(&(entries.len() as u32).to_le_bytes());

    for (i, &(len, max_len, offset)) in data_meta.iter().enumerate() {
        out.extend_from_slice(&key_offsets[i].to_le_bytes());
        out.extend_from_slice(&0x0204u16.to_le_bytes());
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&max_len.to_le_bytes());
        out.extend_from_slice(&offset.to_le_bytes());
    }
    out.extend_from_slice(&key_table);
    out.extend_from_slice(&data_table);
    out
}

#[test]
fn syncs_ps3_profile_idempotently_caches_native_icons_and_preserves_history_when_files_disappear() {
    let tmp = TempDir::new().expect("tempdir");
    let db = DatabaseManager::open(&tmp.path().join("sync.db")).expect("open db");
    let services = AppServices::with_dirs(
        db.clone(),
        AdapterRegistry::with_phase_one_adapters(),
        tmp.path().join("backups"),
        tmp.path().join("cache").join("covers"),
    );

    // Build synthetic RPCS3 tree with firmware, 2 local users, save, trophy dir, and an HDD game with ICON0.PNG
    let rpcs3_root = tmp.path().join("RPCS3");
    fs::create_dir_all(rpcs3_root.join("dev_flash").join("vsh").join("etc")).unwrap();
    fs::create_dir_all(rpcs3_root.join("dev_flash").join("vsh").join("module")).unwrap();
    fs::create_dir_all(
        rpcs3_root
            .join("dev_hdd0")
            .join("home")
            .join("00000001")
            .join("savedata")
            .join("BLUS30443-SAVE"),
    )
    .unwrap();
    fs::create_dir_all(
        rpcs3_root
            .join("dev_hdd0")
            .join("home")
            .join("00000001")
            .join("trophy"),
    )
    .unwrap();
    fs::create_dir_all(rpcs3_root.join("dev_hdd0").join("home").join("00000002")).unwrap();

    let exe_path = rpcs3_root.join("rpcs3.exe");
    fs::write(&exe_path, b"MZ-rpcs3").unwrap();
    fs::write(rpcs3_root.join("RPCS3.log"), "RPCS3 v0.0.34-17105\n").unwrap();
    fs::write(
        rpcs3_root
            .join("dev_flash")
            .join("vsh")
            .join("etc")
            .join("version.txt"),
        "release:04.9100:\n",
    )
    .unwrap();
    fs::write(
        rpcs3_root
            .join("dev_flash")
            .join("vsh")
            .join("module")
            .join("vsh.self"),
        b"vsh",
    )
    .unwrap();
    fs::write(
        rpcs3_root
            .join("dev_hdd0")
            .join("home")
            .join("00000001")
            .join("localusername"),
        "ClemsonPS3",
    )
    .unwrap();
    fs::write(
        rpcs3_root
            .join("dev_hdd0")
            .join("home")
            .join("00000002")
            .join("localusername"),
        "SegundoUsuario",
    )
    .unwrap();

    // Native dev_hdd0/game title with PARAM.SFO + ICON0.PNG + EBOOT.BIN
    let hdd_game = rpcs3_root.join("dev_hdd0").join("game").join("NPUB30001");
    fs::create_dir_all(hdd_game.join("USRDIR")).unwrap();
    let eboot_path = hdd_game.join("USRDIR").join("EBOOT.BIN");
    fs::write(&eboot_path, b"EBOOT").unwrap();
    fs::write(
        hdd_game.join("PARAM.SFO"),
        build_synthetic_param_sfo("Journey", "NPUB30001", "01.02"),
    )
    .unwrap();
    fs::write(hdd_game.join("ICON0.PNG"), MINIMAL_PNG).unwrap();

    // 1. Configuring RPCS3 executable automatically triggers console profile sync!
    services
        .configure_emulator_path("rpcs3", &exe_path.to_string_lossy())
        .expect("configure rpcs3");

    let overview = services
        .console_profile_sync_service()
        .get_console_profile_overview("ps3")
        .expect("get ps3 profile overview");

    assert_eq!(
        overview.emulator_status.value,
        Some(EmulatorStatus::Available)
    );
    assert_eq!(overview.emulator_status.origin, DataOrigin::NativePlatform);
    assert_eq!(overview.firmware_version.value.as_deref(), Some("4.91"));
    assert_eq!(overview.firmware_version.origin, DataOrigin::NativePlatform);
    assert_eq!(overview.local_users.len(), 2);
    assert_eq!(overview.hub_stats.games_available, 1);
    assert_eq!(overview.hub_stats.games_missing_on_disk, 0);
    assert_eq!(
        overview
            .profile
            .as_ref()
            .expect("persisted profile")
            .trophy_sync_readiness,
        TrophySyncReadiness::Phase4Planned.as_str()
    );

    // Native ICON0.PNG must have been auto-cached into authorized cover_cache_dir
    let games = services.list_games_sanitized().expect("list games");
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].title, "Journey");
    let cover = games[0]
        .cover_path
        .as_deref()
        .expect("ICON0.PNG should be auto-cached");
    assert!(
        cover.contains("covers"),
        "auto-imported cover must reside in authorized cover cache dir: {cover}"
    );

    // Record a real 30-minute session for Journey
    let emu_id = services
        .repos
        .list_emulators()
        .unwrap()
        .into_iter()
        .find(|e| e.slug == "rpcs3")
        .unwrap()
        .id;
    let sess = services
        .session_service()
        .start_recorded_session(&games[0].id, None, &emu_id, Some("2026-11-25T18:00:00Z"))
        .expect("start session");
    services
        .session_service()
        .finish_session(&sess.id, Some("2026-11-25T18:30:00Z"), "clean_exit")
        .expect("finish session");

    // 2. Running manual sync a second time MUST be 100% idempotent
    let second_sync = services
        .console_profile_sync_service()
        .sync_console_profile("ps3", SyncTrigger::Manual)
        .expect("second idempotent sync");
    assert_eq!(second_sync.games_discovered, 0);
    assert_eq!(second_sync.covers_imported, 0);
    assert_eq!(second_sync.versions_available, 1);
    assert_eq!(second_sync.versions_marked_missing, 0);
    assert_eq!(second_sync.local_users_detected, 2);

    // 3. Remove EBOOT.BIN, remove second user, and remove rpcs3.exe -> sync preserves ALL history (Adjustment B)
    fs::remove_file(&eboot_path).unwrap();
    fs::remove_dir_all(rpcs3_root.join("dev_hdd0").join("home").join("00000002")).unwrap();
    fs::remove_file(&exe_path).unwrap();

    let after_removal = services
        .console_profile_sync_service()
        .sync_console_profile("ps3", SyncTrigger::Manual)
        .expect("sync after file removals");
    assert_eq!(after_removal.emulator_status, EmulatorStatus::MissingOnDisk);
    assert_eq!(after_removal.versions_marked_missing, 1);
    assert_eq!(after_removal.local_users_marked_missing, 2);

    let overview_after = services
        .console_profile_sync_service()
        .get_console_profile_overview("ps3")
        .expect("overview after removal");
    assert_eq!(
        overview_after.emulator_status.value,
        Some(EmulatorStatus::MissingOnDisk)
    );
    assert_eq!(overview_after.hub_stats.games_total, 1);
    assert_eq!(overview_after.hub_stats.games_available, 0);
    assert_eq!(overview_after.hub_stats.games_missing_on_disk, 1);
    assert_eq!(
        overview_after.hub_stats.total_playtime_seconds, 1800,
        "playtime MUST be preserved when game & emulator go missing"
    );
    assert_eq!(overview_after.hub_stats.total_sessions, 1);

    // 4. Restore EBOOT.BIN & rpcs3.exe -> sync restores availability cleanly!
    fs::write(&exe_path, b"MZ-rpcs3").unwrap();
    fs::write(&eboot_path, b"EBOOT").unwrap();
    let restored = services
        .console_profile_sync_service()
        .sync_console_profile("ps3", SyncTrigger::Manual)
        .expect("sync after restore");
    assert_eq!(restored.emulator_status, EmulatorStatus::Available);
    assert_eq!(restored.versions_available, 1);
    assert_eq!(restored.versions_marked_missing, 0);

    let repo = ConsoleProfileRepository::new(db);
    let users = repo.list_local_users("ps3").expect("users");
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].availability_status, "available");
    assert_eq!(users[1].availability_status, "missing_on_disk");
}

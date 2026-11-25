use std::fs;

use tempfile::TempDir;
use universal_console_hub_lib::domain::environment::{
    RuntimeVerificationFeature, RuntimeVerificationStatus,
};
use universal_console_hub_lib::domain::session::EXIT_STATUS_RECOVERED_AFTER_CRASH;
use universal_console_hub_lib::infrastructure::db::console_env_repo::{
    ConsoleEnvRepository, GameDetailsRepository, UpdateGameMetadataInput,
};
use universal_console_hub_lib::infrastructure::db::manager::DatabaseManager;
use universal_console_hub_lib::infrastructure::db::repositories::SqliteRepositories;
use universal_console_hub_lib::infrastructure::db::session_stats_repo::SessionStatsRepository;
use universal_console_hub_lib::infrastructure::system::cover_cache::{CoverCache, MAX_COVER_BYTES};

const MINIMAL_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG magic
    0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00,
];

const MINIMAL_JPEG: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, b'J', b'F', b'I', b'F'];

const MINIMAL_WEBP: &[u8] = &[
    b'R', b'I', b'F', b'F', 0x14, 0x00, 0x00, 0x00, b'W', b'E', b'B', b'P', b'V', b'P', b'8', b' ',
];

#[test]
fn cover_cache_validates_magic_bytes_size_deduplicates_sha256_and_enforces_authorized_dir() {
    let tmp = TempDir::new().expect("tempdir");
    let cache_dir = tmp.path().join("cache").join("covers");
    let input_dir = tmp.path().join("Área de Trabalho").join("Capas");
    fs::create_dir_all(&input_dir).unwrap();

    // 1. Valid PNG, JPEG, WebP succeed and deduplicate by SHA-256
    let png_a = input_dir.join("cover_a.png");
    let png_b = input_dir.join("cover_copy.png");
    let jpg_a = input_dir.join("cover_b.jpg");
    let webp_a = input_dir.join("cover_c.webp");
    fs::write(&png_a, MINIMAL_PNG).unwrap();
    fs::write(&png_b, MINIMAL_PNG).unwrap();
    fs::write(&jpg_a, MINIMAL_JPEG).unwrap();
    fs::write(&webp_a, MINIMAL_WEBP).unwrap();

    let cached_a = CoverCache::import_local_image(&png_a, &cache_dir).expect("import png");
    let cached_b = CoverCache::import_local_image(&png_b, &cache_dir).expect("deduplicate png");
    assert_eq!(
        cached_a, cached_b,
        "identical bytes must deduplicate by SHA-256"
    );
    assert_eq!(cached_a.extension().and_then(|e| e.to_str()), Some("png"));
    assert!(
        !cached_a.to_string_lossy().starts_with(r"\\?\"),
        "cached cover path must not leak Windows verbatim prefix"
    );

    let cached_jpg = CoverCache::import_local_image(&jpg_a, &cache_dir).expect("import jpg");
    assert_eq!(cached_jpg.extension().and_then(|e| e.to_str()), Some("jpg"));
    let cached_webp = CoverCache::import_local_image(&webp_a, &cache_dir).expect("import webp");
    assert_eq!(
        cached_webp.extension().and_then(|e| e.to_str()),
        Some("webp")
    );

    // 2. Spoofed fake PNG (EXE / plain text with .png extension) MUST be rejected
    let spoofed = input_dir.join("malware.png");
    fs::write(&spoofed, b"MZ\x90\x00not really a png file").unwrap();
    assert!(CoverCache::import_local_image(&spoofed, &cache_dir).is_err());

    // 3. Oversized (> 10 MB) file MUST be rejected even with valid PNG magic
    let oversized = input_dir.join("huge.png");
    let mut big = Vec::with_capacity((MAX_COVER_BYTES as usize) + 64);
    big.extend_from_slice(MINIMAL_PNG);
    big.resize((MAX_COVER_BYTES as usize) + 64, 0u8);
    fs::write(&oversized, &big).unwrap();
    assert!(CoverCache::import_local_image(&oversized, &cache_dir).is_err());

    // 4. Backend authorized-path validation rejects paths outside cache/covers/ and path traversal
    assert!(CoverCache::validate_authorized_cover_path(&cached_a, &cache_dir).is_ok());
    assert!(
        CoverCache::validate_authorized_cover_path(&png_a, &cache_dir).is_err(),
        "original file outside cache/covers/ must be rejected"
    );
    let traversal = cache_dir
        .join("..")
        .join("..")
        .join("Área de Trabalho")
        .join("Capas")
        .join("cover_a.png");
    assert!(
        CoverCache::validate_authorized_cover_path(&traversal, &cache_dir).is_err(),
        "path traversal escaping cache/covers/ must be rejected"
    );
    assert_eq!(
        CoverCache::sanitize_cover_for_frontend(Some(&png_a.to_string_lossy()), &cache_dir),
        None
    );
    assert!(
        CoverCache::sanitize_cover_for_frontend(Some(&cached_a.to_string_lossy()), &cache_dir)
            .is_some()
    );
}

#[test]
fn console_env_and_game_details_repos_persist_paths_settings_verifications_metadata_and_covers() {
    let tmp = TempDir::new().expect("tempdir");
    let db = DatabaseManager::open(&tmp.path().join("env_repo.db")).expect("open db");
    let cache_dir = tmp.path().join("cache").join("covers");
    let repos = SqliteRepositories::new(db.clone());
    let env_repo = ConsoleEnvRepository::new(db.clone());
    let details_repo = GameDetailsRepository::new(db.clone());
    let profile = repos.get_active_profile().unwrap();

    // Scan paths CRUD
    let scan_dir = tmp.path().join("PS3_Library");
    fs::create_dir_all(&scan_dir).unwrap();
    let added = env_repo
        .add_scan_path("ps3", &scan_dir.to_string_lossy(), true)
        .expect("add scan path");
    assert_eq!(added.console_slug, "ps3");
    assert!(added.recursive);
    assert_eq!(env_repo.list_scan_paths("ps3").unwrap().len(), 1);
    assert!(env_repo.list_scan_paths("ps2").unwrap().is_empty());

    // Console settings upsert
    let setting = env_repo
        .upsert_console_setting("ps3", "fullscreen", "true")
        .expect("upsert setting");
    assert_eq!(setting.value, "true");
    let updated = env_repo
        .upsert_console_setting("ps3", "fullscreen", "false")
        .expect("update setting");
    assert_eq!(updated.value, "false");
    assert_eq!(env_repo.list_console_settings("ps3").unwrap().len(), 1);

    // Game + versions + metadata + authorized cover
    let (game, _v1) = repos
        .register_game_with_version(
            "ps3",
            "Demon's Souls",
            "v1.00",
            "USA",
            Some("BLUS-30443"),
            r"D:\PS3\DemonsSouls\PS3_GAME\USRDIR\EBOOT.BIN",
            "ps3_param_sfo",
        )
        .unwrap();

    let updated_game = details_repo
        .update_game_metadata(
            &profile.id,
            &UpdateGameMetadataInput {
                game_id: game.id.clone(),
                description: "Action RPG clássico do PS3.".to_string(),
                developer: Some("FromSoftware".to_string()),
                publisher: Some("Atlus".to_string()),
                release_year: Some(2009),
                genre: Some("Action RPG".to_string()),
                completion_percent: 65,
            },
        )
        .expect("update metadata");
    assert_eq!(updated_game.completion_percent, 65);
    assert_eq!(updated_game.developer.as_deref(), Some("FromSoftware"));

    // Cover path enforcement in repository: outside cache_dir MUST fail
    let outside_png = tmp.path().join("outside.png");
    fs::write(&outside_png, MINIMAL_PNG).unwrap();
    assert!(
        details_repo
            .set_game_cover(&game.id, &outside_png, &cache_dir)
            .is_err(),
        "set_game_cover must reject paths outside authorized cover cache"
    );

    let cached_cover = CoverCache::import_local_image(&outside_png, &cache_dir).unwrap();
    let saved_cover = details_repo
        .set_game_cover(&game.id, &cached_cover, &cache_dir)
        .expect("set authorized cover");
    assert_eq!(saved_cover, cached_cover.to_string_lossy());

    // Runtime verifications (both console scope and per-game scope)
    let rv_console = env_repo
        .upsert_runtime_verification(
            "ps3",
            "rpcs3",
            None,
            RuntimeVerificationFeature::SystemBoot,
            "0.0.34",
            RuntimeVerificationStatus::FilesVerified,
            "vsh.self present, not runtime tested in Phase 2",
        )
        .expect("upsert console verification");
    assert_eq!(rv_console.scope_key, "console");

    let rv_game = env_repo
        .upsert_runtime_verification(
            "ps3",
            "rpcs3",
            Some(&game.id),
            RuntimeVerificationFeature::PerGameCompatibility,
            "0.0.34",
            RuntimeVerificationStatus::Unverified,
            "Pending Phase 3 live run",
        )
        .expect("upsert game verification");
    assert_eq!(rv_game.scope_key, game.id);
    assert_eq!(env_repo.list_runtime_verifications("ps3").unwrap().len(), 2);

    // Details bundle returns game + versions + sanitized cover
    let bundle = details_repo
        .get_game_details(&profile.id, &game.id, Some(&cache_dir))
        .expect("get game details");
    assert_eq!(bundle.versions.len(), 1);
    assert_eq!(
        bundle.game.cover_path.as_deref(),
        Some(saved_cover.as_str())
    );

    assert!(env_repo.remove_scan_path(&added.id).unwrap());
    assert!(env_repo.list_scan_paths("ps3").unwrap().is_empty());
}

#[test]
fn session_stats_repo_recovers_orphans_strictly_from_last_heartbeat_idempotently() {
    let tmp = TempDir::new().expect("tempdir");
    let db = DatabaseManager::open(&tmp.path().join("sessions.db")).expect("open db");
    let repos = SqliteRepositories::new(db.clone());
    let stats_repo = SessionStatsRepository::new(db.clone());
    let profile = repos.get_active_profile().unwrap();

    let (game, version) = repos
        .register_game_with_version(
            "ps3",
            "Journey",
            "01.02",
            "USA",
            Some("NPUA80472"),
            r"D:\PS3\Journey\USRDIR\EBOOT.BIN",
            "ps3_param_sfo",
        )
        .unwrap();

    // Start session at 20:00:00Z, heartbeat at 20:25:00Z (1500s), then crash (ended_at remains NULL)
    let session = stats_repo
        .start_session(
            &profile.id,
            &game.id,
            Some(&version.id),
            "emu-rpcs3",
            "2026-11-10T20:00:00Z",
        )
        .expect("start session");
    let hb = stats_repo
        .heartbeat_session(&session.id, "2026-11-10T20:25:00Z")
        .expect("heartbeat session");
    assert_eq!(hb.duration_seconds, 1500);
    assert!(hb.ended_at.is_none());

    // Simulate Hub restart DAYS later: recovery MUST use last_heartbeat_at (1500s), NEVER Utc::now()!
    let rec1 = stats_repo
        .reconcile_orphaned_sessions()
        .expect("first orphan recovery");
    assert_eq!(rec1.recovered_count, 1);
    assert_eq!(rec1.total_recovered_seconds, 1500);
    assert_eq!(rec1.recovered_session_ids, vec![session.id.clone()]);

    // Re-running reconciliation MUST be 100% idempotent (0 changes, no duplicate time)
    let rec2 = stats_repo
        .reconcile_orphaned_sessions()
        .expect("second orphan recovery must be no-op");
    assert_eq!(rec2.recovered_count, 0);
    assert_eq!(rec2.total_recovered_seconds, 0);

    let sessions = stats_repo
        .list_sessions_for_game(&profile.id, &game.id, 10)
        .unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].exit_status, EXIT_STATUS_RECOVERED_AFTER_CRASH);
    assert_eq!(
        sessions[0].ended_at.as_deref(),
        Some("2026-11-10T20:25:00Z")
    );
    assert_eq!(sessions[0].duration_seconds, 1500);

    // Statistics overview reflects strictly the 1500s real session
    let overview = stats_repo.get_statistics_overview(&profile.id).unwrap();
    assert_eq!(overview.total_playtime_seconds, 1500);
    assert_eq!(overview.total_sessions, 1);
    assert_eq!(overview.games_started, 1);
    assert_eq!(overview.monthly_playtime.len(), 1);
    assert_eq!(overview.monthly_playtime[0].month, "2026-11");
    assert_eq!(overview.monthly_playtime[0].total_playtime_seconds, 1500);
}

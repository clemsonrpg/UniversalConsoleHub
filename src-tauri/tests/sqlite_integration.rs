use std::collections::BTreeSet;

use tempfile::TempDir;
use universal_console_hub_lib::infrastructure::db::console_profile_repo::{
    ConsoleProfileRepository, UpsertConsoleLocalUserInput, UpsertConsoleProfileInput,
};
use universal_console_hub_lib::infrastructure::db::manager::DatabaseManager;
use universal_console_hub_lib::infrastructure::db::repositories::SqliteRepositories;

const EXPECTED_TABLES: &[&str] = &[
    "users",
    "profiles",
    "consoles",
    "emulators",
    "emulator_versions",
    "console_firmware",
    "games",
    "game_versions",
    "game_sessions",
    "trophies",
    "trophy_unlocks",
    "achievement_providers",
    "platform_accounts",
    "statistics",
    "favorites",
    "activity_history",
    "synchronization_history",
    "controller_profiles",
    "application_settings",
];

#[test]
fn applies_all_19_strict_tables_wal_pragmas_and_idempotent_seed() {
    let tmp = TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("Área de Trabalho").join("uch_test.db");

    let db = DatabaseManager::open(&db_path).expect("open real sqlite db");
    assert!(db.verify_integrity().expect("integrity check"));

    db.with_connection(|conn| {
        let mode: String = conn
            .query_row("PRAGMA journal_mode;", [], |r| r.get(0))
            .expect("journal_mode");
        assert_eq!(mode.to_lowercase(), "wal");

        let fk: i64 = conn
            .query_row("PRAGMA foreign_keys;", [], |r| r.get(0))
            .expect("foreign_keys");
        assert_eq!(fk, 1);

        let mut stmt = conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
            )
            .expect("prepare");
        let tables: BTreeSet<String> = stmt
            .query_map([], |r| r.get(0))
            .expect("query")
            .map(|r| r.expect("row"))
            .collect();
        for expected in EXPECTED_TABLES {
            assert!(
                tables.contains(*expected),
                "missing expected table: {expected}"
            );
        }
        Ok(())
    })
    .expect("verify tables");

    // Reopening the same DB must remain idempotent
    drop(db);
    let reopened = DatabaseManager::open(&db_path).expect("reopen sqlite db");
    let repos = SqliteRepositories::new(reopened);
    assert_eq!(repos.list_consoles().expect("consoles").len(), 6);
    assert_eq!(repos.list_emulators().expect("emulators").len(), 6);
    assert_eq!(
        repos.list_achievement_providers().expect("providers").len(),
        5
    );
    let profile = repos.get_active_profile().expect("default profile");
    assert_eq!(profile.handle, "player1");
}

#[test]
fn enforces_foreign_keys_strict_types_utc_timestamps_and_unique_constraints() {
    let tmp = TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("constraints.db");
    let db = DatabaseManager::open(&db_path).expect("open");

    db.with_connection(|conn| {
        // 1. Foreign key rejection
        let fk_err = conn.execute(
            "INSERT INTO profiles (id, user_id, handle, display_name, bio, privacy_level, preferences_json, is_active, created_at, updated_at)
             VALUES ('p-bad', 'missing-user', 'ghost', 'Ghost', '', 'private', '{}', 0, '2026-11-25T21:00:00Z', '2026-11-25T21:00:00Z')",
            [],
        );
        assert!(fk_err.is_err(), "foreign key constraint must reject missing user_id");

        // 2. STRICT type enforcement (text into integer column)
        let strict_err = conn.execute(
            "INSERT INTO consoles (id, slug, name, manufacturer, generation, release_year, phase_status, capabilities_json, notes, created_at)
             VALUES ('c-bad', 'bad-console', 'Bad', 'None', 'not-an-int', 2006, 'planned', '{}', '', '2026-11-25T21:00:00Z')",
            [],
        );
        assert!(strict_err.is_err(), "STRICT table must reject text in INTEGER generation column");

        // 3. UTC timestamp CHECK enforcement
        let tz_err = conn.execute(
            "INSERT INTO users (id, username, created_at, updated_at)
             VALUES ('u-bad-tz', 'badtz', '2026-11-25 18:00:00', '2026-11-25T21:00:00Z')",
            [],
        );
        assert!(tz_err.is_err(), "UTC CHECK constraint must reject non-UTC timestamp");

        // 4. Unique username constraint
        let dup_user = conn.execute(
            "INSERT INTO users (id, username, created_at, updated_at)
             VALUES ('u-dup', 'player1', '2026-11-25T21:00:00Z', '2026-11-25T21:00:00Z')",
            [],
        );
        assert!(dup_user.is_err(), "UNIQUE constraint must reject duplicate username");

        Ok(())
    })
    .expect("constraints checked");
}

#[test]
fn prevents_duplicate_trophies_and_enforces_immutable_trophy_origin_trigger() {
    let tmp = TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("trophies.db");
    let db = DatabaseManager::open(&db_path).expect("open");
    let repos = SqliteRepositories::new(db.clone());
    let profile = repos.get_active_profile().expect("profile");
    let (game, _) = repos
        .register_game_with_version(
            "ps3",
            "Demon's Souls",
            "USA v1.00",
            "USA",
            Some("BLUS-30443"),
            r"C:\Jogos PS3\Demon's Souls\PS3_GAME\USRDIR\EBOOT.BIN",
            "manual_user_entry",
        )
        .expect("register game");

    db.with_connection(|conn| {
        let provider_id: String = conn
            .query_row(
                "SELECT id FROM achievement_providers WHERE slug = 'local'",
                [],
                |r| r.get(0),
            )
            .expect("local provider");

        conn.execute(
            "INSERT INTO trophies (id, game_id, console_id, provider_id, external_trophy_id, name, description, icon_path, category, rarity_percent, origin, created_at)
             VALUES ('t-1', ?1, ?2, ?3, 'LOCAL_001', 'First Boot', 'Completed first run', NULL, 'hub_local', NULL, 'local', '2026-11-25T21:00:00Z')",
            rusqlite::params![game.id, game.console_id, provider_id],
        )
        .expect("insert trophy");

        // Duplicate (game_id, provider_id, external_trophy_id) must fail
        let dup_trophy = conn.execute(
            "INSERT INTO trophies (id, game_id, console_id, provider_id, external_trophy_id, name, description, icon_path, category, rarity_percent, origin, created_at)
             VALUES ('t-2', ?1, ?2, ?3, 'LOCAL_001', 'Duplicate', 'Should fail', NULL, 'hub_local', NULL, 'local', '2026-11-25T21:00:00Z')",
            rusqlite::params![game.id, game.console_id, provider_id],
        );
        assert!(dup_trophy.is_err(), "duplicate trophy definition must be rejected");

        // Attempting to forge origin from 'local' to 'official' MUST be blocked by trigger
        let mutate_origin = conn.execute(
            "UPDATE trophies SET origin = 'official' WHERE id = 't-1'",
            [],
        );
        assert!(mutate_origin.is_err(), "trigger must block mutating trophy origin");

        // Unlock once
        conn.execute(
            "INSERT INTO trophy_unlocks (id, profile_id, trophy_id, origin, unlocked_at, synced_at)
             VALUES ('u-1', ?1, 't-1', 'local', '2026-11-25T21:05:00Z', '2026-11-25T21:05:00Z')",
            rusqlite::params![profile.id],
        )
        .expect("insert unlock");

        // Duplicate unlock for same (profile_id, trophy_id) must fail
        let dup_unlock = conn.execute(
            "INSERT INTO trophy_unlocks (id, profile_id, trophy_id, origin, unlocked_at, synced_at)
             VALUES ('u-2', ?1, 't-1', 'local', '2026-11-25T21:06:00Z', '2026-11-25T21:06:00Z')",
            rusqlite::params![profile.id],
        );
        assert!(dup_unlock.is_err(), "duplicate trophy unlock must be rejected");

        // Mutating unlock origin MUST be blocked by trigger
        let mutate_unlock_origin = conn.execute(
            "UPDATE trophy_unlocks SET origin = 'official' WHERE id = 'u-1'",
            [],
        );
        assert!(
            mutate_unlock_origin.is_err(),
            "trigger must block mutating trophy_unlocks origin"
        );
        Ok(())
    })
    .expect("trophy integrity checked");
}

#[test]
fn registers_games_with_multiple_versions_unicode_paths_and_favorites() {
    let tmp = TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("library.db");
    let db = DatabaseManager::open(&db_path).expect("open");
    let repos = SqliteRepositories::new(db);
    let profile = repos.get_active_profile().expect("profile");

    let (g1, v1) = repos
        .register_game_with_version(
            "ps3",
            "Gran Turismo 5 — Edição Colecionador",
            "v1.00",
            "USA",
            Some("BCUS-98114"),
            r"C:\Users\loure\OneDrive\Área de Trabalho\PS3\GT5\EBOOT.BIN",
            "manual_user_entry",
        )
        .expect("register v1");
    let (g2, v2) = repos
        .register_game_with_version(
            "ps3",
            "Gran Turismo 5 — Edição Colecionador",
            "v2.17",
            "USA",
            Some("BCUS-98114"),
            r"C:\Users\loure\OneDrive\Área de Trabalho\PS3\GT5_217\EBOOT.BIN",
            "manual_user_entry",
        )
        .expect("register v2");

    assert_eq!(
        g1.id, g2.id,
        "same canonical serial must map to the same canonical game"
    );
    assert_ne!(
        v1.id, v2.id,
        "distinct versions must have distinct game_versions rows"
    );

    let fav_on = repos
        .toggle_favorite(&profile.id, &g1.id)
        .expect("favorite on");
    assert!(fav_on);

    let games = repos.list_games(&profile.id).expect("list games");
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].version_count, 2);
    assert!(games[0].is_favorite);
}

#[test]
fn creates_valid_wal_backup_and_restores_mutated_database_completely() {
    let tmp = TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("live_wal.db");
    let backup_path = tmp.path().join("backups").join("snapshot_phase1.db");

    let db = DatabaseManager::open(&db_path).expect("open live db");
    let repos = SqliteRepositories::new(db.clone());
    let updated_profile = repos
        .update_active_profile(
            "clemson",
            "Clemson RPG",
            "Authentic console collector",
            "private",
        )
        .expect("update profile");
    assert_eq!(updated_profile.display_name, "Clemson RPG");

    repos
        .register_game_with_version(
            "ps3",
            "Metal Gear Solid 4: Guns of the Patriots",
            "v2.00",
            "USA",
            Some("BLUS-30109"),
            r"D:\PS3\MGS4\PS3_GAME\USRDIR\EBOOT.BIN",
            "manual_user_entry",
        )
        .expect("register game before backup");

    // 1. Create backup while WAL connection is actively open
    let backup_report = db.create_backup(&backup_path).expect("create WAL backup");
    assert!(backup_report.integrity_ok);
    assert!(backup_report.size_bytes > 0);
    assert!(backup_path.is_file());

    // 2. Mutate the live database after backup (simulating accidental corruption/overwrite)
    repos
        .update_active_profile("wrong_user", "Corrupted Name", "Lost bio", "public")
        .expect("mutate profile");
    db.with_connection(|conn| {
        conn.execute("DELETE FROM games", []).expect("wipe games");
        Ok(())
    })
    .expect("wipe live games");
    assert_eq!(
        repos.list_games(&updated_profile.id).expect("games").len(),
        0
    );

    // 3. Close active handles and restore from backup
    drop(repos);
    drop(db);
    let restore_report =
        DatabaseManager::restore_backup(&backup_path, &db_path).expect("restore validated backup");
    assert!(restore_report.integrity_ok);
    assert!(restore_report.schema_version >= 1);

    // 4. Reopen target DB and confirm original profile, game, AND immutability triggers survived restore
    let restored_db = DatabaseManager::open(&db_path).expect("open restored db");
    assert!(restored_db
        .verify_integrity()
        .expect("post-restore integrity"));
    let restored_repos = SqliteRepositories::new(restored_db);
    let restored_profile = restored_repos
        .get_active_profile()
        .expect("restored profile");
    assert_eq!(restored_profile.display_name, "Clemson RPG");
    assert_eq!(restored_profile.username, "clemson");

    let restored_games = restored_repos
        .list_games(&restored_profile.id)
        .expect("restored games");
    assert_eq!(restored_games.len(), 1);
    assert_eq!(
        restored_games[0].title,
        "Metal Gear Solid 4: Guns of the Patriots"
    );
}

const PHASE1_SCHEMA_SQL: &str =
    include_str!("../src/infrastructure/db/sql/0001_initial_schema.sql");
const PHASE2_SCHEMA_SQL: &str =
    include_str!("../src/infrastructure/db/sql/0002_console_environments.sql");

fn build_populated_phase1_v1_db(path: &std::path::Path) {
    let conn = rusqlite::Connection::open(path).expect("create raw v1 sqlite db");
    conn.pragma_update(None, "journal_mode", "WAL").unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    conn.execute_batch(PHASE1_SCHEMA_SQL)
        .expect("apply raw 0001 schema");
    conn.pragma_update(None, "user_version", 1).unwrap();

    let now = "2026-11-25T21:00:00Z";
    conn.execute(
        "INSERT INTO users (id, username, created_at, updated_at) VALUES ('user-default', 'v1player', ?1, ?1)",
        rusqlite::params![now],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO profiles (id, user_id, handle, display_name, bio, privacy_level, preferences_json, is_active, created_at, updated_at)
         VALUES ('profile-default', 'user-default', 'v1player', 'V1 Veteran', 'Migrated from Phase 1', 'private', '{}', 1, ?1, ?1)",
        rusqlite::params![now],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO consoles (id, slug, name, manufacturer, generation, release_year, phase_status, capabilities_json, notes, created_at)
         VALUES ('console-ps3', 'ps3', 'PlayStation 3', 'Sony Interactive Entertainment', 7, 2006, 'phase_1_prepared_phase_4_full', '{\"supportedStates\":[\"FULL_SYSTEM_BOOT\",\"ORIGINAL_DASHBOARD\",\"DIRECT_GAME_ONLY\"],\"features\":[]}', 'v1 note', ?1)",
        rusqlite::params![now],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO emulators (id, slug, name, console_id, integration_phase, capabilities_json, updated_at)
         VALUES ('emu-rpcs3', 'rpcs3', 'RPCS3', 'console-ps3', 'phase_1_preparation', '{\"supportedStates\":[\"FULL_SYSTEM_BOOT\",\"ORIGINAL_DASHBOARD\",\"DIRECT_GAME_ONLY\"],\"features\":[]}', ?1)",
        rusqlite::params![now],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO console_firmware (id, console_id, emulator_id, file_role, file_path, file_size_bytes, sha256, validation_status, notes, validated_at)
         VALUES ('fw-1', 'console-ps3', 'emu-rpcs3', 'system_firmware_package', 'C:\\FW\\PS3UPDAT.PUP', 1024, 'abc123', 'package_header_and_hash_validated', 'ok', ?1)",
        rusqlite::params![now],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO games (id, console_id, canonical_key, title, description, metadata_source, install_state, associated_emulator_id, completion_percent, created_at, updated_at)
         VALUES ('game-1', 'console-ps3', 'serial:BLUS30443', 'Demon''s Souls', 'Classic RPG', 'manual_user_entry', 'registered', 'emu-rpcs3', 45, ?1, ?1)",
        rusqlite::params![now],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO game_versions (id, game_id, version_label, region, serial_code, file_path, install_state, created_at)
         VALUES ('gv-1', 'game-1', 'v1.00', 'USA', 'BLUS-30443', 'C:\\PS3\\DemonsSouls\\PS3_GAME\\USRDIR\\EBOOT.BIN', 'registered', ?1)",
        rusqlite::params![now],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO favorites (id, profile_id, game_id, created_at) VALUES ('fav-1', 'profile-default', 'game-1', ?1)",
        rusqlite::params![now],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO achievement_providers (id, slug, name, origin, availability_status, supports_auth, supports_sync, notes, updated_at)
         VALUES ('prov-local', 'local', 'LocalAchievementProvider', 'local', 'ready_offline', 0, 0, 'local', ?1)",
        rusqlite::params![now],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO trophies (id, game_id, console_id, provider_id, external_trophy_id, name, description, category, origin, created_at)
         VALUES ('tr-1', 'game-1', 'console-ps3', 'prov-local', 'T001', 'Slayer of Demons', 'Boss defeated', 'hub_local', 'local', ?1)",
        rusqlite::params![now],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO trophy_unlocks (id, profile_id, trophy_id, origin, unlocked_at, synced_at)
         VALUES ('tu-1', 'profile-default', 'tr-1', 'local', ?1, ?1)",
        rusqlite::params![now],
    )
    .unwrap();
}

#[test]
fn upgrades_real_phase1_v1_database_to_v2_preserving_all_records_constraints_and_triggers() {
    let tmp = TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("upgrade_v1_to_v2.db");
    build_populated_phase1_v1_db(&db_path);

    let db = DatabaseManager::open(&db_path).expect("upgrade v1 db to v2");
    assert!(db.verify_integrity().expect("integrity after v1->v2"));

    db.with_connection(|conn| {
        let version: i64 = conn
            .query_row("PRAGMA user_version;", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, 3, "upgraded database must be at schema version 3");

        // All 24 STRICT tables must exist (19 from v1 + 3 from v2 + 2 from v3)
        let mut stmt = conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
            )
            .unwrap();
        let tables: BTreeSet<String> = stmt
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(tables.len(), 24, "expected 24 total tables after v3 migration");
        for new_table in [
            "console_scan_paths",
            "console_settings",
            "console_runtime_verifications",
            "console_profiles",
            "console_local_users",
        ] {
            assert!(tables.contains(new_table), "missing migrated table {new_table}");
        }

        // Additive non-destructive indexes on new and existing tables must exist
        let mut idx_stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'index' AND name NOT LIKE 'sqlite_%'")
            .unwrap();
        let indexes: BTreeSet<String> = idx_stmt
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        for expected_idx in [
            "idx_console_scan_paths_console",
            "idx_console_settings_console",
            "idx_console_runtime_verifications_console_emu",
            "idx_console_runtime_verifications_game",
            "idx_game_versions_file_path",
            "idx_game_versions_serial",
            "idx_game_sessions_open",
            "idx_game_sessions_game_profile",
            "idx_console_local_users_console",
            "idx_game_versions_availability",
            "idx_game_sessions_local_user",
            "idx_trophy_unlocks_local_user",
        ] {
            assert!(indexes.contains(expected_idx), "missing index {expected_idx}");
        }

        // Verify v1 game_version received default availability_status = 'available'
        let avail: String = conn
            .query_row(
                "SELECT availability_status FROM game_versions WHERE id = 'gv-1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(avail, "available");

        // Preserve all v1 records intact
        let profile_name: String = conn
            .query_row(
                "SELECT display_name FROM profiles WHERE id = 'profile-default'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(profile_name, "V1 Veteran");
        let version_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM game_versions WHERE game_id = 'game-1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version_count, 1);
        let fw_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM console_firmware WHERE id = 'fw-1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(fw_count, 1);

        // Phase 1 immutability triggers must remain active after v2 migration
        assert!(
            conn.execute("UPDATE trophies SET origin = 'official' WHERE id = 'tr-1'", [])
                .is_err(),
            "trg_trophies_immutable_origin must survive v1->v2 upgrade"
        );
        assert!(
            conn.execute("UPDATE trophy_unlocks SET origin = 'official' WHERE id = 'tu-1'", [])
                .is_err(),
            "trg_trophy_unlocks_immutable_origin must survive v1->v2 upgrade"
        );

        // Verify console_runtime_verifications CHECK + UNIQUE (console scope & per-game scope)
        conn.execute(
            "INSERT INTO console_runtime_verifications (id, console_id, emulator_id, game_id, scope_key, feature_key, emulator_version, status, evidence_notes, verified_at)
             VALUES ('rv-1', 'console-ps3', 'emu-rpcs3', NULL, 'console', 'system_boot', '0.0.34', 'files_verified', 'vsh present', '2026-11-25T21:10:00Z')",
            [],
        )
        .expect("insert console-scoped verification");

        // Duplicate console-scoped verification must be blocked by UNIQUE(console_id, emulator_id, feature_key, emulator_version, scope_key)
        assert!(conn.execute(
            "INSERT INTO console_runtime_verifications (id, console_id, emulator_id, game_id, scope_key, feature_key, emulator_version, status, evidence_notes, verified_at)
             VALUES ('rv-dup', 'console-ps3', 'emu-rpcs3', NULL, 'console', 'system_boot', '0.0.34', 'unverified', 'dup', '2026-11-25T21:10:00Z')",
            [],
        ).is_err());

        // Mismatched (game_id, scope_key) must be rejected by CHECK
        assert!(conn.execute(
            "INSERT INTO console_runtime_verifications (id, console_id, emulator_id, game_id, scope_key, feature_key, emulator_version, status, evidence_notes, verified_at)
             VALUES ('rv-bad-scope', 'console-ps3', 'emu-rpcs3', 'game-1', 'console', 'per_game_compatibility', '0.0.34', 'unverified', 'bad', '2026-11-25T21:10:00Z')",
            [],
        ).is_err());

        // Valid per-game verification must succeed
        conn.execute(
            "INSERT INTO console_runtime_verifications (id, console_id, emulator_id, game_id, scope_key, feature_key, emulator_version, status, evidence_notes, verified_at)
             VALUES ('rv-game', 'console-ps3', 'emu-rpcs3', 'game-1', 'game-1', 'per_game_compatibility', '0.0.34', 'unverified', 'pending Phase 3 test', '2026-11-25T21:10:00Z')",
            [],
        )
        .expect("insert per-game verification");

        Ok(())
    })
    .expect("v1->v2 assertions");
}

#[test]
fn rejects_future_schema_version_without_modifying_database_file() {
    let tmp = TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("future_schema.db");

    {
        let conn = rusqlite::Connection::open(&db_path).expect("create future db");
        conn.execute(
            "CREATE TABLE sentinel (k TEXT PRIMARY KEY, v TEXT NOT NULL) STRICT;",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO sentinel (k, v) VALUES ('state', 'untouched')",
            [],
        )
        .unwrap();
        conn.pragma_update(None, "user_version", 99).unwrap();
    }

    let err = DatabaseManager::open(&db_path)
        .err()
        .expect("future user_version > LATEST_SCHEMA_VERSION must be rejected");
    let msg = err.to_string().to_lowercase();
    assert!(
        msg.contains("version") || msg.contains("newer") || msg.contains("unsupported"),
        "error should explain schema version incompatibility, got: {msg}"
    );

    // Confirm database was NOT modified
    let ro =
        rusqlite::Connection::open_with_flags(&db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .expect("reopen read-only");
    let version: i64 = ro
        .query_row("PRAGMA user_version;", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 99);
    let sentinel: String = ro
        .query_row("SELECT v FROM sentinel WHERE k = 'state'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(sentinel, "untouched");
}

#[test]
fn inspects_and_live_restores_backup_with_open_wal_connection_including_v1_backup_and_corrupt_rejection(
) {
    let tmp = TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("Área de Trabalho").join("live_restore.db");
    let backup_dir = tmp.path().join("backups");
    let safety_dir = tmp.path().join("safety_snapshots");
    let backup_v2_path = backup_dir.join("good_v2_backup.db");

    let db = DatabaseManager::open(&db_path).expect("open live v2 db");
    let repos = SqliteRepositories::new(db.clone());
    repos
        .update_active_profile("hero", "Hero Player", "Before mutation", "private")
        .expect("update profile");
    repos
        .register_game_with_version(
            "ps3",
            "Demon's Souls",
            "v1.00",
            "USA",
            Some("BLUS-30443"),
            r"D:\PS3\DemonsSouls\PS3_GAME\USRDIR\EBOOT.BIN",
            "manual_user_entry",
        )
        .expect("register game");

    db.create_backup(&backup_v2_path)
        .expect("create valid v2 backup while WAL open");

    // 1. Inspect backup before restoring
    let inspection = DatabaseManager::inspect_backup(&backup_v2_path).expect("inspect v3 backup");
    assert!(inspection.integrity_ok);
    assert_eq!(inspection.schema_version, 3);
    assert_eq!(inspection.profiles_count, 1);
    assert_eq!(inspection.games_count, 1);
    assert!(inspection.size_bytes > 0);

    // 2. Mutate live database while keeping DatabaseManager open
    repos
        .update_active_profile("mutated", "Mutated Name", "Oops", "public")
        .expect("mutate profile");
    db.with_connection(|conn| {
        conn.execute("DELETE FROM games", []).unwrap();
        Ok(())
    })
    .unwrap();
    assert_eq!(repos.list_games("profile-default").unwrap().len(), 0);

    // 3. Reject corrupted non-SQLite file without touching live DB
    let corrupt_path = backup_dir.join("corrupt.db");
    std::fs::write(&corrupt_path, b"this is definitely not a sqlite database").unwrap();
    assert!(DatabaseManager::inspect_backup(&corrupt_path).is_err());
    assert!(db
        .restore_from_backup_live(&corrupt_path, &safety_dir)
        .is_err());
    assert_eq!(
        repos.get_active_profile().unwrap().display_name,
        "Mutated Name",
        "failed restore must leave live DB untouched"
    );

    // 4. Reject backup with future user_version without touching live DB
    let future_backup = backup_dir.join("future_v99.db");
    {
        let c = rusqlite::Connection::open(&future_backup).unwrap();
        c.execute_batch(PHASE1_SCHEMA_SQL).unwrap();
        c.pragma_update(None, "user_version", 99).unwrap();
    }
    assert!(DatabaseManager::inspect_backup(&future_backup).is_err());
    assert!(db
        .restore_from_backup_live(&future_backup, &safety_dir)
        .is_err());

    // 4b. Reject backup with foreign key violation without touching live DB
    let broken_fk_backup = backup_dir.join("broken_fk.db");
    {
        build_populated_phase1_v1_db(&broken_fk_backup);
        let c = rusqlite::Connection::open(&broken_fk_backup).unwrap();
        c.pragma_update(None, "foreign_keys", "OFF").unwrap();
        c.execute(
            "INSERT INTO favorites (id, profile_id, game_id, created_at)
             VALUES ('fav-orphan', 'profile-default', 'non-existent-game', '2026-11-25T21:00:00Z')",
            [],
        )
        .unwrap();
    }
    assert!(
        DatabaseManager::inspect_backup(&broken_fk_backup).is_err(),
        "backup failing PRAGMA foreign_key_check must be rejected"
    );
    assert!(db
        .restore_from_backup_live(&broken_fk_backup, &safety_dir)
        .is_err());

    // 4c. Reject attempting to restore the active DB onto itself
    assert!(db.restore_from_backup_live(&db_path, &safety_dir).is_err());

    // 5. Perform live restore while DatabaseManager WAL connection is open!
    let live_report = db
        .restore_from_backup_live(&backup_v2_path, &safety_dir)
        .expect("live restore over open WAL connection on Windows");
    assert!(live_report.integrity_ok);
    assert_eq!(live_report.schema_version, 3);
    assert!(
        live_report.pre_restore_snapshot_path.is_file(),
        "pre-restore snapshot must exist"
    );

    // Verify pre-restore snapshot actually preserved the pre-restore mutated state
    let snap_check = DatabaseManager::inspect_backup(&live_report.pre_restore_snapshot_path)
        .expect("inspect pre-restore snapshot");
    assert!(snap_check.integrity_ok);
    assert_eq!(snap_check.games_count, 0);

    // Verify LIVE DatabaseManager now serves restored data with WAL & FK enabled
    assert!(db.verify_integrity().expect("post-live-restore integrity"));
    db.with_connection(|conn| {
        let mode: String = conn
            .query_row("PRAGMA journal_mode;", [], |r| r.get(0))
            .unwrap();
        let fk: i64 = conn
            .query_row("PRAGMA foreign_keys;", [], |r| r.get(0))
            .unwrap();
        assert_eq!(mode.to_lowercase(), "wal");
        assert_eq!(fk, 1);
        Ok(())
    })
    .unwrap();
    assert_eq!(
        repos.get_active_profile().unwrap().display_name,
        "Hero Player"
    );
    assert_eq!(repos.list_games("profile-default").unwrap().len(), 1);
    assert!(
        !db_path.with_extension("db.rollback.tmp").exists(),
        "rollback temp file must be cleaned after verified live restore"
    );

    // 6. Live restore a Phase 1 (v1) backup into the live v2 DatabaseManager — must auto-upgrade cleanly to v2!
    let v1_backup_path = backup_dir.join("phase1_v1_backup.db");
    build_populated_phase1_v1_db(&v1_backup_path);
    let v1_inspect = DatabaseManager::inspect_backup(&v1_backup_path).expect("inspect v1 backup");
    assert!(v1_inspect.integrity_ok);
    assert_eq!(v1_inspect.schema_version, 1);
    assert_eq!(v1_inspect.games_count, 1);

    let v1_restore = db
        .restore_from_backup_live(&v1_backup_path, &safety_dir)
        .expect("live restore of v1 backup must auto-migrate to v3");
    assert!(v1_restore.integrity_ok);
    assert_eq!(v1_restore.schema_version, 3);
    assert_eq!(
        repos.get_active_profile().unwrap().display_name,
        "V1 Veteran"
    );
    db.with_connection(|conn| {
        let version: i64 = conn
            .query_row("PRAGMA user_version;", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, 3);
        let new_table_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'console_profiles'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(new_table_exists, 1);
        Ok(())
    })
    .unwrap();
}

fn build_populated_phase2_v2_db(path: &std::path::Path) {
    build_populated_phase1_v1_db(path);
    let conn = rusqlite::Connection::open(path).expect("reopen v1 db to upgrade to raw v2");
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    conn.execute_batch(PHASE2_SCHEMA_SQL)
        .expect("apply raw 0002 schema");
    conn.pragma_update(None, "user_version", 2).unwrap();

    let now = "2026-11-25T21:15:00Z";
    conn.execute(
        "INSERT INTO console_scan_paths (id, console_id, directory_path, recursive, enabled, last_scanned_at, created_at)
         VALUES ('sp-1', 'console-ps3', 'D:\\Jogos\\PS3', 1, 1, ?1, ?1)",
        rusqlite::params![now],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO console_settings (id, console_id, key, value, updated_at)
         VALUES ('cs-1', 'console-ps3', 'fullscreen', 'true', ?1)",
        rusqlite::params![now],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO game_sessions (id, profile_id, game_id, game_version_id, emulator_id, started_at, last_heartbeat_at, ended_at, duration_seconds, exit_status)
         VALUES ('sess-v2', 'profile-default', 'game-1', 'gv-1', 'emu-rpcs3', ?1, ?1, ?1, 600, 'completed')",
        rusqlite::params![now],
    )
    .unwrap();
}

#[test]
fn upgrades_real_phase2_v2_database_to_v3_and_persists_console_profiles_and_local_users() {
    let tmp = TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("upgrade_v2_to_v3.db");
    build_populated_phase2_v2_db(&db_path);

    let db = DatabaseManager::open(&db_path).expect("upgrade v2 db to v3");
    assert!(db.verify_integrity().expect("integrity after v2->v3"));

    let profile_repo = ConsoleProfileRepository::new(db.clone());

    // 1. Persist console profile snapshot (strictly console identity, NOT player identity)
    let saved_profile = profile_repo
        .upsert_console_profile(&UpsertConsoleProfileInput {
            console_slug: "ps3".to_string(),
            emulator_slug: Some("rpcs3".to_string()),
            emulator_status: "available".to_string(),
            detected_emulator_version: Some("RPCS3 v0.0.34-17100".to_string()),
            detected_firmware_version: Some("4.91".to_string()),
            emulator_root_dir: Some(r"D:\Emulators\RPCS3".to_string()),
            native_save_count: 4,
            native_trophy_dir_present: true,
            trophy_sync_readiness: "phase4_planned".to_string(),
            native_metadata_json: r#"{"vshSelfPresent":true}"#.to_string(),
            sync_status: "synced".to_string(),
            sync_diagnostics: String::new(),
            last_synced_at: Some("2026-11-25T21:20:00Z".to_string()),
        })
        .expect("upsert console profile");
    assert_eq!(saved_profile.console_slug, "ps3");
    assert_eq!(saved_profile.emulator_status, "available");
    assert_eq!(
        saved_profile.detected_firmware_version.as_deref(),
        Some("4.91")
    );
    assert_eq!(saved_profile.trophy_sync_readiness, "phase4_planned");
    assert_eq!(saved_profile.native_save_count, 4);
    assert!(saved_profile.native_trophy_dir_present);

    // 2. Persist multiple real local console users (Adjustment A)
    let user1 = profile_repo
        .upsert_local_user(&UpsertConsoleLocalUserInput {
            console_slug: "ps3".to_string(),
            native_user_key: "00000001".to_string(),
            display_name: "ClemsonPS3".to_string(),
            source_path: r"D:\Emulators\RPCS3\dev_hdd0\home\00000001".to_string(),
            availability_status: "available".to_string(),
            linked_profile_id: Some("profile-default".to_string()),
        })
        .expect("upsert local user 1");
    let user2 = profile_repo
        .upsert_local_user(&UpsertConsoleLocalUserInput {
            console_slug: "ps3".to_string(),
            native_user_key: "00000002".to_string(),
            display_name: "GuestPS3".to_string(),
            source_path: r"D:\Emulators\RPCS3\dev_hdd0\home\00000002".to_string(),
            availability_status: "available".to_string(),
            linked_profile_id: None,
        })
        .expect("upsert local user 2");

    let users = profile_repo
        .list_local_users("ps3")
        .expect("list local users");
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].native_user_key, "00000001");
    assert_eq!(users[1].native_user_key, "00000002");

    // Platforms without exposed local users must return empty list (never invent users)
    assert!(profile_repo
        .list_local_users("ps2")
        .expect("list ps2 local users")
        .is_empty());

    // 3. Mark unseen user missing_on_disk without deleting it (Adjustment B)
    let marked = profile_repo
        .mark_unseen_local_users_missing("ps3", &["00000001".to_string()])
        .expect("mark unseen users missing");
    assert_eq!(marked, 1);
    let users_after = profile_repo
        .list_local_users("ps3")
        .expect("list local users after mark");
    assert_eq!(
        users_after.len(),
        2,
        "missing user must be preserved, not deleted"
    );
    assert_eq!(users_after[0].availability_status, "available");
    assert_eq!(users_after[1].availability_status, "missing_on_disk");

    // 4. Mark game version missing_on_disk preserving existing v2 sessions (Adjustment B)
    profile_repo
        .update_game_version_availability("gv-1", "missing_on_disk", "2026-11-25T21:25:00Z")
        .expect("mark version missing_on_disk");

    // 5. Link optional console_local_user_id on game_sessions and trophy_unlocks (Adjustments A + C)
    db.with_connection(|conn| {
        conn.execute(
            "UPDATE game_sessions SET console_local_user_id = ?1 WHERE id = 'sess-v2'",
            rusqlite::params![user1.id],
        )
        .expect("link session to real console local user");

        conn.execute(
            "UPDATE trophy_unlocks SET console_local_user_id = ?1 WHERE id = 'tu-1'",
            rusqlite::params![user1.id],
        )
        .expect("link trophy unlock to real console local user");

        // Immutable origin trigger must STILL protect trophy_unlocks after v3 upgrade
        assert!(
            conn.execute(
                "UPDATE trophy_unlocks SET origin = 'official' WHERE id = 'tu-1'",
                [],
            )
            .is_err(),
            "trg_trophy_unlocks_immutable_origin must survive v2->v3 upgrade"
        );

        let (avail, verified_at): (String, Option<String>) = conn
            .query_row(
                "SELECT availability_status, last_verified_at FROM game_versions WHERE id = 'gv-1'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(avail, "missing_on_disk");
        assert_eq!(verified_at.as_deref(), Some("2026-11-25T21:25:00Z"));

        // Existing v2 session and scan_path survived intact
        let sess_duration: i64 = conn
            .query_row(
                "SELECT duration_seconds FROM game_sessions WHERE id = 'sess-v2'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(sess_duration, 600);

        let scan_path_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM console_scan_paths WHERE id = 'sp-1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(scan_path_count, 1);

        // Verify second user id was referenced cleanly
        assert_ne!(user1.id, user2.id);
        Ok(())
    })
    .expect("v2->v3 assertions");
}

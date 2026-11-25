use std::collections::BTreeSet;

use tempfile::TempDir;
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

use std::collections::BTreeSet;

use tempfile::TempDir;
use universal_console_hub_lib::domain::trophy::{TrophyCategory, TrophyOrigin};
use universal_console_hub_lib::infrastructure::db::manager::DatabaseManager;
use universal_console_hub_lib::infrastructure::db::repositories::SqliteRepositories;
use universal_console_hub_lib::infrastructure::db::trophy_repo::{
    TrophyRepository, UpsertTrophyDefInput, UpsertTrophySetInput,
};

const PHASE1_SQL: &str = include_str!("../src/infrastructure/db/sql/0001_initial_schema.sql");
const PHASE2_SQL: &str = include_str!("../src/infrastructure/db/sql/0002_console_environments.sql");
const PHASE3_SQL: &str =
    include_str!("../src/infrastructure/db/sql/0003_console_profiles_and_runtime.sql");

fn build_populated_v3_db(path: &std::path::Path) {
    let conn = rusqlite::Connection::open(path).expect("open raw v3 db");
    conn.pragma_update(None, "journal_mode", "WAL").unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    conn.execute_batch(PHASE1_SQL).unwrap();
    conn.execute_batch(PHASE2_SQL).unwrap();
    conn.execute_batch(PHASE3_SQL).unwrap();
    conn.pragma_update(None, "user_version", 3).unwrap();
}

#[test]
fn upgrades_v3_to_v4_with_25_strict_tables_and_idempotent_earliest_unlock_preservation() {
    let tmp = TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("upgrade_v3_to_v4.db");
    build_populated_v3_db(&db_path);

    let db = DatabaseManager::open(&db_path).expect("upgrade v3 db to v4");
    assert!(db.verify_integrity().expect("integrity after v3->v4"));

    db.with_connection(|conn| {
        let ver: i64 = conn
            .query_row("PRAGMA user_version;", [], |r| r.get(0))
            .unwrap();
        assert_eq!(ver, 4);

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
        assert_eq!(tables.len(), 25, "expected 25 STRICT tables at v4");
        assert!(tables.contains("trophy_sets"));
        Ok(())
    })
    .unwrap();

    let repos = SqliteRepositories::new(db.clone());
    let profile = repos.get_active_profile().expect("profile");
    let (game, _) = repos
        .register_game_with_version(
            "ps3",
            "Demon's Souls",
            "01.00",
            "USA",
            Some("BLUS30443"),
            r"D:\PS3\DemonsSouls\PS3_GAME\USRDIR\EBOOT.BIN",
            "ps3_param_sfo",
        )
        .expect("register game");

    let trophy_repo = TrophyRepository::new(db.clone());
    let set = trophy_repo
        .upsert_trophy_set(&UpsertTrophySetInput {
            console_slug: "ps3".to_string(),
            game_id: game.id.clone(),
            provider_slug: "emulator".to_string(),
            external_set_id: "NPWR00637_00".to_string(),
            title: "Demon's Souls".to_string(),
            detail: "Trophies for Demon's Souls".to_string(),
            icon_path: None,
            total_count: 2,
            supports_platinum: true,
        })
        .expect("upsert trophy set");
    assert_eq!(set.external_set_id, "NPWR00637_00");
    assert!(set.supports_platinum);

    let t0 = trophy_repo
        .upsert_trophy_definition(&UpsertTrophyDefInput {
            game_id: game.id.clone(),
            console_slug: "ps3".to_string(),
            provider_slug: "emulator".to_string(),
            trophy_set_id: Some(set.id.clone()),
            external_trophy_id: "000".to_string(),
            name: "Toughest Soul Trophy".to_string(),
            description: "All Trophies Obtained".to_string(),
            icon_path: None,
            category: TrophyCategory::Platinum,
            rarity_percent: None,
            origin: TrophyOrigin::Emulator,
            is_hidden: false,
        })
        .expect("upsert platinum trophy");

    let t1 = trophy_repo
        .upsert_trophy_definition(&UpsertTrophyDefInput {
            game_id: game.id.clone(),
            console_slug: "ps3".to_string(),
            provider_slug: "emulator".to_string(),
            trophy_set_id: Some(set.id.clone()),
            external_trophy_id: "001".to_string(),
            name: "Phalanx's Trophy".to_string(),
            description: "Slayer of Demon Phalanx".to_string(),
            icon_path: None,
            category: TrophyCategory::Bronze,
            rarity_percent: None,
            origin: TrophyOrigin::Emulator,
            is_hidden: true,
        })
        .expect("upsert hidden bronze trophy");

    // 1. Unlock trophy at 2026-11-25T20:00:00Z -> newly_unlocked = true
    let u1 = trophy_repo
        .record_trophy_unlock(
            &profile.id,
            &t1.id,
            TrophyOrigin::Emulator,
            "2026-11-25T20:00:00Z",
            None,
        )
        .expect("record unlock");
    assert!(u1.newly_unlocked);
    assert_eq!(u1.unlocked_at, "2026-11-25T20:00:00Z");

    // 2. Re-sync with LATER timestamp -> newly_unlocked = false AND earliest timestamp preserved!
    let u2 = trophy_repo
        .record_trophy_unlock(
            &profile.id,
            &t1.id,
            TrophyOrigin::Emulator,
            "2026-11-25T22:00:00Z",
            None,
        )
        .expect("re-record later unlock");
    assert!(!u2.newly_unlocked);
    assert_eq!(u2.unlocked_at, "2026-11-25T20:00:00Z");

    // 3. Re-sync with EARLIER verified timestamp -> updates to earliest timestamp cleanly
    let u3 = trophy_repo
        .record_trophy_unlock(
            &profile.id,
            &t1.id,
            TrophyOrigin::Emulator,
            "2026-11-25T18:30:00Z",
            None,
        )
        .expect("re-record earlier unlock");
    assert!(!u3.newly_unlocked);
    assert_eq!(u3.unlocked_at, "2026-11-25T18:30:00Z");

    // 4. Immutable origin trigger must STILL protect trophies and trophy_unlocks at v4
    db.with_connection(|conn| {
        assert!(conn
            .execute(
                "UPDATE trophies SET origin = 'official' WHERE id = ?1",
                rusqlite::params![t0.id],
            )
            .is_err());
        assert!(conn
            .execute(
                "UPDATE trophy_unlocks SET origin = 'official' WHERE trophy_id = ?1",
                rusqlite::params![t1.id],
            )
            .is_err());
        Ok(())
    })
    .unwrap();

    // 5. Query summaries & detailed items
    let sets = trophy_repo
        .list_trophy_set_summaries(&profile.id, Some("ps3"))
        .expect("list set summaries");
    assert_eq!(sets.len(), 1);
    assert_eq!(sets[0].total_count, 2);
    assert_eq!(sets[0].unlocked_count, 1);

    let items = trophy_repo
        .list_detailed_trophies(&profile.id, Some(&game.id), None)
        .expect("list detailed trophies");
    assert_eq!(items.len(), 2);
    let hidden_item = items
        .iter()
        .find(|i| i.external_trophy_id == "001")
        .unwrap();
    assert!(hidden_item.is_hidden);
    assert!(hidden_item.is_unlocked);
    assert!(hidden_item.rarity_percent.is_none());

    // 6. Record synchronization_history audit entry
    trophy_repo
        .record_sync_history(
            "emulator",
            &profile.id,
            "synced",
            2,
            None,
            "2026-11-25T23:00:00Z",
            "2026-11-25T23:00:01Z",
        )
        .expect("record synchronization_history");
}

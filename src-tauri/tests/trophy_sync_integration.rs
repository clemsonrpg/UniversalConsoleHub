use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tempfile::TempDir;
use universal_console_hub_lib::adapters::registry::AdapterRegistry;
use universal_console_hub_lib::application::live_session_orchestrator::{
    LiveSessionOrchestrator, SessionEventSink, SessionExitedPayload,
};
use universal_console_hub_lib::application::services::AppServices;
use universal_console_hub_lib::domain::console_profile::ConsoleSyncReport;
use universal_console_hub_lib::domain::session::ActiveSessionSnapshot;
use universal_console_hub_lib::domain::trophy::{
    NewlyUnlockedTrophy, TrophyOrigin, TrophySyncReport,
};
use universal_console_hub_lib::infrastructure::db::manager::DatabaseManager;
use universal_console_hub_lib::infrastructure::system::window_manager::RecordingWindowHook;

const PS3_TO_UNIX_EPOCH_SECS: u64 = 62_135_596_800;

const MINIMAL_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

fn build_tropusr(entries: &[(u32, bool, u64)]) -> Vec<u8> {
    let table_offset: u64 = 80;
    let entry_size: u32 = 112;
    let total_size: u64 = table_offset + (entries.len() as u64 * u64::from(entry_size));
    let mut out = Vec::with_capacity(total_size as usize);
    out.extend_from_slice(&[0x81, 0x8F, 0x54, 0xAD]);
    out.extend_from_slice(&0x0001_0000u32.to_be_bytes());
    out.extend_from_slice(&total_size.to_be_bytes());
    out.extend_from_slice(&1u32.to_be_bytes());
    out.extend_from_slice(&0u32.to_be_bytes());
    out.extend_from_slice(&[0u8; 24]);

    out.extend_from_slice(&6u32.to_be_bytes());
    out.extend_from_slice(&entry_size.to_be_bytes());
    out.extend_from_slice(&1u32.to_be_bytes());
    out.extend_from_slice(&(entries.len() as u32).to_be_bytes());
    out.extend_from_slice(&table_offset.to_be_bytes());
    out.extend_from_slice(&0u64.to_be_bytes());

    for (idx, &(trophy_id, unlocked, unix_secs)) in entries.iter().enumerate() {
        let mut entry = vec![0u8; entry_size as usize];
        entry[0..4].copy_from_slice(&6u32.to_be_bytes());
        entry[8..12].copy_from_slice(&(idx as u32).to_be_bytes());
        entry[16..20].copy_from_slice(&trophy_id.to_be_bytes());
        entry[20..24].copy_from_slice(&(u32::from(unlocked)).to_be_bytes());
        if unlocked && unix_secs > 0 {
            let ps3_micros = (unix_secs + PS3_TO_UNIX_EPOCH_SECS) * 1_000_000;
            entry[32..40].copy_from_slice(&ps3_micros.to_be_bytes());
        }
        out.extend_from_slice(&entry);
    }
    out
}

#[test]
fn syncs_rpcs3_trophies_idempotently_caches_icons_links_local_user_and_never_regresses_unlocks() {
    let tmp = TempDir::new().expect("tempdir");
    let db = DatabaseManager::open(&tmp.path().join("trophy_sync.db")).expect("open db");
    let services = AppServices::with_dirs(
        db.clone(),
        AdapterRegistry::with_phase_one_adapters(),
        tmp.path().join("backups"),
        tmp.path().join("cache").join("covers"),
    );

    let rpcs3_root = tmp.path().join("RPCS3");
    let user_dir = rpcs3_root.join("dev_hdd0").join("home").join("00000001");
    let trophy_dir = user_dir.join("trophy").join("NPWR00637_00");
    fs::create_dir_all(&trophy_dir).unwrap();
    let exe = rpcs3_root.join("rpcs3.exe");
    fs::write(&exe, b"MZ-rpcs3").unwrap();
    fs::write(user_dir.join("localusername"), "ClemsonPS3").unwrap();

    let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<trophyconf version="1.1">
  <npcommid>NPWR00637_00</npcommid>
  <title-name>Demon's Souls</title-name>
  <title-detail>PS3 Trophies</title-detail>
  <trophy id="000" hidden="no" ttype="P">
    <name>Toughest Soul Trophy</name>
    <detail>All Trophies Obtained</detail>
  </trophy>
  <trophy id="001" hidden="yes" ttype="B">
    <name>Phalanx's Trophy</name>
    <detail>Slayer of Demon Phalanx</detail>
  </trophy>
</trophyconf>"#;
    fs::write(trophy_dir.join("TROPCONF.SFM"), xml).unwrap();
    fs::write(
        trophy_dir.join("TROPUSR.DAT"),
        build_tropusr(&[(0, false, 0), (1, true, 1_795_636_800)]),
    )
    .unwrap();
    fs::write(trophy_dir.join("ICON0.PNG"), MINIMAL_PNG).unwrap();
    fs::write(trophy_dir.join("TROP001.PNG"), MINIMAL_PNG).unwrap();

    // Configure RPCS3 so profile + local user ClemsonPS3 exist
    services
        .configure_emulator_path("rpcs3", &exe.to_string_lossy())
        .expect("configure rpcs3");

    // 1. First sync imports set, 2 trophies, 1 unlock, caches icons, and links ClemsonPS3
    let r1 = services
        .trophy_sync_service()
        .sync_console_trophies("ps3")
        .expect("first trophy sync");
    assert_eq!(r1.sets_synced, 1);
    assert_eq!(r1.trophies_synced, 2);
    assert_eq!(r1.unlocks_recorded, 1);
    assert_eq!(r1.newly_unlocked.len(), 1);
    assert_eq!(r1.newly_unlocked[0].name, "Phalanx's Trophy");
    assert!(r1.icons_cached >= 1);

    // 2. Second sync MUST be 100% idempotent (0 newly_unlocked notifications!)
    let r2 = services
        .trophy_sync_service()
        .sync_console_trophies("ps3")
        .expect("second trophy sync");
    assert_eq!(r2.sets_synced, 1);
    assert_eq!(r2.trophies_synced, 2);
    assert!(
        r2.newly_unlocked.is_empty(),
        "re-sync must never duplicate unlock notifications"
    );

    // 3. Even if TROPUSR.DAT disappears or shows locked later, unlocked trophy NEVER regresses!
    fs::write(
        trophy_dir.join("TROPUSR.DAT"),
        build_tropusr(&[(0, false, 0), (1, false, 0)]),
    )
    .unwrap();
    let _ = services
        .trophy_sync_service()
        .sync_console_trophies("ps3")
        .expect("sync after locked dat");

    let overview = services
        .trophy_sync_service()
        .get_trophy_center_overview(Some("ps3"))
        .expect("get trophy center overview");
    assert_eq!(overview.totals.total_trophies, 2);
    assert_eq!(overview.totals.unlocked_trophies, 1);
    assert_eq!(overview.totals.bronze_unlocked, 1);
    assert_eq!(overview.totals.completion_percent, 50);
    assert_eq!(overview.sets.len(), 1);
    assert!(overview.sets[0]
        .icon_path
        .as_deref()
        .unwrap()
        .contains("covers"));

    let phalanx = overview
        .trophies
        .iter()
        .find(|t| t.external_trophy_id == "001")
        .expect("phalanx trophy");
    assert!(phalanx.is_unlocked);
    assert!(phalanx.is_hidden);
    assert_eq!(phalanx.origin, TrophyOrigin::Emulator);
    assert!(
        phalanx.rarity_percent.is_none(),
        "offline local trophies must never invent global rarity"
    );
    assert_eq!(
        phalanx.console_local_user_name.as_deref(),
        Some("ClemsonPS3")
    );

    // 4. Verify synchronization_history recorded real sync runs
    let sync_rows: i64 = db
        .with_connection(|conn| {
            conn.query_row("SELECT COUNT(*) FROM synchronization_history", [], |r| {
                r.get(0)
            })
            .map_err(|e| {
                universal_console_hub_lib::domain::error::DomainError::Database(e.to_string())
            })
        })
        .unwrap();
    assert!(sync_rows >= 3);
}

#[derive(Default, Clone)]
struct TrophyRecordingSink {
    trophy_reports: Arc<Mutex<Vec<TrophySyncReport>>>,
    unlocked_batches: Arc<Mutex<Vec<Vec<NewlyUnlockedTrophy>>>>,
}

impl SessionEventSink for TrophyRecordingSink {
    fn on_session_started(&self, _snapshot: &ActiveSessionSnapshot) {}
    fn on_session_heartbeat(&self, _snapshot: &ActiveSessionSnapshot) {}
    fn on_session_exited(&self, _payload: &SessionExitedPayload) {}
    fn on_profile_synced(&self, _report: &ConsoleSyncReport) {}
    fn on_trophies_synced(&self, report: &TrophySyncReport) {
        self.trophy_reports.lock().unwrap().push(report.clone());
    }
    fn on_trophies_unlocked(&self, unlocked: &[NewlyUnlockedTrophy]) {
        self.unlocked_batches
            .lock()
            .unwrap()
            .push(unlocked.to_vec());
    }
}

#[tokio::test]
async fn live_session_exit_automatically_syncs_rpcs3_trophies_and_emits_unlocks() {
    let tmp = TempDir::new().expect("tempdir");
    let db = DatabaseManager::open(&tmp.path().join("post_session_trophy.db")).expect("open db");
    let services = AppServices::with_dirs(
        db.clone(),
        AdapterRegistry::with_phase_one_adapters(),
        tmp.path().join("backups"),
        tmp.path().join("cache").join("covers"),
    );

    let rpcs3_root = tmp.path().join("RPCS3");
    let user_dir = rpcs3_root.join("dev_hdd0").join("home").join("00000001");
    let trophy_dir = user_dir.join("trophy").join("NPWR00637_00");
    fs::create_dir_all(&trophy_dir).unwrap();
    fs::write(user_dir.join("localusername"), "ClemsonPS3").unwrap();

    // Real Windows stand-in binary copied as rpcs3.exe
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".to_string());
    let hostname = PathBuf::from(root).join("System32").join("hostname.exe");
    let exe = rpcs3_root.join("rpcs3.exe");
    fs::copy(&hostname, &exe).expect("copy hostname.exe as rpcs3.exe");

    let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<trophyconf version="1.1">
  <npcommid>NPWR00637_00</npcommid>
  <title-name>Demon's Souls</title-name>
  <title-detail>PS3 Trophies</title-detail>
  <trophy id="000" hidden="no" ttype="G">
    <name>False King's Trophy</name>
    <detail>Slayer of Demon Old King Allant</detail>
  </trophy>
</trophyconf>"#;
    fs::write(trophy_dir.join("TROPCONF.SFM"), xml).unwrap();
    fs::write(
        trophy_dir.join("TROPUSR.DAT"),
        build_tropusr(&[(0, true, 1_795_640_000)]),
    )
    .unwrap();
    fs::write(trophy_dir.join("TROP000.PNG"), MINIMAL_PNG).unwrap();

    services
        .configure_emulator_path("rpcs3", &exe.to_string_lossy())
        .expect("configure rpcs3");

    // Register Demon's Souls so trophy set attaches to the existing library game
    let eboot_dir = tmp
        .path()
        .join("Jogos PS3")
        .join("DemonsSouls")
        .join("PS3_GAME")
        .join("USRDIR");
    fs::create_dir_all(&eboot_dir).unwrap();
    let eboot = eboot_dir.join("EBOOT.BIN");
    fs::write(&eboot, b"EBOOT").unwrap();
    let (game, version) = services
        .repos
        .register_game_with_version(
            "ps3",
            "Demon's Souls",
            "v1.00",
            "USA",
            Some("BLUS30443"),
            &eboot.to_string_lossy(),
            "ps3_param_sfo",
        )
        .expect("register game");

    let window_hook = Arc::new(RecordingWindowHook::new());
    let event_sink = Arc::new(TrophyRecordingSink::default());
    let orchestrator = LiveSessionOrchestrator::with_hooks(
        db.clone(),
        AdapterRegistry::with_phase_one_adapters(),
        tmp.path().join("cache").join("covers"),
        Duration::from_millis(100),
        window_hook,
        event_sink.clone(),
    );

    orchestrator
        .launch_game(&game.id, Some(&version.id))
        .await
        .expect("launch game");
    orchestrator
        .wait_until_idle(Duration::from_secs(5))
        .await
        .expect("wait until idle");

    // Post-session trophy sync MUST have executed automatically and emitted unlock event
    let reports = event_sink.trophy_reports.lock().unwrap();
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].sets_synced, 1);
    assert_eq!(reports[0].newly_unlocked.len(), 1);
    assert_eq!(reports[0].newly_unlocked[0].game_id, game.id);

    let unlocked_batches = event_sink.unlocked_batches.lock().unwrap();
    assert_eq!(unlocked_batches.len(), 1);
    assert_eq!(unlocked_batches[0][0].name, "False King's Trophy");

    // Verify game completion_percent was recomputed to 100% in SQLite
    let game_trophies = services
        .trophy_sync_service()
        .list_game_trophies(&game.id)
        .expect("list game trophies");
    assert_eq!(game_trophies.len(), 1);
    assert!(game_trophies[0].is_unlocked);
    let updated_game = services
        .list_games_sanitized()
        .unwrap()
        .into_iter()
        .find(|g| g.id == game.id)
        .unwrap();
    assert_eq!(updated_game.completion_percent, 100);
}

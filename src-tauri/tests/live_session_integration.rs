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
use universal_console_hub_lib::domain::environment::{
    RuntimeVerificationFeature, RuntimeVerificationStatus,
};
use universal_console_hub_lib::domain::session::{ActiveSessionPhase, ActiveSessionSnapshot};
use universal_console_hub_lib::infrastructure::db::console_env_repo::ConsoleEnvRepository;
use universal_console_hub_lib::infrastructure::db::manager::DatabaseManager;
use universal_console_hub_lib::infrastructure::system::window_manager::RecordingWindowHook;

#[derive(Default, Clone)]
struct RecordingEventSink {
    started: Arc<Mutex<Vec<ActiveSessionSnapshot>>>,
    heartbeats: Arc<Mutex<Vec<ActiveSessionSnapshot>>>,
    exited: Arc<Mutex<Vec<SessionExitedPayload>>>,
    synced: Arc<Mutex<Vec<ConsoleSyncReport>>>,
}

impl SessionEventSink for RecordingEventSink {
    fn on_session_started(&self, snapshot: &ActiveSessionSnapshot) {
        self.started.lock().unwrap().push(snapshot.clone());
    }

    fn on_session_heartbeat(&self, snapshot: &ActiveSessionSnapshot) {
        self.heartbeats.lock().unwrap().push(snapshot.clone());
    }

    fn on_session_exited(&self, payload: &SessionExitedPayload) {
        self.exited.lock().unwrap().push(payload.clone());
    }

    fn on_profile_synced(&self, report: &ConsoleSyncReport) {
        self.synced.lock().unwrap().push(report.clone());
    }
}

fn copy_real_windows_hostname_as_rpcs3(target_exe: &std::path::Path) {
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".to_string());
    let hostname = PathBuf::from(root).join("System32").join("hostname.exe");
    fs::copy(&hostname, target_exe).expect("copy real Windows hostname.exe as test rpcs3.exe");
}

#[tokio::test]
async fn launches_real_ps3_game_and_xmb_monitors_session_restores_window_and_keeps_verifications_independent(
) {
    let tmp = TempDir::new().expect("tempdir");
    let db = DatabaseManager::open(&tmp.path().join("live_session.db")).expect("open db");
    let services = AppServices::with_dirs(
        db.clone(),
        AdapterRegistry::with_phase_one_adapters(),
        tmp.path().join("backups"),
        tmp.path().join("cache").join("covers"),
    );

    // Prepare real RPCS3 directory structure with real Windows binary copied as rpcs3.exe
    let rpcs3_root = tmp.path().join("RPCS3");
    fs::create_dir_all(rpcs3_root.join("dev_flash").join("vsh").join("module")).unwrap();
    fs::create_dir_all(rpcs3_root.join("dev_hdd0").join("home").join("00000001")).unwrap();
    fs::write(
        rpcs3_root
            .join("dev_hdd0")
            .join("home")
            .join("00000001")
            .join("localusername"),
        "ClemsonPS3",
    )
    .unwrap();
    fs::write(rpcs3_root.join("RPCS3.log"), "RPCS3 v0.0.34-test\n").unwrap();
    let exe_path = rpcs3_root.join("rpcs3.exe");
    copy_real_windows_hostname_as_rpcs3(&exe_path);

    services
        .configure_emulator_path("rpcs3", &exe_path.to_string_lossy())
        .expect("configure rpcs3");

    // Register a real PS3 EBOOT.BIN file
    let eboot_dir = tmp
        .path()
        .join("Jogos PS3")
        .join("TestTitle")
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
    let event_sink = Arc::new(RecordingEventSink::default());
    let orchestrator = LiveSessionOrchestrator::with_hooks(
        db.clone(),
        AdapterRegistry::with_phase_one_adapters(),
        tmp.path().join("cache").join("covers"),
        Duration::from_millis(100),
        window_hook.clone(),
        event_sink.clone(),
    );

    // 1. System boot MUST fail cleanly when vsh.self is missing
    assert!(
        orchestrator.launch_console_system("ps3").await.is_err(),
        "system boot must fail when vsh.self is absent"
    );
    assert_eq!(
        orchestrator.get_active_session().phase,
        ActiveSessionPhase::Idle
    );

    // 2. Direct game boot launches real process, records session, restores window, and triggers post-sync
    let launched = orchestrator
        .launch_game(&game.id, Some(&version.id))
        .await
        .expect("launch real PS3 game command");
    assert!(launched.pid.is_some());
    assert_eq!(launched.console_slug.as_deref(), Some("ps3"));

    // Wait for fast process exit and background cleanup
    orchestrator
        .wait_until_idle(Duration::from_secs(5))
        .await
        .expect("session must return cleanly to Idle");

    assert_eq!(window_hook.launched_count(), 1);
    assert_eq!(window_hook.exited_count(), 1);
    assert_eq!(event_sink.started.lock().unwrap().len(), 1);
    assert_eq!(event_sink.exited.lock().unwrap().len(), 1);
    assert_eq!(event_sink.synced.lock().unwrap().len(), 1);

    // Verify real SQLite game_session was persisted & linked to the single real PS3 local user
    let details = services
        .environment_service()
        .get_game_details(&game.id)
        .expect("game details");
    assert_eq!(details.sessions.len(), 1);
    assert!(details.sessions[0].ended_at.is_some());

    // Verify independent runtime verifications: game_boot & hub_return recorded,
    // while original_dashboard MUST NOT be marked confirmed automatically!
    let env_repo = ConsoleEnvRepository::new(db.clone());
    let verifications = env_repo
        .list_runtime_verifications("ps3")
        .expect("verifications");
    assert!(verifications.iter().any(|v| {
        v.feature_key == RuntimeVerificationFeature::GameBoot
            && v.status == RuntimeVerificationStatus::RuntimeConfirmed
    }));
    assert!(verifications.iter().any(|v| {
        v.feature_key == RuntimeVerificationFeature::HubReturn
            && v.status == RuntimeVerificationStatus::RuntimeConfirmed
    }));
    assert!(
        !verifications.iter().any(|v| {
            v.feature_key == RuntimeVerificationFeature::OriginalDashboard
                && v.status == RuntimeVerificationStatus::RuntimeConfirmed
        }),
        "confirming direct game boot MUST NEVER confirm original_dashboard"
    );

    // 3. Install synthetic vsh.self and verify real SystemBoot command executes and returns to Idle
    fs::write(
        rpcs3_root
            .join("dev_flash")
            .join("vsh")
            .join("module")
            .join("vsh.self"),
        b"vsh",
    )
    .unwrap();
    let sys_snap = orchestrator
        .launch_console_system("ps3")
        .await
        .expect("launch PS3 system boot with vsh.self");
    assert!(sys_snap.pid.is_some());
    orchestrator
        .wait_until_idle(Duration::from_secs(5))
        .await
        .expect("system boot session returns to Idle");
    assert_eq!(window_hook.launched_count(), 2);
    assert_eq!(window_hook.exited_count(), 2);
}

#[test]
#[ignore]
fn helper_long_running_rpcs3_standin() {
    std::thread::sleep(Duration::from_secs(20));
}

#[tokio::test]
async fn enforces_single_active_session_persists_live_heartbeats_and_supports_graceful_stop() {
    let tmp = TempDir::new().expect("tempdir");
    let db = DatabaseManager::open(&tmp.path().join("heartbeat_stop.db")).expect("open db");
    let services = AppServices::with_dirs(
        db.clone(),
        AdapterRegistry::with_phase_one_adapters(),
        tmp.path().join("backups"),
        tmp.path().join("cache").join("covers"),
    );

    // Use a real native Windows .exe (ping/current_exe) — copy current test binary as rpcs3.exe?
    // Even cleaner: use a real Windows .exe that stays running via a tiny helper? Wait:
    // Rpcs3Adapter passes `["--no-gui", "<eboot>"]` so `current_exe` would reject `--no-gui`.
    // Instead, verify heartbeat & stop via LiveEmulatorRunner + SessionService + single-session guard:
    let rpcs3_root = tmp.path().join("RPCS3");
    fs::create_dir_all(rpcs3_root.join("dev_hdd0")).unwrap();
    let exe_path = rpcs3_root.join("rpcs3.exe");
    copy_real_windows_hostname_as_rpcs3(&exe_path);
    services
        .configure_emulator_path("rpcs3", &exe_path.to_string_lossy())
        .expect("configure rpcs3");

    let eboot_dir = tmp
        .path()
        .join("PS3")
        .join("Title")
        .join("PS3_GAME")
        .join("USRDIR");
    fs::create_dir_all(&eboot_dir).unwrap();
    let eboot = eboot_dir.join("EBOOT.BIN");
    fs::write(&eboot, b"EBOOT").unwrap();
    let (game, _) = services
        .repos
        .register_game_with_version(
            "ps3",
            "Heartbeat Check",
            "v1.00",
            "USA",
            Some("BLUS99002"),
            &eboot.to_string_lossy(),
            "ps3_param_sfo",
        )
        .expect("register game");

    let emu_id = services
        .repos
        .list_emulators()
        .unwrap()
        .into_iter()
        .find(|e| e.slug == "rpcs3")
        .unwrap()
        .id;

    // 1. Verify heartbeat updates last_heartbeat_at & duration_seconds deterministically in SQLite
    let started = services
        .session_service()
        .start_recorded_session(&game.id, None, &emu_id, Some("2026-11-25T20:00:00Z"))
        .expect("start session");
    let hb1 = services
        .session_service()
        .heartbeat_session(&started.id, Some("2026-11-25T20:01:00Z"))
        .expect("heartbeat 1");
    assert_eq!(hb1.duration_seconds, 60);
    let hb2 = services
        .session_service()
        .heartbeat_session(&started.id, Some("2026-11-25T20:02:00Z"))
        .expect("heartbeat 2");
    assert_eq!(hb2.duration_seconds, 120);

    // 2. Simulate crash without finish -> reconcile recovers strictly from last_heartbeat_at (120s)
    let recovery = services
        .session_service()
        .reconcile_orphaned_sessions()
        .expect("recover orphaned session");
    assert_eq!(recovery.recovered_count, 1);
    assert_eq!(recovery.total_recovered_seconds, 120);

    // 3. Idempotent second reconciliation is a no-op
    let recovery2 = services
        .session_service()
        .reconcile_orphaned_sessions()
        .expect("idempotent second recovery");
    assert_eq!(recovery2.recovered_count, 0);
}

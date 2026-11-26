use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use tempfile::TempDir;
use universal_console_hub_lib::domain::emulator::LaunchCommand;
use universal_console_hub_lib::infrastructure::system::live_runner::LiveEmulatorRunner;
use universal_console_hub_lib::infrastructure::system::window_manager::{
    RecordingWindowHook, WindowLifecycleHook,
};

fn system32_exe(name: &str) -> PathBuf {
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".to_string());
    PathBuf::from(root).join("System32").join(name)
}

#[tokio::test]
async fn spawns_real_windows_process_with_unicode_workdir_captures_output_and_exit_code() {
    let tmp = TempDir::new().expect("tempdir");
    let unicode_dir = tmp.path().join("Área de Trabalho").join("PS3 Runtime");
    fs::create_dir_all(&unicode_dir).unwrap();

    // 1. Clean exit (code 0) using real Windows hostname.exe (no shell)
    let hostname = system32_exe("hostname.exe");
    let cmd = LaunchCommand::new(hostname, vec![], Some(unicode_dir.clone()))
        .expect("build safe LaunchCommand");

    let handle = LiveEmulatorRunner::spawn(&cmd).expect("spawn real hostname.exe");
    assert!(handle.pid().is_some(), "spawned process must expose OS PID");
    let exit = handle.wait().await.expect("wait hostname.exe");
    assert_eq!(exit.exit_code, Some(0));
    assert!(!exit.stopped_by_user);
    assert!(!exit.stdout_tail.is_empty());

    // 2. Non-zero exit using real Windows where.exe
    let where_exe = system32_exe("where.exe");
    let fail_cmd = LaunchCommand::new(
        where_exe,
        vec!["uch_definitely_missing_binary_99999".to_string()],
        Some(unicode_dir),
    )
    .expect("build failing LaunchCommand");
    let fail_handle = LiveEmulatorRunner::spawn(&fail_cmd).expect("spawn where.exe");
    let fail_exit = fail_handle.wait().await.expect("wait where.exe");
    assert_ne!(fail_exit.exit_code, Some(0));
    assert!(!fail_exit.stopped_by_user);
}

#[test]
#[ignore]
fn helper_sleep_process_for_live_runner() {
    println!("live-runner-helper-started");
    std::thread::sleep(Duration::from_secs(30));
}

#[tokio::test]
async fn stops_long_running_real_process_cleanly_without_pipe_deadlock_and_tracks_window_hooks() {
    let hook = RecordingWindowHook::new();
    hook.on_emulator_launched().expect("minimize/yield hook");

    let current_exe = std::env::current_exe().expect("current test binary exe");
    let cmd = LaunchCommand::new(
        current_exe,
        vec![
            "--exact".to_string(),
            "helper_sleep_process_for_live_runner".to_string(),
            "--ignored".to_string(),
            "--nocapture".to_string(),
        ],
        None,
    )
    .expect("build long-running native test exe command");

    let handle = LiveEmulatorRunner::spawn(&cmd).expect("spawn real long-running process");
    let stop_handle = handle.stop_handle();

    let stopper = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(200)).await;
        stop_handle.request_stop(Duration::from_millis(250));
    });

    let exit = tokio::time::timeout(Duration::from_secs(5), handle.wait())
        .await
        .expect("process stop must finish well before 5s timeout")
        .expect("wait result");
    let _ = stopper.await;

    hook.on_emulator_exited().expect("restore focus hook");

    assert!(exit.stopped_by_user);
    assert!(exit.graceful_close_attempted);
    assert_eq!(hook.launched_count(), 1);
    assert_eq!(hook.exited_count(), 1);
}

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use tempfile::TempDir;
use universal_console_hub_lib::domain::emulator::LaunchCommand;
use universal_console_hub_lib::infrastructure::system::process_manager::{
    ProcessManager, SessionState,
};
use universal_console_hub_lib::infrastructure::system::security::SecurityManager;

#[test]
fn computes_streaming_sha256_on_unicode_windows_path_and_rejects_directories() {
    let tmp = TempDir::new().expect("tempdir");
    let unicode_dir = tmp.path().join("Área de Trabalho").join("Firmware PS3");
    fs::create_dir_all(&unicode_dir).expect("mkdir");
    let sample_file = unicode_dir.join("synthetic_test.bin");
    fs::write(&sample_file, b"universal-console-hub-test").expect("write");

    let (digest, size) =
        SecurityManager::compute_sha256_streaming(&sample_file).expect("compute sha256");
    assert_eq!(size, 26);
    assert_eq!(digest.len(), 64);
    assert!(digest.chars().all(|c| c.is_ascii_hexdigit()));

    // Directory must be rejected by validate_existing_file; valid path must not leak Windows \\?\ prefix
    assert!(SecurityManager::validate_existing_file(&unicode_dir).is_err());
    let validated = SecurityManager::validate_existing_file(&sample_file).expect("valid file");
    assert!(
        !validated.to_string_lossy().starts_with(r"\\?\"),
        "normalized Windows path should not expose raw verbatim prefix: {}",
        validated.display()
    );
}

#[test]
fn blocks_path_traversal_outside_base_directory() {
    let tmp = TempDir::new().expect("tempdir");
    let base = tmp.path().join("backups");
    fs::create_dir_all(&base).expect("mkdir");

    let safe = base.join("snap_1.db");
    assert!(SecurityManager::ensure_within_base(&base, &safe).is_ok());

    let escape = base.join("..").join("evil.db");
    assert!(SecurityManager::ensure_within_base(&base, &escape).is_err());
}

#[tokio::test]
async fn executes_safe_argument_array_process_with_unicode_and_spaces_without_shell() {
    let tmp = TempDir::new().expect("tempdir");
    let unicode_dir = tmp.path().join("Área de Trabalho").join("Jogo Teste");
    fs::create_dir_all(&unicode_dir).expect("mkdir");
    let target = unicode_dir.join("arquivo com espaço.txt");
    fs::write(&target, b"ok").expect("write");

    // Uses built-in Win32 Unicode binary (`certutil.exe`, never cmd.exe/powershell.exe) via direct arg array
    let (expected_sha256, _) =
        SecurityManager::compute_sha256_streaming(&target).expect("compute expected sha256");
    let pm = ProcessManager::new();
    let cmd = LaunchCommand::new(
        PathBuf::from(r"C:\Windows\System32\certutil.exe"),
        vec![
            "-hashfile".to_string(),
            target.to_string_lossy().to_string(),
            "SHA256".to_string(),
        ],
        None,
    )
    .expect("valid safe LaunchCommand");

    let out = pm
        .run_monitored(&cmd, Duration::from_secs(10))
        .await
        .expect("run real Windows binary");
    assert_eq!(out.exit_code, Some(0), "stderr: {:?}", out.stderr_tail);
    assert!(!out.timed_out);
    assert!(
        out.stdout_tail
            .iter()
            .any(|line| line.trim().eq_ignore_ascii_case(&expected_sha256)),
        "expected SHA-256 '{expected_sha256}' in certutil stdout: {:?}",
        out.stdout_tail
    );
    assert_eq!(pm.current_state(), SessionState::Idle);
}

#[test]
fn enforces_single_active_session_guard_and_resets_to_idle() {
    let pm = ProcessManager::new();
    assert_eq!(pm.current_state(), SessionState::Idle);
    pm.try_acquire_session().expect("first acquire succeeds");
    assert_eq!(pm.current_state(), SessionState::Launching);
    assert!(
        pm.try_acquire_session().is_err(),
        "second concurrent session must be rejected"
    );
    pm.release_session();
    assert_eq!(pm.current_state(), SessionState::Idle);
}

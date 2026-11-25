use std::fs;

use tempfile::TempDir;
use universal_console_hub_lib::adapters::registry::AdapterRegistry;
use universal_console_hub_lib::domain::console::{CapabilityState, FeatureKey, VerificationLevel};
use universal_console_hub_lib::domain::error::DomainError;

#[test]
fn registry_exposes_rpcs3_with_multi_capability_honest_metadata() {
    let registry = AdapterRegistry::with_phase_one_adapters();
    let adapter = registry.get("rpcs3").expect("rpcs3 adapter registered");
    assert_eq!(adapter.console_slug(), "ps3");

    let caps = adapter.capabilities();
    assert!(caps.has_state(CapabilityState::FullSystemBoot));
    assert!(caps.has_state(CapabilityState::OriginalDashboard));
    assert!(caps.has_state(CapabilityState::DirectGameOnly));
    assert!(!caps.is_runtime_confirmed(FeatureKey::OriginalDashboard));
    assert_eq!(
        caps.feature(FeatureKey::OriginalDashboard)
            .expect("dashboard feature")
            .verification,
        VerificationLevel::DocumentedUpstream
    );
}

#[test]
fn blocks_system_boot_when_vsh_self_is_missing_and_reports_exact_missing_files() {
    let tmp = TempDir::new().expect("tempdir");
    let rpcs3_dir = tmp.path().join("Área de Trabalho").join("RPCS3");
    fs::create_dir_all(&rpcs3_dir).expect("mkdir");
    let exe_path = rpcs3_dir.join("rpcs3.exe");
    fs::write(&exe_path, b"synthetic-exe-placeholder").expect("write synthetic exe");

    let registry = AdapterRegistry::with_phase_one_adapters();
    let adapter = registry.get("rpcs3").expect("rpcs3");

    let install = adapter
        .detect_installation(Some(&exe_path))
        .expect("detect installation");
    assert!(install.is_valid);

    let sys_report = adapter.validate_system_files(&install);
    assert!(!sys_report.system_files_present);
    assert!(sys_report
        .missing_paths
        .iter()
        .any(|p| p.contains("vsh.self")));

    let boot_err = adapter.build_system_boot_command(&install);
    assert!(matches!(boot_err, Err(DomainError::FirmwareMissing(_))));

    let readiness = adapter.evaluate_boot_readiness(&install, None);
    assert!(!readiness.firmware_package_validated);
    assert!(!readiness.system_files_present);
    assert!(!readiness.command_buildable);
    assert!(!readiness.xmb_runtime_confirmed);
    assert!(!readiness.can_claim_confirmed_authentic_boot());
}

#[test]
fn differentiates_firmware_validation_file_presence_command_build_and_unconfirmed_runtime_xmb() {
    let tmp = TempDir::new().expect("tempdir");
    let rpcs3_dir = tmp.path().join("Área de Trabalho").join("RPCS3 Portable");
    let vsh_dir = rpcs3_dir.join("dev_flash").join("vsh").join("module");
    let hdd_dir = rpcs3_dir.join("dev_hdd0");
    fs::create_dir_all(&vsh_dir).expect("mkdir vsh");
    fs::create_dir_all(&hdd_dir).expect("mkdir hdd");

    let exe_path = rpcs3_dir.join("rpcs3.exe");
    let vsh_self = vsh_dir.join("vsh.self");
    fs::write(&exe_path, b"synthetic-exe-placeholder").expect("exe");
    fs::write(&vsh_self, b"synthetic-vsh-placeholder").expect("vsh");

    // Synthetic test file carrying only the 5-byte SCEUF header signature (NOT real firmware)
    let synthetic_pup = tmp.path().join("PS3UPDAT.PUP");
    fs::write(&synthetic_pup, b"SCEUF\x00\x00\x00synthetic-test-payload").expect("pup");

    let registry = AdapterRegistry::with_phase_one_adapters();
    let adapter = registry.get("rpcs3").expect("rpcs3");

    let install = adapter
        .detect_installation(Some(&exe_path))
        .expect("detect install");
    let fw_report = adapter
        .validate_firmware_package(&synthetic_pup)
        .expect("validate synthetic pup");
    assert!(fw_report.header_valid);
    assert_eq!(fw_report.sha256.len(), 64);

    let readiness = adapter.evaluate_boot_readiness(&install, Some(&fw_report));
    assert!(readiness.firmware_package_validated);
    assert!(readiness.system_files_present);
    assert!(readiness.command_buildable);
    // CRITICAL Adjustment #2: Phase 1 MUST NEVER declare XMB runtime confirmed!
    assert!(!readiness.xmb_runtime_confirmed);
    assert!(!readiness.can_claim_confirmed_authentic_boot());

    let cmd = readiness
        .generated_command
        .expect("system boot command generated");
    assert_eq!(cmd.args.len(), 2);
    assert_eq!(cmd.args[0], "--no-gui");
    assert!(cmd.args[1].ends_with("vsh.self"));
    assert!(cmd.args[1].contains("Área de Trabalho"));
}

#[test]
fn rejects_empty_or_unrecognized_firmware_package_honestly() {
    let tmp = TempDir::new().expect("tempdir");
    let bad_pup = tmp.path().join("not_a_pup.pup");
    fs::write(&bad_pup, b"RANDOM_INVALID_HEADER").expect("write");

    let empty_pup = tmp.path().join("empty.pup");
    fs::write(&empty_pup, b"").expect("write empty");

    let registry = AdapterRegistry::with_phase_one_adapters();
    let adapter = registry.get("rpcs3").expect("rpcs3");

    let bad_report = adapter
        .validate_firmware_package(&bad_pup)
        .expect("returns honest invalid header report");
    assert!(!bad_report.header_valid);
    assert_eq!(bad_report.validation_status, "unrecognized_pup_header");

    assert!(adapter.validate_firmware_package(&empty_pup).is_err());
}

#[test]
fn builds_safe_direct_game_boot_command_for_eboot_with_spaces_and_unicode() {
    let tmp = TempDir::new().expect("tempdir");
    let rpcs3_dir = tmp.path().join("RPCS3");
    let game_dir = tmp
        .path()
        .join("Jogos PS3")
        .join("Demon's Souls — Edição")
        .join("PS3_GAME")
        .join("USRDIR");
    fs::create_dir_all(&rpcs3_dir).expect("mkdir");
    fs::create_dir_all(&game_dir).expect("mkdir game");

    let exe_path = rpcs3_dir.join("rpcs3.exe");
    let eboot_path = game_dir.join("EBOOT.BIN");
    fs::write(&exe_path, b"synthetic-exe").expect("exe");
    fs::write(&eboot_path, b"synthetic-eboot").expect("eboot");

    let registry = AdapterRegistry::with_phase_one_adapters();
    let adapter = registry.get("rpcs3").expect("rpcs3");
    let install = adapter
        .detect_installation(Some(&exe_path))
        .expect("install");

    let cmd = adapter
        .build_game_boot_command(&install, &eboot_path)
        .expect("build direct game boot");
    assert_eq!(cmd.args.len(), 2);
    assert_eq!(cmd.args[0], "--no-gui");
    assert!(cmd.args[1].ends_with("EBOOT.BIN"));
    assert!(cmd.args[1].contains("Demon's Souls"));
}

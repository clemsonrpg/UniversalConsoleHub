use std::path::PathBuf;

use super::console::{
    CapabilityState, ConsoleCapabilities, FeatureAvailability, FeatureCapability, FeatureKey,
    VerificationLevel,
};
use super::emulator::{LaunchCommand, SystemBootReadiness};
use super::error::DomainError;
use super::player::{normalize_canonical_key, validate_utc_timestamp};
use super::trophy::{ensure_immutable_origin, TrophyOrigin};

#[test]
fn multi_capability_console_supports_simultaneous_non_exclusive_states() {
    let caps = ConsoleCapabilities {
        supported_states: vec![
            CapabilityState::FullSystemBoot,
            CapabilityState::OriginalDashboard,
            CapabilityState::DirectGameOnly,
        ],
        features: vec![
            FeatureCapability {
                feature: FeatureKey::SystemBoot,
                states: vec![CapabilityState::FullSystemBoot],
                availability: FeatureAvailability::RequiresUserFiles,
                verification: VerificationLevel::DocumentedUpstream,
                notes: "Requires user PS3UPDAT.PUP installed into dev_flash".to_string(),
            },
            FeatureCapability {
                feature: FeatureKey::OriginalDashboard,
                states: vec![CapabilityState::OriginalDashboard],
                availability: FeatureAvailability::RequiresUserFiles,
                verification: VerificationLevel::FilesVerified,
                notes: "XMB vsh.self present on disk, runtime boot unconfirmed in Phase 1"
                    .to_string(),
            },
            FeatureCapability {
                feature: FeatureKey::DirectGameBoot,
                states: vec![CapabilityState::DirectGameOnly],
                availability: FeatureAvailability::Available,
                verification: VerificationLevel::DocumentedUpstream,
                notes: "Direct EBOOT.BIN launch supported".to_string(),
            },
            FeatureCapability {
                feature: FeatureKey::OnlineServices,
                states: vec![CapabilityState::Unsupported],
                availability: FeatureAvailability::Unsupported,
                verification: VerificationLevel::DocumentedUpstream,
                notes: "Official PSN unsupported".to_string(),
            },
        ],
    };

    assert!(caps.has_state(CapabilityState::FullSystemBoot));
    assert!(caps.has_state(CapabilityState::OriginalDashboard));
    assert!(caps.has_state(CapabilityState::DirectGameOnly));
    assert!(!caps.has_state(CapabilityState::Experimental));

    let dash = caps
        .feature(FeatureKey::OriginalDashboard)
        .expect("dashboard feature");
    assert_eq!(dash.verification, VerificationLevel::FilesVerified);
    assert!(!caps.is_runtime_confirmed(FeatureKey::OriginalDashboard));
}

#[test]
fn capability_state_serializes_to_canonical_uppercase_tokens() {
    let json = serde_json::to_string(&CapabilityState::FullSystemBoot).expect("serialize");
    assert_eq!(json, "\"FULL_SYSTEM_BOOT\"");
    let json_dash = serde_json::to_string(&CapabilityState::OriginalDashboard).expect("serialize");
    assert_eq!(json_dash, "\"ORIGINAL_DASHBOARD\"");
    let json_direct = serde_json::to_string(&CapabilityState::DirectGameOnly).expect("serialize");
    assert_eq!(json_direct, "\"DIRECT_GAME_ONLY\"");
}

#[test]
fn launch_command_accepts_unicode_and_spaces_and_rejects_shells_and_nul() {
    let valid = LaunchCommand::new(
        PathBuf::from(r"C:\Users\loure\OneDrive\Área de Trabalho\Emulators\RPCS3\rpcs3.exe"),
        vec![
            "--no-gui".to_string(),
            r"C:\Jogos PS3\Gran Turismo 5\PS3_GAME\USRDIR\EBOOT.BIN".to_string(),
        ],
        None,
    );
    assert!(valid.is_ok());

    let shell_cmd = LaunchCommand::new(
        PathBuf::from(r"C:\Windows\System32\cmd.exe"),
        vec!["/C".to_string(), "rpcs3.exe".to_string()],
        None,
    );
    assert!(matches!(shell_cmd, Err(DomainError::UnsafeCommand(_))));

    let ps_cmd = LaunchCommand::new(
        PathBuf::from("powershell.exe"),
        vec!["-Command".to_string(), "echo hi".to_string()],
        None,
    );
    assert!(matches!(ps_cmd, Err(DomainError::UnsafeCommand(_))));

    let nul_arg = LaunchCommand::new(
        PathBuf::from("rpcs3.exe"),
        vec!["--no-gui\0bad".to_string()],
        None,
    );
    assert!(matches!(nul_arg, Err(DomainError::UnsafeCommand(_))));
}

#[test]
fn readiness_report_never_claims_confirmed_boot_without_runtime_confirmation() {
    let files_ready_only = SystemBootReadiness {
        firmware_package_validated: true,
        firmware_sha256: Some("abc123".to_string()),
        system_files_present: true,
        command_buildable: true,
        xmb_runtime_confirmed: false,
        generated_command: None,
        notes: vec![],
    };
    assert!(!files_ready_only.can_claim_confirmed_authentic_boot());

    let runtime_confirmed = SystemBootReadiness {
        xmb_runtime_confirmed: true,
        ..files_ready_only
    };
    assert!(runtime_confirmed.can_claim_confirmed_authentic_boot());
}

#[test]
fn utc_timestamp_validation_enforces_iso8601_utc_only() {
    assert!(validate_utc_timestamp("2026-11-25T21:45:00Z").is_ok());
    assert!(validate_utc_timestamp("2026-11-25T21:45:00.123Z").is_ok());
    assert!(validate_utc_timestamp("2026-11-25 21:45:00").is_err());
    assert!(validate_utc_timestamp("2026-11-25T21:45:00-03:00").is_err());
    assert!(validate_utc_timestamp("not-a-date").is_err());
}

#[test]
fn canonical_game_key_prefers_normalized_serial_or_clean_title_slug() {
    assert_eq!(
        normalize_canonical_key("Demon's Souls (USA)", Some("BLUS-30443")).unwrap(),
        "serial:BLUS30443"
    );
    assert_eq!(
        normalize_canonical_key("Shadow of the Colossus!", None).unwrap(),
        "title:shadow-of-the-colossus"
    );
    assert!(normalize_canonical_key("   ", None).is_err());
}

#[test]
fn trophy_origin_is_strictly_immutable() {
    assert!(ensure_immutable_origin(TrophyOrigin::Local, TrophyOrigin::Local).is_ok());
    assert!(ensure_immutable_origin(TrophyOrigin::Emulator, TrophyOrigin::Emulator).is_ok());
    assert!(matches!(
        ensure_immutable_origin(TrophyOrigin::Local, TrophyOrigin::Official),
        Err(DomainError::ImmutableTrophyOrigin { .. })
    ));
    assert!(matches!(
        ensure_immutable_origin(TrophyOrigin::Emulator, TrophyOrigin::Official),
        Err(DomainError::ImmutableTrophyOrigin { .. })
    ));
}

use std::path::PathBuf;

use super::console::{
    CapabilityState, ConsoleCapabilities, FeatureAvailability, FeatureCapability, FeatureKey,
    VerificationLevel,
};
use super::console_profile::{
    AvailabilityStatus, DataOrigin, EmulatorStatus, ProfileField, SyncTrigger, TrophySyncReadiness,
};
use super::emulator::{LaunchCommand, SystemBootReadiness};
use super::environment::{
    ConsoleEnvironmentMode, RuntimeVerificationFeature, RuntimeVerificationRecord,
    RuntimeVerificationStatus,
};
use super::error::DomainError;
use super::player::{normalize_canonical_key, validate_utc_timestamp};
use super::scanner::ScanProgressEvent;
use super::session::{
    compute_elapsed_seconds_utc, ActiveSessionPhase, LaunchTarget,
    EXIT_STATUS_RECOVERED_AFTER_CRASH,
};
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

#[test]
fn derives_console_environment_mode_honestly_from_multi_capabilities() {
    let ps3_caps = ConsoleCapabilities {
        supported_states: vec![
            CapabilityState::FullSystemBoot,
            CapabilityState::OriginalDashboard,
            CapabilityState::DirectGameOnly,
        ],
        features: vec![],
    };
    assert_eq!(
        ConsoleEnvironmentMode::from_capabilities(&ps3_caps),
        ConsoleEnvironmentMode::AuthenticSystemCapable
    );

    let xbox360_caps = ConsoleCapabilities {
        supported_states: vec![
            CapabilityState::DirectGameOnly,
            CapabilityState::PartialDashboard,
        ],
        features: vec![],
    };
    assert_eq!(
        ConsoleEnvironmentMode::from_capabilities(&xbox360_caps),
        ConsoleEnvironmentMode::IntegratedAlternativeOnly
    );

    let ps4_caps = ConsoleCapabilities {
        supported_states: vec![
            CapabilityState::Experimental,
            CapabilityState::DirectGameOnly,
        ],
        features: vec![],
    };
    assert_eq!(
        ConsoleEnvironmentMode::from_capabilities(&ps4_caps),
        ConsoleEnvironmentMode::IntegratedAlternativeOnly
    );
}

#[test]
fn runtime_verification_tracks_all_seven_independent_features_and_scope_keys() {
    let all = RuntimeVerificationFeature::all();
    assert_eq!(all.len(), 7);
    for feat in all {
        assert_eq!(
            RuntimeVerificationFeature::parse(feat.as_str()).expect("roundtrip feature"),
            feat
        );
    }
    assert!(RuntimeVerificationFeature::parse("universal_compatibility").is_err());

    assert_eq!(
        RuntimeVerificationStatus::parse("runtime_confirmed").unwrap(),
        RuntimeVerificationStatus::RuntimeConfirmed
    );
    assert!(RuntimeVerificationStatus::parse("fake_status").is_err());

    assert_eq!(
        RuntimeVerificationRecord::compute_scope_key(None).unwrap(),
        "console"
    );
    assert_eq!(
        RuntimeVerificationRecord::compute_scope_key(Some("game-42")).unwrap(),
        "game-42"
    );
    assert!(RuntimeVerificationRecord::compute_scope_key(Some("   ")).is_err());
}

#[test]
fn scan_progress_event_serializes_with_tauri_channel_tagged_format() {
    let evt = ScanProgressEvent::GameDiscovered {
        title: "Demon's Souls".to_string(),
        serial_code: Some("BLUS30443".to_string()),
        file_path: r"D:\PS3\DemonsSouls\PS3_GAME\USRDIR\EBOOT.BIN".to_string(),
    };
    let json = serde_json::to_value(&evt).expect("serialize ScanProgressEvent");
    assert_eq!(json["event"], "gameDiscovered");
    assert_eq!(json["data"]["title"], "Demon's Souls");
    assert_eq!(json["data"]["serialCode"], "BLUS30443");
}

#[test]
fn session_elapsed_calculation_uses_strict_utc_bounds_and_never_negative() {
    assert_eq!(EXIT_STATUS_RECOVERED_AFTER_CRASH, "recovered_after_crash");
    assert_eq!(
        compute_elapsed_seconds_utc("2026-11-25T20:00:00Z", "2026-11-25T20:15:30Z").unwrap(),
        930
    );
    // Clamps clock skew to 0 instead of returning negative duration
    assert_eq!(
        compute_elapsed_seconds_utc("2026-11-25T20:15:30Z", "2026-11-25T20:00:00Z").unwrap(),
        0
    );
    // Rejects non-UTC timestamps
    assert!(compute_elapsed_seconds_utc("2026-11-25 20:00:00", "2026-11-25T20:05:00Z").is_err());
}

#[test]
fn console_profile_fields_enforce_explicit_data_origin_and_enum_roundtrips() {
    let native_fw = ProfileField::native("4.91".to_string());
    assert_eq!(native_fw.origin, DataOrigin::NativePlatform);
    assert_eq!(native_fw.value.as_deref(), Some("4.91"));

    let hub_hours = ProfileField::hub(3600i64);
    assert_eq!(hub_hours.origin, DataOrigin::HubRecorded);
    assert_eq!(hub_hours.value, Some(3600));

    let missing_psn: ProfileField<String> =
        ProfileField::unavailable("PSN oficial não é suportada.");
    assert_eq!(missing_psn.origin, DataOrigin::Unavailable);
    assert!(missing_psn.value.is_none());
    assert!(missing_psn.notes.is_some());

    let serialized = serde_json::to_value(&native_fw).expect("serialize ProfileField");
    assert_eq!(serialized["origin"], "native_platform");
    assert_eq!(serialized["value"], "4.91");

    assert_eq!(
        AvailabilityStatus::parse("missing_on_disk").unwrap(),
        AvailabilityStatus::MissingOnDisk
    );
    assert_eq!(
        EmulatorStatus::parse("unconfigured").unwrap(),
        EmulatorStatus::Unconfigured
    );
    assert_eq!(
        TrophySyncReadiness::parse("phase4_planned").unwrap(),
        TrophySyncReadiness::Phase4Planned
    );
    assert_eq!(
        SyncTrigger::parse("session_ended").unwrap(),
        SyncTrigger::SessionEnded
    );
    assert!(AvailabilityStatus::parse("deleted").is_err());
    assert!(TrophySyncReadiness::parse("faked").is_err());
}

#[test]
fn live_session_state_machine_enforces_single_active_session_transitions() {
    assert!(ActiveSessionPhase::Idle.can_transition_to(ActiveSessionPhase::Launching));
    assert!(!ActiveSessionPhase::Idle.can_transition_to(ActiveSessionPhase::Running));
    assert!(ActiveSessionPhase::Launching.can_transition_to(ActiveSessionPhase::Running));
    assert!(ActiveSessionPhase::Launching.can_transition_to(ActiveSessionPhase::Idle));
    assert!(!ActiveSessionPhase::Running.can_transition_to(ActiveSessionPhase::Launching));
    assert!(ActiveSessionPhase::Running.can_transition_to(ActiveSessionPhase::Exiting));
    assert!(ActiveSessionPhase::Exiting.can_transition_to(ActiveSessionPhase::Idle));

    let sys_target = LaunchTarget::SystemBoot {
        console_slug: "ps3".to_string(),
    };
    let sys_json = serde_json::to_value(&sys_target).expect("serialize SystemBoot");
    assert_eq!(sys_json["kind"], "system_boot");
    assert_eq!(sys_json["consoleSlug"], "ps3");

    let game_target = LaunchTarget::DirectGameBoot {
        console_slug: "ps3".to_string(),
        game_id: "game-1".to_string(),
        game_version_id: Some("gv-1".to_string()),
    };
    let game_json = serde_json::to_value(&game_target).expect("serialize DirectGameBoot");
    assert_eq!(game_json["kind"], "direct_game_boot");
    assert_eq!(game_json["gameId"], "game-1");
}

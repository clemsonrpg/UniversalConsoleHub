use std::fs;
use std::path::PathBuf;

use tempfile::TempDir;
use universal_console_hub_lib::adapters::profiles::{
    profile_provider_for_console, Ps3ProfileProvider,
};
use universal_console_hub_lib::domain::console_profile::{
    ConsoleProfileProvider, EmulatorStatus, TrophySyncReadiness,
};

const MINIMAL_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

#[test]
fn reads_real_synthetic_rpcs3_tree_with_unicode_firmware_multiple_users_saves_and_artwork() {
    let tmp = TempDir::new().expect("tempdir");
    let rpcs3_root = tmp.path().join("Área de Trabalho").join("RPCS3 Portable");
    fs::create_dir_all(rpcs3_root.join("dev_flash").join("vsh").join("etc")).unwrap();
    fs::create_dir_all(rpcs3_root.join("dev_flash").join("vsh").join("module")).unwrap();
    fs::create_dir_all(
        rpcs3_root
            .join("dev_hdd0")
            .join("home")
            .join("00000001")
            .join("savedata")
            .join("BLUS30443-SAVE01"),
    )
    .unwrap();
    fs::create_dir_all(
        rpcs3_root
            .join("dev_hdd0")
            .join("home")
            .join("00000001")
            .join("savedata")
            .join("BLUS30443-SAVE02"),
    )
    .unwrap();
    fs::create_dir_all(
        rpcs3_root
            .join("dev_hdd0")
            .join("home")
            .join("00000001")
            .join("trophy")
            .join("NPWR00637_00"),
    )
    .unwrap();
    fs::create_dir_all(
        rpcs3_root
            .join("dev_hdd0")
            .join("home")
            .join("00000002")
            .join("savedata")
            .join("BCUS98114-SAVE"),
    )
    .unwrap();
    let hdd_game_dir = rpcs3_root.join("dev_hdd0").join("game");
    fs::create_dir_all(&hdd_game_dir).unwrap();

    let exe_path = rpcs3_root.join("rpcs3.exe");
    fs::write(&exe_path, b"MZ-synthetic-rpcs3").unwrap();
    fs::write(
        rpcs3_root.join("RPCS3.log"),
        "RPCS3 v0.0.34-17105-abcdef01 Alpha | master\nSYS: booted\n",
    )
    .unwrap();
    fs::write(
        rpcs3_root
            .join("dev_flash")
            .join("vsh")
            .join("etc")
            .join("version.txt"),
        "release:04.9100:\nbuild:94095:\n",
    )
    .unwrap();
    fs::write(
        rpcs3_root
            .join("dev_flash")
            .join("vsh")
            .join("module")
            .join("vsh.self"),
        b"SCE-synthetic-vsh",
    )
    .unwrap();
    fs::write(
        rpcs3_root
            .join("dev_hdd0")
            .join("home")
            .join("00000001")
            .join("localusername"),
        "ClemsonPS3\n",
    )
    .unwrap();
    fs::write(
        rpcs3_root
            .join("dev_hdd0")
            .join("home")
            .join("00000002")
            .join("localusername"),
        "PlayerDois",
    )
    .unwrap();

    let provider = Ps3ProfileProvider::new();
    let snap = provider.collect_native_snapshot(Some(&exe_path));

    assert_eq!(snap.console_slug, "ps3");
    assert_eq!(snap.emulator_slug.as_deref(), Some("rpcs3"));
    assert_eq!(snap.emulator_status, EmulatorStatus::Available);
    assert_eq!(
        snap.detected_emulator_version.as_deref(),
        Some("RPCS3 v0.0.34-17105-abcdef01 Alpha | master")
    );
    assert_eq!(snap.detected_firmware_version.as_deref(), Some("4.91"));
    assert!(snap.vsh_or_system_boot_ready);
    assert_eq!(snap.native_save_count, 3);
    assert!(snap.native_trophy_dir_present);
    assert_eq!(snap.trophy_sync_readiness, TrophySyncReadiness::Ready);
    assert_eq!(snap.local_users.len(), 2);
    assert_eq!(snap.local_users[0].native_user_key, "00000001");
    assert_eq!(snap.local_users[0].display_name, "ClemsonPS3");
    assert_eq!(snap.local_users[1].native_user_key, "00000002");
    assert_eq!(snap.local_users[1].display_name, "PlayerDois");
    assert_eq!(snap.native_game_scan_dirs.len(), 1);

    // Artwork discovery for PS3 game folder (PS3_GAME/USRDIR/EBOOT.BIN -> PS3_GAME/ICON0.PNG + PIC1.PNG)
    let ps3_game_dir = tmp
        .path()
        .join("Jogos PS3")
        .join("Demon's Souls")
        .join("PS3_GAME");
    fs::create_dir_all(ps3_game_dir.join("USRDIR")).unwrap();
    let eboot = ps3_game_dir.join("USRDIR").join("EBOOT.BIN");
    let icon0 = ps3_game_dir.join("ICON0.PNG");
    let pic1 = ps3_game_dir.join("PIC1.PNG");
    fs::write(&eboot, b"EBOOT").unwrap();
    fs::write(&icon0, MINIMAL_PNG).unwrap();
    fs::write(&pic1, MINIMAL_PNG).unwrap();

    let art = provider
        .find_native_artwork_for_game(&eboot)
        .expect("should detect native ICON0.PNG and PIC1.PNG");
    assert_eq!(art.icon_path.as_deref(), Some(icon0.as_path()));
    assert_eq!(art.background_path.as_deref(), Some(pic1.as_path()));
}

#[test]
fn never_invents_users_and_reports_unconfigured_or_missing_emulator_honestly() {
    let provider = Ps3ProfileProvider::new();

    // 1. Unconfigured
    let uncfg = provider.collect_native_snapshot(None);
    assert_eq!(uncfg.emulator_status, EmulatorStatus::Unconfigured);
    assert!(uncfg.local_users.is_empty());
    assert_eq!(uncfg.native_save_count, 0);
    assert!(!uncfg.native_trophy_dir_present);

    // 2. Missing executable on disk (Adjustment B)
    let missing_path = PathBuf::from(r"Z:\NonExistent\RPCS3\rpcs3.exe");
    let missing = provider.collect_native_snapshot(Some(&missing_path));
    assert_eq!(missing.emulator_status, EmulatorStatus::MissingOnDisk);
    assert!(missing.local_users.is_empty());
    assert!(!missing.diagnostics.is_empty());

    // 3. Available exe but NO dev_hdd0/home -> MUST return 0 users (never invent)
    let tmp = TempDir::new().expect("tempdir");
    let exe = tmp.path().join("rpcs3.exe");
    fs::write(&exe, b"MZ").unwrap();
    let bare = provider.collect_native_snapshot(Some(&exe));
    assert_eq!(bare.emulator_status, EmulatorStatus::Available);
    assert!(
        bare.local_users.is_empty(),
        "must never invent PS3 local users when dev_hdd0/home is absent"
    );

    // 4. Non-PS3 platform (PS2) reports honest unavailable native trophies & zero invented users
    let ps2_provider = profile_provider_for_console("ps2");
    let ps2_snap = ps2_provider.collect_native_snapshot(None);
    assert_eq!(ps2_snap.console_slug, "ps2");
    assert!(ps2_snap.local_users.is_empty());
    assert_eq!(
        ps2_snap.trophy_sync_readiness,
        TrophySyncReadiness::Unavailable
    );
}

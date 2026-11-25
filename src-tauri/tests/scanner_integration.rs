use std::fs;
use std::path::Path;

use tempfile::TempDir;
use universal_console_hub_lib::adapters::discovery::discovery_for_console;
use universal_console_hub_lib::adapters::discovery::param_sfo::{
    build_synthetic_param_sfo, infer_ps3_region, parse_param_sfo,
};
use universal_console_hub_lib::adapters::discovery::ps3_scanner::Ps3GameDiscovery;
use universal_console_hub_lib::domain::scanner::{GameDiscoveryPort, ScanProgressEvent};

#[test]
fn parses_valid_synthetic_param_sfo_and_never_panics_on_malformed_buffers() {
    let valid_bytes = build_synthetic_param_sfo(&[
        ("APP_VER", "01.04"),
        ("CATEGORY", "DG"),
        ("TITLE", "Demon's Souls"),
        ("TITLE_ID", "BLUS30443"),
    ]);

    let meta = parse_param_sfo(&valid_bytes).expect("parse valid synthetic PARAM.SFO");
    assert_eq!(meta.title.as_deref(), Some("Demon's Souls"));
    assert_eq!(meta.title_id.as_deref(), Some("BLUS30443"));
    assert_eq!(meta.app_ver.as_deref(), Some("01.04"));
    assert_eq!(meta.category.as_deref(), Some("DG"));
    assert_eq!(infer_ps3_region("BLUS30443"), "USA");
    assert_eq!(infer_ps3_region("BLES00932"), "EUR");
    assert_eq!(infer_ps3_region("BCJS30022"), "JPN");
    assert_eq!(infer_ps3_region("BLAS50142"), "ASIA");
    assert_eq!(infer_ps3_region("CUSTOM999"), "UNKNOWN");

    // Truncated / malformed / hostile buffers MUST return Err and NEVER panic
    assert!(parse_param_sfo(&[]).is_err());
    assert!(parse_param_sfo(b"NOT_A_PSF_HEADER_1234567890").is_err());
    assert!(parse_param_sfo(&valid_bytes[..12]).is_err());
    assert!(parse_param_sfo(&valid_bytes[..24]).is_err());

    // Hostile header claiming 0xFFFF_FFFF entries
    let mut hostile = vec![0u8; 32];
    hostile[0..4].copy_from_slice(b"\0PSF");
    hostile[16..20].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(parse_param_sfo(&hostile).is_err());

    // Out-of-bounds key/data table offsets
    let mut oob = valid_bytes.clone();
    oob[8..12].copy_from_slice(&999_999u32.to_le_bytes());
    assert!(parse_param_sfo(&oob).is_err());
}

fn create_synthetic_ps3_disc_folder(
    base: &Path,
    folder_name: &str,
    title: &str,
    title_id: &str,
    app_ver: &str,
) {
    let ps3_game = base.join(folder_name).join("PS3_GAME");
    let usrdir = ps3_game.join("USRDIR");
    fs::create_dir_all(&usrdir).expect("create synthetic PS3_GAME/USRDIR");
    let sfo = build_synthetic_param_sfo(&[
        ("APP_VER", app_ver),
        ("CATEGORY", "DG"),
        ("TITLE", title),
        ("TITLE_ID", title_id),
    ]);
    fs::write(ps3_game.join("PARAM.SFO"), sfo).expect("write PARAM.SFO");
    fs::write(usrdir.join("EBOOT.BIN"), b"SYNTHETIC_EBOOT_STUB").expect("write EBOOT.BIN");
}

#[test]
fn ps3_scanner_discovers_unicode_titles_ignores_corrupt_sfo_and_is_idempotent() {
    let tmp = TempDir::new().expect("tempdir");
    let root = tmp.path().join("Área de Trabalho").join("Jogos PS3");
    fs::create_dir_all(&root).unwrap();

    create_synthetic_ps3_disc_folder(
        &root,
        "Demon's Souls — Edição USA",
        "Demon's Souls",
        "BLUS30443",
        "01.00",
    );

    // Also create a PS3 HDD-style title (dev_hdd0/game/NPUA80472)
    let hdd_game = root.join("dev_hdd0").join("game").join("NPUA80472");
    fs::create_dir_all(hdd_game.join("USRDIR")).unwrap();
    fs::write(
        hdd_game.join("PARAM.SFO"),
        build_synthetic_param_sfo(&[
            ("APP_VER", "01.02"),
            ("CATEGORY", "HG"),
            ("TITLE", "Journey"),
            ("TITLE_ID", "NPUA80472"),
        ]),
    )
    .unwrap();
    fs::write(
        hdd_game.join("USRDIR").join("EBOOT.BIN"),
        b"SYNTHETIC_HDD_EBOOT",
    )
    .unwrap();

    // Create a broken entry with corrupted PARAM.SFO (must produce diagnostic, NOT crash or import garbage)
    let broken = root.join("Corrupted Title").join("PS3_GAME");
    fs::create_dir_all(broken.join("USRDIR")).unwrap();
    fs::write(broken.join("PARAM.SFO"), b"CORRUPT_HEADER").unwrap();
    fs::write(broken.join("USRDIR").join("EBOOT.BIN"), b"STUB").unwrap();

    // Create unrelated file
    fs::write(root.join("readme.txt"), b"ignore me").unwrap();

    let scanner = Ps3GameDiscovery;
    let mut events = Vec::new();
    let batch1 = scanner
        .discover_in_directory(&root, true, &mut |evt| events.push(evt))
        .expect("run ps3 discovery");

    assert_eq!(batch1.candidates.len(), 2, "expected 2 valid PS3 titles");
    assert_eq!(
        batch1.diagnostics.len(),
        1,
        "expected 1 diagnostic for corrupted PARAM.SFO"
    );
    assert!(events
        .iter()
        .any(|e| matches!(e, ScanProgressEvent::GameDiscovered { .. })));

    let demons = batch1
        .candidates
        .iter()
        .find(|c| c.serial_code.as_deref() == Some("BLUS30443"))
        .expect("Demon's Souls candidate");
    assert_eq!(demons.title, "Demon's Souls");
    assert_eq!(demons.region, "USA");
    assert_eq!(demons.version_label, "01.00");
    assert_eq!(demons.metadata_source, "ps3_param_sfo");
    assert!(
        !demons.file_path.to_string_lossy().starts_with(r"\\?\"),
        "discovered path must not leak Windows verbatim prefix"
    );

    let journey = batch1
        .candidates
        .iter()
        .find(|c| c.serial_code.as_deref() == Some("NPUA80472"))
        .expect("Journey HDD candidate");
    assert_eq!(journey.title, "Journey");
    assert_eq!(journey.region, "USA");

    // Second run must yield identical results deterministically
    let batch2 = scanner
        .discover_in_directory(&root, true, &mut |_| {})
        .expect("re-scan idempotently");
    assert_eq!(batch1.candidates, batch2.candidates);
}

#[test]
fn platform_extension_discovery_handles_ps2_wii_xbox_xbox360_and_ps4_honestly() {
    let tmp = TempDir::new().expect("tempdir");

    // PS2
    let ps2_dir = tmp.path().join("PS2");
    fs::create_dir_all(&ps2_dir).unwrap();
    fs::write(ps2_dir.join("Shadow of the Colossus.chd"), b"stub").unwrap();
    fs::write(ps2_dir.join("notes.txt"), b"ignore").unwrap();
    let ps2_scanner = discovery_for_console("ps2").expect("ps2 scanner");
    let ps2_batch = ps2_scanner
        .discover_in_directory(&ps2_dir, true, &mut |_| {})
        .expect("scan ps2");
    assert_eq!(ps2_batch.candidates.len(), 1);
    assert_eq!(ps2_batch.candidates[0].title, "Shadow of the Colossus");
    assert_eq!(
        ps2_batch.candidates[0].metadata_source,
        "filesystem_filename_scan"
    );

    // PS4 (matches eboot.bin case-insensitively and derives title from parent folder)
    let ps4_dir = tmp.path().join("PS4").join("Bloodborne_Test");
    fs::create_dir_all(&ps4_dir).unwrap();
    fs::write(ps4_dir.join("eboot.bin"), b"stub").unwrap();
    fs::write(ps4_dir.join("other.bin"), b"ignore").unwrap();
    let ps4_scanner = discovery_for_console("ps4").expect("ps4 scanner");
    let ps4_batch = ps4_scanner
        .discover_in_directory(&tmp.path().join("PS4"), true, &mut |_| {})
        .expect("scan ps4");
    assert_eq!(ps4_batch.candidates.len(), 1);
    assert_eq!(ps4_batch.candidates[0].title, "Bloodborne_Test");
    assert_eq!(ps4_batch.candidates[0].console_slug, "ps4");

    assert!(discovery_for_console("wii").is_some());
    assert!(discovery_for_console("xbox").is_some());
    assert!(discovery_for_console("xbox360").is_some());
    assert!(discovery_for_console("unknown-console").is_none());
}

use std::fs;

use tempfile::TempDir;
use universal_console_hub_lib::adapters::trophies::rpcs3_trophy_reader::{
    parse_tropconf_sfm, parse_tropusr_dat, Rpcs3TrophyReader,
};
use universal_console_hub_lib::domain::trophy::{TrophyCategory, TrophyReaderPort};

const PS3_TO_UNIX_EPOCH_SECS: u64 = 62_135_596_800;

const MINIMAL_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

fn build_synthetic_tropusr_dat(entries: &[(u32, bool, u64)]) -> Vec<u8> {
    let header_len: usize = 48;
    let dir_entry_len: usize = 32;
    let table_offset: u64 = (header_len + dir_entry_len) as u64;
    let entry_size: u32 = 112;
    let total_size: u64 = table_offset + (entries.len() as u64 * u64::from(entry_size));

    let mut out = Vec::with_capacity(total_size as usize);
    // Header (48 bytes)
    out.extend_from_slice(&[0x81, 0x8F, 0x54, 0xAD]);
    out.extend_from_slice(&0x0001_0000u32.to_be_bytes());
    out.extend_from_slice(&total_size.to_be_bytes());
    out.extend_from_slice(&1u32.to_be_bytes()); // table_count = 1
    out.extend_from_slice(&0u32.to_be_bytes());
    out.extend_from_slice(&[0u8; 24]);

    // Table directory entry for Type 6 (32 bytes)
    out.extend_from_slice(&6u32.to_be_bytes()); // table_type = 6
    out.extend_from_slice(&entry_size.to_be_bytes());
    out.extend_from_slice(&1u32.to_be_bytes()); // seq
    out.extend_from_slice(&(entries.len() as u32).to_be_bytes());
    out.extend_from_slice(&table_offset.to_be_bytes());
    out.extend_from_slice(&0u64.to_be_bytes());

    // Type-6 entries (112 bytes each)
    for (idx, &(trophy_id, unlocked, unix_secs)) in entries.iter().enumerate() {
        let mut entry = vec![0u8; entry_size as usize];
        entry[0..4].copy_from_slice(&6u32.to_be_bytes());
        entry[4..8].copy_from_slice(&0x60u32.to_be_bytes());
        entry[8..12].copy_from_slice(&(idx as u32).to_be_bytes());
        entry[16..20].copy_from_slice(&trophy_id.to_be_bytes());
        entry[20..24].copy_from_slice(&(u32::from(unlocked)).to_be_bytes());
        if unlocked && unix_secs > 0 {
            let ps3_micros = (unix_secs + PS3_TO_UNIX_EPOCH_SECS) * 1_000_000;
            entry[32..40].copy_from_slice(&ps3_micros.to_be_bytes());
            entry[40..48].copy_from_slice(&ps3_micros.to_be_bytes());
        }
        out.extend_from_slice(&entry);
    }
    out
}

#[test]
fn parses_synthetic_rpcs3_tropconf_tropusr_icons_and_unicode_tree_read_only() {
    let tmp = TempDir::new().expect("tempdir");
    let rpcs3_root = tmp.path().join("Área de Trabalho").join("RPCS3");
    let trophy_dir = rpcs3_root
        .join("dev_hdd0")
        .join("home")
        .join("00000001")
        .join("trophy")
        .join("NPWR00637_00");
    fs::create_dir_all(&trophy_dir).unwrap();

    let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<trophyconf version="1.1">
  <npcommid>NPWR00637_00</npcommid>
  <title-name>Demon&apos;s Souls</title-name>
  <title-detail>Trophies for Demon&apos;s Souls &amp; Nexus</title-detail>
  <trophy id="000" hidden="no" ttype="P">
    <name>Toughest Soul Trophy</name>
    <detail>All Trophies Obtained</detail>
  </trophy>
  <trophy id="001" hidden="yes" ttype="B">
    <name>Phalanx&apos;s Trophy</name>
    <detail>Slayer of Demon &quot;Phalanx&quot;</detail>
  </trophy>
  <trophy id="002" hidden="no" ttype="G">
    <name>False King&apos;s Trophy</name>
    <detail>Slayer of Demon False King</detail>
  </trophy>
</trophyconf>"#;

    fs::write(trophy_dir.join("TROPCONF.SFM"), xml).unwrap();
    // Unlock trophy 001 at 2026-11-25T20:00:00Z (unix = 1795636800), keep 000 & 002 locked
    let tropusr =
        build_synthetic_tropusr_dat(&[(0, false, 0), (1, true, 1_795_636_800), (2, false, 0)]);
    fs::write(trophy_dir.join("TROPUSR.DAT"), tropusr).unwrap();
    fs::write(trophy_dir.join("ICON0.PNG"), MINIMAL_PNG).unwrap();
    fs::write(trophy_dir.join("TROP000.PNG"), MINIMAL_PNG).unwrap();
    fs::write(trophy_dir.join("TROP001.PNG"), MINIMAL_PNG).unwrap();

    let reader = Rpcs3TrophyReader::new();
    let batch = reader.read_trophy_sets(&rpcs3_root);

    assert!(
        batch.diagnostics.is_empty(),
        "diagnostics: {:?}",
        batch.diagnostics
    );
    assert_eq!(batch.sets.len(), 1);
    let set = &batch.sets[0];
    assert_eq!(set.console_slug, "ps3");
    assert_eq!(set.native_user_key, "00000001");
    assert_eq!(set.external_set_id, "NPWR00637_00");
    assert_eq!(set.title, "Demon's Souls");
    assert_eq!(set.detail, "Trophies for Demon's Souls & Nexus");
    assert!(set.icon_path.is_some());
    assert!(set.supports_platinum());
    assert_eq!(set.trophies.len(), 3);

    assert_eq!(set.trophies[0].external_trophy_id, "000");
    assert_eq!(set.trophies[0].category, TrophyCategory::Platinum);
    assert!(!set.trophies[0].is_hidden);
    assert!(set.trophies[0].icon_path.is_some());

    assert_eq!(set.trophies[1].external_trophy_id, "001");
    assert_eq!(set.trophies[1].category, TrophyCategory::Bronze);
    assert!(set.trophies[1].is_hidden);
    assert_eq!(set.trophies[1].name, "Phalanx's Trophy");
    assert_eq!(set.trophies[1].description, "Slayer of Demon \"Phalanx\"");

    assert_eq!(set.unlocks.len(), 1);
    assert_eq!(set.unlocks[0].external_trophy_id, "001");
    assert_eq!(set.unlocks[0].unlocked_at_utc, "2026-11-25T20:00:00Z");
    assert_eq!(set.unlocks[0].native_user_key, "00000001");
}

#[test]
fn rejects_corrupted_or_truncated_tropconf_and_tropusr_without_panicking() {
    assert!(parse_tropconf_sfm(b"").is_err());
    assert!(parse_tropconf_sfm(b"<broken-xml").is_err());
    assert!(parse_tropusr_dat(&[], "00000001").is_err());
    assert!(parse_tropusr_dat(b"BADMAGIC1234567890", "00000001").is_err());
    // Truncated header with valid magic
    let mut trunc = vec![0x81, 0x8F, 0x54, 0xAD];
    trunc.extend_from_slice(&[0u8; 20]);
    assert!(parse_tropusr_dat(&trunc, "00000001").is_err());

    // Directory with corrupted files logs diagnostics and returns cleanly without panic
    let tmp = TempDir::new().expect("tempdir");
    let bad_dir = tmp
        .path()
        .join("dev_hdd0")
        .join("home")
        .join("00000001")
        .join("trophy")
        .join("NPWR_BAD");
    fs::create_dir_all(&bad_dir).unwrap();
    fs::write(bad_dir.join("TROPCONF.SFM"), b"not xml at all").unwrap();
    fs::write(bad_dir.join("TROPUSR.DAT"), b"corrupted").unwrap();

    let reader = Rpcs3TrophyReader::new();
    let batch = reader.read_trophy_sets(tmp.path());
    assert!(batch.sets.is_empty());
    assert!(!batch.diagnostics.is_empty());
}

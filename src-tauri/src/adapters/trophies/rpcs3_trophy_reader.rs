use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, SecondsFormat};

use crate::domain::error::DomainError;
use crate::domain::player::now_utc_rfc3339;
use crate::domain::trophy::{
    ParsedTrophyDef, ParsedTrophySet, ParsedTrophyUnlock, TrophyCategory, TrophyReadBatch,
    TrophyReaderPort,
};

const MAX_TROPHY_FILE_BYTES: usize = 8 * 1024 * 1024;
const TROPUSR_MAGIC: &[u8; 4] = &[0x81, 0x8F, 0x54, 0xAD];
const TROPUSR_HEADER_LEN: usize = 48;
const TROPUSR_DIR_ENTRY_LEN: usize = 32;
const TROPUSR_TABLE_TYPE_UNLOCK: u32 = 6;
const PS3_TO_UNIX_EPOCH_SECS: u64 = 62_135_596_800;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TropConfData {
    pub npcommid: Option<String>,
    pub title: String,
    pub detail: String,
    pub trophies: Vec<ParsedTrophyDef>,
}

fn decode_xml_entities(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(amp_idx) = rest.find('&') {
        out.push_str(&rest[..amp_idx]);
        let after_amp = &rest[amp_idx + 1..];
        if let Some(semi_idx) = after_amp.find(';') {
            let entity = &after_amp[..semi_idx];
            match entity {
                "amp" => out.push('&'),
                "lt" => out.push('<'),
                "gt" => out.push('>'),
                "quot" => out.push('"'),
                "apos" => out.push('\''),
                _ if entity.starts_with("#x") || entity.starts_with("#X") => {
                    if let Ok(code) = u32::from_str_radix(&entity[2..], 16) {
                        if let Some(ch) = char::from_u32(code) {
                            out.push(ch);
                        }
                    }
                }
                _ if entity.starts_with('#') => {
                    if let Ok(code) = entity[1..].parse::<u32>() {
                        if let Some(ch) = char::from_u32(code) {
                            out.push(ch);
                        }
                    }
                }
                _ => {
                    out.push('&');
                    out.push_str(entity);
                    out.push(';');
                }
            }
            rest = &after_amp[semi_idx + 1..];
        } else {
            out.push_str(&rest[amp_idx..]);
            return out;
        }
    }
    out.push_str(rest);
    out
}

fn extract_tag_text(xml: &str, tag: &str) -> Option<String> {
    let open_prefix = format!("<{tag}");
    let start_pos = xml.find(&open_prefix)?;
    let after_open = &xml[start_pos + open_prefix.len()..];
    let gt_pos = after_open.find('>')?;
    let body_start = &after_open[gt_pos + 1..];
    let close_tag = format!("</{tag}>");
    let end_pos = body_start.find(&close_tag)?;
    Some(decode_xml_entities(body_start[..end_pos].trim()))
}

fn extract_xml_attr(attrs: &str, name: &str) -> Option<String> {
    for quote in ['"', '\''] {
        let needle = format!("{name}={quote}");
        if let Some(pos) = attrs.find(&needle) {
            let value_start = &attrs[pos + needle.len()..];
            if let Some(end) = value_start.find(quote) {
                return Some(decode_xml_entities(value_start[..end].trim()));
            }
        }
    }
    None
}

/// Pure, bounds-checked parser for Sony PS3 / RPCS3 `TROPCONF.SFM` (and `TROP.SFM`).
pub fn parse_tropconf_sfm(bytes: &[u8]) -> Result<TropConfData, DomainError> {
    if bytes.is_empty() || bytes.len() > MAX_TROPHY_FILE_BYTES {
        return Err(DomainError::Validation(format!(
            "invalid TROPCONF.SFM size ({} bytes)",
            bytes.len()
        )));
    }

    let raw_str = String::from_utf8_lossy(bytes);
    let cleaned = raw_str
        .trim_start_matches('\u{feff}')
        .trim_matches(|c: char| c == '\0' || c.is_whitespace());

    if !cleaned.contains("<trophyconf") || !cleaned.contains("</trophyconf>") {
        return Err(DomainError::Validation(
            "TROPCONF.SFM does not contain a valid <trophyconf> XML document".to_string(),
        ));
    }

    let npcommid = extract_tag_text(cleaned, "npcommid").filter(|s| !s.is_empty());
    let title = extract_tag_text(cleaned, "title-name")
        .filter(|s| !s.is_empty())
        .or_else(|| npcommid.clone())
        .ok_or_else(|| {
            DomainError::Validation("TROPCONF.SFM missing <title-name> and <npcommid>".to_string())
        })?;
    let detail = extract_tag_text(cleaned, "title-detail").unwrap_or_default();

    let mut trophies = Vec::new();
    let mut cursor = cleaned;
    while let Some(start_idx) = cursor.find("<trophy ") {
        let after_start = &cursor[start_idx + "<trophy ".len()..];
        let Some(tag_end) = after_start.find('>') else {
            break;
        };
        let attrs = &after_start[..tag_end];
        let body_and_rest = &after_start[tag_end + 1..];
        let Some(close_idx) = body_and_rest.find("</trophy>") else {
            break;
        };
        let body = &body_and_rest[..close_idx];

        let raw_id = extract_xml_attr(attrs, "id").unwrap_or_default();
        let ext_id = if let Ok(num) = raw_id.parse::<u32>() {
            format!("{num:03}")
        } else {
            raw_id
        };
        let ttype = extract_xml_attr(attrs, "ttype").unwrap_or_else(|| "B".to_string());
        let hidden = extract_xml_attr(attrs, "hidden").is_some_and(|v| {
            v.eq_ignore_ascii_case("yes") || v == "1" || v.eq_ignore_ascii_case("true")
        });

        if !ext_id.is_empty() {
            let category = TrophyCategory::from_ps3_ttype(&ttype).unwrap_or(TrophyCategory::Bronze);
            let name = extract_tag_text(body, "name")
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| format!("Trophy {ext_id}"));
            let description = extract_tag_text(body, "detail").unwrap_or_default();

            trophies.push(ParsedTrophyDef {
                external_trophy_id: ext_id,
                name,
                description,
                category,
                is_hidden: hidden,
                icon_path: None,
            });
        }

        cursor = &body_and_rest[close_idx + "</trophy>".len()..];
    }

    if trophies.is_empty() {
        return Err(DomainError::Validation(
            "TROPCONF.SFM contained zero <trophy> definitions".to_string(),
        ));
    }

    Ok(TropConfData {
        npcommid,
        title,
        detail,
        trophies,
    })
}

fn read_be_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let end = offset.checked_add(4)?;
    let slice: [u8; 4] = bytes.get(offset..end)?.try_into().ok()?;
    Some(u32::from_be_bytes(slice))
}

fn read_be_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    let end = offset.checked_add(8)?;
    let slice: [u8; 8] = bytes.get(offset..end)?.try_into().ok()?;
    Some(u64::from_be_bytes(slice))
}

fn ps3_timestamp_to_utc_rfc3339(raw: u64) -> Option<String> {
    if raw == 0 {
        return None;
    }
    let secs_total = raw / 1_000_000;
    let unix_secs: i64 = if secs_total >= PS3_TO_UNIX_EPOCH_SECS {
        i64::try_from(secs_total - PS3_TO_UNIX_EPOCH_SECS).ok()?
    } else if raw >= 1_000_000_000_000_000 {
        i64::try_from(secs_total).ok()?
    } else if raw >= 1_000_000_000 {
        i64::try_from(raw).ok()?
    } else {
        return None;
    };

    // Sanity check range (2005-01-01 to 2100-01-01)
    if !(1_104_537_600..=4_102_444_800).contains(&unix_secs) {
        return None;
    }

    DateTime::from_timestamp(unix_secs, 0).map(|dt| dt.to_rfc3339_opts(SecondsFormat::Secs, true))
}

/// Bounds-checked big-endian parser for Sony PS3 / RPCS3 `TROPUSR.DAT` (read-only).
pub fn parse_tropusr_dat(
    bytes: &[u8],
    native_user_key: &str,
) -> Result<Vec<ParsedTrophyUnlock>, DomainError> {
    if bytes.len() < TROPUSR_HEADER_LEN || bytes.len() > MAX_TROPHY_FILE_BYTES {
        return Err(DomainError::Validation(format!(
            "invalid TROPUSR.DAT size ({} bytes)",
            bytes.len()
        )));
    }
    if bytes.get(0..4) != Some(TROPUSR_MAGIC.as_slice()) {
        return Err(DomainError::Validation(
            "TROPUSR.DAT magic signature (81 8F 54 AD) not found".to_string(),
        ));
    }

    let table_count = read_be_u32(bytes, 16)
        .ok_or_else(|| DomainError::Validation("failed reading TROPUSR table_count".to_string()))?
        as usize;

    if table_count > 64 {
        return Err(DomainError::Validation(format!(
            "unreasonable TROPUSR table_count: {table_count}"
        )));
    }

    let mut unlocks = Vec::new();
    for i in 0..table_count {
        let dir_off = TROPUSR_HEADER_LEN
            .checked_add(i.checked_mul(TROPUSR_DIR_ENTRY_LEN).ok_or_else(|| {
                DomainError::Validation("TROPUSR directory offset overflow".to_string())
            })?)
            .ok_or_else(|| {
                DomainError::Validation("TROPUSR directory offset overflow".to_string())
            })?;

        let Some(table_type) = read_be_u32(bytes, dir_off) else {
            break;
        };
        if table_type != TROPUSR_TABLE_TYPE_UNLOCK {
            continue;
        }

        let entry_size = read_be_u32(bytes, dir_off + 4).unwrap_or(0) as usize;
        let entry_count = read_be_u32(bytes, dir_off + 12).unwrap_or(0) as usize;
        let table_offset = read_be_u64(bytes, dir_off + 16).unwrap_or(0) as usize;

        if entry_size < 48 || entry_count > 4096 {
            return Err(DomainError::Validation(format!(
                "invalid TROPUSR type-6 table geometry (entry_size={entry_size}, entry_count={entry_count})"
            )));
        }

        for idx in 0..entry_count {
            let Some(off) = idx
                .checked_mul(entry_size)
                .and_then(|d| table_offset.checked_add(d))
            else {
                break;
            };
            let Some(end) = off.checked_add(entry_size) else {
                break;
            };
            if end > bytes.len() {
                break;
            }

            let Some(trophy_id) = read_be_u32(bytes, off + 16) else {
                continue;
            };
            let Some(unlock_state) = read_be_u32(bytes, off + 20) else {
                continue;
            };
            if unlock_state == 0 {
                continue;
            }

            let ts1 = read_be_u64(bytes, off + 32).unwrap_or(0);
            let ts2 = read_be_u64(bytes, off + 40).unwrap_or(0);
            let unlocked_at_utc = ps3_timestamp_to_utc_rfc3339(ts1)
                .or_else(|| ps3_timestamp_to_utc_rfc3339(ts2))
                .unwrap_or_else(now_utc_rfc3339);

            unlocks.push(ParsedTrophyUnlock {
                external_trophy_id: format!("{trophy_id:03}"),
                unlocked_at_utc,
                native_user_key: native_user_key.to_string(),
            });
        }
    }

    Ok(unlocks)
}

#[derive(Debug, Default, Clone)]
pub struct Rpcs3TrophyReader;

impl Rpcs3TrophyReader {
    pub fn new() -> Self {
        Self
    }

    fn find_file_case_tolerant(dir: &Path, names: &[&str]) -> Option<PathBuf> {
        for name in names {
            let p = dir.join(name);
            if p.is_file() {
                return Some(p);
            }
        }
        None
    }

    fn read_single_trophy_dir(
        set_dir: &Path,
        native_user_key: &str,
        diagnostics: &mut Vec<String>,
    ) -> Option<ParsedTrophySet> {
        let conf_path = Self::find_file_case_tolerant(
            set_dir,
            &["TROPCONF.SFM", "tropconf.sfm", "TROP.SFM", "trop.sfm"],
        )?;

        let conf_bytes = match fs::read(&conf_path) {
            Ok(b) => b,
            Err(err) => {
                diagnostics.push(format!("Falha ao ler '{}': {err}", conf_path.display()));
                return None;
            }
        };

        let mut conf = match parse_tropconf_sfm(&conf_bytes) {
            Ok(c) => c,
            Err(err) => {
                diagnostics.push(format!(
                    "TROPCONF.SFM inválido/ignorado em '{}': {err}",
                    conf_path.display()
                ));
                return None;
            }
        };

        let external_set_id = conf.npcommid.unwrap_or_else(|| {
            set_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("UNKNOWN_SET")
                .to_string()
        });

        let set_icon = Self::find_file_case_tolerant(set_dir, &["ICON0.PNG", "icon0.png"]);
        for trophy in &mut conf.trophies {
            let upper = format!("TROP{}.PNG", trophy.external_trophy_id);
            let lower = format!("trop{}.png", trophy.external_trophy_id);
            trophy.icon_path = Self::find_file_case_tolerant(set_dir, &[&upper, &lower]);
        }

        let unlocks = if let Some(usr_path) =
            Self::find_file_case_tolerant(set_dir, &["TROPUSR.DAT", "tropusr.dat"])
        {
            match fs::read(&usr_path) {
                Ok(bytes) => match parse_tropusr_dat(&bytes, native_user_key) {
                    Ok(u) => u,
                    Err(err) => {
                        diagnostics.push(format!(
                            "TROPUSR.DAT inválido/ignorado em '{}': {err}",
                            usr_path.display()
                        ));
                        Vec::new()
                    }
                },
                Err(err) => {
                    diagnostics.push(format!("Falha ao ler '{}': {err}", usr_path.display()));
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        Some(ParsedTrophySet {
            console_slug: "ps3".to_string(),
            native_user_key: native_user_key.to_string(),
            external_set_id,
            title: conf.title,
            detail: conf.detail,
            icon_path: set_icon,
            trophies: conf.trophies,
            unlocks,
        })
    }
}

impl TrophyReaderPort for Rpcs3TrophyReader {
    fn console_slug(&self) -> &'static str {
        "ps3"
    }

    fn read_trophy_sets(&self, emulator_root_dir: &Path) -> TrophyReadBatch {
        let mut batch = TrophyReadBatch::default();
        let home_dir = emulator_root_dir.join("dev_hdd0").join("home");
        if !home_dir.is_dir() {
            return batch;
        }

        let Ok(user_entries) = fs::read_dir(&home_dir) else {
            return batch;
        };

        let mut user_dirs: Vec<PathBuf> = user_entries
            .flatten()
            .filter(|e| {
                e.file_type()
                    .is_ok_and(|ft| !ft.is_symlink() && ft.is_dir())
            })
            .map(|e| e.path())
            .collect();
        user_dirs.sort();

        for user_dir in user_dirs {
            let Some(user_key) = user_dir
                .file_name()
                .and_then(|n| n.to_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
            else {
                continue;
            };

            let trophy_root = user_dir.join("trophy");
            if !trophy_root.is_dir() {
                continue;
            }
            let Ok(set_entries) = fs::read_dir(&trophy_root) else {
                continue;
            };

            let mut set_dirs: Vec<PathBuf> = set_entries
                .flatten()
                .filter(|e| {
                    e.file_type()
                        .is_ok_and(|ft| !ft.is_symlink() && ft.is_dir())
                })
                .map(|e| e.path())
                .collect();
            set_dirs.sort();

            for set_dir in set_dirs {
                if let Some(parsed) =
                    Self::read_single_trophy_dir(&set_dir, &user_key, &mut batch.diagnostics)
                {
                    batch.sets.push(parsed);
                }
            }
        }

        batch
    }
}

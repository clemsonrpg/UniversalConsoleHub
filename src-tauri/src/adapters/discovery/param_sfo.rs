use std::collections::BTreeMap;

use crate::domain::error::DomainError;

const PSF_MAGIC: &[u8; 4] = b"\0PSF";
const HEADER_SIZE: usize = 20;
const INDEX_ENTRY_SIZE: usize = 16;
const MAX_SFO_ENTRIES: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamSfoMetadata {
    pub title: Option<String>,
    pub title_id: Option<String>,
    pub app_ver: Option<String>,
    pub category: Option<String>,
    pub version: Option<String>,
    pub raw_strings: BTreeMap<String, String>,
    pub raw_integers: BTreeMap<String, u32>,
}

fn read_u16_le(bytes: &[u8], offset: usize) -> Result<u16, DomainError> {
    let end = offset
        .checked_add(2)
        .ok_or_else(|| DomainError::Validation("PARAM.SFO offset overflow".to_string()))?;
    let slice = bytes.get(offset..end).ok_or_else(|| {
        DomainError::Validation("PARAM.SFO truncated while reading u16".to_string())
    })?;
    Ok(u16::from_le_bytes([slice[0], slice[1]]))
}

fn read_u32_le(bytes: &[u8], offset: usize) -> Result<u32, DomainError> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| DomainError::Validation("PARAM.SFO offset overflow".to_string()))?;
    let slice = bytes.get(offset..end).ok_or_else(|| {
        DomainError::Validation("PARAM.SFO truncated while reading u32".to_string())
    })?;
    Ok(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

/// Parses a PlayStation `PARAM.SFO` (PSF) binary buffer with strict bounds checking.
/// Never panics on truncated, malformed, or hostile inputs.
pub fn parse_param_sfo(bytes: &[u8]) -> Result<ParamSfoMetadata, DomainError> {
    if bytes.len() < HEADER_SIZE {
        return Err(DomainError::Validation(
            "PARAM.SFO buffer smaller than 20-byte PSF header".to_string(),
        ));
    }

    if bytes.get(0..4) != Some(PSF_MAGIC.as_slice()) {
        return Err(DomainError::Validation(
            "invalid PARAM.SFO magic header (expected \\0PSF)".to_string(),
        ));
    }

    let key_table_start = usize::try_from(read_u32_le(bytes, 8)?)
        .map_err(|_| DomainError::Validation("PARAM.SFO key_table_start overflow".to_string()))?;
    let data_table_start = usize::try_from(read_u32_le(bytes, 12)?)
        .map_err(|_| DomainError::Validation("PARAM.SFO data_table_start overflow".to_string()))?;
    let entries_count = usize::try_from(read_u32_le(bytes, 16)?)
        .map_err(|_| DomainError::Validation("PARAM.SFO entry count overflow".to_string()))?;

    if entries_count > MAX_SFO_ENTRIES {
        return Err(DomainError::Validation(format!(
            "PARAM.SFO entry count {entries_count} exceeds safety cap {MAX_SFO_ENTRIES}"
        )));
    }

    let index_table_bytes = entries_count.checked_mul(INDEX_ENTRY_SIZE).ok_or_else(|| {
        DomainError::Validation("PARAM.SFO index table size overflow".to_string())
    })?;
    let index_table_end = HEADER_SIZE
        .checked_add(index_table_bytes)
        .ok_or_else(|| DomainError::Validation("PARAM.SFO index table end overflow".to_string()))?;

    if index_table_end > bytes.len()
        || key_table_start < index_table_end
        || key_table_start > bytes.len()
        || data_table_start < key_table_start
        || data_table_start > bytes.len()
    {
        return Err(DomainError::Validation(
            "PARAM.SFO header table offsets are out of bounds".to_string(),
        ));
    }

    let mut raw_strings = BTreeMap::new();
    let mut raw_integers = BTreeMap::new();

    for idx in 0..entries_count {
        let entry_offset = HEADER_SIZE
            .checked_add(idx.checked_mul(INDEX_ENTRY_SIZE).ok_or_else(|| {
                DomainError::Validation("PARAM.SFO entry offset overflow".to_string())
            })?)
            .ok_or_else(|| {
                DomainError::Validation("PARAM.SFO entry offset overflow".to_string())
            })?;

        let key_rel = usize::from(read_u16_le(bytes, entry_offset)?);
        let data_fmt = read_u16_le(bytes, entry_offset + 2)?;
        let data_len = usize::try_from(read_u32_le(bytes, entry_offset + 4)?)
            .map_err(|_| DomainError::Validation("PARAM.SFO data_len overflow".to_string()))?;
        let data_max_len = usize::try_from(read_u32_le(bytes, entry_offset + 8)?)
            .map_err(|_| DomainError::Validation("PARAM.SFO data_max_len overflow".to_string()))?;
        let data_rel = usize::try_from(read_u32_le(bytes, entry_offset + 12)?)
            .map_err(|_| DomainError::Validation("PARAM.SFO data_offset overflow".to_string()))?;

        if data_len > data_max_len {
            return Err(DomainError::Validation(
                "PARAM.SFO entry data_len exceeds data_max_len".to_string(),
            ));
        }

        let key_start = key_table_start
            .checked_add(key_rel)
            .ok_or_else(|| DomainError::Validation("PARAM.SFO key offset overflow".to_string()))?;
        let key_slice = bytes.get(key_start..data_table_start).ok_or_else(|| {
            DomainError::Validation("PARAM.SFO key offset out of bounds".to_string())
        })?;
        let nul_pos = key_slice.iter().position(|&b| b == 0).ok_or_else(|| {
            DomainError::Validation("PARAM.SFO key missing NUL terminator".to_string())
        })?;
        let key = String::from_utf8_lossy(&key_slice[..nul_pos])
            .trim()
            .to_string();
        if key.is_empty() {
            continue;
        }

        let data_start = data_table_start
            .checked_add(data_rel)
            .ok_or_else(|| DomainError::Validation("PARAM.SFO data offset overflow".to_string()))?;
        let data_end = data_start
            .checked_add(data_len)
            .ok_or_else(|| DomainError::Validation("PARAM.SFO data end overflow".to_string()))?;
        let data_slice = bytes.get(data_start..data_end).ok_or_else(|| {
            DomainError::Validation("PARAM.SFO data slice out of bounds".to_string())
        })?;

        match data_fmt {
            // 0x0004 = UTF-8 (not necessarily NUL-terminated), 0x0204 = UTF-8 NUL-terminated
            0x0004 | 0x0204 => {
                let effective = match data_slice.iter().position(|&b| b == 0) {
                    Some(pos) => &data_slice[..pos],
                    None => data_slice,
                };
                let value = String::from_utf8_lossy(effective)
                    .replace(['\r', '\n'], " ")
                    .trim()
                    .to_string();
                if !value.is_empty() {
                    raw_strings.insert(key, value);
                }
            }
            // 0x0404 = little-endian u32
            0x0404 if data_slice.len() >= 4 => {
                let val = u32::from_le_bytes([
                    data_slice[0],
                    data_slice[1],
                    data_slice[2],
                    data_slice[3],
                ]);
                raw_integers.insert(key, val);
            }
            _ => {
                // Ignore unknown vendor field types safely without failing the whole file
            }
        }
    }

    Ok(ParamSfoMetadata {
        title: raw_strings.get("TITLE").cloned(),
        title_id: raw_strings.get("TITLE_ID").cloned(),
        app_ver: raw_strings.get("APP_VER").cloned(),
        category: raw_strings.get("CATEGORY").cloned(),
        version: raw_strings.get("VERSION").cloned(),
        raw_strings,
        raw_integers,
    })
}

/// Infers region code from a standard PlayStation 3 `TITLE_ID` prefix.
pub fn infer_ps3_region(title_id: &str) -> &'static str {
    let upper = title_id.trim().to_ascii_uppercase();
    if upper.starts_with("BLUS")
        || upper.starts_with("BCUS")
        || upper.starts_with("NPUB")
        || upper.starts_with("NPUA")
    {
        "USA"
    } else if upper.starts_with("BLES")
        || upper.starts_with("BCES")
        || upper.starts_with("NPEB")
        || upper.starts_with("NPEA")
    {
        "EUR"
    } else if upper.starts_with("BLJM")
        || upper.starts_with("BCJS")
        || upper.starts_with("BLJS")
        || upper.starts_with("NPJB")
        || upper.starts_with("NPJA")
    {
        "JPN"
    } else if upper.starts_with("BLAS")
        || upper.starts_with("BCAS")
        || upper.starts_with("NPHB")
        || upper.starts_with("NPHA")
    {
        "ASIA"
    } else {
        "UNKNOWN"
    }
}

/// Builds a valid synthetic `PARAM.SFO` binary buffer for non-copyrighted test fixtures.
pub fn build_synthetic_param_sfo(entries: &[(&str, &str)]) -> Vec<u8> {
    let mut sorted: Vec<(&str, &str)> = entries.to_vec();
    sorted.sort_by_key(|(k, _)| *k);

    let mut key_table = Vec::new();
    let mut data_table = Vec::new();
    let mut index_entries: Vec<(u16, u16, u32, u32, u32)> = Vec::with_capacity(sorted.len());

    for (key, value) in &sorted {
        let key_offset = key_table.len() as u16;
        key_table.extend_from_slice(key.as_bytes());
        key_table.push(0);

        // Align each data value on a 4-byte boundary (standard PSF layout)
        while data_table.len() % 4 != 0 {
            data_table.push(0);
        }
        let data_offset = data_table.len() as u32;
        let val_bytes = value.as_bytes();
        let data_len = (val_bytes.len() + 1) as u32;
        let mut padded_len = data_len;
        while padded_len % 4 != 0 {
            padded_len += 1;
        }
        data_table.extend_from_slice(val_bytes);
        data_table.push(0);
        while (data_table.len() as u32) < data_offset + padded_len {
            data_table.push(0);
        }

        index_entries.push((key_offset, 0x0204, data_len, padded_len, data_offset));
    }

    while key_table.len() % 4 != 0 {
        key_table.push(0);
    }

    let key_table_start = (HEADER_SIZE + index_entries.len() * INDEX_ENTRY_SIZE) as u32;
    let data_table_start = key_table_start + (key_table.len() as u32);

    let mut out = Vec::with_capacity(data_table_start as usize + data_table.len());
    out.extend_from_slice(PSF_MAGIC);
    out.extend_from_slice(&0x0000_0101u32.to_le_bytes());
    out.extend_from_slice(&key_table_start.to_le_bytes());
    out.extend_from_slice(&data_table_start.to_le_bytes());
    out.extend_from_slice(&(index_entries.len() as u32).to_le_bytes());

    for (key_off, fmt, len, max_len, data_off) in index_entries {
        out.extend_from_slice(&key_off.to_le_bytes());
        out.extend_from_slice(&fmt.to_le_bytes());
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&max_len.to_le_bytes());
        out.extend_from_slice(&data_off.to_le_bytes());
    }
    out.extend_from_slice(&key_table);
    out.extend_from_slice(&data_table);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_synthetic_param_sfo_and_rejects_bad_lengths() {
        let sfo = build_synthetic_param_sfo(&[
            ("TITLE", "Test Title\nSubtitle"),
            ("TITLE_ID", "BCUS98114"),
            ("APP_VER", "02.17"),
        ]);
        let parsed = parse_param_sfo(&sfo).expect("parse synthetic");
        assert_eq!(parsed.title.as_deref(), Some("Test Title Subtitle"));
        assert_eq!(parsed.title_id.as_deref(), Some("BCUS98114"));
        assert_eq!(parsed.app_ver.as_deref(), Some("02.17"));
        assert_eq!(infer_ps3_region("BCUS98114"), "USA");

        // Corrupt data_len > data_max_len in first entry
        let mut corrupted = sfo.clone();
        corrupted[24..28].copy_from_slice(&999u32.to_le_bytes());
        assert!(parse_param_sfo(&corrupted).is_err());
    }
}

use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use serde_json::json;

use crate::domain::console_profile::{
    ConsoleProfileProvider, DetectedLocalUser, EmulatorStatus, NativeConsoleArtworkCandidate,
    NativeConsoleProfileSnapshot, TrophySyncReadiness,
};
use crate::infrastructure::system::security::SecurityManager;

const MAX_TEXT_PROBE_BYTES: usize = 4096;

#[derive(Debug, Default, Clone)]
pub struct Ps3ProfileProvider;

impl Ps3ProfileProvider {
    pub fn new() -> Self {
        Self
    }

    fn read_bounded_utf8(path: &Path, limit: usize) -> Option<String> {
        let mut file = File::open(path).ok()?;
        let mut buf = vec![0u8; limit];
        let read_len = file.read(&mut buf).ok()?;
        if read_len == 0 {
            return None;
        }
        Some(String::from_utf8_lossy(&buf[..read_len]).into_owned())
    }

    fn read_rpcs3_log_version(root_dir: &Path) -> Option<String> {
        let log_path = root_dir.join("RPCS3.log");
        if !log_path.is_file() {
            return None;
        }
        let text = Self::read_bounded_utf8(&log_path, 512)?;
        text.lines()
            .next()
            .filter(|line| line.contains("RPCS3"))
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
    }

    /// Parses Sony PS3 `dev_flash/vsh/etc/version.txt` (typically `release:04.9100:`).
    pub(crate) fn parse_ps3_firmware_version(raw: &str) -> Option<String> {
        for line in raw.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("release:") {
                let token = rest.split(':').next().unwrap_or("").trim();
                if let Some(normalized) = Self::normalize_fw_release(token) {
                    return Some(normalized);
                }
            }
        }
        // Fallback: first printable non-empty version-like line
        raw.lines()
            .map(str::trim)
            .find(|l| !l.is_empty() && l.len() <= 32 && l.chars().any(|c| c.is_ascii_digit()))
            .map(str::to_string)
    }

    fn normalize_fw_release(raw: &str) -> Option<String> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return None;
        }
        if let Some((major_raw, minor_raw)) = trimmed.split_once('.') {
            if !major_raw.chars().all(|c| c.is_ascii_digit())
                || !minor_raw.chars().all(|c| c.is_ascii_digit())
            {
                return None;
            }
            let major = major_raw.trim_start_matches('0');
            let major = if major.is_empty() { "0" } else { major };
            let mut minor = minor_raw.trim_end_matches('0').to_string();
            while minor.len() < 2 {
                minor.push('0');
            }
            return Some(format!("{major}.{minor}"));
        }
        Some(trimmed.to_string())
    }

    fn read_firmware_version(root_dir: &Path) -> Option<String> {
        let version_txt = root_dir
            .join("dev_flash")
            .join("vsh")
            .join("etc")
            .join("version.txt");
        if !version_txt.is_file() {
            return None;
        }
        let text = Self::read_bounded_utf8(&version_txt, MAX_TEXT_PROBE_BYTES)?;
        Self::parse_ps3_firmware_version(&text)
    }

    fn count_child_directories(dir: &Path) -> i64 {
        let Ok(entries) = fs::read_dir(dir) else {
            return 0;
        };
        let mut count = 0i64;
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if !ft.is_symlink() && ft.is_dir() {
                    count += 1;
                }
            }
        }
        count
    }

    fn find_file_case_tolerant(dir: &Path, candidates: &[&str]) -> Option<PathBuf> {
        for name in candidates {
            let direct = dir.join(name);
            if direct.is_file() {
                return Some(direct);
            }
        }
        None
    }
}

impl ConsoleProfileProvider for Ps3ProfileProvider {
    fn console_slug(&self) -> &'static str {
        "ps3"
    }

    fn collect_native_snapshot(
        &self,
        configured_exe: Option<&Path>,
    ) -> NativeConsoleProfileSnapshot {
        let Some(exe) = configured_exe else {
            return NativeConsoleProfileSnapshot {
                console_slug: "ps3".to_string(),
                emulator_slug: Some("rpcs3".to_string()),
                emulator_status: EmulatorStatus::Unconfigured,
                detected_emulator_version: None,
                detected_firmware_version: None,
                emulator_root_dir: None,
                vsh_or_system_boot_ready: false,
                local_users: Vec::new(),
                native_game_scan_dirs: Vec::new(),
                native_save_count: 0,
                native_trophy_dir_present: false,
                trophy_sync_readiness: TrophySyncReadiness::Phase4Planned,
                native_metadata_json: json!({
                    "vshSelfPresent": false,
                    "devHdd0Present": false,
                    "devFlashPresent": false,
                    "ps3HomeUserDirsCount": 0,
                    "trophyDirsCount": 0
                })
                .to_string(),
                diagnostics: vec![
                    "RPCS3 ainda não configurado. Configure o executável rpcs3.exe para sincronizar dados nativos do PlayStation 3."
                        .to_string(),
                ],
            };
        };

        let valid_exe = match SecurityManager::validate_existing_file(exe) {
            Ok(path) => path,
            Err(_) => {
                return NativeConsoleProfileSnapshot {
                    console_slug: "ps3".to_string(),
                    emulator_slug: Some("rpcs3".to_string()),
                    emulator_status: EmulatorStatus::MissingOnDisk,
                    detected_emulator_version: None,
                    detected_firmware_version: None,
                    emulator_root_dir: exe.parent().map(Path::to_path_buf),
                    vsh_or_system_boot_ready: false,
                    local_users: Vec::new(),
                    native_game_scan_dirs: Vec::new(),
                    native_save_count: 0,
                    native_trophy_dir_present: false,
                    trophy_sync_readiness: TrophySyncReadiness::Phase4Planned,
                    native_metadata_json: json!({
                        "vshSelfPresent": false,
                        "devHdd0Present": false,
                        "devFlashPresent": false,
                        "ps3HomeUserDirsCount": 0,
                        "trophyDirsCount": 0
                    })
                    .to_string(),
                    diagnostics: vec![format!(
                        "Executável RPCS3 configurado não encontrado no disco: '{}'. Histórico do Hub preservado.",
                        exe.display()
                    )],
                };
            }
        };

        let Some(root_dir) = valid_exe.parent().map(Path::to_path_buf) else {
            return NativeConsoleProfileSnapshot {
                console_slug: "ps3".to_string(),
                emulator_slug: Some("rpcs3".to_string()),
                emulator_status: EmulatorStatus::MissingOnDisk,
                detected_emulator_version: None,
                detected_firmware_version: None,
                emulator_root_dir: None,
                vsh_or_system_boot_ready: false,
                local_users: Vec::new(),
                native_game_scan_dirs: Vec::new(),
                native_save_count: 0,
                native_trophy_dir_present: false,
                trophy_sync_readiness: TrophySyncReadiness::Phase4Planned,
                native_metadata_json: "{}".to_string(),
                diagnostics: vec![
                    "Não foi possível determinar o diretório raiz do RPCS3.".to_string()
                ],
            };
        };

        let mut diagnostics = Vec::new();
        let detected_emulator_version = Self::read_rpcs3_log_version(&root_dir);
        let detected_firmware_version = Self::read_firmware_version(&root_dir);

        let dev_flash_dir = root_dir.join("dev_flash");
        let dev_hdd0_dir = root_dir.join("dev_hdd0");
        let vsh_self = dev_flash_dir.join("vsh").join("module").join("vsh.self");
        let vsh_present = vsh_self.is_file();
        let dev_hdd0_present = dev_hdd0_dir.is_dir();
        let dev_flash_present = dev_flash_dir.is_dir();

        if !vsh_present {
            diagnostics.push(
                "dev_flash/vsh/module/vsh.self ausente; instale seu PS3UPDAT.PUP no RPCS3 para habilitar o boot do XMB."
                    .to_string(),
            );
        }

        let mut local_users = Vec::new();
        let mut native_save_count = 0i64;
        let mut trophy_dirs_count = 0usize;
        let mut home_user_dirs_count = 0usize;

        let home_dir = dev_hdd0_dir.join("home");
        if home_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&home_dir) {
                let mut user_dirs = Vec::new();
                for entry in entries.flatten() {
                    if let Ok(ft) = entry.file_type() {
                        if !ft.is_symlink() && ft.is_dir() {
                            user_dirs.push(entry.path());
                        }
                    }
                }
                user_dirs.sort();
                home_user_dirs_count = user_dirs.len();

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

                    // Read real localusername ONLY when present and non-empty (never invent users)
                    let username_file = user_dir.join("localusername");
                    if username_file.is_file() {
                        if let Some(raw_name) = Self::read_bounded_utf8(&username_file, 256) {
                            let clean_name = raw_name
                                .lines()
                                .next()
                                .unwrap_or("")
                                .trim()
                                .trim_matches('\0')
                                .to_string();
                            if !clean_name.is_empty() {
                                local_users.push(DetectedLocalUser {
                                    native_user_key: user_key.clone(),
                                    display_name: clean_name,
                                    source_path: user_dir.clone(),
                                });
                            }
                        }
                    }

                    let user_savedata = user_dir.join("savedata");
                    if user_savedata.is_dir() {
                        native_save_count += Self::count_child_directories(&user_savedata);
                    }

                    let user_trophy_dir = user_dir.join("trophy");
                    if user_trophy_dir.is_dir() {
                        trophy_dirs_count += 1;
                    }
                }
            }
        }

        // Also count legacy top-level dev_hdd0/savedata if present
        let root_savedata = dev_hdd0_dir.join("savedata");
        if root_savedata.is_dir() {
            native_save_count += Self::count_child_directories(&root_savedata);
        }

        let mut native_game_scan_dirs = Vec::new();
        let hdd_game_dir = dev_hdd0_dir.join("game");
        if hdd_game_dir.is_dir() {
            native_game_scan_dirs.push(hdd_game_dir);
        }

        let native_trophy_dir_present = trophy_dirs_count > 0;
        let metadata = json!({
            "vshSelfPresent": vsh_present,
            "devHdd0Present": dev_hdd0_present,
            "devFlashPresent": dev_flash_present,
            "ps3HomeUserDirsCount": home_user_dirs_count,
            "trophyDirsCount": trophy_dirs_count
        });

        NativeConsoleProfileSnapshot {
            console_slug: "ps3".to_string(),
            emulator_slug: Some("rpcs3".to_string()),
            emulator_status: EmulatorStatus::Available,
            detected_emulator_version,
            detected_firmware_version,
            emulator_root_dir: Some(root_dir),
            vsh_or_system_boot_ready: vsh_present && dev_hdd0_present,
            local_users,
            native_game_scan_dirs,
            native_save_count,
            native_trophy_dir_present,
            trophy_sync_readiness: TrophySyncReadiness::Phase4Planned,
            native_metadata_json: metadata.to_string(),
            diagnostics,
        }
    }

    fn find_native_artwork_for_game(
        &self,
        game_file_path: &Path,
    ) -> Option<NativeConsoleArtworkCandidate> {
        // Standard PS3 layout: <game_root>/USRDIR/EBOOT.BIN -> <game_root>/ICON0.PNG & PIC1.PNG
        let parent = game_file_path.parent()?;
        let game_root = if parent
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.eq_ignore_ascii_case("USRDIR"))
        {
            parent.parent()?
        } else {
            parent
        };

        let icon_path = Self::find_file_case_tolerant(game_root, &["ICON0.PNG", "icon0.png"]);
        let background_path = Self::find_file_case_tolerant(game_root, &["PIC1.PNG", "pic1.png"]);

        if icon_path.is_none() && background_path.is_none() {
            return None;
        }

        Some(NativeConsoleArtworkCandidate {
            game_file_path: game_file_path.to_path_buf(),
            icon_path,
            background_path,
        })
    }
}

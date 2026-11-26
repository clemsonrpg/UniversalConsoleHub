pub mod ps3_profile;

use std::path::Path;

use serde_json::json;

use crate::domain::console_profile::{
    ConsoleProfileProvider, EmulatorStatus, NativeConsoleArtworkCandidate,
    NativeConsoleProfileSnapshot, TrophySyncReadiness,
};
use crate::infrastructure::system::security::SecurityManager;

pub use ps3_profile::Ps3ProfileProvider;

/// Honest platform-aware fallback provider for Phase 5 consoles prior to their
/// dedicated platform readers. Never fabricates users, firmware, or native trophies.
pub struct GenericPlatformProfileProvider {
    console_slug: &'static str,
    emulator_slug: &'static str,
    supports_native_trophies: bool,
}

impl GenericPlatformProfileProvider {
    pub const fn new(
        console_slug: &'static str,
        emulator_slug: &'static str,
        supports_native_trophies: bool,
    ) -> Self {
        Self {
            console_slug,
            emulator_slug,
            supports_native_trophies,
        }
    }
}

impl ConsoleProfileProvider for GenericPlatformProfileProvider {
    fn console_slug(&self) -> &'static str {
        self.console_slug
    }

    fn collect_native_snapshot(
        &self,
        configured_exe: Option<&Path>,
    ) -> NativeConsoleProfileSnapshot {
        let (emulator_status, emulator_root_dir, diagnostics) = match configured_exe {
            None => (
                EmulatorStatus::Unconfigured,
                None,
                vec![format!(
                    "Integração nativa do emulador '{}' planejada para a Fase 5. Estatísticas do Hub permanecem disponíveis.",
                    self.emulator_slug
                )],
            ),
            Some(exe) => match SecurityManager::validate_existing_file(exe) {
                Ok(valid) => (
                    EmulatorStatus::Available,
                    valid.parent().map(Path::to_path_buf),
                    vec![format!(
                        "Executável '{}' encontrado; leitor nativo dedicado desta plataforma entra na Fase 5.",
                        self.emulator_slug
                    )],
                ),
                Err(_) => (
                    EmulatorStatus::MissingOnDisk,
                    exe.parent().map(Path::to_path_buf),
                    vec![format!(
                        "Executável configurado '{}' não encontrado no disco. Histórico do Hub preservado.",
                        exe.display()
                    )],
                ),
            },
        };

        let trophy_sync_readiness = if self.supports_native_trophies {
            TrophySyncReadiness::Phase4Planned
        } else {
            TrophySyncReadiness::Unavailable
        };

        NativeConsoleProfileSnapshot {
            console_slug: self.console_slug.to_string(),
            emulator_slug: Some(self.emulator_slug.to_string()),
            emulator_status,
            detected_emulator_version: None,
            detected_firmware_version: None,
            emulator_root_dir,
            vsh_or_system_boot_ready: false,
            local_users: Vec::new(),
            native_game_scan_dirs: Vec::new(),
            native_save_count: 0,
            native_trophy_dir_present: false,
            trophy_sync_readiness,
            native_metadata_json: json!({
                "nativeReaderPhase": "phase_5_planned",
                "platformNativeTrophies": self.supports_native_trophies
            })
            .to_string(),
            diagnostics,
        }
    }

    fn find_native_artwork_for_game(
        &self,
        _game_file_path: &Path,
    ) -> Option<NativeConsoleArtworkCandidate> {
        None
    }
}

pub fn profile_provider_for_console(console_slug: &str) -> Box<dyn ConsoleProfileProvider> {
    match console_slug {
        "ps3" => Box::new(Ps3ProfileProvider::new()),
        "ps2" => Box::new(GenericPlatformProfileProvider::new("ps2", "pcsx2", false)),
        "wii" => Box::new(GenericPlatformProfileProvider::new("wii", "dolphin", false)),
        "xbox" => Box::new(GenericPlatformProfileProvider::new("xbox", "xemu", false)),
        "xbox360" => Box::new(GenericPlatformProfileProvider::new(
            "xbox360",
            "xenia_canary",
            true,
        )),
        "ps4" => Box::new(GenericPlatformProfileProvider::new("ps4", "shadps4", true)),
        _ => Box::new(GenericPlatformProfileProvider::new(
            "unknown", "unknown", false,
        )),
    }
}

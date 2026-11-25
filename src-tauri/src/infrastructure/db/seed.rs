use rusqlite::{params, Connection};

use crate::domain::console::{
    CapabilityState, ConsoleCapabilities, FeatureAvailability, FeatureCapability, FeatureKey,
    VerificationLevel,
};
use crate::domain::error::DomainError;
use crate::domain::player::now_utc_rfc3339;

struct ConsoleSeed {
    id: &'static str,
    slug: &'static str,
    name: &'static str,
    manufacturer: &'static str,
    generation: i64,
    release_year: i64,
    phase_status: &'static str,
    notes: &'static str,
    emulator_id: &'static str,
    emulator_slug: &'static str,
    emulator_name: &'static str,
    integration_phase: &'static str,
    capabilities: ConsoleCapabilities,
}

fn feature(
    feature: FeatureKey,
    states: Vec<CapabilityState>,
    availability: FeatureAvailability,
    verification: VerificationLevel,
    notes: &str,
) -> FeatureCapability {
    FeatureCapability {
        feature,
        states,
        availability,
        verification,
        notes: notes.to_string(),
    }
}

fn priority_consoles() -> Vec<ConsoleSeed> {
    vec![
        ConsoleSeed {
            id: "console-ps3",
            slug: "ps3",
            name: "PlayStation 3",
            manufacturer: "Sony Interactive Entertainment",
            generation: 7,
            release_year: 2006,
            phase_status: "phase_2_console_environment",
            notes: "Prioridade 1 via RPCS3. Fase 1–2: detecção, validação local de firmware/vsh.self, scanner PARAM.SFO e ambiente independente. Boot ao vivo do XMB e jogos PS3 entram na Fase 3.",
            emulator_id: "emu-rpcs3",
            emulator_slug: "rpcs3",
            emulator_name: "RPCS3",
            integration_phase: "phase_2_console_environment",
            capabilities: ConsoleCapabilities {
                supported_states: vec![
                    CapabilityState::FullSystemBoot,
                    CapabilityState::OriginalDashboard,
                    CapabilityState::DirectGameOnly,
                ],
                features: vec![
                    feature(
                        FeatureKey::SystemBoot,
                        vec![CapabilityState::FullSystemBoot],
                        FeatureAvailability::RequiresUserFiles,
                        VerificationLevel::DocumentedUpstream,
                        "Requer PS3UPDAT.PUP fornecida legalmente pelo usuário instalada em dev_flash/vsh/module/vsh.self.",
                    ),
                    feature(
                        FeatureKey::OriginalDashboard,
                        vec![CapabilityState::OriginalDashboard],
                        FeatureAvailability::RequiresUserFiles,
                        VerificationLevel::DocumentedUpstream,
                        "PS3 XMB original suportado via RPCS3; confirmação em tempo de execução ocorre na Fase 3.",
                    ),
                    feature(
                        FeatureKey::DirectGameBoot,
                        vec![CapabilityState::DirectGameOnly],
                        FeatureAvailability::Available,
                        VerificationLevel::DocumentedUpstream,
                        "Execução direta via EBOOT.BIN (--no-gui) na Fase 3.",
                    ),
                    feature(
                        FeatureKey::LocalTrophies,
                        vec![CapabilityState::DirectGameOnly],
                        FeatureAvailability::Planned,
                        VerificationLevel::DocumentedUpstream,
                        "Leitura local somente leitura de TROPCONF.SFM e TROPUSR.DAT planejada para Fase 4.",
                    ),
                    feature(
                        FeatureKey::OnlineServices,
                        vec![CapabilityState::Unsupported],
                        FeatureAvailability::Unsupported,
                        VerificationLevel::DocumentedUpstream,
                        "PSN oficial não é suportada.",
                    ),
                ],
            },
        },
        ConsoleSeed {
            id: "console-ps2",
            slug: "ps2",
            name: "PlayStation 2",
            manufacturer: "Sony Interactive Entertainment",
            generation: 6,
            release_year: 2000,
            phase_status: "planned_phase_5",
            notes: "Integração via PCSX2 (Qt) planejada para a Fase 5. Exige BIOS PS2 legalmente fornecida pelo usuário.",
            emulator_id: "emu-pcsx2",
            emulator_slug: "pcsx2",
            emulator_name: "PCSX2 (Qt)",
            integration_phase: "planned_phase_5",
            capabilities: ConsoleCapabilities {
                supported_states: vec![
                    CapabilityState::FullSystemBoot,
                    CapabilityState::OriginalDashboard,
                    CapabilityState::DirectGameOnly,
                ],
                features: vec![
                    feature(
                        FeatureKey::SystemBoot,
                        vec![CapabilityState::FullSystemBoot],
                        FeatureAvailability::Planned,
                        VerificationLevel::DocumentedUpstream,
                        "Boot completo do BIOS PS2 com arquivo legal do usuário (Fase 5).",
                    ),
                    feature(
                        FeatureKey::OriginalDashboard,
                        vec![CapabilityState::OriginalDashboard],
                        FeatureAvailability::Planned,
                        VerificationLevel::DocumentedUpstream,
                        "PS2 Browser / System Configuration original via PCSX2 Qt (Fase 5).",
                    ),
                    feature(
                        FeatureKey::DirectGameBoot,
                        vec![CapabilityState::DirectGameOnly],
                        FeatureAvailability::Planned,
                        VerificationLevel::DocumentedUpstream,
                        "Execução direta via -batch -fullscreen (Fase 5).",
                    ),
                    feature(
                        FeatureKey::OnlineServices,
                        vec![CapabilityState::Unsupported],
                        FeatureAvailability::Unsupported,
                        VerificationLevel::DocumentedUpstream,
                        "Serviços online oficiais indisponíveis.",
                    ),
                ],
            },
        },
        ConsoleSeed {
            id: "console-wii",
            slug: "wii",
            name: "Nintendo Wii",
            manufacturer: "Nintendo",
            generation: 7,
            release_year: 2006,
            phase_status: "planned_phase_5",
            notes: "Integração via Dolphin planejada para a Fase 5. Wii System Menu exige NAND legalmente dumpada pelo usuário.",
            emulator_id: "emu-dolphin",
            emulator_slug: "dolphin",
            emulator_name: "Dolphin",
            integration_phase: "planned_phase_5",
            capabilities: ConsoleCapabilities {
                supported_states: vec![
                    CapabilityState::FullSystemBoot,
                    CapabilityState::OriginalDashboard,
                    CapabilityState::DirectGameOnly,
                ],
                features: vec![
                    feature(
                        FeatureKey::SystemBoot,
                        vec![CapabilityState::FullSystemBoot],
                        FeatureAvailability::Planned,
                        VerificationLevel::DocumentedUpstream,
                        "Boot via -n 0000000100000002 com NAND do usuário (Fase 5).",
                    ),
                    feature(
                        FeatureKey::OriginalDashboard,
                        vec![CapabilityState::OriginalDashboard],
                        FeatureAvailability::Planned,
                        VerificationLevel::DocumentedUpstream,
                        "Wii System Menu original via Dolphin (Fase 5).",
                    ),
                    feature(
                        FeatureKey::DirectGameBoot,
                        vec![CapabilityState::DirectGameOnly],
                        FeatureAvailability::Planned,
                        VerificationLevel::DocumentedUpstream,
                        "Execução direta via -b -e (Fase 5).",
                    ),
                    feature(
                        FeatureKey::OnlineServices,
                        vec![CapabilityState::Unsupported],
                        FeatureAvailability::Unsupported,
                        VerificationLevel::DocumentedUpstream,
                        "WiiConnect24 / Nintendo Wi-Fi Connection oficiais encerrados.",
                    ),
                ],
            },
        },
        ConsoleSeed {
            id: "console-xbox",
            slug: "xbox",
            name: "Xbox Original",
            manufacturer: "Microsoft",
            generation: 6,
            release_year: 2001,
            phase_status: "planned_phase_5",
            notes: "Integração via xemu planejada para a Fase 5. Exige MCPX, BIOS e imagem HDD qcow2 fornecidos pelo usuário.",
            emulator_id: "emu-xemu",
            emulator_slug: "xemu",
            emulator_name: "xemu",
            integration_phase: "planned_phase_5",
            capabilities: ConsoleCapabilities {
                supported_states: vec![
                    CapabilityState::FullSystemBoot,
                    CapabilityState::OriginalDashboard,
                    CapabilityState::DirectGameOnly,
                ],
                features: vec![
                    feature(
                        FeatureKey::SystemBoot,
                        vec![CapabilityState::FullSystemBoot],
                        FeatureAvailability::Planned,
                        VerificationLevel::DocumentedUpstream,
                        "Configuração MCPX/BIOS/HDD em xemu.toml sem -dvd_path (Fase 5).",
                    ),
                    feature(
                        FeatureKey::OriginalDashboard,
                        vec![CapabilityState::OriginalDashboard],
                        FeatureAvailability::Planned,
                        VerificationLevel::DocumentedUpstream,
                        "Dashboard Xbox original presente na imagem HDD do usuário (Fase 5).",
                    ),
                    feature(
                        FeatureKey::DirectGameBoot,
                        vec![CapabilityState::DirectGameOnly],
                        FeatureAvailability::Planned,
                        VerificationLevel::DocumentedUpstream,
                        "Execução direta via -full-screen -dvd_path (Fase 5).",
                    ),
                    feature(
                        FeatureKey::OnlineServices,
                        vec![CapabilityState::Unsupported],
                        FeatureAvailability::Unsupported,
                        VerificationLevel::DocumentedUpstream,
                        "Xbox Live original oficial encerrado.",
                    ),
                ],
            },
        },
        ConsoleSeed {
            id: "console-xbox360",
            slug: "xbox360",
            name: "Xbox 360",
            manufacturer: "Microsoft",
            generation: 7,
            release_year: 2005,
            phase_status: "planned_phase_5",
            notes: "Integração via Xenia Canary planejada para a Fase 5. Execução direta de títulos é o modo primário; dashboard dash.xex possui suporte parcial/limitado.",
            emulator_id: "emu-xenia-canary",
            emulator_slug: "xenia-canary",
            emulator_name: "Xenia Canary",
            integration_phase: "planned_phase_5",
            capabilities: ConsoleCapabilities {
                supported_states: vec![
                    CapabilityState::DirectGameOnly,
                    CapabilityState::PartialDashboard,
                ],
                features: vec![
                    feature(
                        FeatureKey::OriginalDashboard,
                        vec![CapabilityState::PartialDashboard],
                        FeatureAvailability::Partial,
                        VerificationLevel::DocumentedUpstream,
                        "Suporte limitado/nicho a dash.xex; não é um boot completo estável out-of-the-box.",
                    ),
                    feature(
                        FeatureKey::DirectGameBoot,
                        vec![CapabilityState::DirectGameOnly],
                        FeatureAvailability::Planned,
                        VerificationLevel::DocumentedUpstream,
                        "Execução direta de .xex/.iso (modo primário na Fase 5).",
                    ),
                    feature(
                        FeatureKey::OnlineServices,
                        vec![CapabilityState::Unsupported],
                        FeatureAvailability::Unsupported,
                        VerificationLevel::DocumentedUpstream,
                        "Xbox Live oficial não é suportado.",
                    ),
                ],
            },
        },
        ConsoleSeed {
            id: "console-ps4",
            slug: "ps4",
            name: "PlayStation 4",
            manufacturer: "Sony Interactive Entertainment",
            generation: 8,
            release_year: 2013,
            phase_status: "planned_phase_5_experimental",
            notes: "Integração experimental via shadPS4 planejada para a Fase 5. Compatibilidade altamente dependente de build e título; Orbis OS completo não é suportado.",
            emulator_id: "emu-shadps4",
            emulator_slug: "shadps4",
            emulator_name: "shadPS4",
            integration_phase: "planned_phase_5_experimental",
            capabilities: ConsoleCapabilities {
                supported_states: vec![
                    CapabilityState::Experimental,
                    CapabilityState::DirectGameOnly,
                ],
                features: vec![
                    feature(
                        FeatureKey::SystemBoot,
                        vec![CapabilityState::Unsupported],
                        FeatureAvailability::Unsupported,
                        VerificationLevel::DocumentedUpstream,
                        "Boot completo do Orbis OS não é suportado pelo shadPS4.",
                    ),
                    feature(
                        FeatureKey::OriginalDashboard,
                        vec![CapabilityState::Unsupported],
                        FeatureAvailability::Unsupported,
                        VerificationLevel::DocumentedUpstream,
                        "Dashboard PS4 original indisponível.",
                    ),
                    feature(
                        FeatureKey::DirectGameBoot,
                        vec![CapabilityState::Experimental, CapabilityState::DirectGameOnly],
                        FeatureAvailability::Experimental,
                        VerificationLevel::DocumentedUpstream,
                        "Execução direta experimental de títulos compatíveis (Fase 5).",
                    ),
                    feature(
                        FeatureKey::OnlineServices,
                        vec![CapabilityState::Unsupported],
                        FeatureAvailability::Unsupported,
                        VerificationLevel::DocumentedUpstream,
                        "PSN oficial não é suportada.",
                    ),
                ],
            },
        },
    ]
}

pub fn seed_defaults(conn: &mut Connection) -> Result<(), DomainError> {
    let now = now_utc_rfc3339();
    let tx = conn
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
        .map_err(|e| DomainError::Database(format!("failed to start seed tx: {e}")))?;

    // 1. Default offline local user & active profile
    tx.execute(
        "INSERT OR IGNORE INTO users (id, username, created_at, updated_at)
         VALUES ('user-default', 'player1', ?1, ?1)",
        params![now],
    )
    .map_err(|e| DomainError::Database(format!("seed user failed: {e}")))?;

    tx.execute(
        "INSERT OR IGNORE INTO profiles (id, user_id, handle, display_name, avatar_path, banner_path, bio, privacy_level, preferences_json, is_active, created_at, updated_at)
         VALUES ('profile-default', 'user-default', 'player1', 'Player 1', NULL, NULL, 'Perfil offline local do Universal Console Hub.', 'private', '{}', 1, ?1, ?1)",
        params![now],
    )
    .map_err(|e| DomainError::Database(format!("seed profile failed: {e}")))?;

    // 2. Priority 6 consoles & emulators
    for c in priority_consoles() {
        let caps_json = serde_json::to_string(&c.capabilities)
            .map_err(|e| DomainError::Database(format!("serialize capabilities failed: {e}")))?;

        tx.execute(
            "INSERT OR IGNORE INTO consoles (id, slug, name, manufacturer, generation, release_year, phase_status, capabilities_json, notes, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                c.id,
                c.slug,
                c.name,
                c.manufacturer,
                c.generation,
                c.release_year,
                c.phase_status,
                caps_json,
                c.notes,
                now
            ],
        )
        .map_err(|e| DomainError::Database(format!("seed console {} failed: {e}", c.slug)))?;

        tx.execute(
            "INSERT OR IGNORE INTO emulators (id, slug, name, console_id, executable_path, config_path, detected_version, integration_phase, capabilities_json, updated_at)
             VALUES (?1, ?2, ?3, ?4, NULL, NULL, NULL, ?5, ?6, ?7)",
            params![
                c.emulator_id,
                c.emulator_slug,
                c.emulator_name,
                c.id,
                c.integration_phase,
                caps_json,
                now
            ],
        )
        .map_err(|e| DomainError::Database(format!("seed emulator {} failed: {e}", c.emulator_slug)))?;
    }

    // 3. Five independent achievement providers (Section 13)
    let providers = [
        (
            "prov-local",
            "local",
            "LocalAchievementProvider",
            "local",
            "ready_offline",
            0,
            0,
            "Provider local offline do Universal Console Hub (ativo na Fase 1). Nunca rotula conquistas locais como oficiais.",
        ),
        (
            "prov-emulator",
            "emulator",
            "EmulatorAchievementProvider",
            "emulator",
            "planned_phase_4",
            0,
            0,
            "Leitura somente leitura de troféus locais de emuladores (RPCS3 TROPCONF.SFM/TROPUSR.DAT na Fase 4; RetroAchievements na Fase 6).",
        ),
        (
            "prov-playstation",
            "playstation",
            "PlayStationTrophyProvider",
            "official",
            "unavailable_no_official_api",
            1,
            1,
            "Indisponível sem API oficial autorizada. O Hub nunca simula nem contorna autenticação PSN.",
        ),
        (
            "prov-xbox",
            "xbox",
            "XboxAchievementProvider",
            "official",
            "unavailable_no_official_api",
            1,
            1,
            "Indisponível sem integração oficial autorizada Xbox Live.",
        ),
        (
            "prov-steam",
            "steam",
            "SteamAchievementProvider",
            "official",
            "planned_phase_6_authorized_key",
            1,
            1,
            "Planejado para Fase 6 via Steam Web API autorizada com chave fornecida pelo usuário.",
        ),
    ];

    for (id, slug, name, origin, status, auth, sync, notes) in providers {
        tx.execute(
            "INSERT OR IGNORE INTO achievement_providers (id, slug, name, origin, availability_status, supports_auth, supports_sync, notes, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![id, slug, name, origin, status, auth, sync, notes, now],
        )
        .map_err(|e| DomainError::Database(format!("seed provider {slug} failed: {e}")))?;
    }

    // 4. Default application settings
    let defaults = [
        ("ui.fullscreen_default", "false"),
        ("ui.target_resolution", "1920x1080"),
        ("ui.reduced_motion", "false"),
        ("emulation.launch_timeout_seconds", "45"),
    ];
    for (k, v) in defaults {
        tx.execute(
            "INSERT OR IGNORE INTO application_settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
            params![k, v, now],
        )
        .map_err(|e| DomainError::Database(format!("seed setting {k} failed: {e}")))?;
    }

    tx.commit()
        .map_err(|e| DomainError::Database(format!("commit seed tx failed: {e}")))?;

    Ok(())
}

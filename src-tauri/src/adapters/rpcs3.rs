use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::domain::console::{
    CapabilityState, ConsoleCapabilities, FeatureAvailability, FeatureCapability, FeatureKey,
    VerificationLevel,
};
use crate::domain::emulator::{
    DependencyReport, EmulatorAdapter, EmulatorInstallation, FirmwarePackageReport, LaunchCommand,
    SessionOutcome, SystemBootReadiness, SystemFilesReport,
};
use crate::domain::error::DomainError;
use crate::infrastructure::system::security::SecurityManager;

const PS3_PUP_MAGIC: &[u8; 5] = b"SCEUF";

#[derive(Debug, Default, Clone)]
pub struct Rpcs3Adapter;

impl Rpcs3Adapter {
    pub fn new() -> Self {
        Self
    }

    fn vsh_self_path(root_dir: &Path) -> PathBuf {
        root_dir
            .join("dev_flash")
            .join("vsh")
            .join("module")
            .join("vsh.self")
    }

    fn dev_hdd0_path(root_dir: &Path) -> PathBuf {
        root_dir.join("dev_hdd0")
    }
}

impl EmulatorAdapter for Rpcs3Adapter {
    fn slug(&self) -> &'static str {
        "rpcs3"
    }

    fn display_name(&self) -> &'static str {
        "RPCS3"
    }

    fn console_slug(&self) -> &'static str {
        "ps3"
    }

    fn capabilities(&self) -> ConsoleCapabilities {
        ConsoleCapabilities {
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
                    notes: "Requer PS3UPDAT.PUP fornecida legalmente pelo usuário instalada em dev_flash/vsh/module/vsh.self.".to_string(),
                },
                FeatureCapability {
                    feature: FeatureKey::OriginalDashboard,
                    states: vec![CapabilityState::OriginalDashboard],
                    availability: FeatureAvailability::RequiresUserFiles,
                    verification: VerificationLevel::DocumentedUpstream,
                    notes: "PS3 XMB original via vsh.self; confirmação de execução real reservada à Fase 3.".to_string(),
                },
                FeatureCapability {
                    feature: FeatureKey::DirectGameBoot,
                    states: vec![CapabilityState::DirectGameOnly],
                    availability: FeatureAvailability::Available,
                    verification: VerificationLevel::DocumentedUpstream,
                    notes: "Execução direta de títulos PS3 via EBOOT.BIN (--no-gui).".to_string(),
                },
                FeatureCapability {
                    feature: FeatureKey::LocalTrophies,
                    states: vec![CapabilityState::DirectGameOnly],
                    availability: FeatureAvailability::Planned,
                    verification: VerificationLevel::DocumentedUpstream,
                    notes: "Leitura somente leitura de dev_hdd0/home/<user>/trophy/ planejada para a Fase 4.".to_string(),
                },
                FeatureCapability {
                    feature: FeatureKey::OnlineServices,
                    states: vec![CapabilityState::Unsupported],
                    availability: FeatureAvailability::Unsupported,
                    verification: VerificationLevel::DocumentedUpstream,
                    notes: "PSN oficial não é suportada.".to_string(),
                },
            ],
        }
    }

    fn detect_installation(
        &self,
        configured_exe: Option<&Path>,
    ) -> Result<EmulatorInstallation, DomainError> {
        let exe = configured_exe.ok_or_else(|| {
            DomainError::NotFound(
                "nenhum caminho de executável rpcs3.exe foi configurado pelo usuário".to_string(),
            )
        })?;

        let valid_exe = SecurityManager::validate_existing_file(exe)?;
        let root_dir = valid_exe
            .parent()
            .ok_or_else(|| {
                DomainError::Validation(
                    "não foi possível determinar o diretório raiz do RPCS3".to_string(),
                )
            })?
            .to_path_buf();

        let mut issues = Vec::new();
        let file_name = valid_exe
            .file_name()
            .and_then(|s| s.to_str())
            .map(str::to_ascii_lowercase)
            .unwrap_or_default();
        if file_name != "rpcs3.exe" {
            issues.push(format!(
                "Nome de executável incomum '{file_name}' (esperado 'rpcs3.exe'). Verifique se apontou para o binário correto."
            ));
        }

        let detected_version = self.verify_version(&valid_exe)?;

        Ok(EmulatorInstallation {
            emulator_slug: self.slug().to_string(),
            executable_path: valid_exe,
            root_dir,
            detected_version,
            is_valid: true,
            issues,
        })
    }

    fn verify_version(&self, exe_path: &Path) -> Result<Option<String>, DomainError> {
        // Phase 1 preparation NEVER spawns the emulator binary just to probe version (RPCS3 GUI builds may open a window).
        // Inspect nearby version/log metadata read-only when present.
        let valid = SecurityManager::validate_existing_file(exe_path)?;
        if let Some(root) = valid.parent() {
            let log_path = root.join("RPCS3.log");
            if log_path.is_file() {
                if let Ok(mut file) = File::open(&log_path) {
                    let mut buf = vec![0u8; 512];
                    if let Ok(read_len) = file.read(&mut buf) {
                        let header = String::from_utf8_lossy(&buf[..read_len]);
                        if let Some(first_line) = header.lines().next() {
                            if first_line.contains("RPCS3") {
                                return Ok(Some(first_line.trim().to_string()));
                            }
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    fn check_dependencies(&self, installation: &EmulatorInstallation) -> DependencyReport {
        let mut missing = Vec::new();
        let mut notes = Vec::new();
        if !installation.executable_path.is_file() {
            missing.push("rpcs3.exe".to_string());
        }
        let sys = self.validate_system_files(installation);
        if !sys.system_files_present {
            missing.extend(sys.missing_paths);
        }
        notes.push(
            "Na Fase 1, a verificação é puramente estática/filesystem (sem execução ao vivo)."
                .to_string(),
        );
        DependencyReport {
            all_satisfied: missing.is_empty(),
            missing,
            notes,
        }
    }

    fn validate_firmware_package(
        &self,
        package_path: &Path,
    ) -> Result<FirmwarePackageReport, DomainError> {
        let valid_path = SecurityManager::validate_existing_file(package_path)?;
        let (sha256, size_bytes) = SecurityManager::compute_sha256_streaming(&valid_path)?;
        if size_bytes == 0 {
            return Err(DomainError::Validation(format!(
                "arquivo de firmware '{}' está vazio (0 bytes)",
                valid_path.display()
            )));
        }

        let mut file = File::open(&valid_path).map_err(|e| {
            DomainError::Io(format!(
                "falha ao ler cabeçalho de '{}': {e}",
                valid_path.display()
            ))
        })?;
        let mut header = [0u8; 8];
        let read_len = file.read(&mut header).map_err(|e| {
            DomainError::Io(format!(
                "falha ao inspecionar cabeçalho de '{}': {e}",
                valid_path.display()
            ))
        })?;

        let header_valid = read_len >= PS3_PUP_MAGIC.len() && &header[..5] == PS3_PUP_MAGIC;
        let (validation_status, note) = if header_valid {
            (
                "package_header_and_hash_validated".to_string(),
                "Assinatura de cabeçalho SCEUF e hash SHA-256 validados localmente. Isto NÃO confirma a instalação no dev_flash nem a execução em runtime do XMB.".to_string(),
            )
        } else {
            (
                "unrecognized_pup_header".to_string(),
                "Cabeçalho SCEUF não encontrado nos primeiros bytes. Forneça um arquivo PS3UPDAT.PUP legítimo.".to_string(),
            )
        };

        Ok(FirmwarePackageReport {
            file_path: valid_path,
            file_size_bytes: size_bytes,
            sha256,
            header_valid,
            validation_status,
            notes: vec![note],
        })
    }

    fn validate_system_files(&self, installation: &EmulatorInstallation) -> SystemFilesReport {
        let vsh_self = Self::vsh_self_path(&installation.root_dir);
        let dev_hdd0 = Self::dev_hdd0_path(&installation.root_dir);
        let trophy_dir = dev_hdd0.join("home").join("00000001").join("trophy");

        let mut found_paths = Vec::new();
        let mut missing_paths = Vec::new();
        let mut notes = Vec::new();

        if vsh_self.is_file() {
            found_paths.push(vsh_self);
        } else {
            missing_paths.push("dev_flash/vsh/module/vsh.self".to_string());
        }

        if dev_hdd0.is_dir() {
            found_paths.push(dev_hdd0);
        } else {
            missing_paths.push("dev_hdd0/".to_string());
        }

        if trophy_dir.is_dir() {
            found_paths.push(trophy_dir);
            notes.push("Diretório local de troféus detectado (leitura somente leitura planejada para Fase 4).".to_string());
        }

        let system_files_present = missing_paths.is_empty();
        if !system_files_present {
            notes.push(
                "Instale seu próprio PS3UPDAT.PUP obtido legalmente dentro do RPCS3 antes de tentar o boot do XMB."
                    .to_string(),
            );
        }

        SystemFilesReport {
            system_files_present,
            found_paths,
            missing_paths,
            notes,
        }
    }

    fn evaluate_boot_readiness(
        &self,
        installation: &EmulatorInstallation,
        firmware_report: Option<&FirmwarePackageReport>,
    ) -> SystemBootReadiness {
        let firmware_package_validated = firmware_report.is_some_and(|r| r.header_valid);
        let firmware_sha256 = firmware_report.map(|r| r.sha256.clone());

        let sys_report = self.validate_system_files(installation);
        let cmd_result = self.build_system_boot_command(installation);

        let mut notes = Vec::new();
        if firmware_package_validated {
            notes.push(
                "Estágio 1/4: Pacote PS3UPDAT.PUP validado localmente (cabeçalho + SHA-256)."
                    .to_string(),
            );
        } else {
            notes.push("Estágio 1/4: Nenhum pacote PS3UPDAT.PUP validado registrado.".to_string());
        }

        if sys_report.system_files_present {
            notes.push("Estágio 2/4: Arquivos de sistema encontrados (dev_flash/vsh/module/vsh.self e dev_hdd0).".to_string());
        } else {
            notes.push(format!(
                "Estágio 2/4: Arquivos de sistema ausentes: {}.",
                sys_report.missing_paths.join(", ")
            ));
        }

        let (command_buildable, generated_command) = match cmd_result {
            Ok(cmd) => {
                notes.push(
                    "Estágio 3/4: Comando seguro de inicialização do XMB gerado sem shell."
                        .to_string(),
                );
                (true, Some(cmd))
            }
            Err(err) => {
                notes.push(format!("Estágio 3/4: Comando indisponível ({err})."));
                (false, None)
            }
        };

        // Mandatory Adjustment #2 & Option A: NEVER claim XMB runtime boot confirmed before Phase 3 real validation.
        notes.push(
            "Estágio 4/4: Execução real do XMB NÃO confirmada nas Fases 1–2 (lançamento ao vivo, monitoramento e retorno ao Hub pertencem à Fase 3)."
                .to_string(),
        );

        SystemBootReadiness {
            firmware_package_validated,
            firmware_sha256,
            system_files_present: sys_report.system_files_present,
            command_buildable,
            xmb_runtime_confirmed: false,
            generated_command,
            notes,
        }
    }

    fn build_system_boot_command(
        &self,
        installation: &EmulatorInstallation,
    ) -> Result<LaunchCommand, DomainError> {
        let vsh_self = Self::vsh_self_path(&installation.root_dir);
        if !vsh_self.is_file() {
            return Err(DomainError::FirmwareMissing(format!(
                "vsh.self não encontrado em '{}' — instale seu PS3UPDAT.PUP legalmente no RPCS3 primeiro",
                vsh_self.display()
            )));
        }

        LaunchCommand::new(
            installation.executable_path.clone(),
            vec![
                "--no-gui".to_string(),
                vsh_self.to_string_lossy().to_string(),
            ],
            Some(installation.root_dir.clone()),
        )
    }

    fn build_game_boot_command(
        &self,
        installation: &EmulatorInstallation,
        game_path: &Path,
    ) -> Result<LaunchCommand, DomainError> {
        let valid_game = SecurityManager::validate_existing_file(game_path)?;
        LaunchCommand::new(
            installation.executable_path.clone(),
            vec![
                "--no-gui".to_string(),
                valid_game.to_string_lossy().to_string(),
            ],
            Some(installation.root_dir.clone()),
        )
    }

    fn parse_exit_status(&self, exit_code: Option<i32>, stderr_tail: &str) -> SessionOutcome {
        match exit_code {
            Some(0) => SessionOutcome {
                exit_code,
                clean_exit: true,
                diagnostic: "RPCS3 encerrado normalmente (código 0).".to_string(),
            },
            Some(code) => SessionOutcome {
                exit_code,
                clean_exit: false,
                diagnostic: if stderr_tail.trim().is_empty() {
                    format!("RPCS3 encerrou com código não-zero ({code}).")
                } else {
                    format!("RPCS3 encerrou com código {code}: {}", stderr_tail.trim())
                },
            },
            None => SessionOutcome {
                exit_code: None,
                clean_exit: false,
                diagnostic: "RPCS3 foi encerrado sem código de saída (sinal/término forçado)."
                    .to_string(),
            },
        }
    }
}

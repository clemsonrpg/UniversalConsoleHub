use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::console::ConsoleCapabilities;
use super::error::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchCommand {
    pub executable: PathBuf,
    pub args: Vec<String>,
    pub working_dir: Option<PathBuf>,
}

const DISALLOWED_SHELLS: &[&str] = &[
    "cmd",
    "cmd.exe",
    "powershell",
    "powershell.exe",
    "pwsh",
    "pwsh.exe",
    "wscript.exe",
    "cscript.exe",
];

impl LaunchCommand {
    pub fn new(
        executable: PathBuf,
        args: Vec<String>,
        working_dir: Option<PathBuf>,
    ) -> Result<Self, DomainError> {
        let exe_str = executable.to_string_lossy();
        if exe_str.trim().is_empty() {
            return Err(DomainError::UnsafeCommand(
                "executable path cannot be empty".to_string(),
            ));
        }
        if exe_str.contains('\0') {
            return Err(DomainError::UnsafeCommand(
                "executable path cannot contain NUL bytes".to_string(),
            ));
        }
        let file_name = executable
            .file_name()
            .and_then(|n| n.to_str())
            .map(str::to_ascii_lowercase)
            .unwrap_or_default();
        if DISALLOWED_SHELLS.contains(&file_name.as_str()) {
            return Err(DomainError::UnsafeCommand(format!(
                "direct shell invocation is prohibited: {file_name}"
            )));
        }
        for arg in &args {
            if arg.contains('\0') {
                return Err(DomainError::UnsafeCommand(
                    "command argument cannot contain NUL bytes".to_string(),
                ));
            }
        }
        Ok(Self {
            executable,
            args,
            working_dir,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmulatorInstallation {
    pub emulator_slug: String,
    pub executable_path: PathBuf,
    pub root_dir: PathBuf,
    pub detected_version: Option<String>,
    pub is_valid: bool,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyReport {
    pub all_satisfied: bool,
    pub missing: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirmwarePackageReport {
    pub file_path: PathBuf,
    pub file_size_bytes: u64,
    pub sha256: String,
    pub header_valid: bool,
    pub validation_status: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemFilesReport {
    pub system_files_present: bool,
    pub found_paths: Vec<PathBuf>,
    pub missing_paths: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemBootReadiness {
    pub firmware_package_validated: bool,
    pub firmware_sha256: Option<String>,
    pub system_files_present: bool,
    pub command_buildable: bool,
    pub xmb_runtime_confirmed: bool,
    pub generated_command: Option<LaunchCommand>,
    pub notes: Vec<String>,
}

impl SystemBootReadiness {
    pub fn can_claim_confirmed_authentic_boot(&self) -> bool {
        self.system_files_present && self.command_buildable && self.xmb_runtime_confirmed
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionOutcome {
    pub exit_code: Option<i32>,
    pub clean_exit: bool,
    pub diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmulatorRecord {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub console_id: String,
    pub console_slug: String,
    pub executable_path: Option<String>,
    pub config_path: Option<String>,
    pub detected_version: Option<String>,
    pub integration_phase: String,
    pub capabilities: ConsoleCapabilities,
}

pub trait EmulatorAdapter: Send + Sync {
    fn slug(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn console_slug(&self) -> &'static str;
    fn capabilities(&self) -> ConsoleCapabilities;
    fn detect_installation(
        &self,
        configured_exe: Option<&Path>,
    ) -> Result<EmulatorInstallation, DomainError>;
    fn verify_version(&self, exe_path: &Path) -> Result<Option<String>, DomainError>;
    fn check_dependencies(&self, installation: &EmulatorInstallation) -> DependencyReport;
    fn validate_firmware_package(
        &self,
        package_path: &Path,
    ) -> Result<FirmwarePackageReport, DomainError>;
    fn validate_system_files(&self, installation: &EmulatorInstallation) -> SystemFilesReport;
    fn evaluate_boot_readiness(
        &self,
        installation: &EmulatorInstallation,
        firmware_report: Option<&FirmwarePackageReport>,
    ) -> SystemBootReadiness;
    fn build_system_boot_command(
        &self,
        installation: &EmulatorInstallation,
    ) -> Result<LaunchCommand, DomainError>;
    fn build_game_boot_command(
        &self,
        installation: &EmulatorInstallation,
        game_path: &Path,
    ) -> Result<LaunchCommand, DomainError>;
    fn parse_exit_status(&self, exit_code: Option<i32>, stderr_tail: &str) -> SessionOutcome;
}

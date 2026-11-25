use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CapabilityState {
    FullSystemBoot,
    OriginalDashboard,
    PartialDashboard,
    DirectGameOnly,
    Experimental,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureKey {
    SystemBoot,
    OriginalDashboard,
    DirectGameBoot,
    SaveManagement,
    LocalTrophies,
    OnlineServices,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureAvailability {
    Available,
    RequiresUserFiles,
    Partial,
    Experimental,
    Planned,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationLevel {
    Unverified,
    DocumentedUpstream,
    FilesVerified,
    RuntimeConfirmed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BootMode {
    SystemBoot,
    DirectGameBoot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureCapability {
    pub feature: FeatureKey,
    pub states: Vec<CapabilityState>,
    pub availability: FeatureAvailability,
    pub verification: VerificationLevel,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleCapabilities {
    pub supported_states: Vec<CapabilityState>,
    pub features: Vec<FeatureCapability>,
}

impl ConsoleCapabilities {
    pub fn has_state(&self, state: CapabilityState) -> bool {
        self.supported_states.contains(&state)
    }

    pub fn feature(&self, key: FeatureKey) -> Option<&FeatureCapability> {
        self.features.iter().find(|item| item.feature == key)
    }

    pub fn is_runtime_confirmed(&self, key: FeatureKey) -> bool {
        self.feature(key)
            .is_some_and(|item| item.verification == VerificationLevel::RuntimeConfirmed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleRecord {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub manufacturer: String,
    pub generation: i64,
    pub release_year: i64,
    pub phase_status: String,
    pub capabilities: ConsoleCapabilities,
    pub notes: String,
}

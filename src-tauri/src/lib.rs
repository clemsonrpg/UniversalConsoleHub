pub mod adapters;
pub mod domain;
pub mod infrastructure;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HubHealthDto {
    pub app_name: String,
    pub version: String,
    pub phase: u8,
}

mod commands {
    use super::HubHealthDto;

    #[tauri::command]
    pub fn ping_hub() -> HubHealthDto {
        HubHealthDto {
            app_name: "Universal Console Hub".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            phase: 1,
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![commands::ping_hub])
        .run(tauri::generate_context!())
        .expect("error while running Universal Console Hub");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_hub_returns_phase_one_health() {
        let health = commands::ping_hub();
        assert_eq!(health.app_name, "Universal Console Hub");
        assert_eq!(health.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(health.phase, 1);
    }
}

use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use orbit_core::manifest::detect_installed_modules;
use orbit_core::module_registry::modules;

use crate::AppState;

type CmdResult<T> = std::result::Result<T, String>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleDto {
    pub id: String,
    pub name: String,
    pub description: String,
    pub phase: u8,
    pub enabled: bool,
    pub implemented: bool,
    pub required_permissions: Vec<String>,
}

#[tauri::command]
pub fn list_modules(state: State<'_, AppState>) -> CmdResult<Vec<ModuleDto>> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let settings = db.load_settings().map_err(|e| e.to_string())?;
    let disabled = &settings.disabled_modules;
    // Honest: phase 1 ships the shell + island; nothing else is marked implemented.
    Ok(modules()
        .into_iter()
        .map(|m| ModuleDto {
            enabled: !disabled.contains(&m.id),
            id: m.id,
            name: m.name,
            description: m.description,
            phase: m.phase,
            implemented: m.phase == 1,
            required_permissions: m.required_permissions,
        })
        .collect())
}

/// Scan the app-data `modules/` directory for installed module manifests.
/// Honest states: Ok / Refused / Invalid — never silently dropped, never
/// auto-granted. The host does not launch or trust modules yet.
#[tauri::command]
pub fn list_installed_modules(app: AppHandle) -> CmdResult<Vec<orbit_core::manifest::InstalledModule>> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app data dir unavailable: {e}"))?
        .join("modules");
    Ok(detect_installed_modules(&dir))
}

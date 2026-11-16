use serde::Serialize;
use tauri::State;

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
            id: m.id,
            name: m.name,
            description: m.description,
            phase: m.phase,
            enabled: !disabled.contains(&m.id),
            implemented: m.phase == 1,
            required_permissions: m.required_permissions,
        })
        .collect())
}

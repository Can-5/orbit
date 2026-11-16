use tauri::State;

use orbit_core::settings::AppSettings;

use crate::AppState;

type CmdResult<T> = std::result::Result<T, String>;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> CmdResult<AppSettings> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.load_settings().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: AppSettings) -> CmdResult<AppSettings> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.save_settings(&settings).map_err(|e| e.to_string())
}

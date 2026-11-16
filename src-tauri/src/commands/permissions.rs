use serde::Serialize;
use tauri::State;

use orbit_core::permissions::Permission;

use crate::AppState;

type CmdResult<T> = std::result::Result<T, String>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionDto {
    pub id: String,
    pub granted: bool,
    pub updated_at: Option<i64>,
    pub label: String,
    pub description: String,
}

#[tauri::command]
pub fn list_permissions(state: State<'_, AppState>) -> CmdResult<Vec<PermissionDto>> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let statuses = db.list_permissions().map_err(|e| e.to_string())?;
    Ok(statuses
        .into_iter()
        .map(|s| {
            let perm: Permission = s.id.parse().unwrap_or(Permission::ClipboardRead);
            let (label, description) = perm.description();
            PermissionDto {
                id: s.id,
                granted: s.granted,
                updated_at: s.updated_at,
                label: label.to_string(),
                description: description.to_string(),
            }
        })
        .collect())
}

#[tauri::command]
pub fn set_permission(
    state: State<'_, AppState>,
    id: String,
    granted: bool,
) -> CmdResult<PermissionDto> {
    let permission: Permission = id.parse().map_err(|e: orbit_core::OrbitError| e.to_string())?;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let status = db.set_permission(permission, granted).map_err(|e| e.to_string())?;
    let (label, description) = permission.description();
    Ok(PermissionDto {
        id: status.id,
        granted: status.granted,
        updated_at: status.updated_at,
        label: label.to_string(),
        description: description.to_string(),
    })
}

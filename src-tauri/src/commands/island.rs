//! IPC surface for Orbit Island. All state lives in the orbit-core
//! `IslandManager`; these handlers validate input and forward intents.

use tauri::{Emitter, State};

use orbit_core::island::{IslandContent, IslandSnapshot, IslandView, Priority};

use crate::AppState;

type CmdResult<T> = std::result::Result<T, String>;

fn broadcast(app: &tauri::AppHandle, snapshot: &IslandSnapshot) {
    // Both the main window and the (optional) overlay window receive this.
    let _ = app.emit_to("main", "orbit://island-changed", snapshot);
    let _ = app.emit_to("island", "orbit://island-changed", snapshot);
}

fn push_and_broadcast(
    app: &tauri::AppHandle,
    state: &AppState,
    content: IslandContent,
    priority: Priority,
    auto_dismiss_ms: Option<u64>,
) -> CmdResult<IslandSnapshot> {
    let snapshot = {
        let mut island = state.island.lock().map_err(|e| e.to_string())?;
        island.push(content, priority, auto_dismiss_ms);
        island.snapshot()
    };
    broadcast(app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub fn island_snapshot(state: State<'_, AppState>) -> CmdResult<IslandSnapshot> {
    let island = state.island.lock().map_err(|e| e.to_string())?;
    Ok(island.snapshot())
}

#[tauri::command]
pub fn island_tick(app: tauri::AppHandle, state: State<'_, AppState>) -> CmdResult<IslandSnapshot> {
    let snapshot = {
        let mut island = state.island.lock().map_err(|e| e.to_string())?;
        island.tick()
    };
    broadcast(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub fn island_push_toast(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    message: String,
) -> CmdResult<IslandSnapshot> {
    let message = message.trim();
    if message.is_empty() {
        return Err("message must not be empty".into());
    }
    if message.chars().count() > 140 {
        return Err("message too long (max 140 chars)".into());
    }
    push_and_broadcast(
        &app,
        &state,
        IslandContent::Toast {
            message: message.to_string(),
        },
        Priority::Normal,
        Some(6_000),
    )
}

#[tauri::command]
pub fn island_dismiss(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: u64,
) -> CmdResult<IslandSnapshot> {
    let snapshot = {
        let mut island = state.island.lock().map_err(|e| e.to_string())?;
        island.dismiss(id);
        island.snapshot()
    };
    broadcast(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub fn island_dismiss_active(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> CmdResult<IslandSnapshot> {
    let snapshot = {
        let mut island = state.island.lock().map_err(|e| e.to_string())?;
        island.dismiss_active();
        island.snapshot()
    };
    broadcast(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub fn island_toggle_expand(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> CmdResult<IslandSnapshot> {
    let snapshot = {
        let mut island = state.island.lock().map_err(|e| e.to_string())?;
        island.toggle_expand();
        island.snapshot()
    };
    broadcast(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub fn island_set_expanded(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    expanded: bool,
) -> CmdResult<IslandSnapshot> {
    let snapshot = {
        let mut island = state.island.lock().map_err(|e| e.to_string())?;
        island.set_expanded(expanded);
        island.snapshot()
    };
    broadcast(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub fn island_start_timer(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    label: String,
    duration_secs: u64,
) -> CmdResult<IslandSnapshot> {
    if duration_secs == 0 || duration_secs > 8 * 60 * 60 {
        return Err("duration_secs must be between 1 and 28800".into());
    }
    let label = label.trim();
    if label.is_empty() || label.chars().count() > 40 {
        return Err("label must be 1–40 characters".into());
    }
    let snapshot = {
        let mut island = state.island.lock().map_err(|e| e.to_string())?;
        island.start_timer(label, duration_secs);
        island.snapshot()
    };
    broadcast(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub fn island_cancel_timer(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> CmdResult<IslandSnapshot> {
    let snapshot = {
        let mut island = state.island.lock().map_err(|e| e.to_string())?;
        island.cancel_timer();
        island.snapshot()
    };
    broadcast(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub fn island_toggle_timer_pause(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> CmdResult<IslandSnapshot> {
    let snapshot = {
        let mut island = state.island.lock().map_err(|e| e.to_string())?;
        island.toggle_timer_pause();
        island.snapshot()
    };
    broadcast(&app, &snapshot);
    Ok(snapshot)
}

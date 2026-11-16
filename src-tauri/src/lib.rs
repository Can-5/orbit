//! Orbit Tauri shell.
//!
//! Thin layer: opens the SQLite database, wires IPC commands to
//! `orbit-core`, and owns window/platform decisions. No business logic here.

mod commands;
#[cfg(target_os = "linux")]
mod downloads;
#[cfg(target_os = "linux")]
mod mpris;
#[cfg(target_os = "linux")]
mod brightness;
#[cfg(target_os = "linux")]
mod sysmon;
mod platform;

use std::sync::Mutex;

use orbit_core::{island::IslandManager, settings::IslandMode, storage::Database};
use tauri::Manager;

/// Shared application state. All locks are short-lived; commands clone what they need.
pub struct AppState {
    pub db: Mutex<Database>,
    pub island: Mutex<IslandManager>,
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("cannot resolve app data dir: {e}"))?;
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("orbit.db");
            let db = Database::open(&db_path)
                .map_err(|e| format!("cannot open database at {}: {e}", db_path.display()))?;
            let settings = db.load_settings().unwrap_or_default();
            app.manage(AppState {
                db: Mutex::new(db),
                island: Mutex::new(IslandManager::new()),
            });

            // Overlay island only when enabled/allowed. On Linux the auto mode
            // prefers the honest in-app island; overlay must be opted into.
            let wants_overlay = settings.island.enabled
                && match settings.island.mode {
                    IslandMode::InApp => false,
                    IslandMode::Overlay => true,
                    IslandMode::Auto => platform::overlay_island_supported(),
                };
            if wants_overlay {
                if let Some(win) = app.get_webview_window("island") {
                    // Floating pill at the top-center of the primary monitor,
                    // like a desktop take on the dynamic island.
                    if let Ok(Some(monitor)) = win.primary_monitor() {
                        let monitor_size = monitor.size();
                        let monitor_origin = monitor.position();
                        let window_size = win
                            .outer_size()
                            .unwrap_or_else(|_| tauri::PhysicalSize::new(480, 44));
                        let x = monitor_origin.x + (monitor_size.width as i32 - window_size.width as i32) / 2;
                        let _ = win.set_position(tauri::PhysicalPosition::new(x, monitor_origin.y));
                    }
                    let _ = win.show();
                }
            }

            // Ambient island feeds (Linux only, opt-in via the island setting).
            // MPRIS now-playing + download-dir metadata watcher: no content is
            // read, no file is modified, no network is used.
            #[cfg(target_os = "linux")]
            if settings.island.enabled {
                let handle = app.handle().clone();
                mpris::start(handle.clone());
                downloads::start(handle.clone());
                if settings.island.system_stats {
                    sysmon::start(handle);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::permissions::list_permissions,
            commands::permissions::set_permission,
            commands::capabilities::get_capabilities,
            commands::modules::list_modules,
            commands::modules::list_installed_modules,
            commands::app::app_info,
            commands::island::island_snapshot,
            commands::island::island_tick,
            commands::island::island_push_toast,
            commands::island::island_dismiss,
            commands::island::island_dismiss_active,
            commands::island::island_toggle_expand,
            commands::island::island_set_expanded,
            commands::island::island_start_timer,
            commands::island::island_cancel_timer,
            commands::island::island_toggle_timer_pause,
            commands::island::island_drop_files,
            commands::island::island_clear_dropped_files,
            commands::island::island_media_toggle,
            commands::island::island_media_next,
            commands::island::island_media_prev,
            commands::island::island_set_media_volume,
            commands::island::island_brightness_get,
            commands::island::island_brightness_set,
        ])
        .build(tauri::generate_context!())
        .expect("error while building Orbit")
        .run(|_app, _event| {});
}

//! Download directory watcher — best effort, zero content reads.
//!
//! Watches the XDG download directory with plain std polling (~1 Hz). For every
//! recently-modified regular file it tracks a `DownloadTask` in the island: label
//! = file name, received = size on disk, total = unknown until complete.
//! ORBIT NEVER OPENS, COPIES OR DELETES ANY FILE HERE — metadata only.
//! Browser downloads appear here because browsers write files (or .part files)
//! into this directory — no browser-specific API is needed and none is used.
//!
//! Detection rules (honest, may mislabel):
//!   - size changed between ticks → download in flight, upsert task
//!   - size unchanged for 3 consecutive ticks → considered complete
//!   - file disappeared (e.g. .part renamed to final) → considered complete
//! Nothing is tracked unless its mtime is younger than STALE_AFTER_SECS or it is
//! actively growing. Useless to track years-old files in the folder.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Manager};

use orbit_core::island::DownloadTask;

use crate::commands;
use crate::AppState;

const POLL_INTERVAL: Duration = Duration::from_secs(1);
const STALE_AFTER_SECS: u64 = 30 * 60; // ignore files older than this unless growing
const MAX_TRACKED: usize = 64;         // hard cap, oldest-done evicted first
const COMPLETE_AFTER_TICKS: u32 = 3;   // size must be stable this many ticks

#[derive(Clone)]
struct Tracked {
    label: String,
    size: u64,
    unchanged_ticks: u32,
    done: bool,
    last_change_ms: u64,
}

pub fn start(app: AppHandle) {
    let dir = resolve_download_dir();
    eprintln!("orbit: watching download dir {} (metadata only)", dir.display());
    std::thread::spawn(move || ambient_loop(app, dir));
}

/// Read XDG_DOWNLOAD_DIR from ~/.config/user-dirs.dirs (values like "$HOME/İndirilenler").
/// Falls back to ~/İndirilenler then ~/Downloads; worst case, current dir.
fn resolve_download_dir() -> PathBuf {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    if let Some(home) = &home {
        let conf = home.join(".config/user-dirs.dirs");
        if let Ok(content) = fs::read_to_string(&conf) {
            if let Some(raw) = parse_xdg_key(&content, "XDG_DOWNLOAD_DIR") {
                let replaced = raw.replace("$HOME", &home.to_string_lossy());
                let p = PathBuf::from(replaced);
                if p.is_dir() {
                    return p;
                }
            }
        }
        for fallback in ["İndirilenler", "Downloads"] {
            let p = home.join(fallback);
            if p.is_dir() {
                return p;
            }
        }
        return home.clone();
    }
    PathBuf::from(".")
}

/// Extract `KEY="value"` from user-dirs.dirs content. Handles quotes and optional spaces.
fn parse_xdg_key(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || !line.starts_with(key) {
            continue;
        }
        let eq = line.find('=')?;
        let value = line[eq + 1..].trim();
        let unquoted = value.trim_matches('"');
        if !unquoted.is_empty() {
            return Some(unquoted.to_string());
        }
    }
    None
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn ambient_loop(app: AppHandle, dir: PathBuf) {
    let mut tracked: HashMap<String, Tracked> = HashMap::new(); // key: absolute path as string
    loop {
        std::thread::sleep(POLL_INTERVAL);
        let mut seen: HashSet<String> = HashSet::new();

        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue; // top-level files only; never recurse, never open
                }
                let Some(name) = path.file_name().and_then(|n| n.to_str()) else { continue };
                if name.starts_with('.') {
                    continue; // hidden files (e.g. browser internals) stay invisible
                }
                let Ok(meta) = entry.metadata() else { continue };
                let size = meta.len();
                let key = path.to_string_lossy().into_owned();
                seen.insert(key.clone());

                let fresh = meta
                    .modified()
                    .ok()
                    .and_then(|m| m.duration_since(UNIX_EPOCH).ok())
                    .map(|d| now_ms().saturating_sub(d.as_millis() as u64) < STALE_AFTER_SECS * 1000)
                    .unwrap_or(false);

                match tracked.get_mut(&key) {
                    Some(t) => {
                        if size != t.size {
                            t.size = size;
                            t.unchanged_ticks = 0;
                            t.last_change_ms = now_ms();
                            push_download(&app, &key, &t.label, size, false);
                        } else if !t.done {
                            t.unchanged_ticks += 1;
                            if t.unchanged_ticks >= COMPLETE_AFTER_TICKS {
                                t.done = true;
                                mark_done(&app, &key);
                            }
                        }
                    }
                    None => {
                        if fresh {
                            if tracked.len() >= MAX_TRACKED {
                                // evict oldest DONE entries first to make room
                                evict_oldest_done(&mut tracked);
                            }
                            if tracked.len() < MAX_TRACKED {
                                tracked.insert(
                                    key.clone(),
                                    Tracked {
                                        label: name.to_string(),
                                        size,
                                        unchanged_ticks: 0,
                                        done: false,
                                        last_change_ms: now_ms(),
                                    },
                                );
                                push_download(&app, &key, name, size, false);
                            }
                        }
                    }
                }
            }
        }

        // Any tracked path that vanished → treat as complete (browser renames .part → final).
        for (key, t) in tracked.iter_mut() {
            if !t.done && !seen.contains(key) {
                t.done = true;
                mark_done(&app, key);
            }
        }

        // Drop ancient done entries so the label table doesn't grow forever.
        tracked.retain(|_, t| !t.done || now_ms().saturating_sub(t.last_change_ms) < STALE_AFTER_SECS * 1000);
    }
}

fn push_download(app: &AppHandle, id: &str, label: &str, received_bytes: u64, done: bool) {
    let state = app.state::<AppState>();
    if let Ok(mut island) = state.island.lock() {
        island.upsert_download(DownloadTask {
            id: id.to_string(),
            label: label.to_string(),
            received_bytes,
            total_bytes: None, // total size is unknown until complete — never guessed
            done,
        });
    }
    commands::island::broadcast_snapshot(app, &state);
}

fn mark_done(app: &AppHandle, id: &str) {
    let state = app.state::<AppState>();
    if let Ok(mut island) = state.island.lock() {
        island.mark_download_done(id);
    }
    commands::island::broadcast_snapshot(app, &state);
}

fn evict_oldest_done(tracked: &mut HashMap<String, Tracked>) {
    // Remove the oldest DONE entry (by last_change_ms) to make room.
    let oldest_done = tracked
        .iter()
        .filter(|(_, t)| t.done)
        .min_by_key(|(_, t)| t.last_change_ms)
        .map(|(k, _)| k.clone());
    if let Some(key) = oldest_done {
        tracked.remove(&key);
    }
}

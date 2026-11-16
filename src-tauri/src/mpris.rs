//! MPRIS now-playing bridge — best effort, honest about everything it can't do.
//!
//! Reads the freedesktop.org MPRIS D-Bus interface via the `gdbus` tool, which
//! ships with GNOME/KDE (glib). No extra packages are installed or required.
//! If `gdbus` is missing, no MPRIS player is running, or the player misbehaves,
//! the island simply shows nothing — unavailable is reported by omission, never
//! by inventing data. Nothing is read beyond the player's public metadata:
//! playback status, title, artist, length, position. No content, no accounts,
//! no network traffic, no telemetry.

use std::process::{Command, Stdio};
use std::time::Duration;

use tauri::{AppHandle, Manager};

use orbit_core::island::NowPlaying;

use crate::commands;
use crate::AppState;

const POLL_INTERVAL: Duration = Duration::from_secs(2);

pub fn start(app: AppHandle) {
    if !gdbus_available() {
        eprintln!("orbit: gdbus not available — now-playing bridge disabled (honest omission)");
        return;
    }
    std::thread::spawn(move || ambient_loop(app));
}

fn gdbus_available() -> bool {
    Command::new("gdbus")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn gdbus_stdout(args: &[&str]) -> Option<String> {
    let out = Command::new("gdbus")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8(out.stdout).ok()
}

fn gdbus_call(dest: &str, method: &str, args: &[&str]) -> Option<String> {
    let mut full = vec![
        "call",
        "--session",
        "--print-reply",
        "--dest",
        dest,
        "--object-path",
        if dest == "org.freedesktop.DBus" {
            "/org/freedesktop/DBus"
        } else {
            "/org/mpris/MediaPlayer2"
        },
        "--method",
        method,
    ];
    full.extend_from_slice(args);
    gdbus_stdout(&full)
}

fn ambient_loop(app: AppHandle) {
    let mut last: Option<NowPlaying> = None;
    loop {
        std::thread::sleep(POLL_INTERVAL);
        let current = read_now_playing();
        if current != last {
            last = current.clone();
            let state = app.state::<AppState>();
            if let Ok(mut island) = state.island.lock() {
                island.set_now_playing(current.clone());
            }
            commands::island::broadcast_snapshot(&app, &state);
        }
    }
}

/// Returns the best-effort now-playing snapshot, or None when the MPRIS
/// interface cannot be read (no player, missing tool, unsupported player).
fn read_now_playing() -> Option<NowPlaying> {
    let names = gdbus_call(
        "org.freedesktop.DBus",
        "org.freedesktop.DBus.ListNames",
        &[],
    )?;
    let player = find_player(&names)?;
    let props = gdbus_call(
        &player,
        "org.freedesktop.DBus.Properties.GetAll",
        &["org.mpris.MediaPlayer2.Player"],
    )?;
    parse_player_props(&props, &player)
}

/// Find the first running MPRIS player bus name, if any.
fn find_player(list_names_reply: &str) -> Option<String> {
    // Example reply: (['org.freedesktop.DBus', 'org.mpris.MediaPlayer2.spotify', ...],)
    list_names_reply
        .split('\'')
        .find(|part| part.starts_with("org.mpris.MediaPlayer2."))
        .map(|s| s.to_string())
}

/// Extract a quoted string property from gdbus `--print-reply` output.
/// Example: xesam:title: <'Title'> → Title
fn string_prop(props: &str, key: &str) -> Option<String> {
    let needle = format!("{key}: <'");
    let start = props.find(&needle)? + needle.len();
    let end = props[start..].find('\'')? + start;
    Some(props[start..end].to_string())
}

/// Extract a quoted string array property's first element.
/// Example: xesam:artist: <['Artist A', 'Artist B']> → Artist A
fn first_of_array_prop(props: &str, key: &str) -> Option<String> {
    let needle = format!("{key}: <['");
    let start = props.find(&needle)? + needle.len();
    let end = props[start..].find('\'')? + start;
    Some(props[start..end].to_string())
}

/// Extract an integer property; gdbus prints e.g. `<uint64 232000000>` or `<int64 232000000>`.
fn uint_prop(props: &str, key: &str) -> Option<u64> {
    let needle = format!("{key}: <");
    let start = props.find(&needle)? + needle.len();
    let end = props[start..].find('>')? + start;
    let raw = props[start..end].trim();
    let digits: String = raw
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

/// Extract a floating-point property; gdbus prints e.g. `<0.5>` for Volume.
fn double_prop(props: &str, key: &str) -> Option<f32> {
    let needle = format!("{key}: <");
    let start = props.find(&needle)? + needle.len();
    let end = props[start..].find('>')? + start;
    props[start..end].trim().parse::<f32>().ok()
}

/// The first running MPRIS player's bus name, if any.
pub(crate) fn current_player() -> Option<String> {
    gdbus_call("org.freedesktop.DBus", "org.freedesktop.DBus.ListNames", &[])
        .and_then(|names| find_player(&names))
}

/// Invoke a method on the MPRIS Player interface. Returns the raw reply on success.
pub(crate) fn call_player(player: &str, method: &str, args: &[&str]) -> Option<String> {
    gdbus_call(player, method, args)
}

/// Set the player's Volume (0.0–1.0). Returns false when the player refuses or has no Volume property.
pub(crate) fn set_player_volume(player: &str, volume: f32) -> bool {
    let value = format!("<{volume}>");
    gdbus_call(player, "org.freedesktop.DBus.Properties.Set",
        &["org.mpris.MediaPlayer2.Player", "Volume", &value]).is_some()
}

/// Parse gdbus GetAll output into NowPlaying. Returns None on any unexpected drift.
fn parse_player_props(props: &str, player: &str) -> Option<NowPlaying> {
    // PlaybackStatus: 'Playing' | 'Paused' | 'Stopped'
    let status = string_prop(props, "PlaybackStatus")?;
    if status == "Stopped" {
        return None; // player exists but is idle — show nothing
    }
    let title = string_prop(props, "xesam:title").unwrap_or_else(|| "(untitled)".to_string());
    let artist = first_of_array_prop(props, "xesam:artist").unwrap_or_default();
    let length_secs = uint_prop(props, "mpris:length").map(|us| us / 1_000_000);
    let position_secs = uint_prop(props, "Position").map(|us| us / 1_000_000).unwrap_or(0);

    Some(NowPlaying {
        player: player
            .trim_start_matches("org.mpris.MediaPlayer2.")
            .to_string(),
        title,
        artist,
        position_secs,
        length_secs,
        playing: status == "Playing",
        volume: double_prop(props, "Volume"),
    })
}

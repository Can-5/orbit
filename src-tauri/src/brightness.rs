//! Screen brightness control via logind (`org.freedesktop.login1`).
//!
//! Active session may set its own backlight without root. We call
//! `SetBrightness("backlight", <device>, <raw>)` on the session object.
//! Reads come from /sys/class/backlight/<device>/{brightness,max_brightness}
//! which are world-readable. Nothing is persisted, nothing leaves the device.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn device_dir() -> Option<(String, PathBuf)> {
    let backlights = PathBuf::from("/sys/class/backlight");
    let mut entries: Vec<_> = fs::read_dir(&backlights).ok()?.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());
    entries
        .into_iter()
        .find(|e| e.path().join("max_brightness").exists())
        .map(|e| (e.file_name().to_string_lossy().to_string(), e.path()))
}

pub fn max_raw() -> Option<u32> {
    let (_, dir) = device_dir()?;
    fs::read_to_string(dir.join("max_brightness")).ok()?.trim().parse().ok()
}

pub fn read_raw() -> Option<u32> {
    let (_, dir) = device_dir()?;
    fs::read_to_string(dir.join("brightness")).ok()?.trim().parse().ok()
}

fn session_paths() -> Vec<String> {
    let out = match Command::new("/usr/bin/busctl")
        .args(["--system", "call", "org.freedesktop.login1", "/org/freedesktop/login1",
               "org.freedesktop.login1.Manager", "ListSessions"])
        .output()
    {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).into_owned(),
        _ => return Vec::new(),
    };
    out.split('"')
        .filter(|t| t.starts_with("/org/freedesktop/login1/session/"))
        .map(str::to_string)
        .collect()
}

/// Returns current brightness as 0.0..=1.0, or None if no backlight / unreadable.
pub fn read_percent() -> Option<f32> {
    let max = max_raw()?;
    if max == 0 { return None; }
    Some(read_raw()? as f32 / max as f32)
}

/// Sets brightness from 0.0..=1.0. Values clamped to [0.02, 1.0] to avoid a black screen.
pub fn set_percent(level: f32) -> Result<(), String> {
    if !level.is_finite() {
        return Err("level must be finite".to_string());
    }
    let level = level.clamp(0.02, 1.0);
    let (device, _) = device_dir().ok_or_else(|| "no backlight device found".to_string())?;
    let max = max_raw().ok_or_else(|| "unreadable max_brightness".to_string())? as f32;
    let raw = (level * max).round() as u32;

    // logind allows a session to change its own backlight only. From inside the
    // graphical session this succeeds; we just write what we want. No probing of
    // other sessions, no privilege games.
    let paths = session_paths();
    if paths.is_empty() {
        return Err("no logind sessions visible to this process".to_string());
    }
    let mut last_err = String::new();
    for session in paths {
        let out = Command::new("/usr/bin/busctl")
            .args([
                "--system", "call", "org.freedesktop.login1", session.as_str(),
                "org.freedesktop.login1.Session", "SetBrightness", "ssu",
                "backlight", &device, &raw.to_string(),
            ])
            .output()
            .map_err(|e| format!("cannot run busctl: {e}"))?;
        if out.status.success() {
            return Ok(());
        }
        last_err = String::from_utf8_lossy(&out.stderr).trim().to_string();
    }
    Err(format!("logind SetBrightness refused: {last_err}"))
}

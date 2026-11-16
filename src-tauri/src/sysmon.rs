//! Local CPU/RAM sampler — best effort, procfs only, never leaves the device.
//!
//! Reads /proc/stat (CPU) and /proc/meminfo (RAM) every two seconds and pushes
//! a coarse snapshot into the island. No content, no process names, no network.
//! On the first tick no CPU delta exists yet, so CPU reports 0.0 until the
//! second sample — that is honest, not a placeholder.

use std::fs;
use std::time::Duration;

use tauri::{AppHandle, Manager};

use orbit_core::island::SystemStats;

use crate::commands;
use crate::AppState;

const POLL_INTERVAL: Duration = Duration::from_secs(2);

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let mut prev_cpu: Option<(u64, u64)> = None;
        loop {
            std::thread::sleep(POLL_INTERVAL);
            let cur = cpu_idle_total();
            let cpu_percent = match (prev_cpu, cur) {
                (Some((pi, pt)), Some((ci, ct))) if ct > pt => {
                    let total = (ct - pt) as f32;
                    let idle = ci.saturating_sub(pi) as f32;
                    ((total - idle) / total * 100.0).clamp(0.0, 100.0)
                }
                _ => 0.0,
            };
            prev_cpu = cur;
            if let Some((ram_percent, ram_used_mb, ram_total_mb)) = mem_stats() {
                let stats = SystemStats { cpu_percent, ram_percent, ram_used_mb, ram_total_mb };
                let state = app.state::<AppState>();
                if let Ok(mut island) = state.island.lock() {
                    island.set_system(Some(stats));
                }
                commands::island::broadcast_snapshot(&app, &state);
            }
        }
    });
}

/// Returns (idle_jiffies, total_jiffies) from the aggregate `cpu` line of /proc/stat.
fn cpu_idle_total() -> Option<(u64, u64)> {
    let content = fs::read_to_string("/proc/stat").ok()?;
    let line = content.lines().next()?;
    let fields: Vec<u64> = line
        .strip_prefix("cpu")?
        .split_whitespace()
        .filter_map(|v| v.parse().ok())
        .collect();
    if fields.len() < 5 {
        return None;
    }
    let idle = fields[3] + fields[4]; // idle + iowait
    let total: u64 = fields.iter().sum();
    Some((idle, total))
}

/// Returns (ram_used_percent, ram_used_mb, ram_total_mb) from /proc/meminfo.
fn mem_stats() -> Option<(f32, u64, u64)> {
    let content = fs::read_to_string("/proc/meminfo").ok()?;
    let mut total_kb = None;
    let mut avail_kb = None;
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            total_kb = parse_kb(rest);
        } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
            avail_kb = parse_kb(rest);
        }
        if total_kb.is_some() && avail_kb.is_some() {
            break;
        }
    }
    let total = total_kb?;
    let avail = avail_kb?;
    if total == 0 {
        return None;
    }
    let used = total.saturating_sub(avail);
    Some((used as f32 / total as f32 * 100.0, used / 1024, total / 1024))
}

fn parse_kb(s: &str) -> Option<u64> {
    s.split_whitespace().next()?.parse().ok()
}

//! Orbit Module Protocol v1 — manifest (`orbit-module.json`) parsing and
//! installed-module detection.
//!
//! Manifests are read *before* launch. Unknown fields are rejected, `format`
//! must be 1, and requesting a permission is only a request — the user grants
//! or denies at runtime via the permission registry. No scripts are ever
//! executed from a manifest (`deny_unknown_fields` everywhere).

use crate::permissions::Permission;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleManifest {
    pub format: u64,
    pub id: String,
    pub name: String,
    pub version: String,
    pub protocol: String,
    pub description: String,
    pub permissions: Vec<String>,
    pub island: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum InstallState {
    /// Parsed and valid; the module may be launched after user grants.
    Ok { path: String },
    /// Parsed but refused: format / protocol / id / permissions unacceptable.
    Refused { path: String, reason: String },
    /// Could not be parsed at all (schema violation, I/O, malformed JSON).
    Invalid { path: String, reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledModule {
    pub id: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
    pub manifest_path: String,
    pub requested_permissions: Vec<String>,
    pub state: InstallState,
}

/// Lowercase slug `[a-z0-9-]{1,64}`, not starting/ending with '-'.
pub fn is_slug(s: &str) -> bool {
    let len = s.len();
    (1..=64).contains(&len)
        && !s.starts_with('-')
        && !s.ends_with('-')
        && s.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// Semver-ish `MAJOR.MINOR.PATCH` (digits only). Full semver parsing is out
/// of scope for the host's gate; it only needs to reject garbage.
pub fn is_semverish(s: &str) -> bool {
    let mut parts = s.split('.');
    let (Some(a), Some(b), Some(c)) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    if parts.next().is_some() {
        return false;
    }
    [a, b, c].iter().all(|p| !p.is_empty() && p.bytes().all(|x| x.is_ascii_digit()))
}

/// Validate a parsed manifest against the Protocol v1 rules. Returns Err with
/// a human-readable reason; never panics.
pub fn validate_manifest(m: &ModuleManifest) -> Result<(), String> {
    if m.format != 1 {
        return Err(format!("unsupported manifest format {} (expected 1)", m.format));
    }
    if !is_slug(&m.id) {
        return Err(format!("id '{}' is not a lowercase slug [a-z0-9-]{{1,64}}", m.id));
    }
    if m.protocol != "1" {
        return Err(format!("protocol '{}' not supported (this host speaks 1)", m.protocol));
    }
    if !is_semverish(&m.version) {
        return Err(format!("version '{}' is not MAJOR.MINOR.PATCH", m.version));
    }
    if m.name.trim().is_empty() {
        return Err("name must not be empty".into());
    }
    // Permissions must be known registry ids. Unknown ids are refused: a module
    // requesting a permission the host does not understand cannot be honored.
    for p in &m.permissions {
        if Permission::from_id(p).is_none() {
            return Err(format!("unknown permission '{}'", p));
        }
    }
    Ok(())
}

/// Parse a manifest file. Never panics; IO and JSON errors become `Invalid`.
pub fn parse_manifest(path: &Path) -> Result<ModuleManifest, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read: {e}"))?;
    if bytes.len() > 256 * 1024 {
        return Err("manifest larger than 256 KiB".into());
    }
    serde_json::from_slice(&bytes).map_err(|e| format!("parse: {e}"))
}

/// Scan `<root>/<id>/orbit-module.json` for every installed module.
/// Skips non-directories and entries without a manifest. Never fails on a
/// single bad module — it is reported as `Invalid`/`Refused`.
pub fn detect_installed_modules(root: &Path) -> Vec<InstalledModule> {
    let mut out = Vec::new();
    let Ok(read_dir) = std::fs::read_dir(root) else {
        return out; // root may not exist yet — that is not an error
    };
    for entry in read_dir.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let manifest_path = dir.join("orbit-module.json");
        if !manifest_path.is_file() {
            continue;
        }
        let mp = manifest_path.to_string_lossy().into_owned();
        match parse_manifest(&manifest_path) {
            Err(reason) => out.push(InstalledModule {
                id: None,
                name: None,
                version: None,
                manifest_path: mp.clone(),
                requested_permissions: Vec::new(),
                state: InstallState::Invalid { path: mp, reason },
            }),
            Ok(m) => {
                let state = match validate_manifest(&m) {
                    Ok(()) => InstallState::Ok { path: mp.clone() },
                    Err(reason) => InstallState::Refused { path: mp.clone(), reason },
                };
                out.push(InstalledModule {
                    id: Some(m.id),
                    name: Some(m.name),
                    version: Some(m.version),
                    manifest_path: mp,
                    requested_permissions: m.permissions,
                    state,
                });
            }
        }
    }
    out.sort_by(|a, b| a.manifest_path.cmp(&b.manifest_path));
    out
}

/// Where Orbit looks for installed modules by default.
pub fn default_modules_dir() -> PathBuf {
    // <data_dir>/orbit/modules — callers (the shell) pass the platform app data dir.
    PathBuf::from("modules")
}

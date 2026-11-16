//! Common serializable models shared across the core and the frontend.

use serde::{Deserialize, Serialize};

/// A statically known Orbit module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    /// Stable id, e.g. "ghostdrop". Used in settings, storage and IPC.
    pub id: String,
    /// Display name.
    pub name: String,
    /// One-line purpose.
    pub description: String,
    /// Development phase (1–5) the module ships in.
    pub phase: u8,
    /// Permissions this module may request (still requires explicit grant).
    pub required_permissions: Vec<String>,
}

//! Orbit core application logic.
//!
//! Everything in this crate is platform-independent: no windowing, no GUI,
//! no OS APIs. Platform-specific capabilities are expressed through the
//! [`capabilities`] registry, which reports honest support states per OS.
//! Platform adapters (Linux / Windows / macOS) live in `src-tauri/platform`.

pub mod capabilities;
pub mod error;
pub mod island;
pub mod models;
pub mod module_registry;
pub mod permissions;
pub mod settings;
pub mod storage;

pub use error::{OrbitError, Result};

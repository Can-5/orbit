use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrbitError {
    #[error("storage error: {0}")]
    Storage(#[from] rusqlite::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("permission denied: {permission} — grant it explicitly in Settings → Permissions")]
    PermissionDenied { permission: String },

    #[error("feature not supported on this platform: {feature} ({reason})")]
    Unsupported { feature: String, reason: String },

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, OrbitError>;

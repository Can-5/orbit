//! SQLite storage layer (portable schema — same file works on Android later).
//!
//! - One database, migrations tracked via `PRAGMA user_version`.
//! - Settings stored as a single JSON document (key = 'app').
//! - Permissions table mirrors the explicit-grant model.
//! - No telemetry table. There is no telemetry.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection};
use serde_json;

use crate::error::Result;
use crate::permissions::{Permission, PermissionStatus};
use crate::settings::AppSettings;

const MIGRATIONS: &[&str] = &[
    // v1 — initial schema
    "CREATE TABLE settings (
        key        TEXT PRIMARY KEY,
        value      TEXT NOT NULL,
        updated_at INTEGER NOT NULL
    );
    CREATE TABLE permissions (
        id         TEXT PRIMARY KEY,
        granted    INTEGER NOT NULL CHECK (granted IN (0, 1)),
        updated_at INTEGER NOT NULL
    );",
];

pub struct Database {
    conn: Connection,
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        Self::init(conn)
    }

    /// In-memory database for tests.
    pub fn in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Self> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        let version: i64 = self.conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        for (idx, sql) in MIGRATIONS.iter().enumerate() {
            let target = (idx + 1) as i64;
            if version < target {
                self.conn.execute_batch(sql)?;
                self.conn.pragma_update(None, "user_version", target)?;
            }
        }
        Ok(())
    }

    pub fn schema_version(&self) -> Result<i64> {
        Ok(self.conn.query_row("PRAGMA user_version", [], |r| r.get(0))?)
    }

    // ----- settings -----

    pub fn load_settings(&self) -> Result<AppSettings> {
        let row: Option<String> = self
            .conn
            .query_row("SELECT value FROM settings WHERE key = 'app'", [], |r| r.get(0))
            .ok();
        match row {
            Some(json) => Ok(serde_json::from_str(&json)?),
            None => Ok(AppSettings::default()),
        }
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<AppSettings> {
        let normalized = settings.clone().validated().map_err(crate::error::OrbitError::InvalidInput)?;
        let json = serde_json::to_string(&normalized)?;
        self.conn.execute(
            "INSERT INTO settings (key, value, updated_at) VALUES ('app', ?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            params![json, now_secs()],
        )?;
        Ok(normalized)
    }

    // ----- permissions -----

    pub fn list_permissions(&self) -> Result<Vec<PermissionStatus>> {
        let mut stmt = self.conn.prepare("SELECT id, granted, updated_at FROM permissions")?;
        let stored: std::collections::HashMap<String, (bool, i64)> = stmt
            .query_map([], |r| {
                let id: String = r.get(0)?;
                let granted: i64 = r.get(1)?;
                let updated: i64 = r.get(2)?;
                Ok((id, (granted != 0, updated)))
            })?
            .collect::<std::result::Result<_, _>>()?;

        Ok(Permission::all()
            .iter()
            .map(|p| {
                let (granted, updated_at) = stored.get(p.id()).copied().unwrap_or((false, 0));
                PermissionStatus {
                    id: p.id().to_string(),
                    granted,
                    updated_at: if updated_at == 0 { None } else { Some(updated_at) },
                }
            })
            .collect())
    }

    pub fn is_granted(&self, permission: Permission) -> Result<bool> {
        let granted: Option<i64> = self
            .conn
            .query_row(
                "SELECT granted FROM permissions WHERE id = ?1",
                params![permission.id()],
                |r| r.get(0),
            )
            .ok();
        Ok(granted.unwrap_or(0) != 0)
    }

    pub fn set_permission(&self, permission: Permission, granted: bool) -> Result<PermissionStatus> {
        let now = now_secs();
        self.conn.execute(
            "INSERT INTO permissions (id, granted, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET granted = excluded.granted, updated_at = excluded.updated_at",
            params![permission.id(), granted as i64, now],
        )?;
        Ok(PermissionStatus {
            id: permission.id().to_string(),
            granted,
            updated_at: Some(now),
        })
    }
}

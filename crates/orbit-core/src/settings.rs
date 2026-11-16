//! Application settings model.
//!
//! Stored as a single JSON document in SQLite (table `settings`, key='app').
//! Unknown/missing fields fall back to defaults so upgrades never wipe or
//! corrupt existing user configuration.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThemeMode {
    Dark,
    Light,
    /// Follow the OS preference (resolved in the frontend via `prefers-color-scheme`).
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Density {
    Compact,
    Comfortable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IslandPosition {
    TopCenter,
    TopLeft,
    TopRight,
    BottomCenter,
}

/// Where the island lives. `Auto` picks overlay on Windows/macOS and the honest
/// in-app panel on Linux (Wayland cannot guarantee a global overlay).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IslandMode {
    /// In-app floating panel inside the main window. Always works.
    InApp,
    /// Separate frameless always-on-top window. Best-effort on Wayland.
    Overlay,
    /// Overlay on Windows/macOS, in-app on Linux.
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IslandSize {
    Compact,
    Default,
    Large,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct IslandSettings {
    /// Master switch. The whole island can be disabled.
    pub enabled: bool,
    pub mode: IslandMode,
    /// Collapsed pill width mode.
    pub size: IslandSize,
    pub position: IslandPosition,
    /// Fade out after a few seconds without interaction.
    pub auto_hide: bool,
    /// Seconds of inactivity before auto-hide (when auto_hide is on).
    pub auto_hide_after_secs: u64,
    /// 0.0–1.0 opacity of the island surface.
    pub opacity: f32,
    /// Show notification sounds.
    pub sounds: bool,
    /// Which quick actions appear in the island's expanded view.
    pub quick_actions: Vec<String>,
    /// Show the coarse local CPU/RAM row in the expanded island. Local samples only.
    pub system_stats: bool,
}

impl Default for IslandSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: IslandMode::Auto,
            size: IslandSize::Default,
            position: IslandPosition::TopCenter,
            auto_hide: false,
            auto_hide_after_secs: 6,
            opacity: 0.92,
            sounds: false,
            quick_actions: vec![
                "focus_timer_25".to_string(),
                "focus_timer_10".to_string(),
                "ghostdrop_toggle".to_string(),
            ],
            system_stats: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub theme_mode: ThemeMode,
    /// Hex accent color, e.g. "#6c8cff".
    pub accent: String,
    pub density: Density,
    pub island: IslandSettings,
    /// Module ids that are disabled. Unknown ids are ignored by the shell.
    pub disabled_modules: Vec<String>,
    /// Reduces motion for users who prefer less animation.
    pub reduce_motion: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::System,
            accent: "#6c8cff".to_string(),
            density: Density::Comfortable,
            island: IslandSettings::default(),
            disabled_modules: Vec::new(),
            reduce_motion: false,
        }
    }
}

impl AppSettings {
    /// Basic validation; returns normalized copy or an error message.
    pub fn validated(mut self) -> std::result::Result<Self, String> {
        let accent = self.accent.trim();
        let valid = accent.len() == 7
            && accent.starts_with('#')
            && accent[1..].chars().all(|c| c.is_ascii_hexdigit());
        if !valid {
            return Err(format!("accent must be #RRGGBB, got {accent:?}"));
        }
        self.accent = accent.to_ascii_lowercase();
        if self.island.opacity.is_nan() {
            self.island.opacity = 0.92;
        }
        self.island.opacity = self.island.opacity.clamp(0.4, 1.0);
        self.island.auto_hide_after_secs = self.island.auto_hide_after_secs.clamp(2, 60);
        Ok(self)
    }
}

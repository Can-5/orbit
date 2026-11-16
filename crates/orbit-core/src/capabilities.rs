//! Platform detection and capability registry.
//!
//! The golden rule: **never pretend a feature works when it does not.**
//! Every capability reports an honest [`Support`] state with a human-readable
//! reason and, where applicable, the working alternative we ship instead.
//! Desktop platforms (Linux/Windows/macOS) and future Android are all listed
//! explicitly so the UI can gate features per platform.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Linux,
    Windows,
    MacOs,
    /// Planned, not yet developed. Listed so capability queries stay honest.
    Android,
}

impl Platform {
    pub fn current() -> Self {
        #[cfg(target_os = "linux")]
        {
            Platform::Linux
        }
        #[cfg(target_os = "windows")]
        {
            Platform::Windows
        }
        #[cfg(target_os = "macos")]
        {
            Platform::MacOs
        }
        #[cfg(target_os = "android")]
        {
            Platform::Android
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos", target_os = "android")))]
        {
            compile_error!("unsupported target for Orbit")
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Platform::Linux => "Linux",
            Platform::Windows => "Windows",
            Platform::MacOs => "macOS",
            Platform::Android => "Android",
        }
    }
}

/// Honest support state for one capability on one platform.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum Support {
    /// Works. Frontend may expose the feature.
    Supported,
    /// Exists but with real limits — the note explains them.
    Limited { note: String },
    /// Not available. The note explains why and what we ship instead.
    Unsupported { note: String },
}

impl Support {
    pub fn is_usable(&self) -> bool {
        matches!(self, Support::Supported | Support::Limited { .. })
    }
    pub fn note(&self) -> Option<&str> {
        match self {
            Support::Supported => None,
            Support::Limited { note } | Support::Unsupported { note } => Some(note),
        }
    }
}

/// Stable capability identifiers used across the backend, frontend and docs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Capability {
    /// Frameless transparent always-on-top island window.
    IslandOverlayWindow,
    /// Global keyboard shortcuts while the app is unfocused.
    GlobalHotkeys,
    /// Subscribe to OS clipboard change events.
    ClipboardMonitor,
    /// Read clipboard contents (only after explicit user grant).
    ClipboardRead,
    /// Capture screen regions (needs OS screen-recording consent on some platforms).
    ScreenCapture,
    /// Per-monitor ruler / screen-measure overlay.
    ScreenRuler,
    /// Move/resize other applications' windows (SnapSpace).
    WindowManagement,
    /// Register the app to start at login.
    Autostart,
    /// Read/write wallpaper and theme settings.
    ThemeControl,
    /// Enumerate running processes and system metrics.
    ProcessMetrics,
    /// Keyboard event listening for KeyGlow (needs explicit grant).
    KeyboardListen,
}

/// The full capability matrix. Queried by the frontend at startup so the UI
/// can gate or annotate features. Keep this in sync with docs/limitations.md.
pub fn capability_support(cap: Capability, platform: Platform) -> Support {
    use Capability::*;
    use Platform::*;
    use Support::*;

    match (cap, platform) {
        // --- Island overlay ---
        (IslandOverlayWindow, Windows) | (IslandOverlayWindow, MacOs) => Supported,
        (IslandOverlayWindow, Linux) => Limited {
            note: "KDE Plasma supports frameless always-on-top windows. On GNOME and under Wayland \
                   an unfocused always-on-top overlay may not be honored; Orbit falls back to an \
                   in-app island panel instead of pretending overlay works."
                .into(),
        },
        (IslandOverlayWindow, Android) => Unsupported {
            note: "Android forbids unrestricted screen overlays for regular apps. A future Android \
                   release uses a Notification/Quick Settings tile with island-style UI."
                .into(),
        },

        // --- Global hotkeys ---
        (GlobalHotkeys, Windows) | (GlobalHotkeys, MacOs) => Supported,
        (GlobalHotkeys, Linux) => Limited {
            note: "X11 supports global hotkeys. Wayland does not allow clients to grab keys globally; \
                KDE Plasma offers KGlobalAccel via portal-like APIs where available, \
                otherwise hotkeys work only while Orbit is focused."
                .into(),
        },
        (GlobalHotkeys, Android) => Unsupported {
            note: "Android has no global hotkeys for unfocused apps.".into(),
        },

        // --- Clipboard monitor ---
        (ClipboardMonitor, Windows) | (ClipboardMonitor, MacOs) | (ClipboardMonitor, Linux) => {
            Limited {
                note: "Event-based where the OS exposes it (X11, macOS, Windows). Wayland requires a \
                       focused layer-shell client; Orbit tracks clipboard only while a portal allows it \
                       or while focused, and never without an explicit user grant."
                    .into(),
            }
        }
        (ClipboardMonitor, Android) => Unsupported {
            note: "Android forbids background clipboard reads; foreground reads only.".into(),
        },

        (ClipboardRead, _) => Supported, // reading after explicit grant is always possible

        // --- Screen capture ---
        (ScreenCapture, Windows) | (ScreenCapture, Linux) => Limited {
            note: "macOS-style consent does not exist on all Linux sessions; on Wayland a portal \
                   (xdg-desktop-portal) prompt is required per capture. Blur/annotate/OCR operate \
                   locally on the captured image."
                .into(),
        },
        (ScreenCapture, MacOs) => Limited {
            note: "Requires Screen Recording consent in System Settings → Privacy & Security.".into(),
        },
        (ScreenCapture, Android) => Limited {
            note: "Android requires a one-time MediaProjection consent per session; no silent capture."
                .into(),
        },

        // --- Screen ruler ---
        (ScreenRuler, Windows) | (ScreenRuler, Linux) => Limited {
            note: "Implemented as a fullscreen transparent overlay; on some Wayland compositors overlay \
                   windows are restricted."
                .into(),
        },
        (ScreenRuler, MacOs) => Supported,
        (ScreenRuler, Android) => Unsupported {
            note: "No over-app measurement overlay without accessibility abuse; out of scope.".into(),
        },

        // --- Window management (SnapSpace) ---
        (WindowManagement, Windows) | (WindowManagement, MacOs) => Limited {
            note: "Can position/resize windows of normal applications via OS APIs. Elevated or sandboxed \
                   apps may refuse."
                .into(),
        },
        (WindowManagement, Linux) => Limited {
            note: "X11 window management via EWMH. Wayland intentionally gives no client control over \
                   other windows; SnapSpace records layouts and reapplies only inside compositor rules \
                   (KDE KWin scripts on KDE) or stays manual."
                .into(),
        },
        (WindowManagement, Android) => Unsupported {
            note: "No cross-app window control on Android.".into(),
        },

        // --- Autostart ---
        (Autostart, Windows) | (Autostart, MacOs) | (Autostart, Linux) => Supported,
        (Autostart, Android) => Unsupported {
            note: "Background start on boot is restricted; use normal app launch instead.".into(),
        },

        // --- Theme control ---
        (ThemeControl, Linux) => Limited {
            note: "KDE Plasma theming via kwriteconfig6/look-and-feel; GNOME via gsettings (color-scheme, \
                   background). Some properties are read-only."
                .into(),
        },
        (ThemeControl, Windows) => Limited {
            note: "Wallpaper and accent color via registry/undocumented APIs (limited); themes itself \
                   only inside Orbit."
                .into(),
        },
        (ThemeControl, MacOs) => Limited {
            note: "Wallpaper via AppleScript; accent/appearance only inside Orbit. No system theme API."
                .into(),
        },
        (ThemeControl, Android) => Unsupported {
            note: "No system-wide theme control.".into(),
        },

        // --- Process metrics ---
        (ProcessMetrics, Windows) | (ProcessMetrics, MacOs) | (ProcessMetrics, Linux) => Supported,
        (ProcessMetrics, Android) => Limited {
            note: "Per-process stats restricted; only Orbit's own usage is visible.".into(),
        },

        // --- Keyboard listen (KeyGlow) ---
        (KeyboardListen, Windows) | (KeyboardListen, MacOs) => Limited {
            note: "Requires explicit user grant; on macOS needs Input Monitoring permission. \
                   KeyGlow never records characters, only key identity for visualization."
                .into(),
        },
        (KeyboardListen, Linux) => Limited {
            note: "X11 via XRecord. Wayland denies global key listening by design; KeyGlow works only \
                   focused inside Orbit or not at all."
                .into(),
        },
        (KeyboardListen, Android) => Unsupported {
            note: "Android has no global keyboard event access.".into(),
        },
    }
}

/// All capabilities with their support state on the given platform.
pub fn capability_report(platform: Platform) -> Vec<(Capability, Support)> {
    const ALL: &[Capability] = &[
        Capability::IslandOverlayWindow,
        Capability::GlobalHotkeys,
        Capability::ClipboardMonitor,
        Capability::ClipboardRead,
        Capability::ScreenCapture,
        Capability::ScreenRuler,
        Capability::WindowManagement,
        Capability::Autostart,
        Capability::ThemeControl,
        Capability::ProcessMetrics,
        Capability::KeyboardListen,
    ];
    ALL.iter().map(|c| (*c, capability_support(*c, platform))).collect()
}

//! Linux specifics. Orbit targets KDE Plasma and GNOME, on X11 and Wayland.

/// Best-effort detection of a Wayland session.
fn is_wayland() -> bool {
    std::env::var_os("WAYLAND_DISPLAY").is_some()
}

/// Which desktop the session is running (best effort, env-based).
pub fn desktop_name() -> String {
    std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .unwrap_or_else(|_| "unknown".into())
}

/// Overlay island decision on Linux.
///
/// GNOME on Wayland does not allow a global always-on-top overlay for regular
/// clients — showing one anyway would be pretending. We return true on X11
/// (override-redirect style window) and false elsewhere; KDE Wayland users can
/// opt in via Settings → Island → Overlay, accepting compositor limits.
pub fn overlay_island_supported() -> bool {
    !is_wayland()
}

/// KWin-specific notes used by docs and honest error messages.
pub const KWIN_NOTE: &str = "KDE Plasma: pencere konumlandırma EWMH/KWin üzerinden yapılır; \
     Wayland oturumunda başka uygulamaların pencerelerine istemci tarafı tam kontrol yoktur.";

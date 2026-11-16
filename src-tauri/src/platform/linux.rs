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
/// KDE Plasma Wayland usually honors client-side always-on-top, so the
/// overlay is offered there as a best-effort floating pill. GNOME on Wayland
/// does not allow a global always-on-top overlay for regular clients, so the
/// in-app island stays the honest fallback on GNOME Wayland. X11 works via
/// EWMH/override-redirect semantics.
pub fn overlay_island_supported() -> bool {
    if !is_wayland() {
        return true;
    }
    desktop_name().to_uppercase().contains("KDE")
}

/// KWin-specific notes used by docs and honest error messages.
pub const KWIN_NOTE: &str = "KDE Plasma: pencere konumlandırma EWMH/KWin üzerinden yapılır; \
     Wayland oturumunda başka uygulamaların pencerelerine istemci tarafı tam kontrol yoktur.";

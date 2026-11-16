//! Platform adapters. Each file isolates OS-specific behavior so the rest of
//! the app never branches on `cfg!` inline. Anything genuinely unsupported is
//! reported honestly via the orbit-core capability matrix.

pub mod linux;
pub mod macos;
pub mod windows;

/// True when the current platform honestly supports a global overlay island.
/// - Windows / macOS: yes, via a frameless always-on-top transparent window.
/// - Linux: only when not on a restricted Wayland session; KDE Plasma on
///   Wayland may honor it, GNOME Wayland generally will not. Default-off is
///   the honest choice.
pub fn overlay_island_supported() -> bool {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        true
    }
    #[cfg(target_os = "linux")]
    {
        linux::overlay_island_supported()
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        false
    }
}

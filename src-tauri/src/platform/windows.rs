//! Windows specifics. Overlay island and window management use Win32 APIs;
//! permissions are enforced through orbit-core's explicit grant model.

/// Windows currently honors frameless always-on-top transparent windows for
/// the island overlay.
pub const OVERLAY_ISLAND_SUPPORTED: bool = true;

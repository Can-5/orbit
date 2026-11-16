//! macOS specifics. Overlay island works as an NSPanel-style floating window;
//! screen recording and input monitoring require explicit user consent in
//! System Settings — Orbit reports those as limited capabilities.

/// macOS honors frameless always-on-top windows (level: floating-ish).
pub const OVERLAY_ISLAND_SUPPORTED: bool = true;

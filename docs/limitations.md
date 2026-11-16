# Platform honesty matrix (Orbit)

Orbit reports what the OS actually lets it do — never a pretended "supported". `orbit-core::capabilities` is the enforcement point; this table is its human-readable mirror.

| Capability | Linux (X11/XWayland) | Linux (Wayland / GNOME) | Linux (KDE Plasma) | Windows 10/11 | macOS |
| --- | --- | --- | --- | --- | --- |
| Island overlay window | Limited | Limited (KWin generally honors; GNOME may not) | Limited→OK | Supported | Supported |
| Global hotkeys | Limited (evdev/XRecord) | Limited (compositor-dependent) | OK via KWin shortcuts | Supported | Supported |
| Clipboard monitor | Limited (per-app read) | Limited | Limited→OK | Supported | Supported |
| Screen capture | Limited (xdg-portal) | Limited (portal only) | OK | Supported | Supported |
| Window management | Limited (wm-dependent) | Limited | OK (KWin scripting) | Supported | Supported |
| Autostart | Limited (.desktop file) | Limited | OK | Supported | Supported |
| Process metrics (CPU/RAM) | OK (/proc) | OK | OK | Supported | Supported |
| Theme control (wallpaper/accent) | Limited (Qt/KDE or gsettings) | Limited | OK (KDE) | Limited | Limited |
| Screen ruler / eyedropper | Limited | Limited | Limited | Supported | Supported |

Support is **per-session** and may change when the compositor does. Orbit queries, fails honestly, and offers the in-app island as the always-works fallback.

## Known hard limits

- **Wayland global hotkeys**: no standard; KDE via KWin D-Bus, GNOME via Shell extensions only. Without them: in-app shortcuts only.
- **Screenshots on Wayland**: xdg-portal Screenshot API arbitrary-region may be unavailable; full-screen via portal always works but shows a system consent dialog.
- **Always-on-top from outside the compositor**: GNOME/Wayland effectively forbids; Orbit degrades to the in-app island and says so.
- **Theme application**: cross-DE wallpapers are compositor-specific; Orbit ships a validated profile but applies only where the OS allows, asking each time otherwise.

## Reporting inaccuracies

File an issue with your `XDG_CURRENT_DESKTOP`, compositor, and session type. The matrix is updated from user-reported reality, not wishful docs.

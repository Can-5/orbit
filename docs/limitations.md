# Platform honesty matrix (Orbit)

Orbit reports what the OS actually lets it do, never a pretended "supported". `orbit-core::capabilities` is the enforcement point. This table is its human-readable mirror.

| Capability | Linux (X11/XWayland) | Linux (Wayland / GNOME) | Linux (KDE Plasma) | Windows 10/11 | macOS |
| --- | --- | --- | --- | --- | --- |
| Island overlay window | Limited | Limited (KWin generally honors; GNOME may not) | Limited to OK | Supported | Supported |
| Global hotkeys | Limited (evdev/XRecord) | Limited (compositor-dependent) | OK via KWin shortcuts | Supported | Supported |
| Clipboard monitor | Limited (per-app read) | Limited | Limited to OK | Supported | Supported |
| Screen capture | Limited (xdg-portal) | Limited (portal only) | OK | Supported | Supported |
| Window management | Limited (wm-dependent) | Limited | OK (KWin scripting) | Supported | Supported |
| Autostart | Limited (.desktop file) | Limited | OK | Supported | Supported |
| Process metrics (CPU/RAM) | OK (/proc) | OK | OK | Supported | Supported |
| Theme control (wallpaper/accent) | Limited (Qt/KDE or gsettings) | Limited | OK (KDE) | Limited | Limited |
| Screen ruler / eyedropper | Limited | Limited | Limited | Supported | Supported |

Support is per-session and can change when the compositor does. Orbit queries, fails honestly, and offers the in-app island as the fallback that always works.

## KDE Plasma notes

KDE is the best-supported Linux path, with caveats:

- The overlay island works on X11 and is best-effort on Wayland. Plasma usually honors always-on-top; when it does not, Orbit uses the in-app island.
- Global shortcuts on Wayland need the XDG GlobalShortcuts portal. Plasma ships it, but Orbit does not implement the portal client yet. That is Phase 5 work. Until then, shortcuts work inside the app only (`Ctrl+Space` toggles the island, `1`-`5` start timers, `Esc` collapses).
- KWin rules can pin the island window. That is a manual setup; Orbit does not edit your KWin configuration without your approval.

## Known hard limits

- **Wayland global hotkeys**: no standard. KDE uses KWin D-Bus, GNOME uses Shell extensions only. Without them, in-app shortcuts only.
- **Screenshots on Wayland**: xdg-portal Screenshot may not expose arbitrary regions. Full-screen via portal always works, but shows a system consent dialog.
- **Always-on-top from outside the compositor**: GNOME/Wayland effectively forbids it. Orbit degrades to the in-app island and says so.
- **Theme application**: wallpapers are compositor-specific. Orbit ships a validated profile but applies it only where the OS allows, asking each time otherwise.

## Reporting inaccuracies

File an issue with your `XDG_CURRENT_DESKTOP`, compositor, and session type. The matrix is updated from user-reported reality, not wishful docs.

# Privacy at Orbit

Orbit's core promise: nothing leaves your machine, nothing happens without your say-so.

## What Orbit never does

- No telemetry, analytics, or crash reporting of any kind.
- No accounts, no sign-in, no cloud sync, no "phoning home".
- No network access by default. The only network-capable component is the opt-in `network` permission, off until granted, and documented as such.
- No keystroke content logging: KeyGlow receives key identity and timing only.
- No silent capture: screen capture, clipboard, keyboard, process metrics, and file watching each require explicit, per-capability grants.

## What stays on your device

- All app state: `orbit.db` (SQLite, WAL) in the platform config dir — settings, permissions, module registry. No history tables by default.
- History/search data (Recall): stored locally, exclusion rules enforced before recording; you can wipe it any time.
- Bookmarks (LinkNest): local files; export only when you ask.
- Themes (ThemePort): local packs; strict parsing; no executable fields.

## How permissions work (explicit-grant registry)

Orbit ships a `permissions` table — every permission defaults to denied, is granted or revoked explicitly by you, and is required by any feature that touches the capability. Scope creep is not possible by construction: a module cannot use a capability its host never granted, and the host logs nothing about your usage.

## Verification

- The codebase contains **no HTTP client dependency**. The only networking would be explicit, opt-in, and greppable — check `Cargo.lock` and the module crates; each module README lists its real dependencies.
- No stage, no CISO dashboard, no remote config. What you run is what you read.

## Reporting a privacy issue

Open a private security advisory on the GitHub repo (once published), or email the maintainer. Privacy bugs are severity-1 and fix-forward, no bounty nonsense attached.

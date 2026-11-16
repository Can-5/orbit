# Privacy at Orbit

Orbit keeps one promise: nothing leaves your machine, and nothing happens without your say-so.

## What Orbit never does

- No telemetry, analytics, or crash reporting of any kind.
- No accounts, no sign-in, no cloud sync. Orbit does not phone home.
- No network access by default. The only network-capable component is the opt-in `network` permission. It stays off until you grant it.
- No keystroke content logging. KeyGlow receives key identity and timing only.
- No silent capture. Screen capture, clipboard, keyboard, process metrics, and file watching each need an explicit grant.

## What stays on your device

- App state: `orbit.db` (SQLite, WAL) in the platform config dir. It holds settings, permissions, and module registry rows. No history tables by default.
- History/search data (Recall): stored locally. Exclusion rules run before anything is recorded, and you can wipe the data any time.
- Bookmarks (LinkNest): local files. Export only when you ask.
- Themes (ThemePort): local packs, strict parsing, no executable fields.

## Local system samples (Linux)

The island can show a CPU/RAM readout. It reads `/proc/stat` and `/proc/meminfo` every 2 seconds and keeps the sample in memory for the island UI. Nothing is written to disk, logged or transmitted. The readout is a toggle in Settings ("CPU/RAM satırı", on by default with the island). The sampler only starts when the island is enabled. It collects no per-process data.

## How permissions work (explicit-grant registry)

Orbit ships a `permissions` table. Every permission starts denied. You grant or revoke each one explicitly, and any feature that touches the capability requires it. A module cannot use a capability its host never granted. Orbit also logs nothing about how you use your grants.

## Verification

- The codebase contains **no HTTP client dependency**. Any networking would be opt-in and greppable. Check `Cargo.lock` and the module crates; each module README lists its real dependencies.
- No stage, no dashboard, no remote config. What you run is what you read.

## Reporting a privacy issue

Open a private security advisory on the GitHub repo. Privacy bugs are severity-1 and get fixed forward. There is no bounty program and no SLA.

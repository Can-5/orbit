# Orbit

> **Your desktop, organized around you.**

Orbit is a privacy-first, open-source desktop companion for Linux (KDE/GNOME), Windows 10/11, and macOS. It unifies twelve focused projects into one coherent experience — without a framework of forced dependencies, without telemetry, and without accounts.

**Honest status:** *pre-alpha / Phase 1.* Orbit launches and runs today as a Rust workspace; the island engine, settings, permissions, and storage are implemented and tested (`orbit-core`, 7 tests green). The Tauri desktop shell is written but **has not been built on this machine** — it needs system WebKitGTK development packages that are not installed (see [Build](#build)). The Svelte frontend **is implemented** (shell UI: dashboard, modules, island preview, permissions, capabilities, settings — static preview mode outside the Tauri host) but the shell has not been built, so live data is untested. Module repos stand alone and ship their own domain logic + tests; wiring them into the host is Phase 2.

![Orbit UI — browser preview](assets/screenshot.jpg)

## The ecosystem (12 repositories)

Orbit itself is one repo. Every module is its own independent repo, its own crate, its own version, its own CI, and its own license — usable without Orbit and without each other. Orbit detects what is installed, enables what you enable, and never requires everything.

| Repo | What it is | Status |
| --- | --- | --- |
| [Can-5/orbit](https://github.com/Can-5/orbit) | This repo. Tauri shell, app-wide settings, permission registry, `orbit-core` engine (island, capabilities, permissions, storage) | Shell written; **webkit2gtk-devel missing on dev machine** |
| [Can-5/orbit-island](https://github.com/Can-5/orbit-island) | Orbit Island — the signature interactive pill: state machine, timer, queueing, plus Module Protocol v1 reference (`SPEC.md`) | Done (domain engine, 8 tests) |
| [Can-5/ghostdrop](https://github.com/Can-5/ghostdrop) | Temporary clipboard & file-drop zone | Domain engine (5 tests) |
| [Can-5/recall](https://github.com/Can-5/recall) | Local searchable history with exclusions | Domain engine (6 tests) |
| [Can-5/renamex](https://github.com/Can-5/renamex) | Batch-rename *planning* (never writes without confirmation) | Domain engine (10 tests) |
| [Can-5/pixelpick](https://github.com/Can-5/pixelpick) | Color science: HEX/RGB/HSL, palettes | Domain engine (7 tests) |
| [Can-5/linknest](https://github.com/Can-5/linknest) | Local-first bookmarks, Markdown/HTML export | Domain engine (10 tests) |
| [Can-5/keyglow](https://github.com/Can-5/keyglow) | Keystroke *visualization* (never sees what you type) | Domain engine (7 tests) |
| [Can-5/cleanshot](https://github.com/Can-5/cleanshot) | Screenshot annotation models (no capture engine) | Domain engine (8 tests) |
| [Can-5/apppulse](https://github.com/Can-5/apppulse) | Metrics modeling: ring buffer, EMA, alert rules | Domain engine (7 tests) |
| [Can-5/snapspace](https://github.com/Can-5/snapspace) | Workspace profile capture/validation, honest restore planning | Domain engine (6 tests) |
| [Can-5/themeport](https://github.com/Can-5/themeport) | Theme engine: WCAG contrast, validated theme packs | Domain engine (11 tests) |

**85 module tests green** across the eleven module repos (`cargo test --locked`), plus 7 in `orbit-core`.

> *Domain engine* means the platform-independent logic (validation, planning, modeling, export, state machines). The OS-facing pieces — global hotkeys, pasteboard hooks, screenshot capture, window management, wallpaper application — are the Orbit host's job and are **not** faked: they are either unimplemented or explicitly reported as unavailable on your platform via the capability registry. Nothing pretends to work when it doesn't.

## Orbit Island

The signature feature: a Dynamic Island–style interactive pill for desktops.

- Priority-aware notification queue (≤16 items, high-priority preempts)
- Toasts, clipboard/screenshot/workspace/system/timer events
- Focus timers with pause/resume; fires once
- Expandable, collapsible, size/position/auto-hide configurable
- Per-platform honest support matrix: `Supported` on Windows 10/11 & macOS (overlay window), `Limited` on Linux (KDE KWin generally OK; GNOME/Wayland clients may ignore always-on-top overlays → falls back to the in-app island)
- **No Apple branding.** Own design language, own code.

## Privacy — the actual rules

- **No telemetry. Ever.** No analytics, no crash pings, no "anonymous usage statistics", no background beacons. There is no server and no account.
- **No network by default.** The link between Orbit and modules is a local parent↔child pipe.
- **Explicit grants only.** Every sensitive capability (clipboard, screen capture, keyboard listening, process metrics, file watching, network) is off until you grant it — per feature, revocable any time. Denied means denied.
- **Your data stays yours.** SQLite on your disk; exports are files you choose; themes are local; history exclusions are enforced by contract.
- See [docs/privacy.md](docs/privacy.md).

## Build

```sh
# Core engine only — works today, no GUI deps
cargo test -p orbit-core

# Desktop shell — needs system WebKitGTK dev packages on Fedora (ask first, do not auto-install):
#   sudo dnf install webkit2gtk4.1-devel gtk3-devel glib2-devel gcc-c++
# Then: cargo run -p orbit-tauri
```

The Svelte frontend is implemented: `pnpm install && pnpm build` (or `pnpm dev` → http://localhost:1420). Outside the Tauri host it runs in an explicitly-labeled static preview mode; live modules/permissions/settings data requires the Tauri shell (`cargo run -p orbit-tauri`), which needs the WebKitGTK dev packages above. `pnpm check` (svelte-check) is clean and the production build succeeds.

## Integration: Orbit Module Protocol v1 (draft)

Modules integrate via an opt-in, documented protocol — not by linking into Orbit:

- Newline-delimited JSON over a child process's stdio; `orbit-module.json` manifest read *before* launch
- Explicit permission negotiation; user grants or denies
- Strict schemas (`deny_unknown_fields`); no code evaluation; no scripts in packs/modules
- See [docs/module-protocol-v1.md](docs/module-protocol-v1.md) and the canonical draft in `orbit-island/SPEC.md`

## Roadmap

- **Phase 1** (now): shell, design system, permissions, island prototype, `orbit-core`
- **Phase 2**: host-side wiring of GhostDrop / Recall / RenameX / PixelPick
- **Phase 3**: LinkNest / CleanShot / AppPulse / SnapSpace
- **Phase 4**: KeyGlow / ThemePort / advanced island
- **Phase 5**: release builds, packages, CI, docs, issue/PR templates

## Contributing / Security / License

- [CONTRIBUTING.md](.github/CONTRIBUTING.md) *(planned)* · [Security & scope](.github/SECURITY.md) *(planned)*
- MIT © The Orbit contributors — see [LICENSE](LICENSE)

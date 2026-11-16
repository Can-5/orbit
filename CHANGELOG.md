# Changelog

All notable changes to Orbit are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) where practical.
This project is pre-alpha; no releases have been tagged yet.

## [Unreleased]

Unreleased changes since the initial scaffold (1b405b0):

### Added

- Svelte 5 + Vite + TypeScript frontend (Phase 1 shell UI: Pano / Modüller /
  Orbit Island / İzinler / Yetenekler / Ayarlar); browser static-preview mode
  outside the Tauri host, honestly labeled as such
- docs/architecture.md, docs/privacy.md, docs/limitations.md,
  docs/module-protocol-v1.md
- assets/screenshot.jpg
- CI for orbit-core (`cargo test --locked -p orbit-core`, release build);
  the Tauri shell is not built in CI (needs system WebKitGTK dev packages)
- `pnpm check` (svelte-check) clean
- Island ambient bridges (Linux, `src-tauri`): MPRIS now-playing watcher and
  `island_media_toggle` / `island_media_next` / `island_media_prev` /
  `island_set_media_volume` commands; an XDG downloads watcher that tracks
  top-level regular files by name and size only; a `sysmon` sampler that reads
  `/proc/stat` and `/proc/meminfo` every 2 seconds and feeds the island's
  CPU/RAM readout (new `island.system_stats` setting, on by default)
- Dropped-files tray: `island_drop_files` / `island_clear_dropped_files`
  commands listing path and size only; nothing is copied, moved or read
- Frontend: black ambient island restyle, live clock, media controls, volume
  slider, CPU/RAM bars and progress bars for downloads; in-app shortcuts
  (`Ctrl+Space`, `1`-`5`, `Esc`)

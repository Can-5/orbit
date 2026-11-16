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

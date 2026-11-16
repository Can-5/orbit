# Contributing to Orbit

Thanks for your interest in Orbit! Orbit is the host application of the
[Orbit ecosystem](https://github.com/Can-5/orbit): a Tauri 2 desktop shell, a
Svelte 5 + Vite + TypeScript frontend, and `orbit-core` — the Rust engine
(island state machine, capabilities, permissions, storage, module registry).
It lives at [Can-5/orbit](https://github.com/Can-5/orbit) and is MIT-licensed.

The companion module crates are independent repos (e.g.
[Can-5/orbit-island](https://github.com/Can-5/orbit-island)) with their own
contributing guides; Orbit does not require all of them installed.

## Build & test

```sh
# Core engine — works today, no GUI dependencies
cargo test --locked -p orbit-core
cargo build --locked -p orbit-core --release

# Frontend (Svelte shell)
pnpm install && pnpm build && pnpm check
```

`pnpm check` runs `svelte-check` and must stay clean; `pnpm build` must
succeed. Outside the Tauri host the frontend runs in an explicitly-labeled
static preview mode — live data requires the shell.

**Tauri shell:** the desktop shell needs system WebKitGTK/GTK development
packages (on Fedora: `webkit2gtk4.1-devel gtk3-devel glib2-devel gcc-c++`)
and is **currently not built in CI** (see [.github/workflows/ci.yml](.github/workflows/ci.yml)
and [docs/limitations.md](docs/limitations.md)). If you can build it locally,
that's welcome, but do not claim shell-dependent behavior works without having
actually built and run it.

## Ground rules

- **Small, focused PRs.** One concern per pull request. Avoid drive-by
  refactors mixed with behavior changes.
- **No telemetry. Ever.** No analytics, no crash pings, no "anonymous usage
  statistics", no background beacons. Any feature that would move data off the
  device requires an explicit, public permission discussion in an issue
  *before* implementation — see [docs/privacy.md](docs/privacy.md).
- **No network I/O by default.** Orbit's link to modules is a local
  parent↔child pipe; new network access requires the same explicit permission
  discussion and user-facing grant flow.
- **No fake features or fake system data.** If something is not implemented,
  it must be reported as unimplemented or unavailable — never simulated,
  stubbed with made-up values, or presented as working. Platform support is
  reported honestly per the capability registry.
- **Tests are required.** Behavior changes in `orbit-core` must include tests
  (`cargo test --locked -p orbit-core`). Frontend changes must keep
  `pnpm check` clean and `pnpm build` green.
- **MIT license.** By contributing, you agree that your contributions are
  licensed under the MIT License, the same license as the project. (Note:
  Orbit does not yet ship a LICENSE file in the repo root — it is expected to
  be added; contributions are still understood to be MIT.)
- **English code.** Source code, comments, documentation, and commit messages
  are written in English.
- **Commit messages.** Conventional-Commit-ish style is appreciated:
  `feat: …`, `fix: …`, `docs: …`, `refactor: …`, `test: …`, `chore: …`.

## Issues and security

- Use the issue templates for bugs and feature requests.
- For vulnerabilities, do **not** open a public issue — follow
  [SECURITY.md](SECURITY.md) instead.

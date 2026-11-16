# Orbit — Architecture

Orbit is a workspace, not a library of duplicated apps. It is the composition layer for a family of small, independent, MIT-licensed modules.

## Layout

```
orbit/
├─ crates/orbit-core/   # platform-independent domain logic (Rust, no Tauri)
│   ├─ capabilities.rs  # per-platform capability matrix + honest Limited/Unsupported notes
│   ├─ permissions.rs   # 10 explicit, opt-in permissions (default denied)
│   ├─ settings.rs      # theme/density/island settings + validation
│   ├─ storage.rs       # SQLite: settings + permissions tables (user_version migrations)
│   ├─ island.rs        # Island priority queue state machine
│   └─ module_registry.rs # 11 modules: phase, permissions, honest status
├─ src-tauri/           # Tauri 2 shell: windows, platform, IPC commands
├─ src/                 # Svelte frontend (NOT implemented yet)
└─ docs/                # this file, privacy, limitations, protocol
```

## Principles

- **One design language.** Sibling module repos are domain-engine crates; Orbit is the only UI host.
- **No fake platform features.** `capabilities.rs` is the single source of truth; the shell calls it before enabling anything.
- **Modules are independent.** Each module repo builds with `cargo build` alone; Orbit uses them in-process (crate type staticlib/rlib-style domain crates) or, later, via the Module Protocol.
- **Island is a pure engine.** `island.rs` / `orbit-island` is a state machine; presentation lives in the frontend and the Tauri overlay window.
- **SQLite is the only persistence layer** (settings + permissions). No external DB, no network.

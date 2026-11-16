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
│   ├─ island.rs        # island priority queue state machine
│   ├─ manifest.rs      # Orbit Module Manifest v1 validation + install detection
│   └─ module_registry.rs # 11 modules: phase, permissions, honest status
├─ src-tauri/           # Tauri 2 shell: windows, platform, IPC commands
├─ src/                 # Svelte frontend (implemented; browser preview mode outside a Tauri host)
└─ docs/                # this file, privacy, limitations, protocol
```

## Principles

- **One design language.** Sibling module repos are domain-engine crates. Orbit is the only UI host.
- **No fake platform features.** `capabilities.rs` is the source of truth, and the shell calls it before enabling anything.
- **Modules are independent.** Each module repo builds with `cargo build` alone. Orbit uses them in-process as domain crates, or later through the Module Protocol.
- **Island is a pure engine.** `island.rs` and `orbit-island` are state machines. Presentation lives in the frontend and the Tauri overlay window.
- **SQLite is the only persistence layer** for settings and permissions. No external DB, no network.

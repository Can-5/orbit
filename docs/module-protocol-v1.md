# Orbit Module Protocol v1 (draft 0.1.0)

> The canonical, maintained draft lives in the **orbit-island** repository: [`SPEC.md`](https://github.com/Can-5/orbit-island/blob/main/SPEC.md). This file is a summary; when they diverge, SPEC.md wins.

## Shape

- Transport: newline-delimited JSON (**NDJSON**) over stdio.
- Discovery: a module ships `orbit-module.json` (`deny_unknown_fields`, slug ids, protocol major `1`). Orbit reads it **before** launching the process. Permission requests in the manifest are requests, never grants.
- Handshake: Orbit → `module.hello`; module → `{ok: true, name, version, protocol: 1}`. Modules may emit events only after the handshake.
- Envelope: `{ "v": 1, "id"?: n, "kind": "request"|"response"|"event", "method", "params"?, "result"?, "error"? }`.
- Namespaces: `module.*`, `island.*`, `clipboard.*`, `screenshot.*`, `files.*`, `system.*`. Permissions map to namespaces one-to-one.

## Security invariants

1. The protocol itself performs no network I/O.
2. Modules are static, auditable binaries — no `eval`, no scripting from manifests.
3. Every message is schema-validated; unknown fields are rejected.
4. Secrets (tokens, keys) never travel over the wire; they are referenced, not transmitted.
5. Data minimization: a module receives only what its capability grants allow.
6. Honest unavailability: if a platform capability is missing, the module must report `unavailable`, never fabricate success.

## Versioning

- Major `1` gates compatibility; minor versions add methods without removing.
- `island.*` surface and `orbit-island` (the reference implementation) track the same minor.

## Status

Draft. Not implemented in the Tauri shell yet. Orbit currently embeds the island engine directly in `orbit-core`; the protocol is the forward path for out-of-process modules.

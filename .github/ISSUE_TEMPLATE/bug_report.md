---
name: Bug report
about: Report a reproducible problem in Orbit
title: "[Bug]: "
labels: [bug]
---

## Describe the bug

A clear and concise description of what the bug is.

## Steps to reproduce

1. …
2. …
3. …

## Expected behavior

What you expected to happen.

## Actual behavior

What actually happened (include the full error message if any).

## Environment

- OS: <!-- e.g. Fedora Linux 43, Windows 11, macOS 15 -->
- Which part: <!-- orbit-core / Tauri shell / Svelte frontend (browser preview or in-shell) -->
- Orbit version / commit: <!-- e.g. v0.1.0 or commit hash -->
- Rust toolchain: <!-- output of `rustc --version` -->
- Node/pnpm versions (frontend issues): <!-- `node --version`, `pnpm --version` -->

## Logs

Paste any relevant logs, test output (`cargo test --locked -p orbit-core`), or
`pnpm check` output here. Use fenced code blocks and redact anything private.

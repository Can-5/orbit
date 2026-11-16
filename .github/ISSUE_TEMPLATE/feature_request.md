---
name: Feature request
about: Suggest a new feature or improvement for Orbit
title: "[Feature]: "
labels: [enhancement]
---

## Problem

What problem does this solve? Who is affected, and how often?

## Proposed solution

Describe what you would like to happen. If you have UX, API, or permission-model
details in mind, include them.

## Alternatives considered

What other approaches did you consider, and why are they not sufficient?

## Does this change require network/OS access?

Answer honestly:

- [ ] No — this stays entirely on-device and off the Tauri/OS permission surface,
      like the rest of Orbit.
- [ ] Yes — it would require network access or new OS-level permissions
      (clipboard, screen capture, keyboard listening, process metrics, file
      watching, wallpaper, …).

If the answer is "yes, data leaves the device by design", this needs an
explicit permission discussion in the issue **before** any implementation — see
[CONTRIBUTING.md](../CONTRIBUTING.md) and
[docs/privacy.md](../docs/privacy.md). Orbit has no telemetry and no network by
default; that is a deliberate design constraint, not an oversight.

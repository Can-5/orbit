## Summary

<!-- What does this PR change, and why? Link any related issue. -->

## Checklist

- [ ] `cargo test --locked -p orbit-core` passes
- [ ] Frontend changes: `pnpm install && pnpm build && pnpm check` passes
- [ ] No telemetry added (no analytics, pings, crash reporting, or stats)
- [ ] No network I/O added without an explicit, user-facing permission discussion
- [ ] No fake data / fake system claims — unimplemented things are reported as
      unimplemented or unavailable, never simulated
- [ ] README / CHANGELOG / docs updated if behavior changed
- [ ] I acknowledge that my contribution is licensed under the MIT License,
      the same license as this project

# Release Runbook

**Audience:** Maintainers/release engineers  
**Owner:** Core maintainers  
**Last reviewed:** 2026-05-02

## Purpose
Provide a deterministic release checklist to reduce risk and last-minute regressions.

## Preconditions
- Release branch is up to date with reviewed changes.
- Version and changelog are finalized.
- CI is green on target branch.

## Preflight checks

1. Clean tree:
```bash
git status --short
```

2. Native audio validation:
```bash
make test-cpp
```

3. Full quality bar:
```bash
make ci
```

4. Desktop smoke run:
```bash
cd apps/desktop && pnpm tauri dev
```
- Verify deck load, play/pause, crossfader movement, and settings open/close.

## Release steps
1. Tag release candidate (`vX.Y.Z-rcN`) if used.
2. Confirm artifacts for supported platform targets.
3. Publish changelog with notable fixes + known limitations.
4. Create final tag (`vX.Y.Z`) and publish release artifacts.

## Post-release checks
- Install artifact on clean machine/profile.
- Verify app launch and basic playback path.
- Monitor incoming issues for regressions in first 24–48 hours.

## Rollback criteria
- Crash on launch.
- Audio playback broken on primary platform.
- Data/config corruption risk.

If rollback criteria are hit: unpublish/mark release as bad, revert offending changes, cut patched release.

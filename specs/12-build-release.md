# 12 — Build and Release

## 1. Toolchain

| Tool | Version policy |
|---|---|
| Rust | latest stable, pinned via `rust-toolchain.toml` |
| Node.js | LTS (20.x at time of writing), pinned via `.nvmrc` |
| pnpm | ≥ 9, pinned via `packageManager` field |
| CMake | ≥ 3.27 |
| Xcode CLT | latest stable on the build host |
| clang | bundled with Xcode CLT or LLVM ≥ 17 cross-platform |

`scripts/setup.sh` installs everything via Homebrew on macOS; equivalent
scripts live in `scripts/setup-linux.sh` and `scripts/setup-windows.ps1`
(populated in later phases).

## 2. Build commands

| Goal | Command |
|---|---|
| Dev run | `pnpm tauri dev` |
| Release build | `pnpm tauri build` |
| Rust-only checks | `cargo test --workspace` |
| C++ engine tests | `ctest --test-dir native/audio/build` |
| UI tests | `pnpm test` |
| Full quality bar | `make ci` (wraps the above + linters) |

## 3. CI pipeline

GitHub Actions workflows in `.github/workflows/`:

- `ci.yml` — fmt, lint, test, build, coverage, file-length linter
- `bench.yml` — nightly benchmarks against baseline
- `release.yml` — triggered by tag push `v*.*.*`
- `licence-audit.yml` — SBOM + MPL-2.0 compatibility check

Matrix:

- macOS-13 (Apple Silicon when self-hosted), macOS-14 (Apple Silicon)
- Linux ubuntu-22.04 (from Phase 2)
- Windows-2022 (Phase 5 evaluation)

## 4. Artefacts

| Platform | Artefact |
|---|---|
| macOS | `PhraseDJ-<version>-mac-arm64.dmg`, signed and notarised |
| macOS | `PhraseDJ-<version>-mac-universal.dmg` from Phase 5 |
| Linux | `PhraseDJ-<version>-linux-x86_64.AppImage` |
| Windows | `PhraseDJ-<version>-win-x64.msi` (Phase 5 eval) |

Each artefact ships with `SHA256SUMS` and a detached signature.

## 5. macOS signing and notarisation

- Apple Developer ID for the open-source project, kept in a dedicated org
  account.
- `tauri.conf.json` configured with team ID and signing identity.
- Notarisation in `release.yml` via `notarytool`, stapling on success.
- Privacy descriptions in `Info.plist` for microphone (live input later) and
  Bonjour (MIDI over network later) — included only when those features
  ship.
- Hardened runtime on; the only entitlement requested is the Audio
  capability needed by CLAP plugin loading.

## 6. Versioning

- Pre-1.0 releases use `0.x.y`. Tag format `v0.x.y`.
- Channels: `dev` (every main commit), `nightly` (every successful main),
  `beta` (manually promoted), `stable` (1.0+).
- A version bump is the only commit on its own PR with the changelog entry.

## 7. Changelog

- `CHANGELOG.md` follows Keep-a-Changelog format.
- One entry per user-visible change, grouped by Added / Changed / Fixed /
  Removed / Security.
- Generated draft from PR labels via `release.yml`; humans curate before
  publishing.

## 8. Distribution

- GitHub Releases is the canonical source.
- A static download page on the project site lists the latest version and
  its checksums.
- Homebrew tap (`mazer666/tap`) added at 1.0.
- Linux: AUR package and Flathub from Phase 5.

## 9. Update mechanism

- Optional in-app update check is **off by default**.
- When enabled, the app fetches `https://<project>/releases/latest.json` at
  startup, verifies the signature, and offers a download.
- Updates are never silent or background-installed.

## 10. Reproducible builds

- Release jobs commit a `lockfiles/` snapshot (Cargo.lock, pnpm-lock.yaml,
  CMakeCache exports) into the release tarball.
- A nightly job verifies that rebuilding from the lockfile snapshot yields
  byte-identical artefacts within tolerance for embedded timestamps.

## 11. Repository hygiene

- `make pre-commit` runs fmt, lint, file-length check.
- Conventional Commits enforced by a tiny commitlint config.
- PR template asks: spec touched? tests added? files within limits?
- Squash-merge default, with a clean conventional title.

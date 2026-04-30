# Contributing to PhraseDJ

Thanks for your interest in contributing! PhraseDJ is developed AI-assisted
("vibe coding") but welcomes human contributions of all kinds.

## Before you start

Read **[`LLM.md`](LLM.md)** — it's short and tells you the rules any code
change must follow. The most important ones:

- Every source file stays under **600 lines** (target 400).
- All settings / constants go in `config/defaults.toml`, not in code.
- Every new public function gets a doc comment.
- Every new module ships with at least one unit test.

## Development setup

```bash
# 1. Clone
git clone https://github.com/mazer666/phodj.git PhraseDJ && cd PhraseDJ

# 2. Install system deps (macOS with Homebrew)
brew install rustup node cmake llvm
rustup-init -y
source "$HOME/.cargo/env"
rustup component add rustfmt clippy

# 3. Install Node tooling
npm install -g pnpm
pnpm install --dir apps/desktop

# 4. Run quality bar
make ci    # format + lint + Rust tests + frontend tests + C++ tests
```

## Running the app

```bash
# Dev mode (hot reload, no audio yet in Phase 0)
cd apps/desktop && pnpm tauri dev
```

## Workflow

1. Open an issue describing the change (unless it's a trivial typo).
2. Fork / branch from `main`.
3. Make your change — small PRs are easier to review.
4. Run `make ci` locally before pushing.
5. Open a PR using the template.

## Commit style

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(audio): add CoreAudio device enumeration
fix(library): handle missing music_root gracefully
docs(specs): clarify stem cache eviction policy
refactor(pdj-core): split config.rs into audio/library/ui modules
test(pdj-core): add settings round-trip property test
chore: bump tauri to 2.1.0
```

## Code of conduct

Be kind. Beginners are welcome. Bad-faith contributions are not.

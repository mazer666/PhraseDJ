# Getting Started (Developers)

**Audience:** New contributors  
**Owner:** Core maintainers  
**Last reviewed:** 2026-05-02

## Purpose
Get PhraseDJ running locally and execute the core quality checks with minimal guesswork.

## Scope
- Local developer setup
- Build/test commands
- First run of desktop app

## Prerequisites

### macOS (primary target)
```bash
brew install rustup node cmake llvm portaudio pkg-config glib
rustup-init -y
source "$HOME/.cargo/env"
rustup component add rustfmt clippy
npm install -g pnpm
```

### Linux (Ubuntu/Debian)
```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libglib2.0-dev portaudio19-dev cmake
rustup-init -y
source "$HOME/.cargo/env"
rustup component add rustfmt clippy
npm install -g pnpm
```

## Clone and install
```bash
git clone https://github.com/mazer666/phodj.git PhraseDJ
cd PhraseDJ
pnpm install --dir apps/desktop
```

## First build and checks

### 1) Build and test native audio engine
```bash
make test-cpp
```

### 2) Run full local quality bar
```bash
make ci
```

### 3) If environment is constrained, run reduced checks
```bash
make ci-minimal
```

## Run the desktop app
```bash
make test-cpp
cd apps/desktop && pnpm tauri dev
```

## Common failure modes
- **`pnpm` not found:** install globally with `npm install -g pnpm`.
- **Rust tools missing (`cargo fmt`/`clippy`):** run `rustup component add rustfmt clippy`.
- **Native build fails (`portaudio`/`glib`):** verify OS package install commands above.
- **Tauri app cannot load audio lib:** rerun `make test-cpp` before launching dev mode.

## Source of truth
- `README.md`
- `CONTRIBUTING.md`
- `Makefile`

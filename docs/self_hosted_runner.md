# Self-Hosted Runner Setup Guide

This guide describes how to configure and run local GitHub Actions self-hosted runners for PhraseDJ. By using your own hardware (local processing power) to run CI workflows, you completely bypass GitHub-hosted runner consumption and do not affect your Actions minutes quota.

---

## 1. Quick Concept

When you push a commit or open a Pull Request, GitHub Actions will trigger the workflow. However, instead of spawning an expensive virtual machine hosted by GitHub, GitHub will delegate the execution to your running local agent (the "self-hosted runner"). 

We have updated `.github/workflows/ci.yml` to target:
- `[self-hosted, macOS]` for macOS builds and tests.
- `[self-hosted, Linux]` for Linux builds and tests.
- `self-hosted` for general tasks (frontend, license audit, file-length checks) which can run on either agent.

---

## 2. Registering a Runner in GitHub

To link your local computer to your repository as a runner:

1. Open your browser and navigate to your GitHub repository: `https://github.com/mazer666/PhraseDJ` (or your fork).
2. Click **Settings** in the top navigation bar.
3. In the left sidebar, expand **Actions** and select **Runners**.
4. Click the **New self-hosted runner** button in the top right.
5. Select your Operating System (**macOS** or **Linux**) and architecture (usually **ARM64** for Apple Silicon Macs or **X64** for Intel).
6. Follow the customized, copy-pasteable commands provided by GitHub under the **Download** and **Configure** sections.

> [!IMPORTANT]
> During the `./config.sh` step, the CLI setup will ask you for labels.
> The runner automatically receives the `self-hosted` label, along with the OS label (like `macOS` or `Linux`). Ensure these match the matrix exactly:
> - On your Mac, ensure it is registered with labels: `self-hosted`, `macOS` (default)
> - On a Linux box (if used), ensure it has: `self-hosted`, `Linux` (default)

---

## 3. Prerequisite Software on the Host

Because the self-hosted runner executes jobs directly on your system, your host environment must have the necessary build tools installed.

### macOS Runner Host
Run these commands to ensure your Mac has everything the CI workflows need:
```bash
# Install package manager + system dependencies
brew install rustup node cmake llvm portaudio pkg-config glib libsndfile
rustup-init -y
source "$HOME/.cargo/env"
rustup component add rustfmt clippy

# Install frontend package manager
npm install -g pnpm

# Install cargo utilities used by the workflow
cargo install cargo-llvm-cov
cargo install cargo-deny
```

### Linux Runner Host (if running Linux workflows locally)
Ensure these packages are installed:
```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libglib2.0-dev portaudio19-dev libsndfile1-dev cmake clang patchelf libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev libsoup-3.0-dev
rustup-init -y
source "$HOME/.cargo/env"
rustup component add rustfmt clippy
npm install -g pnpm
cargo install cargo-llvm-cov
cargo install cargo-deny
```

---

## 4. Launching the Runner

Once configured, start the runner by executing the run script in your runner directory:
```bash
./run.sh
```
Keep this terminal window open. The runner will listen for incoming jobs and execute them securely inside your local machine shell.

---

## 5. Tailoring to a Single Machine (macOS only)

If you **only** have a macOS self-hosted runner and do not want to set up a Linux machine, the jobs targeting `Linux` (like the Linux Rust and Linux C++ builds) will stay "queued" forever. 

To easily tailor the CI to run exclusively on your Mac self-hosted runner:
1. Open `.github/workflows/ci.yml`.
2. Locate the `matrix.os` fields in the `rust` and `cpp` jobs.
3. Change `os: [macOS, Linux]` to `os: [macOS]`.

This will immediately restrict the remote checks to run solely on your local Mac runner.

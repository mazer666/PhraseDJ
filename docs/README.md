# Documentation Hub

This folder should contain **first-class product documentation** for PhraseDJ: accurate, task-oriented, and easy to navigate.

## Documentation Goals

- Help a new contributor become productive in <30 minutes.
- Help a maintainer safely change core systems (audio engine, UI, Tauri bridge, configs).
- Help operators release and debug the app with low risk.
- Keep docs close to implementation reality with explicit owners and review cadence.

## Suggested Information Architecture

### 1) Start Here
- `getting-started.md` — local setup, prerequisites, first run, and first test.
- `architecture-overview.md` — system map of frontend, Tauri commands, Rust state, and native audio.
- `app-flowchart.md` — visual flowcharts for request and playback paths.
- `developer-workflows.md` — common tasks (add command, add UI state, add audio feature).

### 2) Product & Domain
- `product-requirements.md` — target users, user journeys, non-goals.
- `glossary.md` — DJ and audio terminology shared across code and specs.

### 3) Technical Deep Dives
- `audio-engine.md` — buffering, timing, mixing, BPM detection, thread model.
- `state-and-commands.md` — Tauri command contract and state ownership.
- `settings-and-variables.md` — keymap/defaults/schema variables and settings reference.
- `function-reference.md` — current command/function surface index.
- `testing-strategy.md` — test pyramid, coverage priorities, regression playbook.

### 4) Operations
- `release-runbook.md` — versioning, build matrix, signing, smoke checks.
- `incident-response.md` — troubleshooting patterns and known failure modes.
- `performance-playbook.md` — latency budgets and profiling steps.

### 5) Governance
- `documentation-standards.md` — style, templates, required sections, review checklist.
- `documentation-roadmap.md` — gaps, owners, and ETA for improving docs.
- `plugin-system.md` — implementation-oriented plugin overview and extension points.

## Quality Bar for “First-Class” Docs

Each major doc should include:

1. **Purpose and scope** (what this covers / excludes)
2. **Audience** (new dev, maintainer, release engineer)
3. **Canonical workflows** (step-by-step, copy-pasteable commands)
4. **Diagrams or data flows** where complexity exists
5. **Failure modes and troubleshooting**
6. **Links to source of truth** (code paths and specs)
7. **Last reviewed date + owner**

## Current Status

- Existing action plan: `AUDIT_ACTION_PLAN.md`
- Broader technical specs currently live under `/specs`

Use this folder to convert specs + implicit knowledge into practical, maintainable docs.

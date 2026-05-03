# Developer Workflows

**Audience:** Active contributors  
**Owner:** Core maintainers  
**Last reviewed:** 2026-05-02

## Purpose
Canonical step-by-step flows for common changes.

## 1) Add or update a UI component
1. Edit component in `apps/desktop/src/components`.
2. Add/update component tests in same directory (`*.test.tsx`).
3. Run:
   ```bash
   cd apps/desktop && pnpm test
   cd /workspace/PhraseDJ && make lint
   ```

## 2) Add a Tauri command
1. Implement command handler in `apps/desktop/src-tauri/src/commands`.
2. Wire command registration in command module tree.
3. Add/adjust UI call site in `apps/desktop/src/lib/api.ts`.
4. Run:
   ```bash
   cargo test --workspace --all-features
   cd apps/desktop && pnpm test
   ```

## 3) Change shared state behavior
1. Update state definitions/logic in `apps/desktop/src-tauri/src/state.rs` (or relevant store file).
2. Validate calling command invariants.
3. Run:
   ```bash
   cargo test --workspace --all-features
   make lint
   ```

## 4) Modify native audio behavior
1. Edit `native/audio/src` C++ sources.
2. Add/update tests in `native/audio/tests`.
3. Run:
   ```bash
   make test-cpp
   make ci-minimal
   ```

## 5) Documentation and spec updates
1. Update relevant files in `/docs` and `/specs` together when behavior changes.
2. Ensure command examples are reproducible.
3. Keep “Last reviewed” and owner fields current.

## PR checklist
- [ ] Conventional commit message
- [ ] Relevant tests executed
- [ ] Docs/specs updated for behavior changes
- [ ] No new warnings in lint/type checks

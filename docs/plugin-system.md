# Plugin System Overview

**Audience:** Contributors extending PhraseDJ  
**Owner:** Core maintainers  
**Last reviewed:** 2026-05-02

## Purpose
Summarize how PhraseDJ plugin extensibility is structured and where to implement/support it.

## Plugin tracks
1. **CLAP audio plugins** (`.clap` bundles) for DSP effects.
2. **JavaScript scripts** (QuickJS sandbox) for automation and custom workflows.
3. **MCP tools** for local AI/agent integrations.

## Expected host behavior
- Discovery paths and loading for CLAP plugins.
- Sandboxed JS API surface with explicit permissions.
- MCP bridge opt-in with clear user visibility and auditability.
- Failure isolation: plugin/script/tool failures must degrade gracefully.

## Related spec
Primary design details are in `specs/09-plugin-system.md`.

## Integration points to document next
- Actual runtime plugin loader modules and API contracts.
- User-facing install/enable/disable flow.
- Compatibility/versioning matrix.

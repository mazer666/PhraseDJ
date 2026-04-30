# 09 — Plugin System

## 1. Goals

- Allow third parties to add audio effects, UI panels, and AI-driven helpers
  without forking the codebase.
- Keep the core stable: plugin failures must not crash the host.
- License-friendly: choose CLAP for audio plugins (no Steinberg licence
  burden, MIT-spirit standard).
- Welcome AI agents through MCP — they can browse the library, inspect the
  current state, and propose macros.

## 2. Three plugin tracks

| Track | Purpose | Format |
|---|---|---|
| **CLAP audio plugins** | DSP effects on master / decks / stems | `.clap` bundle |
| **JS scripts** | UI actions, automations, custom transitions | `.js` files in a sandbox |
| **MCP tools** | Outside processes (Claude, Gemini, custom agents) | local JSON-RPC over stdio / unix socket |

## 3. CLAP host

- Linked via the official CLAP headers, no third-party host wrappers.
- Plugins discovered in:
  - `<app>/Contents/PlugIns/CLAP` (bundled)
  - `<app-support>/PhraseDJ/plugins/clap`
  - `~/Library/Audio/Plug-Ins/CLAP` (system)
- Each plugin runs in the host process, but with a watchdog: a plugin that
  exceeds a CPU budget for 200 ms is muted and surfaced in a UI warning.
- Parameters auto-mapped to MIDI / scripting via the same intent system as
  built-in controls.

Built-in CLAP plugins shipped with PhraseDJ:

- 3-band EQ
- Filter (LP / HP combo)
- Echo / Delay
- Reverb (Schroeder + simple FDN)
- Beat-roll / Stutter
- Limiter (also used as master)

Each plugin lives in `plugins/<name>/` as its own crate / target with the
600-line file rule.

## 4. JS scripting

- Engine: `rquickjs` (QuickJS bindings).
- Scripts run in a sandbox: only the API surface PhraseDJ exposes is
  reachable; no `require`, no filesystem, no network.
- Manifest per script:

```toml
[script]
id          = "auto-fade-on-end"
name        = "Auto fade-out 8 bars before end"
version     = "0.1.0"
permissions = ["deck.read", "deck.write_fader"]
entrypoint  = "main.js"
```

- Permissions are explicit. The user approves them at install time. They
  appear in `settings.toml` so revocation is trivial.

Example script:

```js
// Auto-fade the deck ending soonest 8 bars before its end.
on('tick', state => {
  const a = state.deckA, b = state.deckB;
  const ending = [a, b].find(d => d.barsRemaining <= 8 && d.fader > 0.05);
  if (ending) ending.setFader(ending.fader * 0.96, { ease: 'cosine' });
});
```

API namespace:

- `engine.*` — read state
- `deck.*` — read/write deck params
- `library.*` — list, query (read-only)
- `macro.*` — create, edit, list
- `event.on/off` — subscribe to ticks, transitions, MIDI

## 5. MCP bridge

- PhraseDJ ships an MCP server (stdio by default; opt-in unix-socket mode for
  same-host AI tools).
- Exposes a curated tool set:
  - `library.search`
  - `track.get_metadata`
  - `track.suggest_transition` (server-side LLM-free heuristic)
  - `deck.get_state`
  - `macro.create`, `macro.list`, `macro.apply`
- The MCP server is **off by default**. Enabling it requires explicit user
  action and shows a clear "AI agent connected" indicator in the UI.
- Audit log captures every tool call.

This makes it straightforward for Claude Code, Gemini, or custom agents to
sit alongside PhraseDJ during a session and propose ideas — while keeping
the user in full control.

## 6. Crate `pdj-plugins` layout

```
pdj-plugins/
  src/lib.rs
  src/clap/mod.rs        host, scanner, watchdog
  src/clap/host.rs
  src/clap/scanner.rs
  src/clap/watchdog.rs
  src/js/mod.rs          quickjs runtime, sandbox
  src/js/api.rs
  src/js/sandbox.rs
  src/mcp/mod.rs         MCP server
  src/mcp/tools.rs
  src/mcp/audit.rs
```

Each file ≤ 400 lines.

## 7. Failure handling

- A CLAP plugin throwing or blocking is bypassed (audio passes dry) and a
  notification is raised.
- A JS script raising an exception logs once, then is suspended until the
  user reloads it.
- An MCP tool call is rate-limited (default: 10 req/s) and gracefully
  returns errors on overload.

## 8. Versioning and compatibility

- The intent / API namespace is versioned (`v1.deck.fader`).
- Breaking changes bump the major namespace; old scripts keep working until
  removed in a later major release.
- Plugin manifests declare a minimum host version; installing an
  incompatible plugin shows a clear error.

## 9. Security posture

- No remote plugin marketplace in MVP — installation is manual file copy.
- Plugins are not auto-updated.
- A "safe boot" launch flag disables all plugins for crash recovery.

## 10. Testing

- A reference CLAP plugin (gain) ships in the repo as a test fixture.
- The JS sandbox has tests that confirm forbidden APIs are unavailable.
- The MCP server has contract tests for every exposed tool.

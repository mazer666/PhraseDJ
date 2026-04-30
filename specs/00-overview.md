# 00 — Overview, Vision, Scope

## 1. Vision statement

PhraseDJ is a free, open-source DJ application built around three convictions:

1. **A clean interface scales with skill.** Beginners should not be paralysed
   by knobs they don't yet understand; advanced users should not be slowed
   down by chrome they don't need.
2. **AI belongs on the user's machine.** Stem separation, phrase detection
   and lyrics alignment all run locally on Apple Silicon (and ONNX-capable
   hardware later). The user's library never leaves the device.
3. **Performances are programmable.** A great transition is a craft skill —
   PhraseDJ records it, lets the user edit it, and replays it with the same
   feel next time.

## 2. Target users

| Persona | Description | Primary needs |
|---|---|---|
| **Hobby DJ at home** | Plays for friends, learns the craft, owns a local music collection | Easy setup, forgiving UI, AI assistance, no subscription |
| **Bar / small-event DJ** | Plays 2–4 hour sets in non-club venues | Crash-free operation, MIDI controller support, panic stop |
| **Producer / curator** | Uses DJ software to audition and prep their own tracks | Stem control, macros, scripting, plugin host |
| **Tinkerer / OSS contributor** | Wants to extend the tool | Clean architecture, scripting, CLAP plugin API, MCP |

The maintainer themselves is a *programming beginner* — therefore the entire
codebase must be readable by a newcomer (see `LLM.md`, rule 2).

## 3. Scope (in)

- Two-deck mixing with low-latency local audio
- Local music library with metadata, beatgrid, key, energy, phrase markers
- Local stem separation (HTDemucs, 4-stem default)
- Lyrics: tag-based, LRC, **online lookup (opt-in)**, local Whisper alignment
- Transition macros: record, save, edit, replay
- MIDI input + keyboard / mouse operation (no controller required)
- CLAP plugin host + JavaScript scripting + MCP bridge
- macOS-first; cross-platform groundwork from day 1

## 4. Scope (out — non-goals)

The following are explicitly **not** part of PhraseDJ. Do not add them, do not
prepare hooks for them:

- Streaming-service integration (Spotify, TIDAL, Beatport, SoundCloud, …)
- Video mixing, lighting control, DMX, visual VJ
- Cloud-synced library or settings
- Telemetry, analytics, A/B testing
- Subscription, paywalls, in-app purchases
- Built-in DRM
- Social features (sharing sets, follow graph, comments)

A future fork or plugin may add any of these; the core must not.

## 5. Quality attributes (-ilities)

| Attribute | Concrete target |
|---|---|
| Reliability | 4 h continuous playback without crash; auto-save every 10 s |
| Performance | < 10 ms audio round-trip; UI 60 fps idle, 120 fps where supported |
| Usability | New user can play their first mix within 10 minutes |
| Maintainability | Files ≤ 600 lines (target 400), public APIs documented |
| Portability | Engine + crates compile on macOS, Linux, Windows; UI runs unchanged |
| Privacy | No outbound traffic except documented opt-in lyrics lookup |
| Accessibility | Keyboard-only operation possible; respects system contrast settings |
| Internationalisation | UI strings externalised; English default, German shipped |

## 6. Project values (= acceptance bar for any PR)

- **Local-first** — feature works offline; online is at most a fallback
- **Externalised settings** — no magic numbers, all in `config/*.toml`
- **Beginner-readable** — code reads like prose, public APIs documented
- **Tested** — every module ships with tests
- **Bounded files** — 600 hard / 400 soft line limit
- **One responsibility per module** — boundaries from `01-architecture.md`
- **No surprise traffic** — only opt-in network calls, all logged

## 7. Document map

| Spec | Topic |
|---|---|
| 00-overview.md | this file |
| 01-architecture.md | tech stack, layers, data flow |
| 02-audio-engine.md | CoreAudio/PortAudio, latency, decoders, beatgrid |
| 03-ai-stems.md | MLX/ONNX, HTDemucs, background pipeline |
| 04-ui-ux.md | modes (Classic / Stem / Macro), layout, design tokens |
| 05-transitions-macros.md | recorder, replay, save & recall |
| 06-lyrics.md | tag/LRC, online lookup, Whisper alignment |
| 07-library.md | local library, schema, scan, metadata |
| 08-midi-input.md | MIDI learn, keyboard map, mouse gestures |
| 09-plugin-system.md | CLAP host, JS scripting, MCP bridge |
| 10-performance.md | latency budget, profiling, benchmarks |
| 11-testing-qa.md | unit / integration / audio tests, CI gates |
| 12-build-release.md | toolchain, CI, notarisation, distribution |
| glossary.md | DJ and engineering terms used throughout |

## 8. Versioning

Pre-1.0 timeline, semantic versioning starts at 1.0:

- **0.1** — end of Phase 1 (audio core + decks)
- **0.3** — end of Phase 2 (stems)
- **0.5** — end of Phase 3 (macros)
- **0.7** — end of Phase 4 (lyrics + live hardening)
- **0.9** — end of Phase 5 (plugins + polish), public beta
- **1.0** — first stable release after public beta feedback

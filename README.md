# PhraseDJ

> A modern, AI-assisted, uncompromisingly clean open-source DJ application.
> macOS-first, fully local, no subscriptions, no cloud lock-in.

**Status:** 🌱 Pre-Alpha — specification and planning phase
**License:** MPL-2.0
**Platform:** macOS 13+ (Apple Silicon optimized) · Linux/Windows from Phase 5

---

## Vision

PhraseDJ combines the visual clarity of Djay Pro with the functional depth of
Virtual DJ on top of an open foundation inspired by Mixxx — without feature
overload, without subscription pressure, without streaming lock-in. Local
music, local AI, full control.

**Three guiding principles:**

1. **Uncluttered.** Every UI element must justify its existence.
2. **Phrase-aware.** AI detects song structure (intro, drop, chorus, outro)
   and surfaces the right transition moment visually.
3. **Programmable.** Every manually performed transition can be saved as a
   macro, edited, and recalled exactly or semi-automatically next time.

## Core features (MVP)

- **Two-deck engine** with low latency (< 10 ms target)
- **Local stem separation** (HTDemucs on Apple MLX, ~30× realtime)
- **Phrase and beatgrid analysis** in the background on import
- **Transition recorder & macros** — manual mix → reusable macro
- **Lyrics** with local LRC lookup and Whisper-based forced alignment
- **MIDI learn** + full keyboard / mouse operation
- **CLAP plugin host** + JavaScript scripting
- **MCP bridge** for AI agents (Claude, Gemini, …)

## What PhraseDJ is **not**

- Not a streaming-service client (Spotify, TIDAL, Beatport)
- No video mixing
- No cloud library, no telemetry
- No subscription, ever

## Repository layout

```
PhraseDJ/
├── README.md            ← this file
├── LLM.md               ← guide for AI agents (vibe-coding rules)
├── PROJECT_PLAN.md      ← 18–24 month roadmap
├── LICENSE              ← MPL-2.0
└── specs/               ← full requirements specification (start at specs/00-overview.md)
```

Once code lands:

```
├── apps/desktop/        ← Tauri app (Rust + TS/React)
├── crates/              ← Rust modules (audio bridge, library, macros, …)
├── native/audio/        ← C++ audio engine (CoreAudio + PortAudio)
├── ml/                  ← MLX/ONNX models and inference wrappers
├── plugins/             ← built-in CLAP plugins (EQ, filter, echo)
└── tests/               ← unit, integration, and audio tests
```

## Tech stack (short form)

| Layer | Technology |
|---|---|
| UI | Tauri 2 + React + TypeScript + Vite |
| App logic | Rust |
| Audio engine | C++20 (CoreAudio on macOS, PortAudio elsewhere) |
| AI inference | Apple MLX (macOS) · ONNX Runtime (cross-platform) |
| Stem model | HTDemucs (4-stem default) |
| Plugins | CLAP standard |
| Scripting | QuickJS (embedded) |
| Database | SQLite |

Details: [`specs/01-architecture.md`](specs/01-architecture.md).

## Quickstart (once Phase 1 ships)

> Not yet runnable — comes with Phase 1, see `PROJECT_PLAN.md`.

```bash
# Prerequisites
brew install rustup node cmake
rustup-init -y

# Build
git clone https://github.com/mazer666/phodj.git PhraseDJ
cd PhraseDJ
pnpm install
pnpm tauri dev
```

## Contributing

PhraseDJ is developed AI-assisted ("vibe coding"). Read [`LLM.md`](LLM.md)
before letting an AI agent touch the code — it defines code structure,
mandatory tests, and style rules (including: max 600 lines per file, target
400, settings externalised).

Issues and PRs are welcome once Phase 1 begins.

## License

[Mozilla Public License 2.0](LICENSE) — open source with file-level copyleft.
Plugins may ship under their own license.

## Acknowledgements

PhraseDJ stands on the shoulders of:

- [Mixxx](https://mixxx.org/) — inspiration for the audio engine
- [Demucs](https://github.com/facebookresearch/demucs) — stem separation
- [Apple MLX](https://github.com/ml-explore/mlx) — on-device AI
- [CLAP](https://cleveraudio.org/) — plugin standard
- [Tauri](https://tauri.app/) — application framework

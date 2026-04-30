# Glossary

DJ and engineering vocabulary used throughout the PhraseDJ specs and code.

## DJ terms

- **Beatgrid** — the time-aligned grid of beats in a track; used for sync,
  loops, and macro anchoring.
- **BPM** — beats per minute; tempo of a track.
- **Camelot** — a key-naming wheel that simplifies harmonic mixing
  (e.g. 8A, 8B).
- **Cue point** — a marker the DJ jumps to instantly; PhraseDJ supports 8
  per track.
- **Crossfader** — the horizontal fader that blends the master output of
  two decks.
- **Drop** — the high-energy moment, often after a build-up; a common
  alignment target for transitions.
- **Echo out** — a transition where the playing track is gradually replaced
  by its delayed echoes while the next track comes in.
- **EQ** — equaliser, here a 3-band low/mid/high in Classic mode; replaced
  by 4 stem faders in Stem mode.
- **Filter** — high-pass / low-pass sweep, often used during transitions.
- **Hot cue** — alternate name for cue points.
- **Keylock** — preserve key while changing tempo (or vice versa).
- **Loop** — a section repeated indefinitely; can be set in or out, halved
  or doubled.
- **Mashup** — combining elements (e.g. vocals from one track over the
  instrumental of another).
- **Master deck** — the deck whose tempo other decks sync to.
- **Phrase** — a musical section like intro, verse, chorus, drop, outro.
- **Pitch fader** — adjusts playback rate (and thus key, unless keylock is
  on).
- **Scratch** — moving the audio back and forth manually for percussive
  effect.
- **Sync** — automatically match BPM and phase of the slave deck to the
  master.
- **Stems** — separated audio components (vocals / drums / bass / other).
- **Transition** — the audible move from one track to the next.

## Engineering terms

- **Audio callback** — the realtime function the OS calls to fill the
  output buffer; must be allocation-free and lock-free.
- **CLAP** — Clever Audio Plug-in API, a modern open plugin standard.
- **CoreAudio** — Apple's low-level audio framework on macOS.
- **CC (MIDI)** — Continuous Controller message; carries a value 0..127.
- **DAW** — Digital Audio Workstation; PhraseDJ is not one but borrows ideas.
- **FFI** — Foreign Function Interface; the bridge from Rust to C/C++ here.
- **FTZ / DAZ** — flush-to-zero / denormals-are-zero; CPU flags for fast FP.
- **Latency** — time from audio source to output (or input to engine).
- **LRC** — text format for time-stamped lyrics (`[mm:ss.xx] line`).
- **LRU** — Least Recently Used; a cache eviction policy.
- **MCP** — Model Context Protocol; lets external AI agents call tools.
- **MIDI learn** — interactive binding of a hardware control to a software
  parameter.
- **MLX** — Apple's ML framework for Apple Silicon.
- **MPL-2.0** — Mozilla Public License 2.0; this project's licence.
- **ONNX** — Open Neural Network Exchange; portable model format.
- **Overlap-add** — segment-based DSP technique used here for stem
  separation.
- **PCM** — Pulse Code Modulation; uncompressed audio samples.
- **Phrase detection** — model that labels song sections (intro, drop, …).
- **PortAudio** — cross-platform audio I/O library.
- **Realtime-safe** — code that runs in the audio callback without
  allocations, locks, or syscalls.
- **Ring buffer** — a fixed-size FIFO used for lock-free hand-off.
- **Round-trip latency** — input event to audible output time.
- **SPSC** — single-producer / single-consumer; the lock-free buffer
  variant used in the engine.
- **Soft takeover** — prevents a hardware fader from jumping to its
  position when its software counterpart has moved separately.
- **Tauri** — the Rust-based desktop app framework hosting the UI.
- **Whisper** — open-source speech-to-text model used for forced lyrics
  alignment via `whisper.cpp`.

# 08 — MIDI, Keyboard, Mouse

## 1. Goals

- Full keyboard / mouse operation as a first-class path. No controller required.
- Optional MIDI controller support with a simple "learn" workflow.
- All mappings in editable config files, never hard-coded.
- Predictable, low-latency input handling.

## 2. Input layers

```
device  →  parser  →  intent  →  command  →  engine
```

| Stage | Owner | Notes |
|---|---|---|
| device | OS / midir | raw events |
| parser | `pdj-midi` / Tauri keyboard hook | per-input class |
| intent | shared | semantic action e.g. `Deck::TogglePlay(A)` |
| command | Rust app layer | turns intent into engine call |
| engine | C++ engine | applies setter / triggers |

## 3. Keyboard map

Default map (English QWERTY) lives in `config/defaults.toml`; the user copy
sits in `<app-support>/PhraseDJ/keymap.toml`. The settings UI provides a
graphical editor; the file format is human-readable.

```toml
[keymap]
"Q" = "deck.A.toggle_play"
"W" = "deck.A.cue"
"E" = "deck.A.sync"
"P" = "deck.B.toggle_play"
"O" = "deck.B.cue"
"I" = "deck.B.sync"
"Space" = "master.pause_toggle"
"F" = "view.toggle_fullscreen"
"Esc" = "view.close_modal"
"Y" = "view.lyrics_cycle"
```

Modifier syntax: `Ctrl+Shift+F`, `Cmd+L`, `Alt+1`. Layout-independent
fallback to physical key codes for non-Latin keyboards.

## 4. Mouse gestures

- Vertical drag on a deck waveform = filter (configurable: filter / pitch /
  echo dry-wet).
- Horizontal drag = scrub. With `Shift`, fast scrub.
- Wheel on a knob = ±1 % step. With `Alt`, ±10 %.
- Double-click on a knob = reset to default.
- Right-click on any control = context menu (MIDI-learn, reset, copy value,
  unbind).

## 5. MIDI

- Library: `midir` (cross-platform) for device enumeration and I/O.
- Listener thread reads messages, converts to intents, posts via mpsc.
- Latency budget: < 5 ms from MIDI event to engine setter.
- Every controller is named in `pdj-midi/profiles/<vendor-model>.toml`;
  defaults shipped for popular cheap controllers (e.g. Pioneer DDJ-200,
  Numark Mixtrack Pro FX, Hercules DJControl Inpulse 200).

### Mapping format (excerpt)

```toml
device = "DDJ-200"
match.manufacturer = "Pioneer"
match.product      = "DDJ-200"

[[bindings]]
when      = { type = "note_on", channel = 0, note = 0x18 }
intent    = "deck.A.toggle_play"

[[bindings]]
when      = { type = "cc", channel = 0, controller = 0x1F }
intent    = "deck.A.fader"
range     = { min = 0, max = 127, target_min = 0.0, target_max = 1.0 }
```

## 6. MIDI learn

- Right-click any control → "MIDI learn".
- Next MIDI message captured becomes the binding for that control.
- The learned binding lands in the user's keymap file with a comment trail
  indicating where it came from.
- "Forget" / "Replace" actions in the bindings settings.

## 7. Conflict resolution

- A single intent can have multiple sources (keyboard + MIDI + UI).
- Multiple sources for the same control: last-write-wins for absolute values,
  relative deltas accumulate.
- A "takeover" mode for hardware faders prevents jumps when a software fader
  has moved without the hardware: the hardware fader is ignored until it
  passes the current software value.

## 8. Crate `pdj-midi` API

```rust
pub fn list_devices() -> Result<Vec<DeviceInfo>>;
pub fn open(device: &str, profile: Option<&Path>) -> Result<Session>;
pub fn close(session: Session) -> Result<()>;
pub fn subscribe() -> impl Stream<Item = MidiEvent>;
pub fn learn_next(target_intent: &str) -> Result<MidiBinding>;
pub fn save_binding(b: MidiBinding) -> Result<()>;
```

Internal modules:

- `mod devices`   — enumeration, hot-plug
- `mod session`   — open/close, threads
- `mod profile`   — TOML loader, validation
- `mod intent`    — string parser for `"deck.A.fader"` etc.
- `mod takeover`  — soft-takeover state per control

Each file ≤ 400 lines.

## 9. Accessibility

- Keyboard-only operation must work even without focus on a specific element
  (global shortcuts).
- Hold-to-repeat on tempo-nudge keys.
- Optional "sticky modifiers" for users who can't hold multiple keys.
- Mouse alternatives for every gesture.

## 10. Testing

- Synthetic MIDI streams replayed in tests verify intent → command mapping.
- Latency test injects a MIDI event and measures engine reaction time.
- Conflict tests cover all combinations of UI/keyboard/MIDI hitting the same
  control.

# 04 — UI / UX

## 1. Design principles

- **Quiet by default.** Dark theme, low-saturation neutrals, accents earned by
  state (playing = warm; cue = blue; warning = amber).
- **One thing per surface.** Each panel answers one question (which track is
  playing, where am I in it, what's coming next).
- **Reveal complexity on demand.** Stem mixer, macro editor, and plugin
  routing are hidden behind a single mode switch, not piled onto one screen.
- **Latency-honest.** Visual elements that drive audio (faders, knobs,
  buttons) feel instantaneous (< 16 ms input-to-render). Decorative
  elements are budget-aware.

## 2. Three modes

The top-right of the toolbar exposes a three-way switch:

| Mode | When to use | What changes |
|---|---|---|
| **Classic** | Beginners, quick mixes | 3-band EQ per deck, simple FX rack |
| **Stem** | Modern mashups, harmonic mixing | EQ replaced by 4 stem faders |
| **Macro** | Programming transitions | Timeline view between decks, automation curves |

Switching modes never stops playback. Settings within each mode are
remembered per-session.

## 3. Default layout

```
┌─ Toolbar ─────────────────────────────────────────────────────┐
│  [Library] [Mode: Classic|Stem|Macro] [Master] [Settings]    │
├──────────────────────────┬─────────────────────────────────────┤
│ Deck A                   │ Deck B                              │
│ ┌────────────────────┐   │ ┌────────────────────┐              │
│ │  Track artwork     │   │ │  Track artwork     │              │
│ │  Title / Artist    │   │ │  Title / Artist    │              │
│ │  Key · BPM · Time  │   │ │  Key · BPM · Time  │              │
│ └────────────────────┘   │ └────────────────────┘              │
│ ▓▓▓▓ overview waveform   │ ▓▓▓▓ overview waveform              │
│ ▓▓▓▓ zoomed waveform     │ ▓▓▓▓ zoomed waveform                │
│ [▶ Cue Sync Loop FX]     │ [▶ Cue Sync Loop FX]                │
├──────────────────────────┴─────────────────────────────────────┤
│  [Stem / EQ controls]   [Crossfader]   [Stem / EQ controls]   │
├──────────────────────────────────────────────────────────────┤
│ Lyrics / Phrase / Macro panel (mode-dependent)                │
├──────────────────────────────────────────────────────────────┤
│ Library browser (collapsible drawer)                          │
└──────────────────────────────────────────────────────────────┘
```

## 4. Waveforms

- **Overview**: full track, 30–50 px tall, single-colour gradient, marks for
  cues, phrases, current position.
- **Zoomed**: ~4 seconds visible, beat lines, tick subdivisions.
- **Stem-coloured option** (Stem mode): each stem contributes a layer
  - Vocals: warm red
  - Drums: yellow
  - Bass: deep blue
  - Other: grey-green
  Layers are alpha-blended, with a checkbox per stem to dim/hide.

Rendering is GPU-driven (`wgpu`) targeting 120 fps on ProMotion, falling back
gracefully to 60 fps. Computation runs on a worker; the audio thread is not
involved.

## 5. Controls

### Faders & knobs

- Linear faders for volume / crossfader / stems.
- Knobs for filter / FX with a dotted "default" indicator.
- Double-click resets to default.
- Right-click opens a small contextual menu (MIDI-learn, reset, copy value).

### Buttons

- Big, low-chrome, with state-based fill rather than hard borders.
- Cue button responds to press AND release distinctly (press = preview-from-
  cue, release = stop unless playing).

## 6. Keyboard-first operation

Defaults (configurable via `keymap.toml`):

| Action | Deck A | Deck B |
|---|---|---|
| Play / Pause | `Q` | `P` |
| Cue | `W` | `O` |
| Sync | `E` | `I` |
| Tempo nudge − | `1` | `8` |
| Tempo nudge + | `2` | `9` |
| Loop in / out | `A` / `S` | `K` / `L` |
| Stem 1 mute (vocals) | `Z` | `M` |
| Stem 2 mute (drums) | `X` | `,` |
| Stem 3 mute (bass) | `C` | `.` |
| Stem 4 mute (other) | `V` | `/` |

Global:

- `Space` — Master pause toggle (panic-safe, fades out)
- `F` — toggle full-screen
- `Ctrl/Cmd + L` — focus library search
- `Ctrl/Cmd + ,` — settings
- `Esc` — close current modal

### Mouse gestures

- Vertical drag on a deck waveform = filter knob (context-aware).
- Horizontal drag = scrub (slow), `Shift+drag` = fast scrub.
- Mouse wheel on knob = fine adjustment (1 % step), `Alt+wheel` = coarse.
- Drag-and-drop file → loads to the deck under the cursor.

## 7. Library browser

- Drawer, collapsible, 320 px default width.
- Columns: artwork, title, artist, BPM, key, length, energy, stem-status.
- Sticky search box, filters (BPM range, key, has-stems, has-macro).
- Crates / smart playlists as a left sub-tree.
- Drag from list → deck loads track; drag → another crate adds to crate.

## 8. Macro / transition editor

Visible only in Macro mode:

- Horizontal timeline aligned to bars (4 / 8 / 16 / 32-bar windows).
- Two horizontal lanes per deck (volume, filter), more lanes for stems.
- Curves are editable points; right-click to delete, drag to move.
- An **anchor marker** locks the macro to the beatgrid — moving the track
  does not desync the macro.
- A simulation slider previews the macro without committing audio.

Details: `specs/05-transitions-macros.md`.

## 9. Lyrics panel

- Bottom panel (Phase 4), karaoke-style scrolling text.
- Active line highlighted with progress mask.
- Source indicator (tag / LRC / Whisper / online) is always shown next to
  the panel header.
- Hide button always one click away.

## 10. Accessibility

- All controls are keyboard-reachable.
- Focus rings respect macOS "Increase contrast" setting.
- Tooltips with longer descriptions on hover, and via a help layer on `?`.
- Color is never the only indicator (icons / labels back it up).

## 11. Internationalisation

- All strings live in `apps/desktop/src/i18n/<locale>.json`.
- Default locale follows OS setting; English and German shipped at 1.0.
- Date / time / number formatting via the `Intl` API.

## 12. Visual tokens

Tokens defined once, in `apps/desktop/src/theme/tokens.ts` (mirrored in
`config/defaults.toml` for non-UI consumers like macro renderers):

- Colour palette (background, surface, divider, text, accent-warm,
  accent-cool, danger)
- Typography scale (12 / 13 / 14 / 16 / 20 / 28 / 36)
- Spacing scale (4 / 8 / 12 / 16 / 24 / 32 / 48)
- Radius scale (2 / 4 / 8 / 16)
- Motion (cubic-bezier curves, durations 80 / 160 / 240 ms)

Adding a new colour/spacing in code, instead of using a token, is a spec
violation.

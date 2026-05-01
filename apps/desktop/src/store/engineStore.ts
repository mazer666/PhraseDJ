/**
 * engineStore.ts — Zustand store for the audio engine state.
 *
 * Mirrors the Rust-side state via periodic polling of `deck_state`.
 * Phase 2 will replace polling with Tauri events for lower overhead, but
 * for Phase 1 a 60-Hz poll is plenty.
 */

import { create } from "zustand";
import { deckApi, mixerApi, type DeckState, type WaveformData } from "../lib/api";

const POLL_INTERVAL_MS = 50; // 20 fps – plenty for transport indicators

interface EngineStore {
  // Per-deck state.
  decks: [DeckState, DeckState];

  // Waveform peak data per deck (null until loaded).
  waveforms: [WaveformData | null, WaveformData | null];

  // Mixer state (kept locally; pushed to engine on change).
  faderA: number;
  faderB: number;
  crossfader: number;
  masterGain: number;

  // Lifecycle.
  startPolling: () => void;
  stopPolling: () => void;

  // Actions.
  load:    (deck: 0 | 1, path: string) => Promise<void>;
  play:    (deck: 0 | 1) => Promise<void>;
  pause:   (deck: 0 | 1) => Promise<void>;
  setFader:      (deck: 0 | 1, value: number) => Promise<void>;
  setCrossfader: (value: number) => Promise<void>;
  setMasterGain: (value: number) => Promise<void>;
  /** Sync a deck's tempo to the opposite deck's BPM. */
  sync:    (deck: 0 | 1) => Promise<void>;
  /** Nudge playback speed by a small delta (positive = faster). */
  nudgeTempo: (deck: 0 | 1, delta: number) => Promise<void>;
  /** Set playback speed ratio directly (1.0 = normal). */
  setTempoRatio: (deck: 0 | 1, ratio: number) => Promise<void>;
}

const blankState = (deck: number): DeckState => ({
  deck,
  loaded: false,
  playing: false,
  position: 0,
  bpm: 0,
  tempo_ratio: 1.0,
});

let pollHandle: number | null = null;

export const useEngineStore = create<EngineStore>((set, get) => ({
  decks: [blankState(0), blankState(1)],
  waveforms: [null, null],
  faderA: 1.0,
  faderB: 1.0,
  crossfader: 0.5,
  masterGain: 1.0,

  startPolling: () => {
    if (pollHandle !== null) return;
    const tick = async () => {
      try {
        const [a, b] = await Promise.all([
          deckApi.state(0),
          deckApi.state(1),
        ]);
        set({ decks: [a, b] });
      } catch (e) {
        // Engine unavailable – leave state as-is.
        // Logged once, not on every tick, to avoid console spam.
      }
    };
    pollHandle = window.setInterval(tick, POLL_INTERVAL_MS);
  },

  stopPolling: () => {
    if (pollHandle !== null) {
      window.clearInterval(pollHandle);
      pollHandle = null;
    }
  },

  load: async (deck, path) => {
    await deckApi.load(deck, path);
    // Compute waveform peaks in the background after load.
    deckApi.waveform(deck).then((data) => {
      set((s) => {
        const next: [WaveformData | null, WaveformData | null] = [...s.waveforms] as [WaveformData | null, WaveformData | null];
        next[deck] = data;
        return { waveforms: next };
      });
    }).catch(() => { /* waveform unavailable — no-op */ });
  },
  play:  (deck) => deckApi.play(deck),
  pause: (deck) => deckApi.pause(deck),

  setFader: async (deck, value) => {
    await mixerApi.setFader(deck, value);
    set(deck === 0 ? { faderA: value } : { faderB: value });
  },

  setCrossfader: async (value) => {
    await mixerApi.setCrossfader(value);
    set({ crossfader: value });
  },

  setMasterGain: async (value) => {
    await mixerApi.setMasterGain(value);
    set({ masterGain: value });
  },

  sync: (deck) => deckApi.sync(deck),

  nudgeTempo: (deck, delta) => deckApi.nudgeTempo(deck, delta),

  setTempoRatio: (deck, ratio) => deckApi.setTempoRatio(deck, ratio),
}));

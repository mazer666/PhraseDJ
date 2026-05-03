/**
 * engineStore.ts — Zustand store for the audio engine state.
 *
 * Mirrors the Rust-side state via periodic polling of `deck_state`.
 * Stem completion events from the Tauri backend trigger automatic
 * hot-swap reloads so the user never needs to manually reload.
 */

import { create } from "zustand";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  deckApi,
  mixerApi,
  type DeckState,
  type WaveformData,
} from "../lib/api";

const POLL_INTERVAL_MS = 50; // 20 fps – plenty for transport indicators

/** Payload from the Rust `forward_stem_events` bridge. */
interface StemStatusEvent {
  track_id: string;
  status: "pending" | "running" | "model_downloading" | "cached" | "failed";
  progress: number | null;
  reason: string | null;
}

interface EngineStore {
  // Per-deck state.
  decks: [DeckState, DeckState];

  // Waveform peak data per deck (null until loaded).
  waveforms: [WaveformData | null, WaveformData | null];

  // Path last loaded per deck (for hot-swap reload after stem analysis).
  loadedPaths: [string | null, string | null];
  loadedTrackIds: [string | null, string | null];

  // Active stem analysis jobs: track_id -> { status, progress, reason }
  stemJobs: Record<
    string,
    { status: string; progress: number; reason?: string }
  >;
  stemStatusForDeck: (
    deck: 0 | 1,
  ) => { status: string; progress: number; reason?: string } | null;

  // Mixer state (kept locally; pushed to engine on change).
  faderA: number;
  faderB: number;
  stemGainsA: [number, number, number, number];
  stemGainsB: [number, number, number, number];
  stemMuteA: [boolean, boolean, boolean, boolean];
  stemMuteB: [boolean, boolean, boolean, boolean];
  crossfader: number;
  masterGain: number;

  // Lifecycle.
  startPolling: () => void;
  stopPolling: () => void;

  // Actions.
  load: (deck: 0 | 1, path: string) => Promise<void>;
  play: (deck: 0 | 1) => Promise<void>;
  pause: (deck: 0 | 1) => Promise<void>;
  setFader: (deck: 0 | 1, value: number) => Promise<void>;
  setCrossfader: (value: number) => Promise<void>;
  setMasterGain: (value: number) => Promise<void>;
  setStemGain: (
    deck: 0 | 1,
    stem: 0 | 1 | 2 | 3,
    value: number,
  ) => Promise<void>;
  setStemOutputGain: (
    deck: 0 | 1,
    stem: 0 | 1 | 2 | 3,
    value: number,
  ) => Promise<void>;
  toggleStemMute: (deck: 0 | 1, stem: 0 | 1 | 2 | 3) => Promise<void>;
  cue: (deck: 0 | 1) => Promise<void>;
  /** Sync a deck's tempo to the opposite deck's BPM. */
  sync: (deck: 0 | 1) => Promise<void>;
  /** Seek a deck to an absolute frame position. */
  seek: (deck: 0 | 1, position: number) => Promise<void>;
  /** Nudge playback speed by a small delta (positive = faster). */
  nudgeTempo: (deck: 0 | 1, delta: number) => Promise<void>;
  /** Set playback speed ratio directly (1.0 = normal). */
  setTempoRatio: (deck: 0 | 1, ratio: number) => Promise<void>;
  autoTransition: (beats: number, fromDeck?: 0 | 1) => Promise<void>;
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
let stemUnlisten: UnlistenFn | null = null;

const sleep = (ms: number) =>
  new Promise<void>((resolve) => window.setTimeout(resolve, ms));

export const useEngineStore = create<EngineStore>((set, get) => ({
  decks: [blankState(0), blankState(1)],
  waveforms: [null, null],
  loadedPaths: [null, null],
  loadedTrackIds: [null, null],
  stemJobs: {},
  stemStatusForDeck: (deck) => {
    const s = get();
    const id = s.loadedTrackIds[deck];
    if (!id) return null;
    return s.stemJobs[id] ?? null;
  },
  faderA: 1.0,
  faderB: 1.0,
  stemGainsA: [1.0, 1.0, 1.0, 1.0],
  stemGainsB: [1.0, 1.0, 1.0, 1.0],
  stemMuteA: [false, false, false, false],
  stemMuteB: [false, false, false, false],
  crossfader: 0.5,
  masterGain: 1.0,

  startPolling: () => {
    if (pollHandle !== null) return;
    const tick = async () => {
      try {
        const [a, b] = await Promise.all([deckApi.state(0), deckApi.state(1)]);
        set({ decks: [a, b] });
      } catch (e) {
        // Engine unavailable – leave state as-is.
      }
    };
    pollHandle = window.setInterval(tick, POLL_INTERVAL_MS);

    // Listen for stem-status events from the Rust backend.
    if (stemUnlisten === null) {
      listen<StemStatusEvent>("stem-status", (event) => {
        const { track_id, status, progress, reason } = event.payload;

        set((s) => {
          const nextJobs = { ...s.stemJobs };
          if (status === "cached" || status === "failed") {
            // Remove terminal jobs after a delay.
            nextJobs[track_id] = {
              status,
              progress: 1.0,
              reason: reason ?? undefined,
            };
          } else {
            // "pending", "running", or "model_downloading"
            nextJobs[track_id] = { status, progress: progress ?? 0.0 };
          }
          return { stemJobs: nextJobs };
        });

        if (status === "cached") {
          // Stems just became available — reload any deck that has
          // this track loaded so stems are hot-swapped in.
          const paths = get().loadedPaths;
          for (const deck of [0, 1] as const) {
            const p = paths[deck];
            if (p) {
              // Re-load triggers the Rust side to detect cached stems
              // and use load_stems instead of load.
              deckApi
                .load(deck, p)
                .then(() => {
                  // Refresh waveform to show stem colours.
                  deckApi
                    .waveform(deck)
                    .then((data) => {
                      set((s) => {
                        const next = [...s.waveforms] as [
                          WaveformData | null,
                          WaveformData | null,
                        ];
                        next[deck] = data;
                        return { waveforms: next };
                      });
                    })
                    .catch(() => {});
                })
                .catch(() => {});
            }
          }

          // Clear the job from the UI after 5 seconds
          setTimeout(() => {
            set((s) => {
              const nextJobs = { ...s.stemJobs };
              delete nextJobs[track_id];
              return { stemJobs: nextJobs };
            });
          }, 5000);
        }
      }).then((fn) => {
        stemUnlisten = fn;
      });
    }
  },

  stopPolling: () => {
    if (pollHandle !== null) {
      window.clearInterval(pollHandle);
      pollHandle = null;
    }
    if (stemUnlisten !== null) {
      stemUnlisten();
      stemUnlisten = null;
    }
  },

  load: async (deck, path) => {
    const trackId = await deckApi.load(deck, path);
    // Remember the path so we can hot-swap stems later.
    set((s) => {
      const next = [...s.loadedPaths] as [string | null, string | null];
      next[deck] = path;
      const ids = [...s.loadedTrackIds] as [string | null, string | null];
      ids[deck] = trackId;
      return { loadedPaths: next, loadedTrackIds: ids };
    });
    // Compute waveform peaks in the background after load.
    deckApi
      .waveform(deck)
      .then((data) => {
        set((s) => {
          const next: [WaveformData | null, WaveformData | null] = [
            ...s.waveforms,
          ] as [WaveformData | null, WaveformData | null];
          next[deck] = data;
          return { waveforms: next };
        });
      })
      .catch(() => {
        /* waveform unavailable — no-op */
      });
  },
  play: (deck) => deckApi.play(deck),
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

  setStemGain: async (deck, stem, value) => {
    set((s) => {
      if (deck === 0) {
        const next = [...s.stemGainsA] as [number, number, number, number];
        next[stem] = value;
        return { stemGainsA: next };
      } else {
        const next = [...s.stemGainsB] as [number, number, number, number];
        next[stem] = value;
        return { stemGainsB: next };
      }
    });
  },
  setStemOutputGain: (deck, stem, value) =>
    mixerApi.setStemGain(deck, stem, value),

  toggleStemMute: async (deck, stem) => {
    const muted = deck === 0 ? get().stemMuteA[stem] : get().stemMuteB[stem];
    const nextMuted = !muted;
    set((s) => {
      if (deck === 0) {
        const m = [...s.stemMuteA] as [boolean, boolean, boolean, boolean];
        m[stem] = nextMuted;
        return { stemMuteA: m };
      }
      const m = [...s.stemMuteB] as [boolean, boolean, boolean, boolean];
      m[stem] = nextMuted;
      return { stemMuteB: m };
    });
  },

  cue: async (deck) => {
    await deckApi.pause(deck).catch(() => {});
    await deckApi.seek(deck, 0);
  },

  sync: (deck) => deckApi.sync(deck),

  seek: (deck, position) => deckApi.seek(deck, position),

  nudgeTempo: (deck, delta) => deckApi.nudgeTempo(deck, delta),

  setTempoRatio: (deck, ratio) => deckApi.setTempoRatio(deck, ratio),

  autoTransition: async (beats, fromDeck) => {
    const s = get();
    const source = fromDeck ?? (s.decks[0].playing ? 0 : 1);
    const target = source === 0 ? 1 : 0;
    const bpm = s.decks[source].bpm > 0 ? s.decks[source].bpm : 120;
    const beatMs = 60000 / bpm;
    const totalMs = beatMs * beats;
    const interval = 33;
    const steps = Math.max(4, Math.floor(totalMs / interval));
    await s.play(target as 0 | 1).catch(() => {});
    await s.sync(target as 0 | 1).catch(() => {});
    for (let i = 0; i <= steps; i++) {
      const t = i / steps;
      const xf = source === 0 ? t : 1 - t;
      await get()
        .setCrossfader(xf)
        .catch(() => {});
      await sleep(interval);
    }
    await get()
      .pause(source as 0 | 1)
      .catch(() => {});
  },
}));

/**
 * api.ts — Typed wrappers around Tauri commands.
 *
 * The React side never calls `invoke()` directly.  Instead it imports
 * functions from this file, so renaming a command requires touching only
 * one place in TS.  Each function mirrors a `#[tauri::command]` in Rust.
 */

import { invoke } from "@tauri-apps/api/core";

// ---------------------------------------------------------------------------
// App-level
// ---------------------------------------------------------------------------

export interface AppStatus {
  version: string;
  audio_running: boolean;
  library_count: number;
}

export const app = {
  version:    () => invoke<string>("app_version"),
  status:     () => invoke<AppStatus>("app_status"),
  keymapLoad: () => invoke<Record<string, string>>("keymap_load"),
};

// ---------------------------------------------------------------------------
// Deck
// ---------------------------------------------------------------------------

export interface DeckState {
  deck: number;
  loaded: boolean;
  playing: boolean;
  position: number;
  bpm: number;
  /** Current playback speed ratio (1.0 = normal). */
  tempo_ratio: number;
}

export const deckApi = {
  load:   (deck: number, path: string) =>
    invoke<void>("deck_load", { deck, path }),
  play:   (deck: number) => invoke<void>("deck_play",  { deck }),
  pause:  (deck: number) => invoke<void>("deck_pause", { deck }),
  seek:   (deck: number, position: number) =>
    invoke<void>("deck_seek", { deck, position }),
  state:  (deck: number) => invoke<DeckState>("deck_state", { deck }),
  setTempoRatio: (deck: number, ratio: number) =>
    invoke<void>("deck_set_tempo_ratio", { deck, ratio }),
  sync:   (deck: number) => invoke<void>("deck_sync", { deck }),
  nudgeTempo: (deck: number, delta: number) =>
    invoke<void>("deck_nudge_tempo", { deck, delta }),
};

// ---------------------------------------------------------------------------
// Mixer
// ---------------------------------------------------------------------------

export const mixerApi = {
  setFader:      (deck: number, value: number) =>
    invoke<void>("mixer_set_fader", { deck, value }),
  setCrossfader: (value: number) =>
    invoke<void>("mixer_set_crossfader", { value }),
  setMasterGain: (value: number) =>
    invoke<void>("mixer_set_master_gain", { value }),
  setStemGain:   (deck: number, stem: number, value: number) =>
    invoke<void>("mixer_set_stem_gain", { deck, stem, value }),
};

// ---------------------------------------------------------------------------
// Library
// ---------------------------------------------------------------------------

export interface Track {
  id: string;
  path: string;
  rel_path: string | null;
  title: string | null;
  artist: string | null;
  album: string | null;
  duration_ms: number | null;
  bpm: number | null;
  key: string | null;
  imported_at: number;
  analyzed_at: number | null;
  analysis_state: "raw" | "beatgrid" | "full" | "failed";
  stems_state: "pending" | "running" | "cached" | "failed";
}

export interface ScanReport {
  added: number;
  duplicate: number;
  skipped: number;
  errors: string[];
}

export interface Query {
  text?: string | null;
  bpm_min?: number | null;
  bpm_max?: number | null;
  limit?: number | null;
}

export const libraryApi = {
  importFile: (path: string) =>
    invoke<string>("library_import_file", { path }),
  scanFolder: (path: string) =>
    invoke<ScanReport>("library_scan_folder", { path }),
  recent:     (limit = 50) =>
    invoke<Track[]>("library_recent", { limit }),
  search:     (query: Query) =>
    invoke<Track[]>("library_search", { query }),
};

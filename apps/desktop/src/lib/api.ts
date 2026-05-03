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

export interface UiSettings {
  sample_rate:     number;
  buffer_size:     number;
  pitch_range_pct: number;
  music_root:      string;
  online_lookup:   boolean;
  target_fps:      number;
  update_check:    boolean;
}

export const app = {
  version:       () => invoke<string>("app_version"),
  status:        () => invoke<AppStatus>("app_status"),
  keymapLoad:    () => invoke<Record<string, string>>("keymap_load"),
  settingsLoad:  () => invoke<UiSettings>("settings_load"),
  settingsSave:  (settings: UiSettings) => invoke<void>("settings_save", { settings }),
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

export interface WaveformData {
  num_bins:     number;
  peaks_min:    number[];
  peaks_max:    number[];
  stem_peaks:   [number[], number[], number[], number[]] | null;
  total_frames: number;
}

export const deckApi = {
  load:   (deck: number, path: string) =>
    invoke<string>("deck_load", { deck, path }),
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
  /** Fetch waveform peak data for canvas rendering. bins ≈ canvas width. */
  waveform: (deck: number, bins = 800) =>
    invoke<WaveformData>("deck_waveform", { deck, bins }),
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

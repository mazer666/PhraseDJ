/**
 * useKeymap.ts — Global keyboard shortcut handler driven by keymap.toml.
 *
 * On mount, fetches the key→intent map from the Rust backend (which merges
 * the bundled default with any user overrides), then registers a `keydown`
 * listener on `window`.  The listener looks up the pressed key in the map
 * and dispatches the matching action to the engine store.
 *
 * Intent string format: "<namespace>.<scope>.<action>"
 * Examples: "deck.A.toggle_play", "deck.B.tempo_nudge_plus"
 *
 * Text inputs and textareas are excluded so that typing in the library
 * search box doesn't accidentally trigger deck controls.
 */

import { useEffect } from "react";
import { app } from "../lib/api";
import { useEngineStore } from "../store/engineStore";

// How much each nudge keypress shifts the tempo ratio.
const NUDGE_DELTA = 0.01;

export function useKeymap(): void {
  const play       = useEngineStore((s) => s.play);
  const pause      = useEngineStore((s) => s.pause);
  const sync       = useEngineStore((s) => s.sync);
  const nudgeTempo = useEngineStore((s) => s.nudgeTempo);
  const decks      = useEngineStore((s) => s.decks);

  useEffect(() => {
    let keymap: Record<string, string> = {};

    // Load keymap from backend asynchronously; the listener is registered
    // immediately with an empty map and starts working once the load resolves.
    app.keymapLoad()
      .then((map) => { keymap = map; })
      .catch(() => { /* leave keymap empty — no shortcuts active */ });

    const onKeyDown = (e: KeyboardEvent) => {
      // Skip if a text input is focused so typing isn't intercepted.
      const tag = (e.target as HTMLElement | null)?.tagName;
      if (tag === "INPUT" || tag === "TEXTAREA") return;

      // Build the lookup key.  Modifiers use the same syntax as keymap.toml.
      const parts: string[] = [];
      if (e.ctrlKey || e.metaKey) parts.push("Ctrl");
      if (e.altKey)  parts.push("Alt");
      if (e.shiftKey) parts.push("Shift");
      parts.push(normaliseKey(e.key));
      const lookupKey = parts.join("+");

      const intent = keymap[lookupKey];
      if (!intent) return;

      // Prevent the browser from acting on recognised shortcuts (e.g. Space
      // scrolling the page).
      e.preventDefault();

      dispatch(intent, { play, pause, sync, nudgeTempo, decks });
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [play, pause, sync, nudgeTempo, decks]);
}

// ---------------------------------------------------------------------------
// Intent dispatcher
// ---------------------------------------------------------------------------

interface Actions {
  play:       (deck: 0 | 1) => Promise<void>;
  pause:      (deck: 0 | 1) => Promise<void>;
  sync:       (deck: 0 | 1) => Promise<void>;
  nudgeTempo: (deck: 0 | 1, delta: number) => Promise<void>;
  decks:      [{ playing: boolean; loaded: boolean }, { playing: boolean; loaded: boolean }];
}

function dispatch(intent: string, actions: Actions): void {
  const parts  = intent.split(".");
  const ns     = parts[0];  // "deck" | "master" | "view" | "macro"
  const scope  = parts[1];  // "A" | "B" | (global intents omit scope)
  const action = parts[2];

  if (ns === "deck") {
    const deckIndex = scope === "A" ? 0 : 1 as 0 | 1;
    dispatchDeckIntent(action, deckIndex, actions);
    return;
  }

  if (ns === "master" && action === "pause_toggle") {
    // Pause both decks if either is playing.
    const anyPlaying = actions.decks[0].playing || actions.decks[1].playing;
    if (anyPlaying) {
      if (actions.decks[0].playing) actions.pause(0);
      if (actions.decks[1].playing) actions.pause(1);
    } else {
      if (actions.decks[0].loaded) actions.play(0);
      if (actions.decks[1].loaded) actions.play(1);
    }
  }

  // "view.*" and "macro.*" intents are handled elsewhere (modals, etc.)
  // and are ignored here without error.
}

function dispatchDeckIntent(action: string, deck: 0 | 1, a: Actions): void {
  switch (action) {
    case "toggle_play":
      if (a.decks[deck].playing) a.pause(deck);
      else if (a.decks[deck].loaded) a.play(deck);
      break;
    case "sync":
      a.sync(deck);
      break;
    case "tempo_nudge_minus":
      a.nudgeTempo(deck, -NUDGE_DELTA);
      break;
    case "tempo_nudge_plus":
      a.nudgeTempo(deck, +NUDGE_DELTA);
      break;
    // "cue", "loop_in", "loop_out", "stem_mute_*" are Phase 1 stubs:
    // the hardware actions will be wired as each feature lands.
    default:
      break;
  }
}

// ---------------------------------------------------------------------------
// Key name normalisation
// ---------------------------------------------------------------------------

/**
 * Convert `KeyboardEvent.key` to the format used in keymap.toml.
 *
 * Single printable characters are uppercased ("q" → "Q").
 * Special keys are passed through as-is ("Space", "Escape", etc.)
 * but "Escape" is aliased to "Esc" to match the config file.
 */
function normaliseKey(key: string): string {
  if (key === "Escape") return "Esc";
  if (key === " ")      return "Space";
  if (key.length === 1) return key.toUpperCase();
  return key;
}

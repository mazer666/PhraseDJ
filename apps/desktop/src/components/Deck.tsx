/**
 * Deck.tsx — One playback deck UI.
 *
 * Phase 1: loaded track, BPM, transport buttons, tempo sync/nudge,
 * and a channel fader.  Waveform rendering arrives later in Phase 1.
 */

import { open } from "@tauri-apps/plugin-dialog";

import { useEngineStore } from "../store/engineStore";

export interface DeckProps {
  side: "A" | "B";
}

export function Deck({ side }: DeckProps): React.JSX.Element {
  const deckIndex = side === "A" ? 0 : 1 as 0 | 1;
  const state      = useEngineStore((s) => s.decks[deckIndex]);
  const fader      = useEngineStore((s) => (side === "A" ? s.faderA : s.faderB));
  const load       = useEngineStore((s) => s.load);
  const play       = useEngineStore((s) => s.play);
  const pause      = useEngineStore((s) => s.pause);
  const setFader   = useEngineStore((s) => s.setFader);
  const sync       = useEngineStore((s) => s.sync);
  const nudgeTempo = useEngineStore((s) => s.nudgeTempo);

  const onChooseFile = async () => {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "Audio",
          extensions: ["wav", "flac", "aif", "aiff", "mp3", "m4a", "aac", "ogg", "opus"],
        },
      ],
    });
    if (typeof selected === "string") {
      await load(deckIndex as 0 | 1, selected);
    }
  };

  // Format frame position as mm:ss.  Phase 1 default sample rate = 44.1 kHz.
  const positionLabel = formatPosition(state.position, 44_100);

  // Show tempo ratio as a signed percentage deviation from 1.0.
  const ratioPct = (state.tempo_ratio - 1.0) * 100;
  const ratioLabel = ratioPct >= 0
    ? `+${ratioPct.toFixed(1)} %`
    : `${ratioPct.toFixed(1)} %`;
  const ratioAtUnity = Math.abs(ratioPct) < 0.05;

  return (
    <div className={`deck deck-${side.toLowerCase()}`}>
      <div className="deck-header">
        <span className="deck-label">DECK {side}</span>
        <span className="deck-bpm">
          {state.bpm > 0 ? `${state.bpm.toFixed(1)} BPM` : "— BPM"}
        </span>
      </div>

      <div className="deck-track">
        <button className="btn-secondary" onClick={onChooseFile}>
          {state.loaded ? "Change track…" : "Load track…"}
        </button>
        <div className="deck-position">{positionLabel}</div>
      </div>

      <div className="deck-transport">
        {state.playing ? (
          <button
            className="btn-primary"
            onClick={() => pause(deckIndex as 0 | 1)}
            disabled={!state.loaded}
          >
            Pause
          </button>
        ) : (
          <button
            className="btn-primary"
            onClick={() => play(deckIndex as 0 | 1)}
            disabled={!state.loaded}
          >
            Play
          </button>
        )}
      </div>

      {/* Tempo row: nudge − | ratio label | sync | nudge + */}
      <div className="deck-tempo-row">
        <button
          className="btn-tempo-nudge"
          title="Tempo nudge − (slow down 1 %)"
          disabled={!state.loaded}
          onClick={() => nudgeTempo(deckIndex as 0 | 1, -0.01)}
        >
          −
        </button>

        <span
          className={`deck-tempo-ratio${ratioAtUnity ? " at-unity" : ""}`}
          title="Playback speed relative to original BPM"
        >
          {ratioAtUnity ? "±0.0 %" : ratioLabel}
        </span>

        <button
          className="btn-sync"
          title="Sync tempo to other deck"
          disabled={!state.loaded || state.bpm <= 0}
          onClick={() => sync(deckIndex as 0 | 1)}
        >
          SYNC
        </button>

        <button
          className="btn-tempo-nudge"
          title="Tempo nudge + (speed up 1 %)"
          disabled={!state.loaded}
          onClick={() => nudgeTempo(deckIndex as 0 | 1, +0.01)}
        >
          +
        </button>
      </div>

      <div className="deck-fader-block">
        <label>Fader</label>
        <input
          type="range"
          min={0}
          max={1}
          step={0.01}
          value={fader}
          onChange={(e) =>
            setFader(deckIndex as 0 | 1, parseFloat(e.target.value))
          }
        />
        <span className="value-label">{Math.round(fader * 100)}</span>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function formatPosition(frames: number, sampleRate: number): string {
  if (!frames || sampleRate <= 0) return "0:00";
  const totalSec = frames / sampleRate;
  const min = Math.floor(totalSec / 60);
  const sec = Math.floor(totalSec % 60);
  return `${min}:${sec.toString().padStart(2, "0")}`;
}

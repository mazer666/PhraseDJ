/**
 * Deck.tsx — One playback deck UI.
 *
 * Phase 1: shows the loaded track path, BPM, transport buttons, and a
 * channel fader.  Waveform rendering arrives later in Phase 1.
 */

import { open } from "@tauri-apps/plugin-dialog";

import { useEngineStore } from "../store/engineStore";

export interface DeckProps {
  side: "A" | "B";
}

export function Deck({ side }: DeckProps): React.JSX.Element {
  const deckIndex = side === "A" ? 0 : 1;
  const state = useEngineStore((s) => s.decks[deckIndex]);
  const fader = useEngineStore((s) => (side === "A" ? s.faderA : s.faderB));
  const load  = useEngineStore((s) => s.load);
  const play  = useEngineStore((s) => s.play);
  const pause = useEngineStore((s) => s.pause);
  const setFader = useEngineStore((s) => s.setFader);

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

/**
 * Deck.tsx — One playback deck UI.
 *
 * Phase 1: loaded track, BPM, overview waveform, transport buttons,
 * tempo sync/nudge, and a channel fader.
 */

import { open } from "@tauri-apps/plugin-dialog";

import { useEngineStore } from "../store/engineStore";
import { WaveformCanvas } from "./WaveformCanvas";

export interface DeckProps {
  side: "A" | "B";
}

export function Deck({ side }: DeckProps): React.JSX.Element {
  const deckIndex = side === "A" ? 0 : 1 as 0 | 1;
  const state      = useEngineStore((s) => s.decks[deckIndex]);
  const waveform   = useEngineStore((s) => s.waveforms[deckIndex]);
  const fader      = useEngineStore((s) => (side === "A" ? s.faderA : s.faderB));
  const stemGains  = useEngineStore((s) => (side === "A" ? s.stemGainsA : s.stemGainsB));
  const load       = useEngineStore((s) => s.load);
  const play       = useEngineStore((s) => s.play);
  const pause      = useEngineStore((s) => s.pause);
  const setFader   = useEngineStore((s) => s.setFader);
  const setStemGain = useEngineStore((s) => s.setStemGain);
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

      {/* Overview waveform — always visible, shows placeholder when not loaded */}
      <WaveformCanvas
        waveform={waveform}
        position={state.position}
        accentColor={side === "A" ? "#e05a2b" : "#2bb3e0"}
      />

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

      <div className="deck-stems-block">
        {[
          { label: "Vocals", index: 0 as const },
          { label: "Drums", index: 1 as const },
          { label: "Bass", index: 2 as const },
          { label: "Other", index: 3 as const },
        ].map(({ label, index }) => (
          <div className="deck-fader-block stem-fader" key={label}>
            <label>{label}</label>
            <input
              type="range"
              min={0}
              max={1.5}
              step={0.01}
              value={stemGains[index]}
              onChange={(e) => setStemGain(deckIndex, index, parseFloat(e.target.value))}
            />
            <span className="value-label">{Math.round(stemGains[index] * 100)}</span>
          </div>
        ))}
      </div>

      <div className="deck-fader-block main-fader">
        <label>Level</label>
        <input
          type="range"
          min={0}
          max={1}
          step={0.01}
          value={fader}
          onChange={(e) =>
            setFader(deckIndex, parseFloat(e.target.value))
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

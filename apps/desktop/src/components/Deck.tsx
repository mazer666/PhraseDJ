/**
 * Deck.tsx — One playback deck UI.
 *
 * Phase 1: loaded track, BPM, overview waveform, transport buttons,
 * tempo sync/nudge, and a channel fader.
 */

import { open } from "@tauri-apps/plugin-dialog";

import { useEngineStore } from "../store/engineStore";
import { WaveformCanvas } from "./WaveformCanvas";
import { Turntable } from "./Turntable";
import { useEffect, useRef, useState } from "react";

export interface DeckProps {
  side: "A" | "B";
}

export function Deck({ side }: DeckProps): React.JSX.Element {
  const deckIndex = side === "A" ? 0 : 1 as 0 | 1;
  const state      = useEngineStore((s) => s.decks[deckIndex]);
  const waveform   = useEngineStore((s) => s.waveforms[deckIndex]);
  const loadedPath = useEngineStore((s) => s.loadedPaths[deckIndex]);
  const fader      = useEngineStore((s) => (side === "A" ? s.faderA : s.faderB));
  const stemGains  = useEngineStore((s) => (side === "A" ? s.stemGainsA : s.stemGainsB));
  const load       = useEngineStore((s) => s.load);
  const play       = useEngineStore((s) => s.play);
  const pause      = useEngineStore((s) => s.pause);
  const setFader   = useEngineStore((s) => s.setFader);
  const setStemGain = useEngineStore((s) => s.setStemGain);
  const setStemOutputGain = useEngineStore((s) => s.setStemOutputGain);
  const setCrossfader = useEngineStore((s) => s.setCrossfader);
  const sync       = useEngineStore((s) => s.sync);
  const nudgeTempo = useEngineStore((s) => s.nudgeTempo);
  const seek = useEngineStore((s) => s.seek);
  const cue = useEngineStore((s) => s.cue);
  const stemStatus = useEngineStore((s) => s.stemStatusForDeck(deckIndex));
  const autoTransition = useEngineStore((s) => s.autoTransition);
  const [autoBeats, setAutoBeats] = useState(16);
  const [zoom, setZoom] = useState(1);
  const [showBeatGrid, setShowBeatGrid] = useState(true);
  const stemMute = useEngineStore((s) => (side === "A" ? s.stemMuteA : s.stemMuteB));
  const toggleStemMute = useEngineStore((s) => s.toggleStemMute);
  const [stemSolo, setStemSolo] = useState<[boolean, boolean, boolean, boolean]>([false, false, false, false]);
  const [eqLow, setEqLow] = useState(1);
  const [eqMid, setEqMid] = useState(1);
  const [eqHigh, setEqHigh] = useState(1);
  const flushTimer = useRef<number | null>(null);
  const lastSent = useRef<[number, number, number, number]>([-1, -1, -1, -1]);

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
  const anySolo = stemSolo.some(Boolean);
  const eqMultiplierForStem = (stem: 0 | 1 | 2 | 3): number => {
    if (stem === 2) return eqLow; // bass -> low
    if (stem === 0) return eqHigh; // vocals -> high
    return eqMid; // drums + other -> mid
  };

  const applyStemGain = (stem: 0 | 1 | 2 | 3, raw: number) =>
    void setStemGain(deckIndex, stem, Math.max(0, Math.min(1.5, raw)));

  useEffect(() => {
    if (flushTimer.current !== null) window.clearTimeout(flushTimer.current);
    flushTimer.current = window.setTimeout(() => {
      for (const stem of [0, 1, 2, 3] as const) {
        const soloGate = anySolo ? (stemSolo[stem] ? 1 : 0) : 1;
        const muteGate = stemMute[stem] ? 0 : 1;
        const processed = Math.max(0, Math.min(1.5, stemGains[stem] * soloGate * muteGate * eqMultiplierForStem(stem)));
        if (Math.abs(lastSent.current[stem] - processed) < 0.001) continue;
        lastSent.current[stem] = processed;
        void setStemOutputGain(deckIndex, stem, processed);
      }
    }, 33);
    return () => {
      if (flushTimer.current !== null) window.clearTimeout(flushTimer.current);
    };
  }, [anySolo, stemSolo, stemMute, stemGains, eqLow, eqMid, eqHigh, deckIndex, setStemOutputGain]);

  const handOverToOtherDeck = () => {
    const target = side === "A" ? 1 : 0;
    void play(target as 0 | 1);
    void sync(target as 0 | 1);
    void setCrossfader(side === "A" ? 1 : 0);
  };


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

      <Turntable
        playing={state.playing}
        position={state.position}
        totalFrames={waveform?.total_frames ?? 0}
        bpm={state.bpm}
        trackLabel={loadedPath ? loadedPath.split(/[\\/]/).pop() : undefined}
        onSeek={(f) => { void seek(deckIndex, f); }}
      />

      {/* Overview waveform — always visible, shows placeholder when not loaded */}
      <div className="waveform-toolbar">
        <label>Zoom</label>
        <input type="range" min={1} max={8} step={1} value={zoom} onChange={(e) => setZoom(parseInt(e.target.value, 10))} />
        <span className="value-label">{zoom}x</span>
        <button className="btn-secondary" onClick={() => setShowBeatGrid((v) => !v)}>{showBeatGrid ? "Beats: On" : "Beats: Off"}</button>
      </div>
      <WaveformCanvas
        waveform={waveform}
        position={state.position}
        accentColor={side === "A" ? "#e05a2b" : "#2bb3e0"}
        deck={deckIndex}
        zoom={zoom}
        bpm={state.bpm}
        tempoRatio={state.tempo_ratio}
        showBeatGrid={showBeatGrid}
      />

      <div className="deck-transport">
        <button className="btn-secondary" onClick={() => void cue(deckIndex)} disabled={!state.loaded}>Cue</button>
        <button className="btn-secondary" onClick={handOverToOtherDeck} disabled={!state.loaded}>Hand Over</button>
        <button className="btn-secondary" onClick={() => autoTransition(autoBeats, deckIndex)} disabled={!state.loaded}>AutoSwitch</button>
        <input className="beats-input" type="number" min={2} max={64} step={2} value={autoBeats} onChange={(e) => setAutoBeats(parseInt(e.target.value || "16", 10))} title="AutoSwitch beats" />
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
        {stemStatus && stemStatus.status !== "cached" && (
          <div className={`stem-status stem-${stemStatus.status}`}>
            {stemStatus.status === "running" || stemStatus.status === "model_downloading"
              ? `Stems analysing… ${Math.round((stemStatus.progress ?? 0) * 100)}%`
              : stemStatus.status === "failed"
              ? `Stems failed: ${stemStatus.reason ?? "unknown error"}`
              : "Stems pending…"}
          </div>
        )}
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
              onChange={(e) => applyStemGain(index, parseFloat(e.target.value))}
            />
            <button className={`btn-chip ${stemMute[index] ? "active" : ""}`} onClick={() => void toggleStemMute(deckIndex, index)}>M</button>
            <button className={`btn-chip ${stemSolo[index] ? "active" : ""}`} onClick={() => setStemSolo((m) => { const n=[...m] as [boolean,boolean,boolean,boolean]; n[index]=!n[index]; return n; })}>S</button>
            <span className="value-label">{Math.round(stemGains[index] * 100)}</span>
          </div>
        ))}
      </div>


      <div className="deck-eq-block">
        <div className="deck-fader-block">
          <label>EQ Low</label>
          <input type="range" min={0} max={1.5} step={0.01} value={eqLow} onChange={(e) => setEqLow(parseFloat(e.target.value))} />
          <span className="value-label">{Math.round(eqLow * 100)}</span>
        </div>
        <div className="deck-fader-block">
          <label>EQ Mid</label>
          <input type="range" min={0} max={1.5} step={0.01} value={eqMid} onChange={(e) => setEqMid(parseFloat(e.target.value))} />
          <span className="value-label">{Math.round(eqMid * 100)}</span>
        </div>
        <div className="deck-fader-block">
          <label>EQ High</label>
          <input type="range" min={0} max={1.5} step={0.01} value={eqHigh} onChange={(e) => setEqHigh(parseFloat(e.target.value))} />
          <span className="value-label">{Math.round(eqHigh * 100)}</span>
        </div>
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

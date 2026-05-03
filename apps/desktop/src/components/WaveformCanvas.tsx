import { useEffect, useRef } from "react";
import { useEngineStore } from "../store/engineStore";
import type { WaveformData } from "../lib/api";

export interface WaveformCanvasProps {
  waveform: WaveformData | null;
  position: number;
  accentColor: string;
  deck: 0 | 1;
  height?: number;
  zoom?: number;
  bpm?: number;
  tempoRatio?: number;
  showBeatGrid?: boolean;
  sampleRate?: number;
}

export function WaveformCanvas({
  waveform,
  position,
  accentColor,
  deck,
  height = 48,
  zoom = 1,
  bpm = 0,
  tempoRatio = 1,
  showBeatGrid = true,
  sampleRate = 44_100,
}: WaveformCanvasProps): React.JSX.Element {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const seek = useEngineStore((s) => s.seek);

  const frameWindow = (total: number) => {
    if (zoom <= 1) return { start: 0, end: total };
    const span = Math.max(Math.floor(total / zoom), 1);
    const center = Math.min(Math.max(position, 0), total);
    const start = Math.max(
      0,
      Math.min(center - Math.floor(span / 2), total - span),
    );
    return { start, end: start + span };
  };

  const handleCanvasClick = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas || !waveform || waveform.total_frames === 0) return;
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const progress = x / rect.width;
    const win = frameWindow(waveform.total_frames);
    const targetFrame = Math.floor(
      win.start + progress * (win.end - win.start),
    );
    seek(deck, targetFrame);
  };

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const dpr = window.devicePixelRatio || 1;
    const w = canvas.offsetWidth;
    const h = canvas.offsetHeight;
    if (canvas.width !== w * dpr || canvas.height !== h * dpr) {
      canvas.width = w * dpr;
      canvas.height = h * dpr;
      ctx.scale(dpr, dpr);
    }
    ctx.clearRect(0, 0, w, h);
    ctx.fillStyle = "rgba(26,26,30,1)";
    ctx.fillRect(0, 0, w, h);

    if (!waveform || waveform.num_bins === 0) {
      ctx.fillStyle = "rgba(255,255,255,0.08)";
      ctx.fillRect(0, Math.floor(h / 2) - 1, w, 2);
      return;
    }

    const { num_bins, peaks_max, stem_peaks, total_frames } = waveform;
    const midY = h / 2;
    const binW = w / num_bins;
    const stemColors = ["#3b82f6", "#ef4444", "#eab308", "#22c55e"];
    const win = frameWindow(total_frames);

    for (let i = 0; i < num_bins; i++) {
      const binStart = (i / num_bins) * total_frames;
      const binEnd = ((i + 1) / num_bins) * total_frames;
      if (binEnd < win.start || binStart > win.end) continue;

      const x = ((binStart - win.start) / (win.end - win.start)) * w;
      const amplitude = Math.min(peaks_max[i], 1.0);
      const barH = amplitude * midY;
      const width = Math.max(1, binW * Math.max(1, zoom));

      ctx.fillStyle = "rgba(255,255,255,0.07)";
      ctx.fillRect(x, midY - midY * 0.1, width, midY * 0.2);

      if (barH > 0.5) {
        if (stem_peaks) {
          const stems = [
            stem_peaks[0][i],
            stem_peaks[1][i],
            stem_peaks[2][i],
            stem_peaks[3][i],
          ];
          const sum = stems[0] + stems[1] + stems[2] + stems[3];
          if (sum > 0) {
            let top = midY - barH;
            let bot = midY;
            for (let s = 0; s < 4; s++) {
              const segH = barH * (stems[s] / sum);
              if (segH <= 0.5) continue;
              const alpha = 0.35 + amplitude * 0.65;
              ctx.fillStyle = hexToRgba(stemColors[s], alpha);
              ctx.fillRect(x, top, width, segH);
              ctx.fillRect(x, bot, width, segH);
              top += segH;
              bot += segH;
            }
          }
        } else {
          ctx.fillStyle = hexToRgba(accentColor, 0.35 + amplitude * 0.65);
          ctx.fillRect(x, midY - barH, width, barH * 2);
        }
      }
    }

    if (showBeatGrid && bpm > 0 && total_frames > 0) {
      const effectiveBpm = Math.max(1, bpm * tempoRatio);
      const framesPerBeat = Math.floor((sampleRate * 60) / effectiveBpm);
      if (framesPerBeat > 0) {
        ctx.strokeStyle = "rgba(255,255,255,0.15)";
        ctx.lineWidth = 1;
        const first = Math.floor(win.start / framesPerBeat) * framesPerBeat;
        for (let f = first; f < win.end; f += framesPerBeat) {
          const x = ((f - win.start) / (win.end - win.start)) * w;
          ctx.beginPath();
          ctx.moveTo(x, 0);
          ctx.lineTo(x, h);
          ctx.stroke();
        }
      }
    }

    const clamped = Math.min(Math.max(position, win.start), win.end);
    const px = ((clamped - win.start) / (win.end - win.start)) * w;
    ctx.fillStyle = "rgba(255,255,255,0.9)";
    ctx.fillRect(px - 1, 0, 2, h);
  }, [
    waveform,
    position,
    accentColor,
    zoom,
    bpm,
    tempoRatio,
    showBeatGrid,
    sampleRate,
  ]);

  return (
    <canvas
      ref={canvasRef}
      className="waveform-canvas"
      style={{
        width: "100%",
        height: `${height}px`,
        display: "block",
        cursor: "pointer",
      }}
      onClick={handleCanvasClick}
    />
  );
}

function hexToRgba(hex: string, alpha: number): string {
  const h = hex.replace("#", "");
  const r = parseInt(h.slice(0, 2), 16);
  const g = parseInt(h.slice(2, 4), 16);
  const b = parseInt(h.slice(4, 6), 16);
  return `rgba(${r},${g},${b},${alpha.toFixed(2)})`;
}

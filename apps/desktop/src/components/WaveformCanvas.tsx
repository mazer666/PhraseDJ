/**
 * WaveformCanvas.tsx — Overview waveform for one deck.
 *
 * Draws the precomputed peak data as a centred symmetrical bar chart and
 * renders a playhead line that tracks the current frame position.
 * Redraws whenever `position` or `waveform` changes; no animation loop
 * needed because the parent poll already drives re-renders at 20 fps.
 */

import { useEffect, useRef } from "react";
import type { WaveformData } from "../lib/api";

export interface WaveformCanvasProps {
  /** Waveform peak data returned by deck_waveform. */
  waveform:     WaveformData | null;
  /** Current playback position in frames (from DeckState). */
  position:     number;
  /** Accent colour for the filled bars (CSS colour string). */
  accentColor:  string;
  /** Canvas height in CSS pixels. */
  height?:      number;
}

export function WaveformCanvas({
  waveform,
  position,
  accentColor,
  height = 48,
}: WaveformCanvasProps): React.JSX.Element {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const dpr    = window.devicePixelRatio || 1;
    const w      = canvas.offsetWidth;
    const h      = canvas.offsetHeight;

    // Resize backing store to match physical pixels.
    if (canvas.width !== w * dpr || canvas.height !== h * dpr) {
      canvas.width  = w * dpr;
      canvas.height = h * dpr;
      ctx.scale(dpr, dpr);
    }

    // Clear to background colour.
    ctx.clearRect(0, 0, w, h);
    ctx.fillStyle = "rgba(26,26,30,1)";
    ctx.fillRect(0, 0, w, h);

    if (!waveform || waveform.num_bins === 0) {
      // No data yet — draw a placeholder centre line.
      ctx.fillStyle = "rgba(255,255,255,0.08)";
      ctx.fillRect(0, Math.floor(h / 2) - 1, w, 2);
      return;
    }

    const { num_bins, peaks_max, stem_peaks, total_frames } = waveform;
    const midY    = h / 2;
    const binW    = w / num_bins;

    // Stem colours: Vocals (Blue), Drums (Red), Bass (Yellow), Other (Green)
    const stemColors = ["#3b82f6", "#ef4444", "#eab308", "#22c55e"];

    // Draw waveform bars.
    for (let i = 0; i < num_bins; i++) {
      const x         = i * binW;
      const amplitude = Math.min(peaks_max[i], 1.0);
      const barH      = amplitude * midY;

      // Dim base colour — always visible even at silence.
      ctx.fillStyle = "rgba(255,255,255,0.07)";
      ctx.fillRect(x, midY - midY * 0.1, binW - 0.5, midY * 0.2);

      // Accent fill proportional to amplitude.
      if (barH > 0.5) {
        if (stem_peaks) {
          // Render stacked stems
          const v = stem_peaks[0][i];
          const d = stem_peaks[1][i];
          const b = stem_peaks[2][i];
          const o = stem_peaks[3][i];
          const sum = v + d + b + o;
          
          if (sum > 0) {
            let currentYTop = midY - barH;
            let currentYBot = midY;
            
            const stems = [v, d, b, o];
            for (let s = 0; s < 4; s++) {
              const fraction = stems[s] / sum;
              const segH = barH * fraction;
              if (segH > 0.5) {
                const alpha = 0.35 + amplitude * 0.65;
                ctx.fillStyle = hexToRgba(stemColors[s], alpha);
                // Draw top half
                ctx.fillRect(x, currentYTop, binW - 0.5, segH);
                // Draw bottom half
                ctx.fillRect(x, currentYBot, binW - 0.5, segH);
                
                currentYTop += segH;
                currentYBot += segH;
              }
            }
          }
        } else {
          // Standard single-colour waveform
          const alpha = 0.35 + amplitude * 0.65;
          ctx.fillStyle = hexToRgba(accentColor, alpha);
          ctx.fillRect(x, midY - barH, binW - 0.5, barH * 2);
        }
      }
    }

    // Draw playhead.
    if (total_frames > 0) {
      const progress = Math.min(position / total_frames, 1.0);
      const px = progress * w;
      ctx.fillStyle = "rgba(255,255,255,0.9)";
      ctx.fillRect(px - 1, 0, 2, h);

      // Tiny diamond marker at the centre.
      ctx.fillStyle = "#fff";
      ctx.beginPath();
      ctx.moveTo(px,     midY - 5);
      ctx.lineTo(px + 4, midY);
      ctx.lineTo(px,     midY + 5);
      ctx.lineTo(px - 4, midY);
      ctx.closePath();
      ctx.fill();
    }
  }, [waveform, position, accentColor]);

  return (
    <canvas
      ref={canvasRef}
      className="waveform-canvas"
      style={{ width: "100%", height: `${height}px`, display: "block" }}
    />
  );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Convert a #rrggbb hex string to an rgba() value with the given alpha. */
function hexToRgba(hex: string, alpha: number): string {
  const h   = hex.replace("#", "");
  const r   = parseInt(h.slice(0, 2), 16);
  const g   = parseInt(h.slice(2, 4), 16);
  const b   = parseInt(h.slice(4, 6), 16);
  return `rgba(${r},${g},${b},${alpha.toFixed(2)})`;
}

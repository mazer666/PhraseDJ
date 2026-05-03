import { useMemo, useRef, useState } from "react";

export interface TurntableProps {
  playing: boolean;
  position: number;
  totalFrames: number;
  bpm: number;
  trackLabel?: string;
  onSeek: (frame: number) => void;
}

export function Turntable({ playing, position, totalFrames, bpm, trackLabel, onSeek }: TurntableProps): React.JSX.Element {
  const [dragging, setDragging] = useState(false);
  const lastAngle = useRef<number | null>(null);

  const rotationDeg = useMemo(() => {
    if (totalFrames <= 0) return 0;
    return (position / totalFrames) * 360 * 100;
  }, [position, totalFrames]);

  const onPointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    (e.target as HTMLElement).setPointerCapture(e.pointerId);
    setDragging(true);
    lastAngle.current = angleFromEvent(e);
  };

  const onPointerMove = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!dragging || totalFrames <= 0) return;
    const nextAngle = angleFromEvent(e);
    const prev = lastAngle.current;
    if (prev == null) {
      lastAngle.current = nextAngle;
      return;
    }
    let delta = nextAngle - prev;
    if (delta > 180) delta -= 360;
    if (delta < -180) delta += 360;
    const frameDelta = Math.round((delta / 360) * (44100 * 1.8));
    onSeek(Math.max(0, Math.min(totalFrames, position + frameDelta)));
    lastAngle.current = nextAngle;
  };

  const onPointerUp = () => {
    setDragging(false);
    lastAngle.current = null;
  };

  return (
    <div className="turntable-wrap">
      <div
        className={`turntable ${dragging ? "dragging" : ""} ${playing ? "spinning" : ""}`}
        style={{ transform: `rotate(${rotationDeg}deg)` }}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
      >
        <div className="vinyl-grooves" />
        <div className="vinyl-label">
          <span className="vinyl-title">{trackLabel ?? "No Track"}</span>
          <span className="vinyl-sub">{bpm > 0 ? `${bpm.toFixed(1)} BPM` : "— BPM"}</span>
        </div>
      </div>
    </div>
  );
}

function angleFromEvent(e: React.PointerEvent<HTMLDivElement>): number {
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
  const cx = rect.left + rect.width / 2;
  const cy = rect.top + rect.height / 2;
  const dx = e.clientX - cx;
  const dy = e.clientY - cy;
  return (Math.atan2(dy, dx) * 180) / Math.PI;
}

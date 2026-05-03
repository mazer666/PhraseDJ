/**
 * Crossfader.tsx — Equal-power crossfader between Deck A and Deck B.
 */

import { useEngineStore } from "../store/engineStore";

export function Crossfader(): React.JSX.Element {
  const value = useEngineStore((s) => s.crossfader);
  const set = useEngineStore((s) => s.setCrossfader);

  return (
    <div className="crossfader">
      <span className="xf-label-l">A</span>
      <input
        type="range"
        min={0}
        max={1}
        step={0.005}
        value={value}
        onChange={(e) => set(parseFloat(e.target.value))}
      />
      <span className="xf-label-r">B</span>
    </div>
  );
}

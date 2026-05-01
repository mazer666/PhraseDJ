/**
 * App.tsx — Root component.
 *
 * Phase 1 layout: header, two decks, crossfader, status bar.
 * Library browser comes later in Phase 1.
 */

import { useEffect } from "react";

import { Crossfader } from "./components/Crossfader";
import { Deck } from "./components/Deck";
import { StatusBar } from "./components/StatusBar";
import { useEngineStore } from "./store/engineStore";

function App(): React.JSX.Element {
  const startPolling = useEngineStore((s) => s.startPolling);
  const stopPolling  = useEngineStore((s) => s.stopPolling);

  // Start polling deck state on mount, stop on unmount.
  useEffect(() => {
    startPolling();
    return () => stopPolling();
  }, [startPolling, stopPolling]);

  return (
    <div className="app-shell">
      <header className="app-header">
        <h1>PhraseDJ</h1>
        <span className="app-subtitle">phrase-aware DJ mixing</span>
      </header>

      <main className="app-main">
        <div className="decks-row">
          <Deck side="A" />
          <Deck side="B" />
        </div>
        <div className="crossfader-row">
          <Crossfader />
        </div>
      </main>

      <StatusBar />
    </div>
  );
}

export default App;

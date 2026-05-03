/**
 * App.tsx — Root component.
 *
 * Phase 1 layout: header, two decks, crossfader, status bar.
 * Library browser comes later in Phase 1.
 */

import { useEffect, useState } from "react";

import { Crossfader } from "./components/Crossfader";
import { Deck } from "./components/Deck";
import { SettingsModal } from "./components/SettingsModal";
import { StatusBar } from "./components/StatusBar";
import { useEngineStore } from "./store/engineStore";
import { useKeymap } from "./hooks/useKeymap";

function App(): React.JSX.Element {
  const startPolling = useEngineStore((s) => s.startPolling);
  const stopPolling = useEngineStore((s) => s.stopPolling);
  const [showSettings, setShowSettings] = useState(false);

  // Start polling deck state on mount, stop on unmount.
  useEffect(() => {
    startPolling();
    return () => stopPolling();
  }, [startPolling, stopPolling]);

  // Register global keyboard shortcuts driven by keymap.toml.
  useKeymap();

  return (
    <div className="app-shell">
      <header className="app-header">
        <h1>PhraseDJ</h1>
        <span className="app-subtitle">phrase-aware DJ mixing</span>
        <div style={{ flex: 1 }} />
        <button
          className="btn-secondary"
          style={{ fontSize: "12px", padding: "4px 10px" }}
          onClick={() => setShowSettings(true)}
          title="Settings (Ctrl+,)"
        >
          ⚙ Settings
        </button>
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

      {showSettings && <SettingsModal onClose={() => setShowSettings(false)} />}
    </div>
  );
}

export default App;

import React from "react";

/**
 * Root component for PhraseDJ.
 *
 * Phase 0: renders a minimal placeholder so we can confirm the Tauri window
 * opens and React mounts correctly.  Real UI panels will be added in Phase 1.
 */
function App(): React.JSX.Element {
  return (
    <div className="app-shell">
      <header className="app-header">
        <h1>PhraseDJ</h1>
        <span className="app-version">v0.0.1 — Phase 0</span>
      </header>

      <main className="app-main">
        <p className="placeholder-text">
          Audio engine loading… (Phase 1)
        </p>
      </main>
    </div>
  );
}

export default App;

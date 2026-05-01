/**
 * StatusBar.tsx — Bottom status bar showing app version and engine state.
 */

import { useEffect, useState } from "react";

import { app, type AppStatus } from "../lib/api";

export function StatusBar(): React.JSX.Element {
  const [status, setStatus] = useState<AppStatus | null>(null);

  useEffect(() => {
    let mounted = true;
    app.status()
      .then((s) => { if (mounted) setStatus(s); })
      .catch(() => { /* commands may be unavailable in dev preview */ });
    return () => { mounted = false; };
  }, []);

  return (
    <footer className="status-bar">
      <span className="status-version">
        PhraseDJ v{status?.version ?? "?"}
      </span>
      <span className="status-engine">
        Audio: {status?.audio_running ? "running" : "offline"}
      </span>
      <span className="status-library">
        Library: {status?.library_count ?? 0} tracks
      </span>
    </footer>
  );
}

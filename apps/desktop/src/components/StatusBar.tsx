import { useEffect, useState } from "react";
import { app, type AppStatus } from "../lib/api";
import { useEngineStore } from "../store/engineStore";

export function StatusBar(): React.JSX.Element {
  const [status, setStatus] = useState<AppStatus | null>(null);
  const stemJobs = useEngineStore((s) => s.stemJobs);

  useEffect(() => {
    let mounted = true;
    app.status()
      .then((s) => { if (mounted) setStatus(s); })
      .catch(() => { /* commands may be unavailable in dev preview */ });
    return () => { mounted = false; };
  }, []);

  // Filter for jobs that are actually running or pending.
  const activeJobs = Object.entries(stemJobs).filter(
    ([_, job]) => job.status === "running" || job.status === "pending" || job.status === "model_downloading"
  );
  
  const isDownloading = activeJobs.some(([_, job]) => job.status === "model_downloading");

  const totalProgress = activeJobs.length > 0
    ? activeJobs.reduce((acc, [_, job]) => acc + job.progress, 0) / activeJobs.length
    : 0;

  return (
    <footer className="status-bar">
      <div className="status-left">
        <span className="status-version">
          PhraseDJ v{status?.version ?? "?"}
        </span>
        <span className="status-engine">
          Audio: {status?.audio_running ? "running" : "offline"}
        </span>
        <span className="status-library">
          Library: {status?.library_count ?? 0} tracks
        </span>
      </div>

      <div className="status-right">
        {activeJobs.length > 0 && (
          <div className="status-stems-progress">
            <span>
              {isDownloading ? "Downloading AI Model" : `Stem Analysis (${activeJobs.length})`}: {Math.round(totalProgress * 100)}%
            </span>
            <div className="progress-track">
              <div className="progress-fill" style={{ width: `${totalProgress * 100}%` }} />
            </div>
          </div>
        )}
      </div>
    </footer>
  );
}

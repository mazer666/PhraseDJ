/**
 * SettingsModal.tsx — Settings panel with reset-to-default.
 *
 * Phase 1 scope: audio output options, music root, lyrics online lookup,
 * UI FPS, and update check.  All fields map directly to UiSettings fields
 * persisted by the `settings_save` Tauri command.
 */

import { useEffect, useState } from "react";
import { app, type UiSettings } from "../lib/api";

const DEFAULTS: UiSettings = {
  sample_rate:     44_100,
  buffer_size:     128,
  pitch_range_pct: 8.0,
  music_root:      "~/Music",
  online_lookup:   false,
  target_fps:      60,
  update_check:    false,
};

interface SettingsModalProps {
  onClose: () => void;
}

export function SettingsModal({ onClose }: SettingsModalProps): React.JSX.Element {
  const [settings, setSettings] = useState<UiSettings>(DEFAULTS);
  const [dirty,    setDirty]    = useState(false);
  const [saving,   setSaving]   = useState(false);
  const [error,    setError]    = useState<string | null>(null);

  useEffect(() => {
    app.settingsLoad().then((s) => setSettings(s)).catch(() => {/* use defaults */});
  }, []);

  function update<K extends keyof UiSettings>(key: K, value: UiSettings[K]) {
    setSettings((s) => ({ ...s, [key]: value }));
    setDirty(true);
    setError(null);
  }

  async function save() {
    setSaving(true);
    try {
      await app.settingsSave(settings);
      setDirty(false);
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  }

  function resetToDefaults() {
    setSettings(DEFAULTS);
    setDirty(true);
    setError(null);
  }

  return (
    /* Backdrop */
    <div className="modal-backdrop" onClick={onClose}>
      {/* Panel — stop click from closing when interacting with content */}
      <div className="modal-panel" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Settings</h2>
          <button className="btn-icon" onClick={onClose} title="Close">✕</button>
        </div>

        <div className="modal-body">
          {/* Audio section */}
          <section className="settings-section">
            <h3>Audio</h3>

            <label className="settings-row">
              <span>Sample rate</span>
              <select
                value={settings.sample_rate}
                onChange={(e) => update("sample_rate", Number(e.target.value))}
              >
                <option value={44100}>44 100 Hz</option>
                <option value={48000}>48 000 Hz</option>
                <option value={96000}>96 000 Hz</option>
              </select>
            </label>

            <label className="settings-row">
              <span>Buffer size</span>
              <select
                value={settings.buffer_size}
                onChange={(e) => update("buffer_size", Number(e.target.value))}
              >
                <option value={64}>64 frames (~1.5 ms)</option>
                <option value={128}>128 frames (~2.9 ms)</option>
                <option value={256}>256 frames (~5.8 ms)</option>
                <option value={512}>512 frames (~11.6 ms)</option>
              </select>
            </label>

            <label className="settings-row">
              <span>Pitch fader range</span>
              <div className="settings-slider-row">
                <input
                  type="range"
                  min={2}
                  max={16}
                  step={1}
                  value={settings.pitch_range_pct}
                  onChange={(e) => update("pitch_range_pct", Number(e.target.value))}
                />
                <span className="value-label">±{settings.pitch_range_pct} %</span>
              </div>
            </label>
          </section>

          {/* Library section */}
          <section className="settings-section">
            <h3>Library</h3>

            <label className="settings-row">
              <span>Music root folder</span>
              <input
                type="text"
                className="settings-text-input"
                value={settings.music_root}
                onChange={(e) => update("music_root", e.target.value)}
                spellCheck={false}
              />
            </label>
          </section>

          {/* Lyrics section */}
          <section className="settings-section">
            <h3>Lyrics</h3>

            <label className="settings-row settings-row-toggle">
              <div>
                <span>Online lyrics lookup</span>
                <p className="settings-hint">
                  Fetches lyrics from LRCLib. Only title, artist and
                  duration are sent — never the audio file.
                </p>
              </div>
              <input
                type="checkbox"
                checked={settings.online_lookup}
                onChange={(e) => update("online_lookup", e.target.checked)}
              />
            </label>
          </section>

          {/* UI section */}
          <section className="settings-section">
            <h3>Display</h3>

            <label className="settings-row">
              <span>Target FPS</span>
              <select
                value={settings.target_fps}
                onChange={(e) => update("target_fps", Number(e.target.value))}
              >
                <option value={30}>30 fps</option>
                <option value={60}>60 fps</option>
                <option value={120}>120 fps (ProMotion)</option>
              </select>
            </label>
          </section>

          {/* Privacy section */}
          <section className="settings-section">
            <h3>Privacy</h3>

            <label className="settings-row settings-row-toggle">
              <div>
                <span>Check for updates on startup</span>
                <p className="settings-hint">No telemetry is ever sent.</p>
              </div>
              <input
                type="checkbox"
                checked={settings.update_check}
                onChange={(e) => update("update_check", e.target.checked)}
              />
            </label>
          </section>
        </div>

        {error && <p className="modal-error">{error}</p>}

        <div className="modal-footer">
          <button className="btn-secondary" onClick={resetToDefaults}>
            Reset to defaults
          </button>
          <div style={{ flex: 1 }} />
          <button className="btn-secondary" onClick={onClose}>Cancel</button>
          <button
            className="btn-primary"
            disabled={!dirty || saving}
            onClick={save}
          >
            {saving ? "Saving…" : "Save"}
          </button>
        </div>
      </div>
    </div>
  );
}

// Vitest global setup — imported by vite.config.ts `setupFiles`.
// Stubs Tauri APIs so tests can run in jsdom without a real Tauri host.

import { vi } from "vitest";

// Stub the Tauri core `invoke` so tests don't blow up on import.
// Individual tests override this via vi.mocked(invoke).mockResolvedValue(...)
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

// Stub the dialog plugin.
vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn().mockResolvedValue(null),
}));

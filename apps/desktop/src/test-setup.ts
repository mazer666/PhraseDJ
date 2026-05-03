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

// Stub the Tauri event plugin so startPolling's listen() call doesn't throw.
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
  emit: vi.fn().mockResolvedValue(undefined),
}));

// jsdom doesn't implement HTMLCanvasElement.getContext — stub it out.
Object.defineProperty(HTMLCanvasElement.prototype, "getContext", {
  value: vi.fn().mockReturnValue({
    clearRect: vi.fn(),
    fillRect: vi.fn(),
    beginPath: vi.fn(),
    moveTo: vi.fn(),
    lineTo: vi.fn(),
    stroke: vi.fn(),
    fill: vi.fn(),
    save: vi.fn(),
    restore: vi.fn(),
    scale: vi.fn(),
    translate: vi.fn(),
    drawImage: vi.fn(),
    getImageData: vi.fn().mockReturnValue({ data: new Uint8ClampedArray(4) }),
    putImageData: vi.fn(),
    createImageData: vi
      .fn()
      .mockReturnValue({ data: new Uint8ClampedArray(4) }),
    setTransform: vi.fn(),
    canvas: null,
  }),
});

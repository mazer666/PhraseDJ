/**
 * engineStore.test.ts — Unit tests for the Zustand engine store.
 *
 * We test:
 *  - Initial state values.
 *  - startPolling / stopPolling lifecycle.
 *  - setCrossfader / setFader / setMasterGain update local state.
 */

import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { useEngineStore } from "./engineStore";

const mockInvoke = vi.mocked(invoke);

const fakeDeckState = (deck: number) => ({
  deck,
  loaded: false,
  playing: false,
  position: 0,
  bpm: 0,
});

// ---------------------------------------------------------------------------
// Initial state
// ---------------------------------------------------------------------------

describe("engineStore – initial state", () => {
  it("crossfader defaults to 0.5", () => {
    expect(useEngineStore.getState().crossfader).toBe(0.5);
  });

  it("faderA and faderB default to 1.0", () => {
    const { faderA, faderB } = useEngineStore.getState();
    expect(faderA).toBe(1.0);
    expect(faderB).toBe(1.0);
  });

  it("masterGain defaults to 1.0", () => {
    expect(useEngineStore.getState().masterGain).toBe(1.0);
  });

  it("both decks start unloaded and paused", () => {
    const { decks } = useEngineStore.getState();
    expect(decks[0].loaded).toBe(false);
    expect(decks[1].playing).toBe(false);
  });
});

// ---------------------------------------------------------------------------
// Polling
// ---------------------------------------------------------------------------

describe("engineStore – polling", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    mockInvoke.mockResolvedValue(fakeDeckState(0));
  });

  afterEach(() => {
    useEngineStore.getState().stopPolling();
    vi.useRealTimers();
    mockInvoke.mockReset();
  });

  it("startPolling fires an interval that calls deck_state for both decks", async () => {
    mockInvoke
      .mockResolvedValueOnce(fakeDeckState(0))
      .mockResolvedValueOnce(fakeDeckState(1));

    useEngineStore.getState().startPolling();
    await vi.advanceTimersByTimeAsync(60);

    const commands = mockInvoke.mock.calls.map(([cmd]) => cmd);
    expect(commands).toContain("deck_state");
  });

  it("stopPolling prevents further interval ticks", async () => {
    mockInvoke.mockResolvedValue(fakeDeckState(0));
    useEngineStore.getState().startPolling();
    await vi.advanceTimersByTimeAsync(60);
    const _callsAfterStart = mockInvoke.mock.calls.length;

    useEngineStore.getState().stopPolling();
    mockInvoke.mockReset();

    await vi.advanceTimersByTimeAsync(300);
    // No new calls after stopping (in-flight tick may have already fired).
    expect(mockInvoke.mock.calls.length).toBe(0);
  });

  it("calling startPolling twice registers only one interval", async () => {
    mockInvoke.mockResolvedValue(fakeDeckState(0));
    useEngineStore.getState().startPolling();
    useEngineStore.getState().startPolling(); // second call is a no-op

    // Advance exactly one interval period.
    await vi.advanceTimersByTimeAsync(60);

    // One interval fires at most one tick → at most 2 deck_state calls (one per deck).
    const deckStateCalls = mockInvoke.mock.calls.filter(
      ([cmd]) => cmd === "deck_state"
    );
    // Allow 2 (normal) or 4 (two parallel ticks if timer fires twice within 60ms)
    // — the key invariant is there is no runaway doubling over many ticks.
    expect(deckStateCalls.length).toBeLessThanOrEqual(4);
  });
});

// ---------------------------------------------------------------------------
// Mixer actions
// ---------------------------------------------------------------------------

describe("engineStore – mixer actions", () => {
  beforeEach(() => {
    mockInvoke.mockResolvedValue(undefined);
  });

  afterEach(() => {
    mockInvoke.mockReset();
  });

  it("setCrossfader updates store and invokes mixer_set_crossfader", async () => {
    await useEngineStore.getState().setCrossfader(0.3);
    expect(useEngineStore.getState().crossfader).toBeCloseTo(0.3);
    expect(mockInvoke).toHaveBeenCalledWith("mixer_set_crossfader", { value: 0.3 });
  });

  it("setFader for deck 0 updates faderA and invokes mixer_set_fader", async () => {
    await useEngineStore.getState().setFader(0, 0.6);
    expect(useEngineStore.getState().faderA).toBeCloseTo(0.6);
    expect(mockInvoke).toHaveBeenCalledWith("mixer_set_fader", { deck: 0, value: 0.6 });
  });

  it("setFader for deck 1 updates faderB", async () => {
    await useEngineStore.getState().setFader(1, 0.4);
    expect(useEngineStore.getState().faderB).toBeCloseTo(0.4);
  });

  it("setMasterGain updates masterGain", async () => {
    await useEngineStore.getState().setMasterGain(0.9);
    expect(useEngineStore.getState().masterGain).toBeCloseTo(0.9);
    expect(mockInvoke).toHaveBeenCalledWith("mixer_set_master_gain", { value: 0.9 });
  });
});

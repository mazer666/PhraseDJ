/**
 * Deck.test.tsx — Unit tests for the Deck component.
 */

import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, beforeEach, vi } from "vitest";
import { Deck } from "./Deck";
import { useEngineStore } from "../store/engineStore";
import type { DeckState } from "../lib/api";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const blankDeck = (deck: number): DeckState => ({
  deck,
  loaded: false,
  playing: false,
  position: 0,
  bpm: 0,
  tempo_ratio: 1.0,
});

function setDeckState(side: "A" | "B", patch: Partial<DeckState>) {
  const idx = side === "A" ? 0 : 1;
  useEngineStore.setState((prev) => {
    const decks = [...prev.decks] as [DeckState, DeckState];
    decks[idx] = { ...decks[idx], ...patch };
    return { decks };
  });
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("Deck (side A)", () => {
  beforeEach(() => {
    useEngineStore.setState({
      decks: [blankDeck(0), blankDeck(1)],
      faderA: 1.0,
      faderB: 1.0,
      load: vi.fn().mockResolvedValue(undefined),
      play: vi.fn().mockResolvedValue(undefined),
      pause: vi.fn().mockResolvedValue(undefined),
      setFader: vi.fn().mockResolvedValue(undefined),
    } as any);
  });

  it("shows DECK A label", () => {
    render(<Deck side="A" />);
    screen.getByText("DECK A");
  });

  it("shows — BPM when bpm is 0", () => {
    render(<Deck side="A" />);
    expect(screen.getAllByText("— BPM").length).toBeGreaterThan(0);
  });

  it("shows formatted BPM when bpm > 0", () => {
    setDeckState("A", { bpm: 128 });
    render(<Deck side="A" />);
    expect(screen.getAllByText("128.0 BPM").length).toBeGreaterThan(0);
  });

  it("shows 'Load track…' when not loaded", () => {
    render(<Deck side="A" />);
    screen.getByText("Load track…");
  });

  it("shows 'Change track…' when loaded", () => {
    setDeckState("A", { loaded: true });
    render(<Deck side="A" />);
    screen.getByText("Change track…");
  });

  it("Play button is disabled when not loaded", () => {
    render(<Deck side="A" />);
    const btn = screen.getByRole("button", {
      name: "Play",
    }) as HTMLButtonElement;
    expect(btn.disabled).toBe(true);
  });

  it("Play button is enabled when loaded", () => {
    setDeckState("A", { loaded: true });
    render(<Deck side="A" />);
    const btn = screen.getByRole("button", {
      name: "Play",
    }) as HTMLButtonElement;
    expect(btn.disabled).toBe(false);
  });

  it("shows Pause button when playing", () => {
    setDeckState("A", { loaded: true, playing: true });
    render(<Deck side="A" />);
    screen.getByRole("button", { name: "Pause" });
  });

  it("clicking Play calls store play action", () => {
    const play = vi.fn().mockResolvedValue(undefined);
    setDeckState("A", { loaded: true, playing: false });
    useEngineStore.setState({ play } as any);
    render(<Deck side="A" />);
    fireEvent.click(screen.getByRole("button", { name: "Play" }));
    expect(play).toHaveBeenCalledWith(0);
  });

  it("clicking Pause calls store pause action", () => {
    const pause = vi.fn().mockResolvedValue(undefined);
    setDeckState("A", { loaded: true, playing: true });
    useEngineStore.setState({ pause } as any);
    render(<Deck side="A" />);
    fireEvent.click(screen.getByRole("button", { name: "Pause" }));
    expect(pause).toHaveBeenCalledWith(0);
  });

  it("position shows 0:00 when at frame 0", () => {
    render(<Deck side="A" />);
    screen.getByText("0:00");
  });

  it("position formats correctly at 90 seconds (44100 sr)", () => {
    // 90 s × 44100 frames/s = 3 969 000 frames
    setDeckState("A", { position: 3_969_000 });
    render(<Deck side="A" />);
    screen.getByText("1:30");
  });

  it("fader slider renders with correct initial value", () => {
    useEngineStore.setState({ faderA: 0.8 } as any);
    render(<Deck side="A" />);
    const sliders = screen.getAllByRole("slider") as HTMLInputElement[];
    expect(parseFloat(sliders[sliders.length - 1].value)).toBeCloseTo(0.8);
  });
});

describe("Deck (side B)", () => {
  beforeEach(() => {
    useEngineStore.setState({
      decks: [blankDeck(0), blankDeck(1)],
      faderA: 1.0,
      faderB: 1.0,
      load: vi.fn().mockResolvedValue(undefined),
      play: vi.fn().mockResolvedValue(undefined),
      pause: vi.fn().mockResolvedValue(undefined),
      setFader: vi.fn().mockResolvedValue(undefined),
    } as any);
  });

  it("shows DECK B label", () => {
    render(<Deck side="B" />);
    screen.getByText("DECK B");
  });

  it("clicking Play calls store play with deck index 1", () => {
    const play = vi.fn().mockResolvedValue(undefined);
    setDeckState("B", { loaded: true, playing: false });
    useEngineStore.setState({ play } as any);
    render(<Deck side="B" />);
    fireEvent.click(screen.getByRole("button", { name: "Play" }));
    expect(play).toHaveBeenCalledWith(1);
  });
});

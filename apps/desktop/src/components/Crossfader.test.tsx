/**
 * Crossfader.test.tsx — Unit tests for the Crossfader component.
 */

import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, beforeEach, vi } from "vitest";
import { Crossfader } from "./Crossfader";
import { useEngineStore } from "../store/engineStore";

function resetStore() {
  useEngineStore.setState({
    crossfader: 0.5,
    setCrossfader: vi.fn().mockResolvedValue(undefined),
  } as any);
}

describe("Crossfader", () => {
  beforeEach(() => { resetStore(); });

  it("renders A and B labels", () => {
    render(<Crossfader />);
    // getByText throws if absent — presence is the assertion.
    screen.getByText("A");
    screen.getByText("B");
  });

  it("range input starts at the store crossfader value", () => {
    useEngineStore.setState({ crossfader: 0.25 } as any);
    render(<Crossfader />);
    const slider = screen.getByRole("slider") as HTMLInputElement;
    expect(parseFloat(slider.value)).toBeCloseTo(0.25);
  });

  it("calls setCrossfader when slider changes", () => {
    const setCrossfader = vi.fn().mockResolvedValue(undefined);
    useEngineStore.setState({ crossfader: 0.5, setCrossfader } as any);
    render(<Crossfader />);
    fireEvent.change(screen.getByRole("slider"), { target: { value: "0.75" } });
    expect(setCrossfader).toHaveBeenCalledWith(0.75);
  });

  it("slider min is 0 and max is 1", () => {
    render(<Crossfader />);
    const slider = screen.getByRole("slider") as HTMLInputElement;
    expect(slider.min).toBe("0");
    expect(slider.max).toBe("1");
  });
});

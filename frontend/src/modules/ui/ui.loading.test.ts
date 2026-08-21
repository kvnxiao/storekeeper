import { renderHook } from "@solidjs/testing-library";
import { createSignal } from "solid-js";
import { afterEach, describe, expect, it, vi } from "vite-plus/test";
import { createLoadingPhase, LOADING_FADE_MS, LOADING_MIN_MS } from "@/modules/ui/ui.loading";

function mountPhase() {
  const [loading, setLoading] = createSignal(false);
  const { result: phase, cleanup } = renderHook(() => createLoadingPhase(loading));
  return { phase, setLoading, cleanup };
}

describe("createLoadingPhase", () => {
  afterEach(() => vi.useRealTimers());

  it("stays unset while nothing is loading", () => {
    vi.useFakeTimers();
    const { phase } = mountPhase();

    vi.advanceTimersByTime(LOADING_MIN_MS + LOADING_FADE_MS);

    expect(phase()).toBeUndefined();
  });

  it("holds a fetch that lands early for the minimum duration", () => {
    vi.useFakeTimers();
    const { phase, setLoading } = mountPhase();

    setLoading(true);
    expect(phase()).toBe("active");

    setLoading(false);
    vi.advanceTimersByTime(LOADING_MIN_MS - 1);
    expect(phase()).toBe("active");

    vi.advanceTimersByTime(1);
    expect(phase()).toBe("fading");

    vi.advanceTimersByTime(LOADING_FADE_MS);
    expect(phase()).toBeUndefined();
  });

  it("fades as soon as a fetch that outlasts the minimum lands", () => {
    vi.useFakeTimers();
    const { phase, setLoading } = mountPhase();

    setLoading(true);
    vi.advanceTimersByTime(LOADING_MIN_MS * 2);
    expect(phase()).toBe("active");

    setLoading(false);
    expect(phase()).toBe("fading");

    vi.advanceTimersByTime(LOADING_FADE_MS);
    expect(phase()).toBeUndefined();
  });

  it("returns to active when loading resumes mid-fade", () => {
    vi.useFakeTimers();
    const { phase, setLoading } = mountPhase();

    setLoading(true);
    setLoading(false);
    vi.advanceTimersByTime(LOADING_MIN_MS);
    expect(phase()).toBe("fading");

    setLoading(true);
    expect(phase()).toBe("active");

    vi.advanceTimersByTime(LOADING_FADE_MS);
    expect(phase()).toBe("active");
  });

  it("measures the hold from the first load, not from a resumed one", () => {
    vi.useFakeTimers();
    const { phase, setLoading } = mountPhase();

    setLoading(true);
    vi.advanceTimersByTime(400);
    setLoading(false);
    vi.advanceTimersByTime(100);
    setLoading(true);
    vi.advanceTimersByTime(100);
    setLoading(false);

    vi.advanceTimersByTime(LOADING_MIN_MS - 600 - 1);
    expect(phase()).toBe("active");

    vi.advanceTimersByTime(1);
    expect(phase()).toBe("fading");
  });

  it("drops the pending fade when the root disposes", () => {
    vi.useFakeTimers();
    const { phase, setLoading, cleanup } = mountPhase();

    setLoading(true);
    setLoading(false);
    cleanup();

    vi.advanceTimersByTime(LOADING_MIN_MS + LOADING_FADE_MS);

    expect(phase()).toBe("active");
  });
});

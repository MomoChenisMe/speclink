import { describe, it, expect, afterEach, vi } from "vitest";
import { render, screen, fireEvent, cleanup, act } from "@testing-library/react";

import { useCopied } from "../components/useCopied";

afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

function Probe() {
  const [copied, markCopied] = useCopied();
  return <button onClick={markCopied}>{copied ? "copied" : "idle"}</button>;
}

describe("useCopied", () => {
  it("觸發後亮起，1.2 秒自動復原", () => {
    vi.useFakeTimers();
    render(<Probe />);
    fireEvent.click(screen.getByRole("button"));
    expect(screen.getByRole("button").textContent).toBe("copied");
    act(() => {
      vi.advanceTimersByTime(1200);
    });
    expect(screen.getByRole("button").textContent).toBe("idle");
  });

  it("重複觸發重新計時，不被前一次的到期提早熄燈", () => {
    vi.useFakeTimers();
    render(<Probe />);
    fireEvent.click(screen.getByRole("button"));
    act(() => {
      vi.advanceTimersByTime(800);
    });
    fireEvent.click(screen.getByRole("button"));
    act(() => {
      vi.advanceTimersByTime(800);
    });
    expect(screen.getByRole("button").textContent).toBe("copied");
    act(() => {
      vi.advanceTimersByTime(400);
    });
    expect(screen.getByRole("button").textContent).toBe("idle");
  });

  it("unmount 取消未到期的計時器，不留晚於拆除觸發的 setState", () => {
    vi.useFakeTimers();
    const { unmount } = render(<Probe />);
    fireEvent.click(screen.getByRole("button"));
    unmount();
    expect(vi.getTimerCount()).toBe(0);
  });
});

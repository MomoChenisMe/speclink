// 觸控板滑動返回（spec desktop-app「詳情頁的導覽與返回」；desktop-swipe-back-finger-lift design D4）：
// 只在原生層送來的手指按下（began）到離開（ended）之間累計朝返回方向的水平位移，左緣「←」指示
// 隨之推出、達門檻轉實心；手指離開的當下結算——已達門檻即返回、未達收回，不等慣性捲動。
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, render } from "@testing-library/react";
import { listen } from "@tauri-apps/api/event";

import { SWIPE_THRESHOLD, SwipeBack } from "../components/SwipeBack";

// 攔下 Tauri 事件層：捕捉 trackpad-gesture 的處理器，測試以 gesture() 模擬手指按下與離開。
type GestureHandler = (e: { payload: { phase: "began" | "ended"; swipeEnabled: boolean } }) => void;
const { gestureHandlers } = vi.hoisted(() => ({ gestureHandlers: [] as GestureHandler[] }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((event: string, handler: GestureHandler) => {
    if (event === "trackpad-gesture") gestureHandlers.push(handler);
    return Promise.resolve(() => {
      const i = gestureHandlers.indexOf(handler);
      if (i >= 0) gestureHandlers.splice(i, 1);
    });
  }),
}));

// 假計時器且從不推進：結算若仍靠計時器，onBack 就不會在 ended 的當下被呼叫。
beforeEach(() => {
  vi.useFakeTimers();
});
afterEach(() => {
  cleanup();
  vi.useRealTimers();
  gestureHandlers.length = 0;
  vi.mocked(listen).mockClear();
});

const indicator = () => document.querySelector("[data-swipe-back]") as HTMLElement;

/** 原生層的手指按下／離開；ended 的 swipeEnabled 恆為 false（Rust 端如此送）。 */
function gesture(phase: "began" | "ended", swipeEnabled = phase === "began") {
  act(() => {
    for (const h of [...gestureHandlers]) h({ payload: { phase, swipeEnabled } });
  });
}

/** 送一個 wheel 事件；`inverted` 對應 WebKit 的自然捲動旗標（缺席＝不帶此欄位）。 */
function wheel(
  target: EventTarget,
  deltaX: number,
  { deltaY = 0, inverted, ctrlKey = false, shiftKey = false }: { deltaY?: number; inverted?: boolean; ctrlKey?: boolean; shiftKey?: boolean } = {},
) {
  const e = new WheelEvent("wheel", { deltaX, deltaY, ctrlKey, shiftKey, bubbles: true, cancelable: true });
  if (inverted !== undefined) Object.defineProperty(e, "webkitDirectionInvertedFromDevice", { value: inverted });
  act(() => {
    target.dispatchEvent(e);
  });
}

/** 一次完整手勢：手指按下、依序送出位移（自然捲動下手指向右＝deltaX 為負）、手指離開。 */
function swipe(deltas: number[], target: EventTarget = window, inverted?: boolean) {
  gesture("began");
  for (const dx of deltas) wheel(target, dx, { inverted });
  gesture("ended");
}

describe("SwipeBack", () => {
  it("手指按著滑過門檻：指示轉實心但不返回；手指離開的當下返回並收回指示", () => {
    const onBack = vi.fn();
    render(<SwipeBack enabled onBack={onBack} />);
    gesture("began");
    for (let i = 0; i < 4; i++) wheel(window, -50);
    expect(indicator().getAttribute("data-armed")).toBe("true");
    expect(onBack).not.toHaveBeenCalled();
    gesture("ended");
    expect(onBack).toHaveBeenCalledTimes(1);
    expect(indicator().getAttribute("data-progress")).toBe("0.00");
  });

  // spec Example「放開時的結算（門檻 120px）」逐列；慣性是手指離開後才送的 wheel。
  it.each([
    { gesture: "向右滑 200px 後放開", result: "立即關閉詳情頁", moves: [-50, -50, -50, -50], momentum: [], backs: 1 },
    { gesture: "向右滑 80px 後放開", result: "指示收回、詳情頁維持", moves: [-40, -40], momentum: [], backs: 0 },
    {
      gesture: "向右滑 150px、未放開再往回滑到 80px 後放開",
      result: "指示收回、詳情頁維持",
      moves: [-50, -50, -50, 70],
      momentum: [],
      backs: 0,
    },
    {
      gesture: "向右滑 200px 後放開，慣性又捲了 300px",
      result: "放開當下關頁，慣性不影響",
      moves: [-50, -50, -50, -50],
      momentum: [-100, -100, -100],
      backs: 1,
    },
  ])("$gesture → $result", ({ moves, momentum, backs }) => {
    const onBack = vi.fn();
    render(<SwipeBack enabled onBack={onBack} />);
    swipe(moves);
    expect(onBack).toHaveBeenCalledTimes(backs);
    expect(indicator().getAttribute("data-progress")).toBe("0.00");
    for (const dx of momentum) wheel(window, dx);
    expect(indicator().getAttribute("data-progress")).toBe("0.00");
    expect(onBack).toHaveBeenCalledTimes(backs);
  });

  it("放開前滑回門檻以下：指示由實心轉回空心，放開不返回", () => {
    const onBack = vi.fn();
    render(<SwipeBack enabled onBack={onBack} />);
    gesture("began");
    for (let i = 0; i < 3; i++) wheel(window, -50);
    expect(indicator().getAttribute("data-armed")).toBe("true");
    wheel(window, 70);
    expect(indicator().hasAttribute("data-armed")).toBe(false);
    gesture("ended");
    expect(onBack).not.toHaveBeenCalled();
  });

  it("沒有手勢階段的輸入（一般滑鼠水平滾輪、Shift＋滾輪）不推進指示、不返回", () => {
    const onBack = vi.fn();
    render(<SwipeBack enabled onBack={onBack} />);
    for (let i = 0; i < 4; i++) wheel(window, -50);
    for (let i = 0; i < 4; i++) wheel(window, -50, { shiftKey: true });
    expect(indicator().getAttribute("data-progress")).toBe("0.00");
    act(() => {
      vi.advanceTimersByTime(1000);
    });
    expect(onBack).not.toHaveBeenCalled();
  });

  it("系統設定不允許兩指滑動（began 的 swipeEnabled 為 false）時不推進、不返回", () => {
    const onBack = vi.fn();
    render(<SwipeBack enabled onBack={onBack} />);
    gesture("began", false);
    for (let i = 0; i < 4; i++) wheel(window, -50);
    expect(indicator().getAttribute("data-progress")).toBe("0.00");
    gesture("ended");
    expect(onBack).not.toHaveBeenCalled();
  });

  it("MayBegin 之後的 Began（tracking 中再收 began）不歸零累計", () => {
    const onBack = vi.fn();
    render(<SwipeBack enabled onBack={onBack} />);
    gesture("began");
    wheel(window, -60);
    gesture("began");
    wheel(window, -60);
    gesture("ended");
    expect(onBack).toHaveBeenCalledTimes(1);
  });

  it("先垂直捲動 40px 再向右滑 200px：整次手勢取消，放開不返回", () => {
    const onBack = vi.fn();
    render(<SwipeBack enabled onBack={onBack} />);
    gesture("began");
    wheel(window, 0, { deltaY: 40 });
    for (let i = 0; i < 4; i++) wheel(window, -50);
    expect(indicator().getAttribute("data-progress")).toBe("0.00");
    gesture("ended");
    expect(onBack).not.toHaveBeenCalled();
  });

  it("純水平 200px 夾少量垂直晃動（累計 8px）不取消，放開即返回", () => {
    const onBack = vi.fn();
    render(<SwipeBack enabled onBack={onBack} />);
    gesture("began");
    for (let i = 0; i < 4; i++) wheel(window, -50, { deltaY: 2 });
    gesture("ended");
    expect(onBack).toHaveBeenCalledTimes(1);
  });

  it("往前方向的滑動與縮放（ctrl＋滾輪）都不理會", () => {
    const onBack = vi.fn();
    render(<SwipeBack enabled onBack={onBack} />);
    swipe([50, 50, 50, 50]);
    gesture("began");
    for (let i = 0; i < 4; i++) wheel(window, -50, { ctrlKey: true });
    expect(indicator().getAttribute("data-progress")).toBe("0.00");
    gesture("ended");
    expect(onBack).not.toHaveBeenCalled();
  });

  /** 寬 600、可視 200 的水平捲動區塊（overflow 可捲動才算），回傳其內的子元素當事件目標。 */
  function scroller(scrollLeft: number, overflowX = "auto") {
    const box = document.createElement("div");
    const cell = document.createElement("span");
    box.appendChild(cell);
    document.body.appendChild(box);
    box.style.overflowX = overflowX;
    Object.defineProperty(box, "scrollWidth", { value: 600 });
    Object.defineProperty(box, "clientWidth", { value: 200 });
    box.scrollLeft = scrollLeft;
    return { box, cell };
  }

  it("手勢開始於仍可往左捲動的水平捲動區塊時不接手；捲到最左才接手", () => {
    const onBack = vi.fn();
    render(<SwipeBack enabled onBack={onBack} />);
    const { box, cell } = scroller(50);
    swipe([-40, -40, -40, -40], cell);
    expect(onBack).not.toHaveBeenCalled();
    box.scrollLeft = 0;
    swipe([-40, -40, -40, -40], cell);
    expect(onBack).toHaveBeenCalledTimes(1);
    box.remove();
  });

  it("自然捲動關閉時手指向右是把區塊往右捲：停在最左的寬表格仍可往右捲就不接手，捲到最右才接手", () => {
    const onBack = vi.fn();
    render(<SwipeBack enabled onBack={onBack} />);
    const { box, cell } = scroller(0);
    swipe([40, 40, 40, 40], cell, false);
    expect(onBack).not.toHaveBeenCalled();
    box.scrollLeft = 400;
    swipe([40, 40, 40, 40], cell, false);
    expect(onBack).toHaveBeenCalledTimes(1);
    box.remove();
  });

  it("只裁切不捲動的區塊（overflow hidden，如截斷的長名稱）不擋返回", () => {
    const onBack = vi.fn();
    render(<SwipeBack enabled onBack={onBack} />);
    const { box, cell } = scroller(0, "hidden");
    swipe([40, 40, 40, 40], cell, false);
    expect(onBack).toHaveBeenCalledTimes(1);
    box.remove();
  });

  it("自然捲動關閉時方向反轉：手指向右＝deltaX 為正", () => {
    const onBack = vi.fn();
    render(<SwipeBack enabled onBack={onBack} />);
    swipe([-40, -40, -40, -40], window, false);
    expect(onBack).not.toHaveBeenCalled();
    swipe([40, 40, 40, 40], window, false);
    expect(onBack).toHaveBeenCalledTimes(1);
  });

  it("停用時不訂閱手勢事件、也不監聽 wheel", () => {
    const addListener = vi.spyOn(window, "addEventListener");
    const onBack = vi.fn();
    render(<SwipeBack enabled={false} onBack={onBack} />);
    expect(listen).not.toHaveBeenCalled();
    expect(addListener.mock.calls.some(([type]) => type === "wheel")).toBe(false);
    swipe([-50, -50, -50, -50]);
    expect(onBack).not.toHaveBeenCalled();
    expect(indicator().getAttribute("data-progress")).toBe("0.00");
    addListener.mockRestore();
  });

  it("手勢中途停用（詳情頁關閉）：指示歸零並解除訂閱", async () => {
    const onBack = vi.fn();
    const { rerender } = render(<SwipeBack enabled onBack={onBack} />);
    gesture("began");
    wheel(window, -SWIPE_THRESHOLD / 2);
    expect(indicator().getAttribute("data-progress")).toBe("0.50");
    rerender(<SwipeBack enabled={false} onBack={onBack} />);
    expect(indicator().getAttribute("data-progress")).toBe("0.00");
    await act(async () => {});
    expect(gestureHandlers).toHaveLength(0);
  });
});

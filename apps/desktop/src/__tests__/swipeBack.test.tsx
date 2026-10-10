// 觸控板滑動返回（spec desktop-app「詳情頁的導覽與返回」；desktop-readonly-detail-pages design D8）：
// 水平滑動朝返回方向累計位移、左緣「←」指示隨之推出，達門檻轉實心；最後一個事件後靜止才
// 判定手勢結束，屆時已達門檻即返回、未達收回。
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, render } from "@testing-library/react";

import { SWIPE_THRESHOLD, SwipeBack } from "../components/SwipeBack";

beforeEach(() => {
  vi.useFakeTimers();
});
afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

const indicator = () => document.querySelector("[data-swipe-back]") as HTMLElement;

/** 送一個 wheel 事件；`inverted` 對應 WebKit 的自然捲動旗標（缺席＝不帶此欄位）。 */
function wheel(target: EventTarget, deltaX: number, deltaY = 0, inverted?: boolean) {
  const e = new WheelEvent("wheel", { deltaX, deltaY, bubbles: true, cancelable: true });
  if (inverted !== undefined) Object.defineProperty(e, "webkitDirectionInvertedFromDevice", { value: inverted });
  act(() => {
    target.dispatchEvent(e);
  });
}
/** 手勢結束：最後一個事件後靜止。 */
const settle = () =>
  act(() => {
    vi.advanceTimersByTime(200);
  });

describe("SwipeBack", () => {
  it("朝返回方向累計達門檻：指示轉實心，靜止後才觸發返回並收回指示", () => {
    const onBack = vi.fn();
    render(<SwipeBack enabled onBack={onBack} />);
    // 自然捲動下手指向右＝deltaX 為負。
    for (let i = 0; i < 4; i++) wheel(window, -40);
    expect(indicator().getAttribute("data-armed")).toBe("true");
    expect(onBack).not.toHaveBeenCalled();
    settle();
    expect(onBack).toHaveBeenCalledTimes(1);
    expect(indicator().getAttribute("data-progress")).toBe("0.00");
  });

  it("未達門檻：指示推出一半後收回，不觸發", () => {
    const onBack = vi.fn();
    render(<SwipeBack enabled onBack={onBack} />);
    wheel(window, -SWIPE_THRESHOLD / 4);
    wheel(window, -SWIPE_THRESHOLD / 4);
    expect(indicator().getAttribute("data-progress")).toBe("0.50");
    expect(indicator().hasAttribute("data-armed")).toBe(false);
    settle();
    expect(onBack).not.toHaveBeenCalled();
    expect(indicator().getAttribute("data-progress")).toBe("0.00");
  });

  it("反向滑回會扣減累計，放開時未達門檻不觸發", () => {
    const onBack = vi.fn();
    render(<SwipeBack enabled onBack={onBack} />);
    wheel(window, -80);
    wheel(window, -80);
    expect(indicator().getAttribute("data-armed")).toBe("true");
    wheel(window, 100);
    expect(indicator().hasAttribute("data-armed")).toBe(false);
    settle();
    expect(onBack).not.toHaveBeenCalled();
  });

  it("垂直為主的捲動與往前方向的滑動都不理會", () => {
    const onBack = vi.fn();
    render(<SwipeBack enabled onBack={onBack} />);
    for (let i = 0; i < 5; i++) wheel(window, -40, 60);
    for (let i = 0; i < 5; i++) wheel(window, 40);
    expect(indicator().getAttribute("data-progress")).toBe("0.00");
    settle();
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
    for (let i = 0; i < 4; i++) wheel(cell, -40);
    expect(indicator().getAttribute("data-progress")).toBe("0.00");
    settle();
    box.scrollLeft = 0;
    for (let i = 0; i < 4; i++) wheel(cell, -40);
    settle();
    expect(onBack).toHaveBeenCalledTimes(1);
    box.remove();
  });

  it("自然捲動關閉時手指向右是把區塊往右捲：停在最左的寬表格仍可往右捲就不接手，捲到最右才接手", () => {
    const onBack = vi.fn();
    render(<SwipeBack enabled onBack={onBack} />);
    const { box, cell } = scroller(0);
    for (let i = 0; i < 4; i++) wheel(cell, 40, 0, false);
    expect(indicator().getAttribute("data-progress")).toBe("0.00");
    settle();
    expect(onBack).not.toHaveBeenCalled();
    box.scrollLeft = 400;
    for (let i = 0; i < 4; i++) wheel(cell, 40, 0, false);
    settle();
    expect(onBack).toHaveBeenCalledTimes(1);
    box.remove();
  });

  it("只裁切不捲動的區塊（overflow hidden，如截斷的長名稱）不擋返回", () => {
    const onBack = vi.fn();
    render(<SwipeBack enabled onBack={onBack} />);
    const { box, cell } = scroller(0, "hidden");
    for (let i = 0; i < 4; i++) wheel(cell, 40, 0, false);
    settle();
    expect(onBack).toHaveBeenCalledTimes(1);
    box.remove();
  });

  it("自然捲動關閉時方向反轉：手指向右＝deltaX 為正", () => {
    const onBack = vi.fn();
    render(<SwipeBack enabled onBack={onBack} />);
    for (let i = 0; i < 4; i++) wheel(window, -40, 0, false);
    settle();
    expect(onBack).not.toHaveBeenCalled();
    for (let i = 0; i < 4; i++) wheel(window, 40, 0, false);
    settle();
    expect(onBack).toHaveBeenCalledTimes(1);
  });

  it("停用時不監聽", () => {
    const onBack = vi.fn();
    render(<SwipeBack enabled={false} onBack={onBack} />);
    for (let i = 0; i < 4; i++) wheel(window, -40);
    settle();
    expect(onBack).not.toHaveBeenCalled();
    expect(indicator().getAttribute("data-progress")).toBe("0.00");
  });
});

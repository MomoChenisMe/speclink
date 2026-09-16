// 波次章與被擋變淡（spec desktop-app「看板卡片的波次與阻擋標示」；add-change-plan-desktop
// design D4 改版）：清單項帶 wave 時進度列最左浮現圓圈數字章，tooltip 合併前置清單；
// blockedBy 非空時整卡變淡（data-blocked），標題列不長任何排程章；缺 wave 時無章不變淡。
import { describe, it, expect, afterEach, vi } from "vitest";
import { render as rtlRender, screen, fireEvent, cleanup, act } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider, type UiLocale } from "../i18n";
import { ChangeCard } from "../components/ChangeCard";
import type { ChangeItem } from "../adapter";

function render(ui: ReactElement, locale: UiLocale = "zh-TW") {
  const wrapper = ({ children }: { children: ReactNode }) => (
    <I18nProvider locale={locale}>{children}</I18nProvider>
  );
  return rtlRender(ui, { wrapper });
}

afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

function card(plan: Partial<ChangeItem>): ChangeItem {
  return { name: "add-c", status: "proposed", totalTasks: 3, completedTasks: 0, ...plan };
}

const cardEl = () => document.querySelector('[data-change="add-c"]') as HTMLElement;
/** 標題列＝Card 的第一個子元素（CardHeader）。 */
const headerEl = () => cardEl().firstElementChild as HTMLElement;

/** hover 章、推進 tooltip 延遲後回傳浮層文字。 */
function hoverTooltip(badge: HTMLElement): string {
  fireEvent.pointerMove(badge, { pointerType: "mouse" });
  act(() => {
    vi.advanceTimersByTime(300);
  });
  return document.querySelector("[data-side]")?.textContent ?? "";
}

describe("ChangeCard 波次章與被擋變淡", () => {
  it("第一波無阻擋：進度列最左圓圈「1」，tooltip「第 1 波」，卡片不變淡", () => {
    vi.useFakeTimers();
    render(<ChangeCard change={card({ wave: 1, blockedBy: [], dependsOn: [], overlaps: [] })} />);
    const wave = screen.getByLabelText("第 1 波");
    expect(wave.textContent).toBe("1");
    expect(wave.getAttribute("title")).toBeNull();
    // 章在進度列內（任務數所在的那一列），且是該列第一個子元素（進度條之前）；
    // 以既有節點定位，不靠測試專用的 data 屬性（缺 wave 時 DOM 才能逐位元不變）。
    const row = wave.parentElement as HTMLElement;
    expect(row.textContent).toContain("0/3");
    expect(row.firstElementChild).toBe(wave);
    expect(headerEl().contains(wave)).toBe(false);
    expect(cardEl().getAttribute("data-blocked")).toBeNull();
    expect(cardEl().className).not.toContain("opacity-60");
    expect(hoverTooltip(wave)).toContain("第 1 波");
  });

  it("第二波被兩項擋住：圓圈「2」、tooltip「第 2 波 · 等待：add-a、add-b」、整卡變淡、標題列無「等 N 項」", () => {
    vi.useFakeTimers();
    render(
      <ChangeCard
        change={card({ wave: 2, blockedBy: ["add-a", "add-b"], dependsOn: ["add-a"], overlaps: [] })}
      />,
    );
    const wave = screen.getByLabelText("第 2 波");
    expect(wave.textContent).toBe("2");
    expect(cardEl().getAttribute("data-blocked")).toBe("true");
    expect(cardEl().className).toContain("opacity-60");
    expect(screen.queryByText(/等 \d+ 項/)).toBeNull();
    expect(headerEl().textContent).not.toMatch(/等待|波/);
    expect(hoverTooltip(wave)).toContain("第 2 波 · 等待：add-a、add-b");
  });

  it("缺 wave（remote 摘要或 plan 成環）時無章、不變淡", () => {
    render(<ChangeCard change={card({})} />);
    expect(screen.queryByLabelText(/^第 \d+ 波$/)).toBeNull();
    expect(cardEl().getAttribute("data-blocked")).toBeNull();
    expect(cardEl().className).not.toContain("opacity-60");
    // 其餘呈現逐位元一致：DOM 不留任何排程痕跡（含測試用屬性）。
    expect(cardEl().outerHTML).not.toMatch(/data-blocked|data-progress|opacity-60/);
    cleanup();
    // 只帶 blockedBy 而無 wave（不合法組合）也不變淡。
    render(<ChangeCard change={card({ blockedBy: ["add-a"] })} />);
    expect(cardEl().getAttribute("data-blocked")).toBeNull();
  });

  it("en 文案：Wave 2 · Waiting for: add-a, add-b", () => {
    vi.useFakeTimers();
    render(
      <ChangeCard change={card({ wave: 2, blockedBy: ["add-a", "add-b"], dependsOn: [], overlaps: [] })} />,
      "en",
    );
    const wave = screen.getByLabelText("Wave 2");
    expect(hoverTooltip(wave)).toContain("Wave 2 · Waiting for: add-a, add-b");
  });
});

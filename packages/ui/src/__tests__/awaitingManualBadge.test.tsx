// 待手動章（spec desktop-app「看板卡片的待手動標示」；design D5）：寫碼任務全
// 完成、僅餘未勾 `[M]` 時卡片浮現行內小章，tooltip 載明剩餘項數。
import { describe, it, expect, afterEach, vi } from "vitest";
import { render as rtlRender, screen, fireEvent, cleanup, act } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { ChangeCard } from "../components/ChangeCard";
import type { ChangeItem } from "../adapter";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

const TOTAL = 10;

/** 一張寫碼進度已知的卡：remaining＝未勾任務數，codeRemaining 由呼叫端給。 */
function card(remaining: number, codeRemaining: number): ChangeItem {
  return {
    name: "c",
    status: "in-progress",
    totalTasks: TOTAL,
    completedTasks: TOTAL - remaining,
    codeTotal: TOTAL - remaining + codeRemaining,
    codeComplete: TOTAL - remaining,
    codeRemaining,
  };
}

describe("ChangeCard 待手動章", () => {
  it("浮現判定逐列成立（spec Example 表）", () => {
    // | codeRemaining | remaining | 待手動章 |
    const rows: [number, number, boolean][] = [
      [0, 1, true],
      [0, 3, true],
      [2, 3, false],
      [0, 0, false],
    ];
    for (const [codeRemaining, remaining, shown] of rows) {
      render(<ChangeCard change={card(remaining, codeRemaining)} />);
      const badge = screen.queryByLabelText("待手動");
      expect(Boolean(badge), `codeRemaining=${codeRemaining} remaining=${remaining}`).toBe(shown);
      cleanup();
    }
  });

  it("codeTotal=0 的全手動變更不浮現章（spec Example 表末列：空真值）", () => {
    // 尚無任何寫碼任務時「寫碼全完成」是空真值——沒開工的卡不該宣告待手動。
    render(
      <ChangeCard
        change={{
          name: "c",
          status: "proposed",
          totalTasks: 2,
          completedTasks: 0,
          codeTotal: 0,
          codeComplete: 0,
          codeRemaining: 0,
        }}
      />,
    );
    expect(screen.queryByLabelText("待手動")).toBeNull();
  });

  it("tooltip 載明剩餘項數（沿審查標示家族：無原生 title）", () => {
    // Scenario「待手動卡片浮現章」：codeTotal=9、codeComplete=9、total=10、complete=9。
    vi.useFakeTimers();
    render(<ChangeCard change={card(1, 0)} />);
    const badge = screen.getByLabelText("待手動");
    expect(badge.getAttribute("title")).toBeNull();
    fireEvent.pointerMove(badge, { pointerType: "mouse" });
    act(() => {
      vi.advanceTimersByTime(300);
    });
    expect(document.querySelector("[data-side]")?.textContent).toContain("待手動·剩 1 項");
  });

  it("en 章文案為 Awaiting manual（放寬後不提 test）", () => {
    // 語意放寬後兩語系的章文案都不得再把 `[M]` 說成只限測試。
    vi.useFakeTimers();
    rtlRender(
      <I18nProvider locale="en">
        <ChangeCard change={card(1, 0)} />
      </I18nProvider>,
    );
    const badge = screen.getByLabelText("Awaiting manual");
    // 與 zh 版同構：無原生 title、tooltip 精確比對（退回 "manual test" 即紅燈）。
    expect(badge.getAttribute("title")).toBeNull();
    fireEvent.pointerMove(badge, { pointerType: "mouse" });
    act(() => {
      vi.advanceTimersByTime(300);
    });
    expect(document.querySelector("[data-side]")?.textContent).toContain("Awaiting manual · 1 left");
  });

  it("remote 摘要缺寫碼進度欄位時章缺席", () => {
    // Scenario「remote 模式無章」：缺欄位＝資料源不供，不猜、不以全量計數代打。
    render(
      <ChangeCard change={{ name: "c", status: "in-progress", totalTasks: 10, completedTasks: 9 }} />,
    );
    expect(screen.queryByLabelText("待手動")).toBeNull();
  });
});

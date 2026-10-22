// spec desktop-app「已封存頁的流程成本摘要」的呈現：四項、每個數字的樣本數、
// 繁中與英文文案、沒有資料時整個摘要不出現。
import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";

import type { ArchivedItem } from "../adapter";
import { I18nProvider, type UiLocale } from "../i18n";
import { ProcessCostSummary } from "../components/ProcessCostSummary";

function item(name: string, fields: Partial<ArchivedItem>): ArchivedItem {
  return { datedName: `2026-10-01-${name}`, date: "2026-10-01", name, ...fields };
}

function renderIn(locale: UiLocale, items: ArchivedItem[]) {
  return render(
    <I18nProvider locale={locale}>
      <ProcessCostSummary items={items} />
    </I18nProvider>,
  ).container;
}

/** 每一格（DocTiles 的磚）的 [大字, 小字]，依呈現順序。 */
function tiles(container: HTMLElement): [string, string][] {
  return [...container.querySelectorAll("[data-process-cost] [data-doc-tile]")].map((tile) => {
    const [value, label] = [...tile.children].map((el) => el.textContent ?? "");
    return [value, label];
  });
}

// Scenario「顯示中位數、P90 與樣本數」。
const THREE = [
  item("a", { docLines: 100, tasksTotal: 4, leadDays: 0 }),
  item("b", { docLines: 200, tasksTotal: 6, leadDays: 0 }),
  item("c", { docLines: 400, tasksTotal: 9, leadDays: 3 }),
];

const WITH_ROUNDS = [
  item("a", { docLines: 100, tasksTotal: 4, leadDays: 0, leadHours: 6, reviewRounds: 3 }),
  item("b", { docLines: 200, tasksTotal: 6, leadDays: 1, leadHours: 30.5, reviewRounds: 1, verifyRounds: 2 }),
];

describe("ProcessCostSummary", () => {
  it("shows medians as the tile values with P90 and sample counts under a 流程成本（中位數） heading", () => {
    const container = renderIn("zh-TW", THREE);
    expect(container.querySelector("[data-process-cost] h3")!.textContent).toBe("流程成本（中位數）");
    expect(tiles(container)).toEqual([
      ["200", "文件行數（3 筆） · P90 400"],
      ["6", "任務數（3 筆）"],
      ["0 天", "建立到封存（3 筆）"],
    ]);
  });

  it("averages an even sample and pairs every number with its own sample count", () => {
    // Scenario「偶數筆取中間兩值平均」＋輪數各自計樣本。
    const container = renderIn("zh-TW", WITH_ROUNDS);
    expect(tiles(container).slice(2)).toEqual([
      ["0.5 天 · 18.3 小時", "建立到封存（2／2 筆）"],
      ["審查 2 · 驗證 2", "品質關卡輪數（2／1 筆）"],
    ]);
  });

  it("follows the UI language", () => {
    const container = renderIn("en", WITH_ROUNDS);
    expect(container.querySelector("[data-process-cost] h3")!.textContent).toBe("Process cost (median)");
    expect(tiles(container)).toEqual([
      ["150", "Doc lines (2 changes) · P90 200"],
      ["5", "Tasks (2 changes)"],
      ["0.5 days · 18.3 hours", "Created to archived (2 / 2 changes)"],
      ["Review 2 · Verify 2", "Quality gate rounds (2 / 1 changes)"],
    ]);
  });

  it("uses the singular form for a single count of one", () => {
    const container = renderIn("en", [item("only", { docLines: 10, tasksTotal: 3 })]);
    expect(tiles(container)).toEqual([
      ["10", "Doc lines (1 change) · P90 10"],
      ["3", "Tasks (1 change)"],
    ]);
  });

  it("keeps every label short enough for a quarter of the default window", () => {
    // DocTiles 的小字會截斷：1440px 預設視窗扣掉側欄後每格約 245px，11px 字約 40 個半形字元。
    // 標籤超過就會把樣本數截掉（spec「每個數字標出樣本數」）。
    for (const locale of ["en", "zh-TW"] as const) {
      for (const [, label] of tiles(renderIn(locale, WITH_ROUNDS))) {
        expect(label.length, label).toBeLessThanOrEqual(36);
      }
    }
  });

  it("leaves out the quality gate tile when no change carries rounds", () => {
    // Scenario「舊記錄沒有輪數時不顯示該項」。
    const container = renderIn("zh-TW", THREE);
    expect(tiles(container).map(([, label]) => label).some((l) => l.startsWith("品質關卡輪數"))).toBe(false);
  });

  it("renders nothing without process cost fields", () => {
    // Scenario「沒有資料時整個摘要不出現」：沒有封存變更，或遠端清單只帶任務數。
    expect(renderIn("zh-TW", []).innerHTML).toBe("");
    expect(renderIn("zh-TW", [item("remote", { tasksTotal: 4 })]).innerHTML).toBe("");
  });
});

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

/** 每個數字一行（dd）的文字，依 data-cost 鍵取用。 */
function numbers(container: HTMLElement): Record<string, string> {
  return Object.fromEntries(
    [...container.querySelectorAll<HTMLElement>("[data-cost]")].map((el) => [el.dataset.cost, el.textContent]),
  );
}

function labels(container: HTMLElement): string[] {
  return [...container.querySelectorAll("[data-cost-item] dt")].map((el) => el.textContent ?? "");
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
  it("shows medians, P90 and sample counts under a 流程成本 heading", () => {
    const container = renderIn("zh-TW", THREE);
    const region = container.querySelector("[data-process-cost]")!;
    expect(region.querySelector("h3")!.textContent).toBe("流程成本");
    expect(labels(container)).toEqual(["文件行數", "任務數", "建立到封存"]);
    expect(numbers(container)).toEqual({
      docLines: "中位數 200 · P90 400 · 3 筆",
      tasks: "中位數 6 · 3 筆",
      leadDays: "中位數 0 天 · 3 筆",
    });
  });

  it("averages an even sample and shows hours next to days", () => {
    // Scenario「偶數筆取中間兩值平均」＋輪數各自計樣本。
    const container = renderIn("zh-TW", WITH_ROUNDS);
    expect(labels(container)).toEqual(["文件行數", "任務數", "建立到封存", "品質關卡輪數"]);
    expect(numbers(container)).toMatchObject({
      leadDays: "中位數 0.5 天 · 2 筆",
      leadHours: "中位數 18.3 小時 · 2 筆",
      reviewRounds: "審查 中位數 2 · 2 筆",
      verifyRounds: "驗證 中位數 2 · 1 筆",
    });
  });

  it("follows the UI language", () => {
    const container = renderIn("en", WITH_ROUNDS);
    expect(container.querySelector("[data-process-cost] h3")!.textContent).toBe("Process cost");
    expect(labels(container)).toEqual(["Doc lines", "Tasks", "Created to archived", "Quality gate rounds"]);
    expect(numbers(container)).toEqual({
      docLines: "median 150 · P90 200 · 2 changes",
      tasks: "median 5 · 2 changes",
      leadDays: "median 0.5 days · 2 changes",
      leadHours: "median 18.3 hours · 2 changes",
      reviewRounds: "Review median 2 · 2 changes",
      verifyRounds: "Verify median 2 · 1 change",
    });
  });

  it("leaves out the quality gate item when no change carries rounds", () => {
    // Scenario「舊記錄沒有輪數時不顯示該項」。
    const container = renderIn("zh-TW", THREE);
    expect(container.querySelector('[data-cost-item="rounds"]')).toBeNull();
  });

  it("renders nothing without process cost fields", () => {
    // Scenario「沒有資料時整個摘要不出現」：沒有封存變更，或遠端清單只帶任務數。
    expect(renderIn("zh-TW", []).innerHTML).toBe("");
    expect(renderIn("zh-TW", [item("remote", { tasksTotal: 4 })]).innerHTML).toBe("");
  });
});

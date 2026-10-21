// spec desktop-app「已封存頁的流程成本摘要」的彙總規則：中位數、P90、樣本數與
// 顯示格式（process-cost-metrics design D4：彙總在呈現層的純函式）。
import { describe, it, expect } from "vitest";

import type { ArchivedItem } from "../adapter";
import { formatCost, median, p90, summarizeProcessCost } from "../processCost";

function item(name: string, fields: Partial<ArchivedItem>): ArchivedItem {
  return { datedName: `2026-10-01-${name}`, date: "2026-10-01", name, ...fields };
}

describe("median and P90", () => {
  it("takes the middle value of an odd count and the mean of the two middle values of an even count", () => {
    expect(median([400, 100, 200])).toBe(200);
    expect(median([1, 0])).toBe(0.5);
    expect(median([7])).toBe(7);
  });

  it("takes the ⌈0.9 × n⌉-th smallest value as P90", () => {
    expect(p90([100, 200, 400])).toBe(400);
    expect(p90([1, 2, 3, 4, 5, 6, 7, 8, 9, 10])).toBe(9);
    // 0.9 × 30 在浮點數是 27.000000000000004——位置仍是第 27 個。
    expect(p90(Array.from({ length: 30 }, (_, i) => i + 1))).toBe(27);
  });
});

describe("summarizeProcessCost", () => {
  it("summarizes doc lines, tasks and lead days with their sample counts", () => {
    // Scenario「顯示中位數、P90 與樣本數」。
    const summary = summarizeProcessCost([
      item("a", { docLines: 100, tasksTotal: 4, leadDays: 0 }),
      item("b", { docLines: 200, tasksTotal: 6, leadDays: 0 }),
      item("c", { docLines: 400, tasksTotal: 9, leadDays: 3 }),
    ]);
    expect(summary).toEqual({
      docLines: { median: 200, p90: 400, samples: 3 },
      tasks: { median: 6, samples: 3 },
      leadDays: { median: 0, samples: 3 },
    });
  });

  it("averages the two middle values of an even sample", () => {
    // Scenario「偶數筆取中間兩值平均」。
    const summary = summarizeProcessCost([
      item("a", { docLines: 10, leadDays: 0, leadHours: 6 }),
      item("b", { docLines: 20, leadDays: 1, leadHours: 30.5 }),
    ]);
    expect(summary.leadDays).toEqual({ median: 0.5, samples: 2 });
    expect(summary.leadHours).toEqual({ median: 18.25, samples: 2 });
    expect(formatCost(summary.leadHours!.median)).toBe("18.3");
  });

  it("counts only the items that carry a field and leaves out numbers with no samples", () => {
    // Scenario「舊記錄沒有輪數時不顯示該項」＋輪數各自計樣本。
    const summary = summarizeProcessCost([
      item("old", { docLines: 300, leadDays: 2 }),
      item("new", { docLines: 100, leadDays: 0, leadHours: 5, reviewRounds: 3 }),
    ]);
    expect(summary.reviewRounds).toEqual({ median: 3, samples: 1 });
    expect(summary.verifyRounds).toBeUndefined();
    expect(summary.leadHours).toEqual({ median: 5, samples: 1 });
    expect(summary.tasks).toBeUndefined();
  });

  it("returns an empty summary when no item carries a process cost field", () => {
    // Scenario「沒有資料時整個摘要不出現」：沒有封存變更，或遠端清單只帶任務數。
    expect(summarizeProcessCost([])).toEqual({});
    expect(
      summarizeProcessCost([item("remote-a", { tasksTotal: 4 }), item("remote-b", { tasksTotal: 6 })]),
    ).toEqual({});
  });
});

describe("formatCost", () => {
  it("shows at most one decimal and integers without a decimal point", () => {
    expect(formatCost(200)).toBe("200");
    expect(formatCost(0.5)).toBe("0.5");
    expect(formatCost(18.25)).toBe("18.3");
    expect(formatCost(2.04)).toBe("2");
    expect(formatCost(0)).toBe("0");
  });
});

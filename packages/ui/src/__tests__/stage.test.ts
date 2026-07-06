// spec 需求「看板欄位由生命週期標記驅動」：全完成＝已就緒 ＞ 有 started＝進行中
// ＞ 其餘＝提案中。矩陣值取自 spec 的 Example 表。
import { describe, it, expect } from "vitest";
import { changeStage } from "../stage";
import type { ChangeItem } from "../adapter";

function ci(total: number, done: number, startedAt?: string): ChangeItem {
  return { name: "c", status: "x", totalTasks: total, completedTasks: done, startedAt };
}

describe("changeStage（標記驅動）", () => {
  it.each([
    // [totalTasks, completedTasks, startedAt, expected]
    [0, 0, undefined, "proposed"], // 無標記、0 任務
    [28, 0, undefined, "proposed"], // 無標記、0/28——剛 propose 完不再錯置為進行中
    [28, 13, undefined, "proposed"], // 無標記、有進度亦留在提案中（欄位由標記驅動）
    [28, 13, "2026-07-06", "in-progress"], // 有標記、未全完成
    [28, 28, undefined, "ready"], // 全完成優先（就算沒標記）
    [28, 28, "2026-07-06", "ready"], // 全完成優先（有標記）
    [0, 0, "2026-07-06", "in-progress"], // 有標記、0 任務——已開工
  ] as const)("total=%i done=%i started=%s → %s", (total, done, startedAt, expected) => {
    expect(changeStage(ci(total, done, startedAt))).toBe(expected);
  });
});

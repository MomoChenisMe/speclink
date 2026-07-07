// spec 需求「看板欄位由生命週期標記驅動」：全完成＝已就緒 ＞ started_at 或任務
// 完成數>0＝進行中 ＞ 其餘＝提案中。矩陣值取自 spec 的 Example「欄位判定矩陣」表。
import { describe, it, expect } from "vitest";
import { changeStage } from "../stage";
import type { ChangeItem } from "../adapter";

function ci(total: number, done: number, startedAt?: string): ChangeItem {
  return { name: "c", status: "x", totalTasks: total, completedTasks: done, startedAt };
}

describe("changeStage（標記驅動）", () => {
  it.each([
    // [totalTasks, completedTasks, startedAt, expected]
    [0, 0, undefined, "proposed"], // 無 | 0 任務 | 提案中
    [28, 0, undefined, "proposed"], // 無 | 0/28 | 提案中——剛 propose 完不錯置
    [28, 3, undefined, "in-progress"], // 無 | 3/28 | 進行中——任務進度涵蓋繞過工具的寫入路徑
    [28, 0, "2026-07-06", "in-progress"], // 有 | 0/28 | 進行中
    [28, 13, "2026-07-06", "in-progress"], // 有 | 13/28 | 進行中
    [28, 28, undefined, "ready"], // 無 | 28/28 | 已就緒（全完成優先）
    [28, 28, "2026-07-06", "ready"], // 有 | 28/28 | 已就緒
    [0, 0, "2026-07-06", "in-progress"], // 有標記、0 任務——已開工
  ] as const)("total=%i done=%i started=%s → %s", (total, done, startedAt, expected) => {
    expect(changeStage(ci(total, done, startedAt))).toBe(expected);
  });
});

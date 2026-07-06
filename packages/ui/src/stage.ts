import type { ChangeItem } from "./adapter";

/** SDD 生命週期階段（不含 Archived——歸檔由歸檔清單另計）。 */
export type Stage = "proposed" | "in-progress" | "ready";

/** 看板欄位順序（左→右流動）。 */
export const STAGES: Stage[] = ["proposed", "in-progress", "ready"];

export const STAGE_LABEL: Record<Stage, string> = {
  proposed: "提案中",
  "in-progress": "進行中",
  ready: "已就緒",
};

/**
 * 由生命週期標記派生 change 的階段（優先序由上而下）：
 * - 任務全完成（總數 > 0 且完成數 == 總數）→ ready（可歸檔，全完成優先）
 * - meta 含 started_at → in-progress（誰開工了才算進行中）
 * - 其餘 → proposed（就算任務已就位——剛 propose 完不再錯置）
 */
export function changeStage(c: ChangeItem): Stage {
  if (c.totalTasks > 0 && c.completedTasks >= c.totalTasks) return "ready";
  if (c.startedAt) return "in-progress";
  return "proposed";
}

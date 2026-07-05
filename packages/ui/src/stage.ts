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
 * 由任務進度派生 change 的生命週期階段：
 * - 0 tasks → proposed（尚在建 artifact）
 * - 全部完成 → ready（可歸檔）
 * - 其餘 → in-progress
 */
export function changeStage(c: ChangeItem): Stage {
  if (c.totalTasks === 0) return "proposed";
  if (c.completedTasks >= c.totalTasks) return "ready";
  return "in-progress";
}

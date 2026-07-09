import type { ChangeItem } from "./adapter";

/** SDD 生命週期階段（不含 Archived——歸檔由歸檔清單另計）。 */
export type Stage = "proposed" | "in-progress" | "ready";

/** 看板欄位順序（左→右流動）。顯示標籤經 i18n：t(`stage.${stage}`)。 */
export const STAGES: Stage[] = ["proposed", "in-progress", "ready"];

/**
 * 各階段的徽章／chip 配色——單一 teal 色相以深淺表達生命週期推進。看板欄計數
 * 徽章與討論欄 promoted chip 共用此單一來源，避免兩處配色分歧。
 */
export const STAGE_BADGE: Record<Stage, string> = {
  proposed: "bg-primary/8 text-primary/70",
  "in-progress": "bg-primary/12 text-primary",
  ready: "bg-primary text-primary-foreground",
};

/**
 * 由生命週期標記派生 change 的階段（優先序由上而下）：
 * - 任務全完成（總數 > 0 且完成數 == 總數）→ ready（可歸檔，全完成優先）
 * - meta 含 started_at 或任務完成數 > 0 → in-progress（任務進度涵蓋手改
 *   tasks.md、agent 直改、git pull 等繞過工具的寫入路徑——派生管顯示，
 *   開工歸屬仍只由 meta 的 started_* 誠實記錄）
 * - 其餘 → proposed（剛 propose 完、全未勾、未開工）
 */
export function changeStage(c: ChangeItem): Stage {
  if (c.totalTasks > 0 && c.completedTasks >= c.totalTasks) return "ready";
  if (c.startedAt || c.completedTasks > 0) return "in-progress";
  return "proposed";
}

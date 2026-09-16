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
 * 各階段的進度條填色——單一 teal 色相深淺階梯（提案中最淺、進行中次之、
 * 已就緒最深）。看板欄位進度條與系統匣面板進度條共用此單一來源，避免
 * 兩處配色分歧。
 */
export const STAGE_BAR: Record<Stage, string> = {
  proposed: "bg-primary/50",
  "in-progress": "bg-primary/75",
  ready: "bg-primary",
};

/** 各階段的圖示色——與 STAGE_BAR 同階梯；看板欄標題與面板分區標題共用。 */
export const STAGE_ICON: Record<Stage, string> = {
  proposed: "text-primary/50",
  "in-progress": "text-primary/75",
  ready: "text-primary",
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

/**
 * 待手動判定的單一入口（spec desktop-app「看板卡片的待手動標示」）：寫碼任務
 * 全完成、僅餘未勾 `[M]` 時回傳剩餘手測項數，其餘回 0——含兩個不浮現的邊界：
 * remote 摘要缺寫碼進度欄位（undefined 不猜、不以全量計數代打），以及
 * codeTotal 為 0 的全手動變更（「寫碼全完成」的空真值不視為待手動）。
 */
export function awaitingManualCount(c: ChangeItem): number {
  const remaining = c.totalTasks - c.completedTasks;
  const show = (c.codeTotal ?? 0) > 0 && c.codeRemaining === 0 && remaining > 0;
  return show ? remaining : 0;
}

/**
 * 波次讀取入口（spec desktop-app「看板卡片的波次與阻擋標示」）：清單項帶 wave
 * 才有波次；remote 摘要、plan 成環或壞 meta 時欄位缺席，回 null——卡片與排程
 * 分頁只讀結果、不自行判斷缺席原因。
 */
export function planWave(c: ChangeItem): number | null {
  return typeof c.wave === "number" ? c.wave : null;
}

/** 阻擋清單讀取入口：只在波次存在時成立（四欄同進同出），其餘回空陣列。 */
export function planBlockedBy(c: ChangeItem): string[] {
  return planWave(c) === null ? [] : (c.blockedBy ?? []);
}

/** 波次文字（card.wave）：卡片章、面板列首、排程分頁共用同一處組裝。 */
export function planWaveLabel(wave: number, t: (key: string) => string): string {
  return t("card.wave").replace("{n}", String(wave));
}

/** 前置清單文字（card.blockedTitle）：名稱以語系分隔符相連，卡片 tooltip 與面板列 title 共用。 */
export function planBlockedLabel(blockedBy: readonly string[], t: (key: string) => string): string {
  return t("card.blockedTitle").replace("{names}", blockedBy.join(t("common.listSeparator")));
}

/*
 * 介面狀態語意色的單一來源（spec desktop-app「介面狀態語意色分層」、
 * server-web-console「後台狀態徽章語意色」共用）。三層色彩角色規則：
 * 主色只做連結／互動／進度，狀態一律語意色，靜態 metadata 一律中性。
 *
 * 三紅分工——三張表各司其職，不合併：
 * - 錯誤訊息與危險動作 → 本表的 `danger`（destructive token）
 * - 品質站未通過章     → reviewStyle.tsx 的 `reviewedNotPassed`（rose）
 * - delta 刪除         → DeltaBadges.tsx 的 `removed`（red）
 */

/** 狀態文字／圖示色：進行中＝藍、成功＝綠、警示＝琥珀、錯誤與危險＝紅。 */
export const SEMANTIC_TONE = {
  inProgress: "text-sky-600 dark:text-sky-400",
  success: "text-emerald-600 dark:text-emerald-400",
  warning: "text-amber-600 dark:text-amber-500",
  danger: "text-destructive",
} as const;

/** 同語意的面色（border＋淡底）：供橫幅、狀態卡與政策衝突面等有底色的區塊用。 */
export const SEMANTIC_SURFACE = {
  inProgress: "border-sky-500/40 bg-sky-500/10",
  success: "border-emerald-500/40 bg-emerald-500/10",
  warning: "border-amber-500/40 bg-amber-500/10",
  danger: "border-destructive/40 bg-destructive/10",
} as const;

/** 語意色鍵集——`SEMANTIC_TONE` 與 `SEMANTIC_SURFACE` 的共同鍵。 */
export type SemanticTone = keyof typeof SEMANTIC_TONE;

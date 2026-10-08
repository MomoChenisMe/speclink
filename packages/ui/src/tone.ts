/*
 * 介面狀態語意色的單一來源（spec desktop-app「介面狀態語意色分層」、
 * server-web-console「後台狀態徽章語意色」共用）。三層色彩角色規則：
 * 主色只做連結／互動／進度，狀態一律語意色，靜態 metadata 一律中性。
 *
 * 表內只保存「狀態→token class」對照：色值與深色值都在 theme.css 的
 * status-* token，這裡不寫原生色階、也不寫深色成對變體。錯誤與危險
 * 沿用 destructive token，不另設 status-danger。
 */

/** 狀態文字／圖示色：進行中＝藍、成功＝綠、警示＝琥珀、錯誤與危險＝紅。 */
export const SEMANTIC_TONE = {
  inProgress: "text-status-progress",
  success: "text-status-success",
  warning: "text-status-warning",
  danger: "text-destructive",
} as const;

/** 同語意的面色（border＋淡底）：供橫幅、狀態卡與政策衝突面等有底色的區塊用。 */
export const SEMANTIC_SURFACE = {
  inProgress: "border-status-progress/40 bg-status-progress/10",
  success: "border-status-success/40 bg-status-success/10",
  warning: "border-status-warning/40 bg-status-warning/10",
  danger: "border-destructive/40 bg-destructive/10",
} as const;

/** 語意色鍵集——`SEMANTIC_TONE` 與 `SEMANTIC_SURFACE` 的共同鍵。 */
export type SemanticTone = keyof typeof SEMANTIC_TONE;

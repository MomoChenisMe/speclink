import { Sparkles } from "lucide-react";

/* 改進討論標示樣式表（spec desktop-app「看板討論卡片的改進標示」「討論抽屜的
   改進標示」共用）：卡片小章、promoted 收合列、已封存側與兩個抽屜的對照只此
   一份——鏡射 reviewStyle.tsx 的集中模式。色調取 indigo：審查／驗證章的紫是
   品質站蓋章專屬，此章標的是討論型別而非狀態，兩者不共色。 */
export const IMPROVE_TONE = "text-indigo-600 dark:text-indigo-400";

/** 抽屜章籤的染色底——透明度對齊 tone.ts 的 /10 慣例。 */
export const IMPROVE_CHIP_TONE = "bg-indigo-500/10";

/** 改進標示圖示（各呈現面自帶尺寸 className）。 */
export const ImproveIcon = Sparkles;

/** 改進討論的 i18n 詞條 key——卡片小章、已封存側與抽屜標示共用同一份文案。 */
export const IMPROVE_LABEL_KEY = "discussion.kindImprove";

/** kind 是否為改進討論（單一合法值，缺席即一般討論）。 */
export function isImproveKind(kind?: string | null): boolean {
  return kind === "improve";
}

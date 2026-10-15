import type { Translator } from "./i18n";

/** 相對時間（天級即可；日期通常是 YYYY-MM-DD）。翻譯函式由呼叫端注入；無日期回 null、
 * 不可解析原樣回傳（可見的失敗，非靜默錯值）。差 1 天回「昨天」，所以 `time.daysAgo` 實際只在
 * 2 天以上出現——它的 `.one` 是字典守門要求的完整性，不會顯示。 */
export function relativeDays(date: string | null | undefined, { t, tn }: Translator): string | null {
  if (!date) return null;
  const parsed = Date.parse(date);
  if (Number.isNaN(parsed)) return date;
  const days = Math.floor((Date.now() - parsed) / 86_400_000);
  if (days <= 0) return t("time.today");
  if (days === 1) return t("time.yesterday");
  return tn("time.daysAgo", days);
}

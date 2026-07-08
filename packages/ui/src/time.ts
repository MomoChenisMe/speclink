/** 相對時間（天級即可；日期通常是 YYYY-MM-DD）。t 由呼叫端注入；無日期回 null、
 * 不可解析原樣回傳（可見的失敗，非靜默錯值）。 */
export function relativeDays(date: string | null | undefined, t: (key: string) => string): string | null {
  if (!date) return null;
  const parsed = Date.parse(date);
  if (Number.isNaN(parsed)) return date;
  const days = Math.floor((Date.now() - parsed) / 86_400_000);
  if (days <= 0) return t("time.today");
  if (days === 1) return t("time.yesterday");
  return t("time.daysAgo").replace("{n}", String(days));
}

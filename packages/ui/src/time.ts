import { useEffect, useState } from "react";

import type { Translator } from "./i18n";

/** 使用者時區今天的日期鍵（YYYY-MM-DD）。 */
function localDayKey(now: Date): string {
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
}

/**
 * 讓元件在使用者時區的每個午夜重繪一次（desktop-uses-list-fields D6）：回傳今天的日期鍵。
 * 計時器排在下一個午夜加 1 秒，每次觸發都重排下一個（提早觸發也不會停）；卸載時清掉。
 * 睡眠跨過午夜時，計時器在喚醒後觸發。
 */
export function useLocalDay(): string {
  const [tick, setTick] = useState(0);
  useEffect(() => {
    const now = new Date();
    const next = new Date(now.getFullYear(), now.getMonth(), now.getDate() + 1, 0, 0, 1);
    const timer = setTimeout(() => setTick((n) => n + 1), next.getTime() - now.getTime());
    return () => clearTimeout(timer);
  }, [tick]);
  return localDayKey(new Date());
}

/** 使用者時區的那一天的午夜（毫秒）。只有日期的 `YYYY-MM-DD` 是使用者時區的那一天，
 * 不照 `Date.parse` 當成 UTC 午夜。解析不出來回 NaN。 */
function localMidnight(date: string): number {
  const dateOnly = /^(\d{4})-(\d{2})-(\d{2})$/.exec(date);
  const d = dateOnly ? new Date(+dateOnly[1], +dateOnly[2] - 1, +dateOnly[3]) : new Date(date);
  return new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();
}

/** 相對時間（天級），以使用者時區的日曆日計算（desktop-uses-list-fields D5）：與現在同一天
 * （或在現在之後）回「今天」、前一天回「昨天」、更早回「N 天前」。翻譯函式由呼叫端注入；
 * 無日期回 null、不可解析原樣回傳（可見的失敗，非靜默錯值）。差 1 天回「昨天」，所以
 * `time.daysAgo` 實際只在 2 天以上出現——它的 `.one` 是字典守門要求的完整性，不會顯示。 */
export function relativeDays(date: string | null | undefined, { t, tn }: Translator): string | null {
  if (!date) return null;
  const then = localMidnight(date);
  if (Number.isNaN(then)) return date;
  // 兩個午夜相差 23 或 25 小時的日子（夏令時間）以四捨五入算成整天。
  const days = Math.round((localMidnight(new Date().toISOString()) - then) / 86_400_000);
  if (days <= 0) return t("time.today");
  if (days === 1) return t("time.yesterday");
  return tn("time.daysAgo", days);
}

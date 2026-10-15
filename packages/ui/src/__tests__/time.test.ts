// relativeDays（fix-english-status-labels-and-plurals）：翻譯物件由呼叫端注入；英文 2 天以上用複數，
// 差 1 天固定寫「昨天」——所以 time.daysAgo 的單數形不會出現（desktop-config「數量為 0 或 2 以上時用複數」）。
import { describe, it, expect } from "vitest";

import { translatorFor } from "../i18n";
import { relativeDays } from "../time";

const daysAgo = (days: number) => new Date(Date.now() - days * 86_400_000 - 3_600_000).toISOString();

describe("relativeDays", () => {
  const en = translatorFor("en");

  it("英文 3 天前寫「3 days ago」", () => {
    expect(relativeDays(daysAgo(3), en)).toBe("3 days ago");
  });

  it("差 1 天寫「Yesterday」，不寫「1 day ago」", () => {
    expect(relativeDays(daysAgo(1), en)).toBe(en.t("time.yesterday"));
  });

  it("繁中 3 天前用繁中字串", () => {
    const zh = translatorFor("zh-TW");
    expect(relativeDays(daysAgo(3), zh)).toBe(zh.t("time.daysAgo").replace("{n}", "3"));
  });

  it("沒有日期回 null，不可解析的字串原樣回傳", () => {
    expect(relativeDays(null, en)).toBeNull();
    expect(relativeDays("not a date", en)).toBe("not a date");
  });
});

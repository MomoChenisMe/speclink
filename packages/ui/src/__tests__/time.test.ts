// relativeDays（fix-english-status-labels-and-plurals）：翻譯物件由呼叫端注入；英文 2 天以上用複數，
// 差 1 天固定寫「昨天」——所以 time.daysAgo 的單數形不會出現（desktop-config「數量為 0 或 2 以上時用複數」）。
// desktop-uses-list-fields D5：以使用者時區的日曆日計算。現在時間一律用假時鐘固定、日期一律以本地時間
// 建構，CI 的 UTC 與使用者的時區都成立。
import { afterEach, beforeEach, describe, it, expect, vi } from "vitest";

import { translatorFor } from "../i18n";
import { relativeDays } from "../time";

/** 本地時間 2026-11-<day> <hour>:<minute> 的 RFC 3339 字串。 */
const local = (day: number, hour: number, minute = 0) => new Date(2026, 10, day, hour, minute).toISOString();

describe("relativeDays", () => {
  const en = translatorFor("en");
  const zh = translatorFor("zh-TW");

  beforeEach(() => {
    vi.useFakeTimers({ toFake: ["Date"] });
    vi.setSystemTime(new Date(2026, 10, 26, 12, 0));
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  it("英文 3 天前寫「3 days ago」", () => {
    expect(relativeDays(local(23, 11), en)).toBe("3 days ago");
  });

  it("差 1 天寫「Yesterday」，不寫「1 day ago」", () => {
    expect(relativeDays(local(25, 11), en)).toBe(en.t("time.yesterday"));
  });

  it("繁中 3 天前用繁中字串", () => {
    expect(relativeDays(local(23, 11), zh)).toBe(zh.t("time.daysAgo").replace("{n}", "3"));
  });

  it("沒有日期回 null，不可解析的字串原樣回傳", () => {
    expect(relativeDays(null, en)).toBeNull();
    expect(relativeDays("not a date", en)).toBe("not a date");
  });

  // delta desktop-app「看板卡片顯示最後更新」Example「modifiedAt 與顯示的文字」。
  it.each([
    // [modifiedAt（本地）, 現在（本地）, 顯示]
    [[26, 0, 10], [26, 23, 50], "今天"],
    [[25, 23, 0], [26, 9, 0], "昨天"],
    [[24, 1, 0], [26, 23, 0], "2 天前"],
    [[23, 12, 0], [26, 8, 0], "3 天前"],
    [[27, 10, 0], [26, 9, 0], "今天"], // 在現在之後
  ] as const)("modifiedAt 11/%j、現在 11/%j → %s", ([d, h, m], [nd, nh, nm], text) => {
    vi.setSystemTime(new Date(2026, 10, nd, nh, nm));
    expect(relativeDays(local(d, h, m), zh)).toBe(text);
  });

  it("只有日期的 YYYY-MM-DD 當成使用者時區的那一天，不當成 UTC 午夜", () => {
    for (const hour of [0, 1, 12, 23]) {
      vi.setSystemTime(new Date(2026, 10, 26, hour, 30));
      expect(relativeDays("2026-11-25", zh)).toBe("昨天");
      expect(relativeDays("2026-11-26", zh)).toBe("今天");
    }
  });
});

// app 層字典的兩語言 key 集合相等（比照 packages/ui 的 MESSAGES 測試）——
// t(key) 缺 key 時靜默回傳 key 本身，單邊新增字串若無此測試會默默漏譯。
import { describe, it, expect } from "vitest";

import { APP_MESSAGES } from "../i18n/messages";

describe("APP_MESSAGES", () => {
  it("keeps the zh-TW and en dictionaries key-equal", () => {
    const zh = Object.keys(APP_MESSAGES["zh-TW"]).sort();
    const en = Object.keys(APP_MESSAGES.en).sort();
    expect(zh).toEqual(en);
    expect(zh.length).toBeGreaterThan(0);
  });
});

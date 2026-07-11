// spec 需求「看板搜尋過濾卡片」的比對層純函式（design D5/D7）：
// 篩選 chips 的 AND 交集、建立時間窗、名稱層 subsequence 模糊比對。
import { describe, it, expect } from "vitest";

import {
  matchesQuery,
  matchesFuzzy,
  matchesCreatedRange,
  matchesFilters,
  EMPTY_FILTERS,
  type BoardFilters,
} from "../search";

const TODAY = "2026-07-11";

describe("matchesFuzzy（名稱層 subsequence，design D7）", () => {
  it("查詢字元依序出現即命中（spec Example：etc → engine-typed-core）", () => {
    expect(matchesFuzzy("etc", "engine-typed-core")).toBe(true);
    expect(matchesFuzzy("dta", "desktop-acp-agent")).toBe(true);
  });

  it("順序不符或字元缺席不命中；不分大小寫；空 query 恆命中", () => {
    expect(matchesFuzzy("cte", "engine-typed-core")).toBe(false);
    expect(matchesFuzzy("xyz", "engine-typed-core")).toBe(false);
    expect(matchesFuzzy("ETC", "Engine-Typed-Core")).toBe(true);
    expect(matchesFuzzy("", "anything")).toBe(true);
    expect(matchesFuzzy("a", null)).toBe(false);
  });
});

describe("matchesCreatedRange（建立時間窗）", () => {
  it("近 7 天／近 30 天／更早 三窗互斥比對", () => {
    expect(matchesCreatedRange("2026-07-08", "7d", TODAY)).toBe(true);
    expect(matchesCreatedRange("2026-07-01", "7d", TODAY)).toBe(false);
    expect(matchesCreatedRange("2026-07-01", "30d", TODAY)).toBe(true);
    expect(matchesCreatedRange("2026-05-01", "30d", TODAY)).toBe(false);
    expect(matchesCreatedRange("2026-05-01", "earlier", TODAY)).toBe(true);
    expect(matchesCreatedRange("2026-07-08", "earlier", TODAY)).toBe(false);
  });

  it("未啟用恆命中；日期缺席時不命中任何啟用中的窗", () => {
    expect(matchesCreatedRange(null, null, TODAY)).toBe(true);
    expect(matchesCreatedRange(null, "7d", TODAY)).toBe(false);
    expect(matchesCreatedRange("not-a-date", "7d", TODAY)).toBe(false);
  });
});

describe("matchesFilters（chips AND 交集，design D5）", () => {
  const chg = {
    createdBy: "Momo <m@x>",
    created: "2026-07-10",
    fromDiscussions: ["collab"],
  };
  const disc = { createdBy: "Ann <a@x>", created: "2026-06-01", slug: "collab" };

  it("未啟用任何 chip 恆命中", () => {
    expect(matchesFilters(EMPTY_FILTERS, chg, TODAY)).toBe(true);
  });

  it("建立者全等比對", () => {
    const f: BoardFilters = { ...EMPTY_FILTERS, createdBy: "Momo <m@x>" };
    expect(matchesFilters(f, chg, TODAY)).toBe(true);
    expect(matchesFilters(f, disc, TODAY)).toBe(false);
  });

  it("來源討論：變更卡以 fromDiscussions 命中、討論卡以自身 slug 命中", () => {
    const f: BoardFilters = { ...EMPTY_FILTERS, fromDiscussion: "collab" };
    expect(matchesFilters(f, chg, TODAY)).toBe(true);
    expect(matchesFilters(f, disc, TODAY)).toBe(true);
    expect(matchesFilters(f, { slug: "other" }, TODAY)).toBe(false);
  });

  it("多 chip AND 交集：任一維度不符即整卡不命中", () => {
    const f: BoardFilters = {
      createdBy: "Momo <m@x>",
      createdWithin: "7d",
      fromDiscussion: "collab",
    };
    expect(matchesFilters(f, chg, TODAY)).toBe(true);
    expect(matchesFilters(f, { ...chg, created: "2026-05-01" }, TODAY)).toBe(false);
    expect(matchesFilters(f, { ...chg, createdBy: "Ann <a@x>" }, TODAY)).toBe(false);
  });
});

describe("matchesQuery（既有子字串規則不回歸）", () => {
  it("去頭尾空白、不分大小寫、子字串命中任一欄位", () => {
    expect(matchesQuery(" GUI ", "GUI 勾任務")).toBe(true);
    expect(matchesQuery("desk", "desktop-acp-agent", null)).toBe(true);
    expect(matchesQuery("", "anything")).toBe(true);
    expect(matchesQuery("zzz", "a", "b")).toBe(false);
  });
});

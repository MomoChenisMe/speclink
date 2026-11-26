// spec 需求「看板搜尋過濾卡片」的比對層純函式（design D5/D7）：
// 篩選 chips 的 AND 交集、建立時間窗、名稱層 subsequence 模糊比對。
import { describe, it, expect } from "vitest";

import {
  matchesQuery,
  matchesFuzzy,
  matchesCreatedRange,
  matchesFilters,
  creatorOwnership,
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
    const f: BoardFilters = { ...EMPTY_FILTERS, creator: { name: "Momo <m@x>" } };
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
      creator: { name: "Momo <m@x>" },
      createdWithin: "7d",
      fromDiscussion: "collab",
    };
    expect(matchesFilters(f, chg, TODAY)).toBe(true);
    expect(matchesFilters(f, { ...chg, created: "2026-05-01" }, TODAY)).toBe(false);
    expect(matchesFilters(f, { ...chg, createdBy: "Ann <a@x>" }, TODAY)).toBe(false);
  });
});

// spec「看板搜尋過濾卡片」Scenario「建立者篩選以識別認人」Example「卡片的歸屬與選項」
// 與 Scenario「本機專案的建立者篩選維持名稱比對」（design D2）。
describe("creatorOwnership（建立者以人為單位）", () => {
  type Card = { id: string; createdBy?: string | null; createdById?: string | null };
  /** 選單的標籤，與選第一個選項時留下的卡。 */
  function run(cards: Card[]) {
    const own = creatorOwnership(cards);
    const f: BoardFilters = { ...EMPTY_FILTERS, creator: own.options[0].key };
    const kept = cards
      .filter((c) => matchesFilters(f, { ...c, creator: own.keyOf(c) }, TODAY))
      .map((c) => c.id);
    return { labels: own.options.map((o) => o.label), kept };
  }

  it.each([
    {
      cards: [
        { id: "A", createdBy: "Ann", createdById: "u_1" },
        { id: "B", createdBy: "Ann Lee", createdById: "u_1" },
      ],
      labels: ["Ann"],
      kept: ["A", "B"],
    },
    {
      cards: [
        { id: "A", createdBy: "Ann", createdById: "u_1" },
        { id: "C", createdBy: "Ann" },
      ],
      labels: ["Ann"],
      kept: ["A", "C"],
    },
    {
      cards: [
        { id: "A", createdBy: "Ann", createdById: "u_1" },
        { id: "D", createdBy: "Ann", createdById: "u_2" },
        { id: "C", createdBy: "Ann" },
      ],
      labels: ["Ann (u_1)", "Ann (u_2)", "Ann"],
      kept: ["A"],
    },
    {
      cards: [
        { id: "E", createdBy: "Bob" },
        { id: "F", createdBy: "Bob" },
      ],
      labels: ["Bob"],
      kept: ["E", "F"],
    },
  ])("$cards.length 張卡 → 選項 $labels、留下 $kept", ({ cards, labels, kept }) => {
    expect(run(cards)).toEqual({ labels, kept });
  });

  it("同名的標籤只接識別的最後 4 個字元", () => {
    const own = creatorOwnership([
      { createdBy: "Ann", createdById: "user_01ab9f" },
      { createdBy: "Ann", createdById: "user_01cd3e" },
    ]);
    expect(own.options.map((o) => o.label)).toEqual(["Ann (ab9f)", "Ann (cd3e)"]);
  });

  it("本機專案沒有識別：選項是名稱、選定後只留名稱全等的卡", () => {
    const cards: Card[] = [
      { id: "1", createdBy: "Bob <bob@example.com>" },
      { id: "2", createdBy: "Ann <ann@example.com>" },
      { id: "3", createdBy: "Bob <bob@example.com>" },
    ];
    const own = creatorOwnership(cards);
    expect(own.options).toEqual([
      { key: { name: "Bob <bob@example.com>" }, label: "Bob <bob@example.com>" },
      { key: { name: "Ann <ann@example.com>" }, label: "Ann <ann@example.com>" },
    ]);
    expect(run(cards).kept).toEqual(["1", "3"]);
  });

  it("識別與名稱都沒有的卡沒有歸屬，選了任何建立者都不留下", () => {
    const own = creatorOwnership([{ createdBy: "Ann" }, {}]);
    expect(own.keyOf({})).toBeNull();
    const f: BoardFilters = { ...EMPTY_FILTERS, creator: own.options[0].key };
    expect(matchesFilters(f, { creator: own.keyOf({}) }, TODAY)).toBe(false);
  });

  it("同名而且識別的最後 4 個字元也相同時，標籤接完整識別", () => {
    const own = creatorOwnership([
      { createdBy: "Ann", createdById: "org1_abcd" },
      { createdBy: "Ann", createdById: "org2_abcd" },
    ]);
    expect(own.options.map((o) => o.label)).toEqual(["Ann (org1_abcd)", "Ann (org2_abcd)"]);
  });

  it("名稱長得像識別的人和那個識別分開：識別與名稱在結構上不同", () => {
    const cards: Card[] = [
      { id: "1", createdBy: "Bob", createdById: "u_1" },
      { id: "2", createdBy: "u_1" },
    ];
    const own = creatorOwnership(cards);
    expect(own.options).toEqual([
      { key: { id: "u_1" }, label: "Bob" },
      { key: { name: "u_1" }, label: "u_1" },
    ]);
    const f: BoardFilters = { ...EMPTY_FILTERS, creator: own.options[1].key };
    expect(cards.filter((c) => matchesFilters(f, { ...c, creator: own.keyOf(c) }, TODAY)).map((c) => c.id)).toEqual(["2"]);
  });

  // 審查 Round 1 的必修：重載後歸屬鍵可能改變，選定的人要跟著走。
  describe("resolve：重載後找回選定的人", () => {
    const C = { createdBy: "Ann" };
    const A = { createdBy: "Ann", createdById: "u_1" };
    const D = { createdBy: "Ann", createdById: "u_2" };

    it("選定的名稱在重載後併進一個識別時，對到那個識別", () => {
      const before = creatorOwnership([C]);
      expect(before.options[0].key).toEqual({ name: "Ann" });
      expect(creatorOwnership([C, A]).resolve({ name: "Ann" })).toEqual({ id: "u_1" });
    });

    it("選定的識別仍在選項裡時不變；同名多出第二個識別時照樣是那個識別", () => {
      expect(creatorOwnership([A, C]).resolve({ id: "u_1" })).toEqual({ id: "u_1" });
      expect(creatorOwnership([A, D, C]).resolve({ id: "u_1" })).toEqual({ id: "u_1" });
      expect(creatorOwnership([A, D, C]).resolve({ name: "Ann" })).toEqual({ name: "Ann" });
    });

    it("選定的人已經不在看板上時回 null", () => {
      expect(creatorOwnership([A]).resolve({ id: "u_9" })).toBeNull();
      expect(creatorOwnership([A]).resolve({ name: "Zed" })).toBeNull();
    });
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

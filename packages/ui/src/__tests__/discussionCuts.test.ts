// 刀卡的組成（discussion-cuts-from-engine design「前端以列點原文把刀對回細節列」）：刀由引擎解析，前端只以
// `line` 把刀對回決定的細節列，並只讀引擎欄位判定轉出。
import { describe, expect, it } from "vitest";

import { cutCards, cutPromotion, splitVerdict, type CutCard } from "../discussionModel";
import { engineCut } from "./engineCuts";

const DECISION = [
  "定論。",
  "- 一般細節",
  "- **cut 1 `a-one`**: 骨架——說明",
  "  續行散文一句。",
  "  - 子項",
  "- **cut 2 `b-two`**: 外殼",
].join("\n");

const CUT_1 = engineCut("**cut 1 `a-one`**: 骨架——說明", {
  cut: "cut 1",
  title: "骨架",
  note: "說明",
  changes: ["a-one"],
  changesInBold: true,
});
const CUT_2 = engineCut("**cut 2 `b-two`**: 外殼", {
  cut: "cut 2",
  title: "外殼",
  changes: ["b-two"],
  changesInBold: true,
  unpromoted: ["b-two"],
});

describe("cutCards（以列點原文把刀對回細節列）", () => {
  it("依 cuts 順序認領 text 等於 line 的細節列，認領到的列自 rest 移除", () => {
    const { cuts, rest } = cutCards(splitVerdict(DECISION).details, [CUT_2, CUT_1]);
    expect(cuts.map((c) => c.id)).toEqual(["cut 2", "cut 1"]);
    expect(rest.map((r) => r.text)).toEqual(["一般細節"]);
    expect(cuts[1]).toEqual<CutCard>({
      id: "cut 1",
      title: "骨架",
      names: ["a-one"],
      namesInBold: true,
      unpromoted: [],
      items: ["說明", "續行散文一句。", "子項"],
    });
  });

  it("items 為 note 接巢狀子項與續行散文的細線列；note 為空時只有巢狀內容", () => {
    const { cuts } = cutCards(splitVerdict(DECISION).details, [CUT_1, CUT_2]);
    expect(cuts[0].items).toEqual(["說明", "續行散文一句。", "子項"]);
    expect(cuts[1].items).toEqual([]);
  });

  it("同一首行出現兩次時各認領一列", () => {
    const md = ["定論。", "- **cut 1 `a`**: 同名", "  - 第一列的子項", "- **cut 1 `a`**: 同名", "  - 第二列的子項"].join("\n");
    const twin = engineCut("**cut 1 `a`**: 同名", { cut: "cut 1", title: "同名", changes: ["a"], changesInBold: true });
    const { cuts, rest } = cutCards(splitVerdict(md).details, [twin, twin]);
    expect(cuts.map((c) => c.items)).toEqual([["第一列的子項"], ["第二列的子項"]]);
    expect(rest).toEqual([]);
  });

  it("找不到對應列的刀仍成卡，items 只有 note；細節列不動", () => {
    const lost = engineCut("**cut 9 `z`**: 不在這裡 <!-- 註解 -->", { cut: "cut 9", title: "不在這裡", note: "說明" });
    const { cuts, rest } = cutCards(splitVerdict(DECISION).details, [lost]);
    expect(cuts).toHaveLength(1);
    expect(cuts[0].items).toEqual(["說明"]);
    expect(rest).toHaveLength(3);
  });

  it("cuts 為空時沒有刀卡，細節列全部留在 rest", () => {
    const details = splitVerdict(DECISION).details;
    expect(cutCards(details, [])).toEqual({ cuts: [], rest: details });
  });

  it("以 `*` 起頭的刀列不是引擎的刀，留在 rest", () => {
    const { cuts, rest } = cutCards(splitVerdict("定論。\n* **cut 3 `add-x`**: 範圍").details, []);
    expect(cuts).toEqual([]);
    expect(rest.map((r) => r.text)).toEqual(["**cut 3 `add-x`**: 範圍"]);
  });
});

// desktop-app「刀的轉出判定」Example 逐列一案：只讀引擎的 changes、changesInBold、unpromoted。
describe("cutPromotion（刀的轉出判定）", () => {
  const card = (names: string[], namesInBold: boolean, unpromoted: string[]): CutCard => ({
    id: "cut 1",
    title: "骨架",
    names,
    namesInBold,
    unpromoted,
    items: [],
  });

  it.each<[string, CutCard, string]>([
    ["a、b 皆在 promoted_to", card(["a", "b"], true, []), "promoted"],
    ["a、b 只有 a 在 promoted_to", card(["a", "b"], true, ["b"]), "partial"],
    ["a 不在 promoted_to", card(["a"], true, ["a"]), "none"],
    ["沒有變更名籤", card([], false, []), "unknown"],
    ["粗體內沒有、粗體之後有 KeyLines、util.rs", card(["KeyLines", "util.rs"], false, []), "unknown"],
  ])("%s → %s", (_row, cut, verdict) => {
    expect(cutPromotion(cut)).toBe(verdict);
  });
});

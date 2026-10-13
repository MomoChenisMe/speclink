// 討論記錄解析器（desktop-discussion-doc-views design D1）：純函式，形狀不符時逐項降級（回 null、空陣列或整條
// 當一般列），不拋錯。案例取自 spec Example 與 detail-content-redesign、desktop-ui-redesign 兩份真實結論。
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { describe, expect, it } from "vitest";

import { listItemsOf, parseListItems } from "../docModel";
import {
  captureTargets,
  commandChips,
  contextRelations,
  countPriorDiscussions,
  matchNames,
  openItems,
  parseConclusion,
  parseContextMeta,
  parseCuts,
  roundStats,
  splitDiscussionSections,
  splitLead,
  splitOption,
  splitRounds,
  splitRows,
  splitVerdict,
} from "../discussionModel";

const DOC = `---
topic: Alpha search
slug: alpha-search
status: concluded
created: 2026-07-01
---

# Discussion: Alpha search

## Context

框架脈絡內容。

## Rounds

### Round 1 — assumptions (2026-07-01)

**Focus**: 範圍界定

## Conclusion

**Decision**: 建置 alpha 搜尋
`;

describe("splitDiscussionSections（區段切分）", () => {
  it("切出脈絡/回合/結論三區段", () => {
    const s = splitDiscussionSections(DOC);
    expect(s).not.toBeNull();
    expect(s!.context).toContain("框架脈絡內容");
    expect(s!.rounds).toContain("範圍界定");
    expect(s!.conclusion).toContain("建置 alpha 搜尋");
  });

  it("非預期格式（缺區段）回 null → 整篇退回", () => {
    expect(splitDiscussionSections("手寫的自由格式記錄，沒有標準區段。")).toBeNull();
    expect(splitDiscussionSections("## Context\n\n只有脈絡。\n")).toBeNull();
  });

  it("輪內文含「## 」開頭行時 rounds 區段完整涵蓋至結構 Conclusion、內文不遺失", () => {
    // spec「區段切分不被輪內文截斷」：與引擎同一結構標題白名單。
    const doc = DOC.replace("**Focus**: 範圍界定", "**Focus**: 範圍界定\n\n## 背景\n首輪本文");
    const s = splitDiscussionSections(doc);
    expect(s).not.toBeNull();
    expect(s!.rounds).toContain("## 背景");
    expect(s!.rounds).toContain("首輪本文");
    expect(s!.conclusion).toContain("建置 alpha 搜尋");
  });

  it("圍欄內的結構標題行不切斷區段、不誤當區段起點", () => {
    // 與引擎同規則：fenced code block 內的行不是結構。
    const doc = DOC.replace("**Focus**: 範圍界定", "**Focus**: 範圍界定\n\n```\n## Conclusion\n```");
    const s = splitDiscussionSections(doc);
    expect(s).not.toBeNull();
    expect(s!.rounds).toContain("```\n## Conclusion\n```");
    expect(s!.conclusion).toContain("建置 alpha 搜尋");
    expect(s!.conclusion).not.toContain("```");
  });

  it("結論區段不吞其後的 pre-scaffold「## Round 」區段", () => {
    // 與引擎同規則：未跳脫的「## Round 」前綴行是區段邊界（pre-scaffold 容忍）。
    const doc = `${DOC}\n## Round 2 — explore (2026-07-02)\n\n次輪本文\n`;
    const s = splitDiscussionSections(doc);
    expect(s).not.toBeNull();
    expect(s!.conclusion).toContain("建置 alpha 搜尋");
    expect(s!.conclusion).not.toContain("次輪本文");
  });
});

describe("結構標題白名單跨語言對齊（引擎 ↔ UI）", () => {
  const REPO_ROOT = () => resolve(dirname(expect.getState().testPath ?? ""), "../../../..");
  const tsSource = () => readFileSync(resolve(REPO_ROOT(), "packages/ui/src/discussionModel.ts"), "utf8");
  const rustSource = () =>
    readFileSync(resolve(REPO_ROOT(), "crates/engine/speclink-core/src/lifecycle/discuss.rs"), "utf8");

  it("Rust 與 TS 的 STRUCTURAL_HEADERS 逐值一致", () => {
    // 兩份白名單只靠註解互指不夠——讀原始檔逐值比對（同 tooltipDelay.test.tsx
    // 的檔案系統斷言慣例；repo root 由測試檔路徑推出）。
    const extract = (source: string): string[] => {
      const m = /const STRUCTURAL_HEADERS[^=]*=[^[]*\[([^\]]+)\]/.exec(source);
      expect(m).not.toBeNull();
      return [...m![1].matchAll(/"([^"]+)"/g)].map((x) => x[1]);
    };
    const rust = extract(rustSource());
    expect(extract(tsSource())).toEqual(rust);
    expect(rust).toEqual(["## Context", "## Rounds", "## Conclusion"]);
  });

  it("圍欄、輪標題與 pre-scaffold 邊界的字面規則兩邊同在", () => {
    // 絆線測試：三條跨語言規則（圍欄定界符、輪標題前綴與 em-dash 分隔、
    // pre-scaffold 邊界前綴）的字面值必須同時存在於兩份原始檔——
    // 任一側改了定界符或前綴，這裡就斷。
    const rustSrc = rustSource();
    const tsSrc = tsSource();
    for (const literal of ["## Round ", "### Round ", "```", "~~~", " — "]) {
      expect(rustSrc, `Rust 缺 ${JSON.stringify(literal)}`).toContain(literal);
      expect(tsSrc, `TS 缺 ${JSON.stringify(literal)}`).toContain(literal);
    }
  });
});

// spec 需求「討論輪以卡片呈現」的輪切分。
const ROUNDS_TEXT = `
<!-- \`### Round N — <mode> (<date>)\` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-08)

**Focus**: 第一輪焦點
**Position**: 總綰一句：
- 列點甲
- 列點乙
**Open**: 未解之一

### Round 2 — interview (2026-07-09)

**Focus**: 第二輪焦點
**Position**: 直答
**Note**: 這行不是欄位
**Ruled out**: 淘汰項
**Open**: 無
`;

describe("splitRounds（輪切分，行掃描解析 scaffold）", () => {
  it("scaffold 記錄解析出輪陣列：輪次、mode、日期、欄位對應", () => {
    const rounds = splitRounds(ROUNDS_TEXT);
    expect(rounds).not.toBeNull();
    expect(rounds!.length).toBe(2);
    expect(rounds![0].round).toBe(1);
    expect(rounds![0].mode).toBe("assumptions");
    expect(rounds![0].date).toBe("2026-07-08");
    expect(rounds![1].round).toBe(2);
    expect(rounds![1].mode).toBe("interview");
    expect(rounds![1].date).toBe("2026-07-09");
    expect(rounds![0].fields.Focus).toBe("第一輪焦點");
  });

  it("Position 標籤行後的列點多行歸屬 Position 欄位", () => {
    const rounds = splitRounds(ROUNDS_TEXT)!;
    expect(rounds[0].fields.Position).toContain("總綰一句");
    expect(rounds[0].fields.Position).toContain("- 列點甲");
    expect(rounds[0].fields.Position).toContain("- 列點乙");
    expect(rounds[0].fields.Open).toBe("未解之一");
  });

  it("來源缺 Ruled out 時欄位對應無該鍵", () => {
    const rounds = splitRounds(ROUNDS_TEXT)!;
    expect(rounds[0].fields["Ruled out"]).toBeUndefined();
    expect(rounds[1].fields["Ruled out"]).toBe("淘汰項");
  });

  it("非四詞白名單的粗體前綴行按內文歸屬當前欄位", () => {
    const rounds = splitRounds(ROUNDS_TEXT)!;
    expect(Object.keys(rounds[1].fields)).not.toContain("Note");
    expect(rounds[1].fields.Position).toContain("**Note**: 這行不是欄位");
  });

  it("圍欄內的 ### 行不觸發輪切分或整篇退回", () => {
    // 與引擎同規則：fenced code block 內的行歸當前輪內文，不是輪標題。
    const text =
      "\n### Round 1 — assumptions (2026-07-08)\n\n**Focus**: 焦點\n\n```\n### 附註\n### Round 9 — fake (2026-01-01)\n```\n";
    const rounds = splitRounds(text);
    expect(rounds).not.toBeNull();
    expect(rounds!.length).toBe(1);
    expect(rounds![0].fields.Focus).toContain("```\n### 附註\n### Round 9 — fake (2026-01-01)\n```");
  });

  it("任一輪標題不符 scaffold 格式時回 null（整篇退回）", () => {
    expect(splitRounds("### Round 1 — assumptions\n\n**Focus**: 缺日期括號\n")).toBeNull();
    expect(
      splitRounds("### Round 1 — assumptions (2026-07-08)\n\n**Focus**: 好輪\n\n### 附註\n\n手寫段落\n"),
    ).toBeNull();
    expect(splitRounds("手寫的自由格式輪記錄，沒有輪標題。")).toBeNull();
  });

  it("零輪（僅 scaffold 註解與空行）回空陣列", () => {
    expect(
      splitRounds("\n<!-- \\`### Round N — <mode> (<date>)\\` entries are appended here by the CLI. -->\n\n"),
    ).toEqual([]);
  });
});

describe("countPriorDiscussions", () => {
  it.each([
    ["Prior discussions: none", null],
    ["Prior discussions: None.", null],
    ["Prior discussions: none — 首次討論這個主題", null],
    ["Prior discussions: NONE（尚無）", null],
    ["Prior discussions: none-legacy-topic", 1],
    ["Prior discussions: alpha-one, beta-two", 2],
    // 與背景的先前討論籤同一套切法：分號也切項、重複名只算一次、括號內的頓號不算。
    ["Prior discussions: alpha-one（甲、乙）；beta-two。", 2],
    ["Prior discussions: alpha-one, alpha-one", 1],
  ])("countPriorDiscussions(%s) → %s：none 開頭的變體寫法都算沒有先前討論", (line, expected) => {
    expect(countPriorDiscussions(`框架脈絡。\n\n${line}`)).toBe(expected);
  });
});

// detail-content-redesign 結論的節錄（形狀照真實記錄：一句定論、一般細節、三條帶巢狀子項的分刀列）。
const DCR_CONCLUSION = [
  "**Decision**: 變更、討論、已封存三種詳情頁的內容呈現改成手冊式三欄閱讀頁，原始 Markdown 格式與模板不動，分三刀落地。",
  "- 範圍與驗收：只改 UI 渲染",
  "- 版面：左欄 240px 文件樹",
  "- **cut 1 `desktop-detail-doc-tree`**: 骨架——三種詳情頁改三欄，九份文件先以現有渲染器落進新骨架",
  "  - 文件樹欄（身分區、文件節點、狀態區）",
  "  - 分頁列退場；已封存詳情頁同步",
  "- **cut 2 `desktop-change-doc-views`**: 變更文件渲染器——提案、設計的新畫法；依賴 cut 1",
  "  - 章節白名單與逐項降級規則落在這一刀",
  "- **cut 3 `desktop-discussion-doc-views`**: 討論文件渲染器——結論分刀卡；最後一刀",
  "  - desktop-app 討論輪、討論結論、討論詳情頁需求 MODIFIED",
  "**Rationale**: 手冊頁已經證明三欄閱讀頁成立。",
  "**Rejected alternatives**:",
  "- 改 Markdown 格式或模板來遷就呈現 — 使用者裁定格式不動，只改 UI",
  "- 一刀落地 — 約 15 條需求",
  "**Deferred**:",
  "- 引擎在 discuss show --json 回拆好的 rounds — 等有第二個消費者再做",
  "- 手冊頁要不要同步加數字磚",
  "**Capture to**: proposal、design、spec（desktop-app MODIFIED）、tasks、LANGUAGE.md",
  "**Next**: /speclink-propose --from-discussion detail-content-redesign（先建 cut 1 `desktop-detail-doc-tree`；cut 3 帶 --last）",
].join("\n");

// desktop-ui-redesign 結論的分刀列節錄：cut 2／3a 把變更名寫在粗體之後的括號裡、粗體內另有範圍詞。
const UIR_DECISION = [
  "整個介面重新設計，分七刀落地。",
  "- 底層不換：React 19＋Tailwind v4",
  "- **cut 0 `fix-drawer-open-dismisses-overlays`**: 獨立 bug 修，不等換皮（已封存）",
  "  - openDetail 清掉可取消浮層",
  "- **cut 2 外殼**（propose 期拆三個 change：`desktop-shell-redesign` 外殼結構、`desktop-notice-relocation` 提示歸位（依賴前者）、`desktop-native-menu` macOS 原生選單（獨立）；三者皆已轉出）",
  "  - 三平台標題列",
  "  - 新原語 dropdown-menu、kbd、page-header",
  "- **cut 3a 看板**（propose 期拆兩個 change：`desktop-board-reskin` 欄與卡、`desktop-list-pages-reskin` 規格／已封存／手冊三頁；皆依賴 desktop-shell-redesign；兩者皆已轉出）",
  "- **cut 5 `tray-panel-restyle`**: 系統匣面板，可與 3a、3b、4 並行",
].join("\n");

describe("parseConclusion", () => {
  it("六詞白名單切出六個欄位；第一個標籤前無內文時 lead 為空", () => {
    const c = parseConclusion(DCR_CONCLUSION);
    expect(c).not.toBeNull();
    expect(Object.keys(c!.fields)).toEqual([
      "Decision",
      "Rationale",
      "Rejected alternatives",
      "Deferred",
      "Capture to",
      "Next",
    ]);
    expect(c!.lead).toBe("");
    expect(c!.fields.Rationale).toBe("手冊頁已經證明三欄閱讀頁成立。");
  });

  it("第一個標籤前的內文歸 lead", () => {
    const c = parseConclusion("前言一段。\n**Decision**: 拍板");
    expect(c).toEqual({ lead: "前言一段。", fields: { Decision: "拍板" } });
  });

  it("零個白名單欄位（手寫自由格式）回 null", () => {
    expect(parseConclusion("就這樣定了，大家都同意這個方向。")).toBeNull();
    expect(parseConclusion("**決定**：中文標籤不在白名單")).toBeNull();
  });
});

describe("splitVerdict", () => {
  it("第一段散文為定論句、頂層列點依序為細節（巢狀子項留在 body）", () => {
    const v = splitVerdict(parseConclusion(DCR_CONCLUSION)!.fields.Decision!);
    expect(v.verdict).toBe(
      "變更、討論、已封存三種詳情頁的內容呈現改成手冊式三欄閱讀頁，原始 Markdown 格式與模板不動，分三刀落地。",
    );
    expect(v.details.map((d) => d.text)).toEqual([
      "範圍與驗收：只改 UI 渲染",
      "版面：左欄 240px 文件樹",
      "**cut 1 `desktop-detail-doc-tree`**: 骨架——三種詳情頁改三欄，九份文件先以現有渲染器落進新骨架",
      "**cut 2 `desktop-change-doc-views`**: 變更文件渲染器——提案、設計的新畫法；依賴 cut 1",
      "**cut 3 `desktop-discussion-doc-views`**: 討論文件渲染器——結論分刀卡；最後一刀",
    ]);
    expect(v.details[2].body).toBe("- 文件樹欄（身分區、文件節點、狀態區）\n- 分頁列退場；已封存詳情頁同步");
  });

  it("首段即列點時定論句為空", () => {
    const v = splitVerdict("- 甲\n- 乙");
    expect(v.verdict).toBe("");
    expect(v.details.map((d) => d.text)).toEqual(["甲", "乙"]);
  });

  it("只有一句時沒有細節", () => {
    expect(splitVerdict("只有一句定論。")).toEqual({ verdict: "只有一句定論。", details: [] });
  });
});

// spec Example「分刀列的解析」逐列一案。
describe("parseCuts", () => {
  const one = (line: string) => parseCuts(listItemsOf(parseListItems(`- ${line}`)));

  it.each<[string, string, string[], string]>([
    ["**cut 1 `desktop-detail-doc-tree`**: 骨架——三種詳情頁改三欄", "cut 1", ["desktop-detail-doc-tree"], "骨架"],
    [
      "**cut 2 `desktop-shell-redesign` `desktop-notice-relocation`**: 外殼 7 項",
      "cut 2",
      ["desktop-shell-redesign", "desktop-notice-relocation"],
      "外殼 7 項",
    ],
    ["**刀三**：系統匣面板", "刀三", [], "系統匣面板"],
    [
      "**cut 2 外殼**（propose 期拆兩個 change：`desktop-shell-redesign` 外殼結構、`desktop-native-menu` 原生選單）",
      "cut 2",
      ["desktop-shell-redesign", "desktop-native-menu"],
      "外殼",
    ],
    ["**第三刀**：系統匣", "第三刀", [], "系統匣"],
    ["**Cut 3a `x-y`**", "Cut 3a", ["x-y"], ""],
  ])("%s → 識別籤 %s、變更名籤 %j、範圍句 %s", (line, id, names, title) => {
    const { cuts, rest } = one(line);
    expect(rest).toEqual([]);
    expect(cuts).toHaveLength(1);
    expect(cuts[0]).toMatchObject({ id, names, title });
  });

  it.each([
    "**範圍與驗收**：只改 UI 渲染",
    "**cutover 計畫**：切換步驟",
    "**Cut scope**：範圍說明",
    "**`desktop-app`**: 粗體內只有反引號",
    "cut 1 `x`：沒有粗體",
  ])("%s 不是分刀，留在定案細節", (line) => {
    const { cuts, rest } = one(line);
    expect(cuts).toEqual([]);
    expect(rest).toHaveLength(1);
  });

  it("「——」之後的文字成為卡內第一條細線列，其後接巢狀子項", () => {
    const details = splitVerdict(parseConclusion(DCR_CONCLUSION)!.fields.Decision!).details;
    const { cuts, rest } = parseCuts(details);
    expect(rest.map((r) => r.text)).toEqual(["範圍與驗收：只改 UI 渲染", "版面：左欄 240px 文件樹"]);
    expect(cuts.map((c) => [c.id, c.title, c.names])).toEqual([
      ["cut 1", "骨架", ["desktop-detail-doc-tree"]],
      ["cut 2", "變更文件渲染器", ["desktop-change-doc-views"]],
      ["cut 3", "討論文件渲染器", ["desktop-discussion-doc-views"]],
    ]);
    expect(cuts[0].items).toEqual([
      "三種詳情頁改三欄，九份文件先以現有渲染器落進新骨架",
      "文件樹欄（身分區、文件節點、狀態區）",
      "分頁列退場；已封存詳情頁同步",
    ]);
  });

  it("粗體內有範圍詞時，粗體之後的括號說明（拿掉外圍括號）成為第一條細線列、變更名取自括號內", () => {
    const { cuts, rest } = parseCuts(splitVerdict(UIR_DECISION).details);
    expect(rest.map((r) => r.text)).toEqual(["底層不換：React 19＋Tailwind v4"]);
    expect(cuts.map((c) => [c.id, c.title, c.names])).toEqual([
      ["cut 0", "獨立 bug 修，不等換皮（已封存）", ["fix-drawer-open-dismisses-overlays"]],
      ["cut 2", "外殼", ["desktop-shell-redesign", "desktop-notice-relocation", "desktop-native-menu"]],
      ["cut 3a", "看板", ["desktop-board-reskin", "desktop-list-pages-reskin"]],
      ["cut 5", "系統匣面板，可與 3a、3b、4 並行", ["tray-panel-restyle"]],
    ]);
    expect(cuts[0].items).toEqual(["openDetail 清掉可取消浮層"]);
    expect(cuts[1].items).toEqual([
      "propose 期拆三個 change：`desktop-shell-redesign` 外殼結構、`desktop-notice-relocation` 提示歸位（依賴前者）、`desktop-native-menu` macOS 原生選單（獨立）；三者皆已轉出",
      "三平台標題列",
      "新原語 dropdown-menu、kbd、page-header",
    ]);
  });

  it("分刀列的縮排續行散文與巢狀子項依序成為卡內細線列，不丟內容", () => {
    const { cuts } = parseCuts(
      splitVerdict(["定論。", "- **cut 1 `a-one`**: 骨架——說明", "  續行散文一句。", "  - 子項"].join("\n")).details,
    );
    expect(cuts[0].items).toEqual(["說明", "續行散文一句。", "子項"]);
  });

  it("含空白的反引號片段不算變更名", () => {
    const { cuts } = one("**cut 4 設定**（先跑 `speclink plan`，再建 `desktop-settings-mode`）");
    expect(cuts[0].names).toEqual(["desktop-settings-mode"]);
  });
});

describe("splitOption（否決與擱置的方案／理由）", () => {
  it.each<[string, string, string | null]>([
    ["改 Markdown 格式或模板來遷就呈現 — 使用者裁定格式不動，只改 UI", "改 Markdown 格式或模板來遷就呈現", "使用者裁定格式不動，只改 UI"],
    ["A——理由甲", "A", "理由甲"],
    ["維持頂部橫幅提示 — 看板被推下去——且很遠", "維持頂部橫幅提示", "看板被推下去——且很遠"],
    ["前者——後者 — 再後", "前者", "後者 — 再後"],
    ["desktop-ui—redesign 沒有空白的破折號不切", "desktop-ui—redesign 沒有空白的破折號不切", null],
    ["手冊頁要不要同步加數字磚", "手冊頁要不要同步加數字磚", null],
  ])("%s", (text, option, reason) => {
    expect(splitOption(text)).toEqual({ option, reason });
  });
});

describe("splitRows", () => {
  it("內文有列點時取頂層列點（巢狀子項跟著該列）", () => {
    expect(splitRows("- 甲\n- 乙\n  - 子")).toEqual(["甲", "乙\n- 子"]);
  });

  it("沒有列點時以頂層的「；」切段，反引號與括號內的「；」不切", () => {
    expect(splitRows("A——理由甲；B——理由乙")).toEqual(["A——理由甲", "B——理由乙"]);
    expect(splitRows("`a；b`；c（d；e）;f； ")).toEqual(["`a；b`", "c（d；e）", "f"]);
  });

  it("空內文為空陣列", () => {
    expect(splitRows("  ")).toEqual([]);
  });
});

describe("captureTargets", () => {
  it("頂層的頓號與逗號切項，括號內不切、反引號拿掉", () => {
    expect(captureTargets("proposal、design、spec（desktop-app、desktop-config MODIFIED）, `tasks`，LANGUAGE.md")).toEqual([
      "proposal",
      "design",
      "spec（desktop-app、desktop-config MODIFIED）",
      "tasks",
      "LANGUAGE.md",
    ]);
  });

  it("整段散文（含句號或換行）回 null", () => {
    expect(captureTargets("寫進設計文件。")).toBeNull();
    expect(captureTargets("proposal\n另起一行")).toBeNull();
    expect(captureTargets("  ")).toBeNull();
  });
});

describe("openItems", () => {
  it.each(["無——寫結論", "無：寫入結論", "無", "None", "none.", "  無——跑 ingest  "])("%s 為收斂標記", (md) => {
    expect(openItems(md)).toEqual({ none: true, items: [md.trim()] });
  });

  it("「無法…」與 none-legacy 不算收斂標記", () => {
    expect(openItems("無法決定；切刀")).toEqual({ none: false, items: ["無法決定", "切刀"] });
    expect(openItems("none-legacy 題目")).toEqual({ none: false, items: ["none-legacy 題目"] });
  });

  it("其餘逐條切開", () => {
    expect(openItems("點綴色留 teal 還是換紫；切刀")).toEqual({
      none: false,
      items: ["點綴色留 teal 還是換紫", "切刀"],
    });
  });
});

describe("roundStats", () => {
  it("mode 依首次出現序計數、天數為不同日期數、首末日", () => {
    const rounds = splitRounds(
      [
        "### Round 1 — assumptions (2026-10-08)",
        "### Round 2 — interview (2026-10-08)",
        "### Round 3 — assumptions (2026-10-09)",
        "### Round 4 — explore (2026-10-11)",
      ].join("\n\n"),
    )!;
    expect(roundStats(rounds)).toEqual({
      modes: [
        { mode: "assumptions", count: 2 },
        { mode: "interview", count: 1 },
        { mode: "explore", count: 1 },
      ],
      days: 3,
      first: "2026-10-08",
      last: "2026-10-11",
    });
  });

  it("零輪", () => {
    expect(roundStats([])).toEqual({ modes: [], days: 0, first: null, last: null });
  });
});

// spec Scenario「背景文件的引言、關聯與記錄列」的收尾四行。
const CONTEXT = [
  "起因段落，提到 desktop-app 與 tray-status-menu。",
  "",
  "第二段提到 add-copilot-tool 與 tray-panel-card-design。",
  "",
  "Related specs: desktop-app（79 條）、tray-status-menu",
  "Related changes: add-copilot-tool: desktop-config, tray-panel-card-design",
  "Prior discussions: card-drawer-header-colors, admin-console-redesign",
  "Source doc: none",
].join("\n");

describe("parseContextMeta", () => {
  it("四行收尾行剝離、名單去括號備註並取冒號前、Source doc none 為空字串", () => {
    expect(parseContextMeta(CONTEXT)).toEqual({
      specs: ["desktop-app", "tray-status-menu"],
      changes: ["add-copilot-tool", "tray-panel-card-design"],
      prior: ["card-drawer-header-colors", "admin-console-redesign"],
      sourceDoc: "",
      body: "起因段落，提到 desktop-app 與 tray-status-menu。\n\n第二段提到 add-copilot-tool 與 tray-panel-card-design。",
    });
  });

  it("行缺席為 null；none 起頭為空陣列；Source doc 其餘為原文；分號也切項、重複名去重", () => {
    const meta = parseContextMeta(
      "內文。\nPrior discussions: None.\n  Related specs: desktop-app (79 reqs); tray-status-menu、desktop-app\nSource doc: docs/brief.md \n",
    );
    expect(meta).toEqual({
      specs: ["desktop-app", "tray-status-menu"],
      changes: null,
      prior: [],
      sourceDoc: "docs/brief.md",
      body: "內文。",
    });
  });

  it("名稱止於括號、句尾標點拿掉、英數段後緊接的中文不算進名稱（真實記錄 spec-manual-page-alignment）", () => {
    const meta = parseContextMeta(
      [
        "內文。",
        "Related specs: desktop-app（桌面 app 呈現、markdown 內容）；desktop-manual-page（手冊頁的側欄樹）。",
        "Related changes: desktop-change-doc-views（在途、apply 中）會改寫 markdown 的兩條需求；desktop-detail-doc-tree定義了三欄骨架；desktop-discussion-doc-views（cut 3，尚未建）。",
        "Prior discussions: config-context-與-rules-gui-編輯, 桌面即時刷新與封存瀏覽。",
      ].join("\n"),
    );
    expect(meta.specs).toEqual(["desktop-app", "desktop-manual-page"]);
    expect(meta.changes).toEqual(["desktop-change-doc-views", "desktop-detail-doc-tree", "desktop-discussion-doc-views"]);
    expect(meta.prior).toEqual(["config-context-與-rules-gui-編輯", "桌面即時刷新與封存瀏覽"]);
  });

  it("圍欄內的同名行不剝", () => {
    const meta = parseContextMeta("```\nRelated specs: x\n```");
    expect(meta.specs).toBeNull();
    expect(meta.body).toBe("```\nRelated specs: x\n```");
  });
});

describe("contextRelations", () => {
  it("收尾行與內文命中的聯集；在途與已封存只收宿主清單裡有的名稱；討論排除本身", () => {
    const meta = parseContextMeta(
      [
        "提到 desktop-app、add-copilot-tool、old-change 與 self-slug、alpha-talk。",
        "Related specs: tray-status-menu",
        "Related changes: gone-change, old-change",
        "Prior discussions: beta-talk",
      ].join("\n"),
    );
    const rel = contextRelations(meta, {
      specs: ["desktop-app", "tray-status-menu"],
      active: ["add-copilot-tool"],
      archived: ["old-change"],
      discussions: ["self-slug", "alpha-talk", "beta-talk"],
      self: "self-slug",
    });
    expect(rel).toEqual({
      specs: ["tray-status-menu", "desktop-app"],
      changes: ["add-copilot-tool"],
      archived: ["old-change"],
      prior: ["beta-talk", "alpha-talk"],
      hits: {
        specs: ["desktop-app"],
        changes: ["add-copilot-tool"],
        archived: ["old-change"],
        discussions: ["alpha-talk"],
      },
    });
  });
});

describe("splitLead", () => {
  it("第一個空行之前為引言段", () => {
    expect(splitLead("第一段\n第一段續行\n\n第二段\n\n第三段")).toEqual({
      lead: "第一段\n第一段續行",
      rest: "第二段\n\n第三段",
    });
  });

  it("只有一段時 rest 為空", () => {
    expect(splitLead("\n\n只有一段\n")).toEqual({ lead: "只有一段", rest: "" });
  });

  it.each(["- 列點\n\n段", "## 標題\n\n段", "```\ncode\n```\n\n段"])("第一段不是散文（%j）時 lead 為空", (body) => {
    expect(splitLead(body)).toEqual({ lead: "", rest: body });
  });
});

describe("matchNames", () => {
  it("前後邊界、依首次出現序去重", () => {
    expect(
      matchNames("內文提到 desktop-app 與 add-copilot-tool，再提 desktop-app。", ["add-copilot-tool", "desktop-app", "desktop"]),
    ).toEqual(["desktop-app", "add-copilot-tool"]);
  });

  it("前綴與反引號內、圍欄內不算", () => {
    expect(matchNames("desktop-app-x 與 `desktop-app`\n```\ndesktop-app\n```", ["desktop-app"])).toEqual([]);
  });

  it("長名優先：短名在長名之內不另算", () => {
    expect(
      matchNames("見 desktop-導覽與專案首頁重構 的結論", ["desktop-導覽", "desktop-導覽與專案首頁重構"]),
    ).toEqual(["desktop-導覽與專案首頁重構"]);
  });

  it("名單為空時零命中", () => {
    expect(matchNames("desktop-app", [])).toEqual([]);
  });
});

describe("commandChips", () => {
  it("擷取 /speclink-* 指令與其旗標、參數，遇到中文或標點停", () => {
    expect(commandChips(parseConclusion(DCR_CONCLUSION)!.fields.Next!)).toEqual([
      "/speclink-propose --from-discussion detail-content-redesign",
    ]);
  });

  it("反引號內外皆算、去重保序", () => {
    expect(commandChips("跑 `/speclink-quality` 後 /speclink-archive x-y；再 /speclink-quality")).toEqual([
      "/speclink-quality",
      "/speclink-archive x-y",
    ]);
  });

  it("中文 slug 的參數完整擷取；英數段後緊接的中文、句尾句點與空白後的中文不算", () => {
    expect(commandChips("/speclink-propose --from-discussion config-context-與-rules-gui-編輯")).toEqual([
      "/speclink-propose --from-discussion config-context-與-rules-gui-編輯",
    ]);
    expect(commandChips("跑 /speclink-apply 的時候；/speclink-archive x-y後再推。最後 /speclink-archive z-w.")).toEqual([
      "/speclink-apply",
      "/speclink-archive x-y",
      "/speclink-archive z-w",
    ]);
  });

  it("沒有指令時為空陣列", () => {
    expect(commandChips("先跑品質關卡。")).toEqual([]);
  });
});

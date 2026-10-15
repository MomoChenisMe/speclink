// 文件解析器（desktop-change-doc-views design D1）：純函式，形狀不符時逐項降級（回 null 或標成一般列），不拋錯。
import { describe, expect, it } from "vitest";

import {
  findingAxis,
  listItemsOf,
  parseCapabilities,
  parseContractRows,
  parseDecisions,
  parseGoalsNonGoals,
  parseImpact,
  parseListItems,
  proposalShape,
  removedScenarios,
  splitAlternative,
  splitH2Sections,
  splitRisk,
  splitScenarioSteps,
  splitTaskText,
} from "../docModel";

describe("splitH2Sections", () => {
  // spec「提案與設計章節以中文標籤呈現」Example「章節對照與畫法」的來源標題。
  it.each([
    ["Summary", "sections.summary"],
    ["Why", "sections.why"],
    ["What Changes", "sections.whatChanges"],
    ["Proposed Solution", "sections.proposedSolution"],
    ["Non-Goals", "sections.nonGoals"],
    ["Alternatives Considered", "sections.alternatives"],
    ["Capabilities", "sections.capabilities"],
    ["Impact", "sections.impact"],
    ["Context", "sections.context"],
    ["Decisions", "sections.decisions"],
    ["Implementation Contract", "sections.contract"],
    ["Risks / Trade-offs", "sections.risks"],
  ])("白名單章節 ## %s 對到 %s", (title, key) => {
    const doc = splitH2Sections(`## ${title}\n\n內文。\n`);
    expect(doc?.sections).toEqual([{ key, title, content: "內文。", subsections: [] }]);
  });

  it("(optional) 於比對前剝除", () => {
    expect(splitH2Sections("## Non-Goals (optional)\n\n- 不做\n")?.sections[0]).toMatchObject({
      key: "sections.nonGoals",
      title: "Non-Goals",
    });
  });

  it("白名單外章節以原文為標題、key 為 null；首個 h2 之前為 preamble", () => {
    const doc = splitH2Sections("# 標題\n\n開場。\n\n## Why\n\n動機。\n\n## Rollout\n\n第一段。\n\n第二段。\n");
    expect(doc?.preamble).toBe("# 標題\n\n開場。");
    expect(doc?.sections.map((s) => [s.key, s.title, s.content])).toEqual([
      ["sections.why", "Why", "動機。"],
      [null, "Rollout", "第一段。\n\n第二段。"],
    ]);
  });

  it("fenced code block 內的 `## ` 不切段", () => {
    const doc = splitH2Sections("## Why\n\n```md\n## 不是標題\n```\n\n尾段。\n");
    expect(doc?.sections).toHaveLength(1);
    expect(doc?.sections[0].content).toBe("```md\n## 不是標題\n```\n\n尾段。");
  });

  it("沒有任何 h2 回 null", () => {
    expect(splitH2Sections("# 自由文件\n\n段落而已。\n")).toBeNull();
  });

  it("New／Modified Capabilities 只在 Capabilities 段內切成子段", () => {
    const md = [
      "## Capabilities",
      "",
      "### New Capabilities",
      "",
      "(none)",
      "",
      "### Modified Capabilities",
      "",
      "- `desktop-app`: 改寫",
      "",
      "## Decisions",
      "",
      "### New Capabilities",
      "",
      "決策內文。",
    ].join("\n");
    const doc = splitH2Sections(md)!;
    expect(doc.sections[0].subsections).toEqual([
      { key: "sections.newCapabilities", title: "New Capabilities", content: "(none)" },
      { key: "sections.modifiedCapabilities", title: "Modified Capabilities", content: "- `desktop-app`: 改寫" },
    ]);
    expect(doc.sections[1].subsections).toEqual([]);
    expect(doc.sections[1].content).toBe("### New Capabilities\n\n決策內文。");
  });
});

describe("parseListItems", () => {
  it("頂層列點與編號項依序成列，巢狀行歸入該項 body，非列點段落歸 prose", () => {
    const md = "開場段落。\n\n- 第一項\n  - 巢狀\n- 第二項\n\n1. 編號一\n2. 編號二\n\n收尾段落。";
    expect(parseListItems(md)).toEqual([
      { kind: "prose", text: "開場段落。" },
      {
        kind: "list",
        items: [
          { text: "第一項", body: "- 巢狀" },
          { text: "第二項", body: "" },
          { text: "編號一", body: "" },
          { text: "編號二", body: "" },
        ],
      },
      { kind: "prose", text: "收尾段落。" },
    ]);
  });

  it("listItemsOf 收齊各段的列點", () => {
    expect(listItemsOf(parseListItems("- a\n\n段落\n\n* b")).map((i) => i.text)).toEqual(["a", "b"]);
  });

  it("圍欄內的 `- ` 不是列點", () => {
    expect(parseListItems("```\n- 不是列點\n```")).toEqual([{ kind: "prose", text: "```\n- 不是列點\n```" }]);
  });
});

describe("splitAlternative", () => {
  it("首個「——」切方案與落敗理由", () => {
    expect(splitAlternative("保留舊畫法——討論裁定改掉——再補一句")).toEqual({
      option: "保留舊畫法",
      reason: "討論裁定改掉——再補一句",
    });
  });

  it("切不到時整條為方案、理由為 null", () => {
    expect(splitAlternative("只有方案")).toEqual({ option: "只有方案", reason: null });
  });
});

describe("parseImpact", () => {
  it("Affected specs 成能力名、Affected code 三組路徑，(none) 為空", () => {
    const md = [
      "- Affected specs: `desktop-app`（7 條 MODIFIED）",
      "- Affected code:",
      "  - New: `a/b.ts`、a/c.ts",
      "  - Modified: d/e.tsx（順手改）",
      "  - Removed: (none)",
    ].join("\n");
    expect(parseImpact(md)).toEqual({
      specs: { names: ["desktop-app"], raw: "`desktop-app`（7 條 MODIFIED）" },
      code: {
        new: [
          { path: "a/b.ts", note: null },
          { path: "a/c.ts", note: null },
        ],
        modified: [{ path: "d/e.tsx", note: "順手改" }],
        removed: [],
      },
      notes: [],
    });
  });

  it("沒有 Affected specs 列時 specs 為 null；其他列點留在 notes", () => {
    const md = "- Affected code:\n  - New: a.ts, b.ts\n  - Modified: c.ts\n  - Removed: (none)\n- 相容性：不變";
    const impact = parseImpact(md)!;
    expect(impact.specs).toBeNull();
    expect(impact.code.new.map((p) => p.path)).toEqual(["a.ts", "b.ts"]);
    expect(impact.notes.map((n) => n.text)).toEqual(["相容性：不變"]);
  });

  it("能力名不成形時保留原文、不切籤", () => {
    const md = "- Affected specs: 無 delta（零行為變化）\n- Affected code:\n  - Modified: a.ts";
    expect(parseImpact(md)?.specs).toEqual({ names: null, raw: "無 delta（零行為變化）" });
  });

  it("「無」為空能力清單", () => {
    expect(parseImpact("- Affected specs: 無\n- Affected code:\n  - New: a.ts")?.specs).toEqual({ names: [], raw: "無" });
  });

  it("不符形狀（無 Affected code 子項）回 null", () => {
    expect(parseImpact("- 影響看板與詳情頁\n- 無 CLI 變動")).toBeNull();
  });

  it("路徑項不成形（含空白的散文）回 null", () => {
    expect(parseImpact("- Affected code:\n  - Modified: a.ts、以及 components 下所有元件")).toBeNull();
  });
});

describe("parseCapabilities", () => {
  it("「`名稱`: 說明」列點成能力籤，其他列點為一般列，(none) 為空", () => {
    expect(parseCapabilities("- `desktop-app`: 修改詳情頁\n- auth：登入\n- 純說明列")).toEqual([
      { name: "desktop-app", text: "修改詳情頁" },
      { name: "auth", text: "登入" },
      { name: null, text: "純說明列" },
    ]);
    expect(parseCapabilities("(none)")).toEqual([]);
  });
});

describe("parseGoalsNonGoals", () => {
  it("以粗體標籤切兩欄", () => {
    const md = "**Goals:**\n\n- 目標一\n- 目標二\n\n**Non-Goals:**\n\n- 非目標一";
    expect(parseGoalsNonGoals(md)).toEqual({ intro: "", goals: "- 目標一\n- 目標二", nonGoals: "- 非目標一" });
  });

  it("缺任一標籤回 null", () => {
    expect(parseGoalsNonGoals("**Goals:**\n\n- 目標一")).toBeNull();
  });
});

describe("parseDecisions", () => {
  it("每個 h3 一張卡：D 編號、去編號標題、頂層列點數", () => {
    const md = "引言。\n\n### D1 面板底與卡片層\n\n- 一\n- 二\n  - 巢狀\n\n### 不帶編號\n\n散文。";
    expect(parseDecisions(md)).toEqual({
      intro: "引言。",
      cards: [
        { id: "D1", title: "面板底與卡片層", bullets: 2, body: "- 一\n- 二\n  - 巢狀" },
        { id: null, title: "不帶編號", bullets: 0, body: "散文。" },
      ],
    });
  });
});

describe("parseContractRows", () => {
  it("「- **鍵**：值」列點成鍵值列", () => {
    expect(parseContractRows("- **Behavior**：開頁\n- **Failure modes**: 降級")).toEqual([
      { key: "Behavior", value: "開頁" },
      { key: "Failure modes", value: "降級" },
    ]);
  });

  it("任一列不符回 null", () => {
    expect(parseContractRows("- **Behavior**：開頁\n- 散文列")).toBeNull();
  });
});

describe("splitRisk", () => {
  it("首個「→」切風險與對策，風險的方括號剝除", () => {
    expect(splitRisk("[字級連動討論側] → spec 已納入 → 不再動")).toEqual({
      risk: "字級連動討論側",
      mitigation: "spec 已納入 → 不再動",
    });
  });

  it("切不到時整條為標題", () => {
    expect(splitRisk("只有風險")).toEqual({ risk: "只有風險", mitigation: null });
  });
});

describe("splitTaskText", () => {
  // spec「任務群組標題與章節標籤同款式」Example「三層切分」。
  it.each([
    ["新增 `a.ts`（design D1）：匯出 x；驗證：npm test", "新增 `a.ts`", "匯出 x；", "npm test", ["D1"], null],
    ["補 i18n 詞條；驗證：i18n 測試全綠", "補 i18n 詞條；", null, "i18n 測試全綠", [], null],
    ["對齊匯出", "對齊匯出", null, null, [], null],
    [
      "改 TaskList（design D5；spec「任務群組標題與章節標籤同款式」）：群組成卡",
      "改 TaskList",
      "群組成卡",
      null,
      ["D5"],
      "spec「任務群組標題與章節標籤同款式」",
    ],
  ])("%s", (text, title, body, verify, decisions, note) => {
    expect(splitTaskText(text)).toEqual({ number: null, title, body, verify, decisions, note });
  });

  it("開頭的「X.Y」序號切到 number，其餘照三層切", () => {
    expect(splitTaskText("1.2 對齊匯出（design D2）：內文")).toMatchObject({ number: "1.2", title: "對齊匯出", decisions: ["D2"] });
    expect(splitTaskText("3 單層序號")).toMatchObject({ number: "3", title: "單層序號" });
  });

  it("半形括號與多個 D 編號", () => {
    expect(splitTaskText("改頁 (design D1, D3)：接線")).toMatchObject({ title: "改頁", decisions: ["D1", "D3"] });
  });
});

describe("removedScenarios", () => {
  it("取 REMOVED-SCENARIO 名單並去掉註解行與 BEFORE 行", () => {
    const block = [
      "<!-- BEFORE: 舊說明。 -->",
      "",
      "需求文。",
      "",
      "<!-- REMOVED-SCENARIO: ready workspace 的區塊順序與分割線 -->",
      "<!-- REMOVED-SCENARIO: 進度條依階段深淺 -->",
    ].join("\n");
    expect(removedScenarios(block)).toEqual({
      names: ["ready workspace 的區塊順序與分割線", "進度條依階段深淺"],
      content: "需求文。",
    });
  });
});

describe("splitScenarioSteps", () => {
  it("開頭的 GIVEN／WHEN／THEN／AND 列點成步驟，其餘為 rest", () => {
    const body = "- **GIVEN** 前提\n- **WHEN** 動作\n- **THEN** 結果\n- **AND** 補充\n\n##### Example: 表\n\n| a |\n| - |";
    expect(splitScenarioSteps(body)).toEqual({
      steps: [
        { label: "GIVEN", text: "前提" },
        { label: "WHEN", text: "動作" },
        { label: "THEN", text: "結果" },
        { label: "AND", text: "補充" },
      ],
      rest: "##### Example: 表\n\n| a |\n| - |",
    });
  });

  it("沒有步驟列點回 null", () => {
    expect(splitScenarioSteps("散文而已。")).toBeNull();
  });
});

describe("findingAxis", () => {
  // spec「詳情頁的工單分頁」Example「階段詞與 accepted token」（accepted 尾 token 已先切掉）。
  it.each([
    ["Correctness: 未處理空清單", "Correctness", "未處理空清單"],
    ["Standards: 命名不一致", "Standards", "命名不一致"],
    ["Standards: (accepted) 語意不清", "Standards", "(accepted) 語意不清"],
    ["Coherence：前後不一", "Coherence", "前後不一"],
    ["沒有前綴的描述", null, "沒有前綴的描述"],
  ])("%s", (input, axis, text) => {
    expect(findingAxis(input)).toEqual({ axis, text });
  });
});

describe("proposalShape", () => {
  const shape = (titles: string[]) => proposalShape(splitH2Sections(titles.map((t) => `## ${t}\n\n內文`).join("\n\n"))!);
  it.each([
    [["Problem", "Root Cause", "Proposed Solution"], "bugFix"],
    [["Summary", "Motivation", "Proposed Solution"], "refactor"],
    [["Why", "What Changes", "Impact"], "feature"],
    [["Context", "Rollout"], null],
  ])("%j → %s", (titles, expected) => {
    expect(shape(titles)).toBe(expected);
  });
});

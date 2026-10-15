// design D1：正式規格 spec.md 依 `### Requirement:` 切段（Purpose、body、scenarios、段尾 @trace），
// 與 SHALL／MUST 關鍵字的 rehype 標示外掛。
import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";

import { PURPOSE_TBD_PREFIX, parseSpecSections, rehypeSpecKeywords } from "../specSections";

/** 比照 archive.rs trace_block 排版的 @trace 註解。 */
function trace(source: string, updated: string): string {
  return ["<!-- @trace", `source: ${source}`, `updated: ${updated}`, "code:", "  - a.rs", "-->"].join("\n");
}

const SPEC = [
  "# demo Specification",
  "",
  "## Purpose",
  "",
  "示範用的規格。",
  "",
  "## Requirements",
  "",
  "### Requirement: 登入",
  "使用者 SHALL 能登入。",
  "",
  "#### Scenario: 正確密碼",
  "- **WHEN** 輸入正確密碼",
  "- **THEN** 進入首頁",
  "",
  "##### Example: 帳密對照",
  "",
  "| 帳號 | 結果 |",
  "| --- | --- |",
  "| alice | 成功 |",
  "",
  "#### Scenario: 錯誤密碼",
  "- **WHEN** 輸入錯誤密碼",
  "- **THEN** 顯示錯誤",
  "",
  "",
  trace("add-login", "2026-07-05"),
  "",
  "---",
  "### Requirement: 登出",
  "使用者 MUST 能登出。",
  "",
  "#### Scenario: 點登出",
  "- **WHEN** 點登出",
  "- **THEN** 回登入頁",
  "",
  trace("add-logout", "2026-09-16T16:37:51+08:00"),
  "",
  "---",
  "### Requirement: 記住我",
  "系統 SHALL NOT 在未勾選時記住帳號。",
  "",
  "```",
  "## 不是標題",
  "### Requirement: 也不是需求",
  "---",
  "```",
  "",
  "#### Scenario: 未勾選",
  "- **WHEN** 未勾選記住我",
  "- **THEN** 下次需重新輸入",
  "",
].join("\n");

describe("parseSpecSections", () => {
  it("切出 Purpose 與三條需求的標題", () => {
    const parsed = parseSpecSections(SPEC);
    expect(parsed).not.toBeNull();
    expect(parsed!.purpose).toBe("示範用的規格。");
    expect(parsed!.requirements.map((r) => r.title)).toEqual(["登入", "登出", "記住我"]);
  });

  it("body 為標題之後到第一個 Scenario 之前", () => {
    const [login] = parseSpecSections(SPEC)!.requirements;
    expect(login.body).toBe("使用者 SHALL 能登入。");
  });

  it("每個 Scenario 一段，Example 歸屬所在 Scenario", () => {
    const [login] = parseSpecSections(SPEC)!.requirements;
    expect(login.scenarios.map((s) => s.title)).toEqual(["正確密碼", "錯誤密碼"]);
    expect(login.scenarios[0].body).toContain("##### Example: 帳密對照");
    expect(login.scenarios[0].body).toContain("| alice | 成功 |");
    expect(login.scenarios[1].body).not.toContain("Example");
  });

  it("段尾 @trace 解析 source 與 updated 日期部分並自內文移除", () => {
    const [login, logout] = parseSpecSections(SPEC)!.requirements;
    expect(login.trace).toEqual({ source: "add-login", updated: "2026-07-05" });
    expect(logout.trace).toEqual({ source: "add-logout", updated: "2026-09-16" });
    const all = [login, logout].flatMap((r) => [r.body, ...r.scenarios.map((s) => s.body)]).join("\n");
    expect(all).not.toContain("@trace");
    expect(all).not.toContain("add-login");
  });

  it("單行的 @trace 註解自內文移除、不吞掉其後內容", () => {
    const parsed = parseSpecSections("### Requirement: A\n內文。\n<!-- @trace -->\n\n#### Scenario: S\n- x\n");
    expect(parsed!.requirements[0].body).toBe("內文。");
    expect(parsed!.requirements[0].scenarios).toEqual([{ title: "S", body: "- x" }]);
  });

  it("無 @trace 的需求 trace 為 null", () => {
    const remember = parseSpecSections(SPEC)!.requirements[2];
    expect(remember.trace).toBeNull();
  });

  it("--- 分隔線不出現在 body 與 scenario，程式碼區塊內的照留", () => {
    const { requirements } = parseSpecSections(SPEC)!;
    const [login, logout, remember] = requirements;
    expect(login.scenarios[1].body).not.toMatch(/^---$/m);
    expect(logout.scenarios[0].body).not.toMatch(/^---$/m);
    // 圍欄內的 `## `、`### Requirement:`、`---` 都不是結構。
    expect(remember.body).toContain("## 不是標題");
    expect(remember.body).toContain("### Requirement: 也不是需求");
    expect(remember.body).toMatch(/^---$/m);
    expect(remember.scenarios.map((s) => s.title)).toEqual(["未勾選"]);
  });

  it("沒有任何需求標題回 null", () => {
    expect(parseSpecSections("# x\n\n## Purpose\n\n只有目的。\n")).toBeNull();
    expect(parseSpecSections("")).toBeNull();
  });

  it("Purpose 缺席為空字串", () => {
    const parsed = parseSpecSections("### Requirement: A\n內文。\n");
    expect(parsed!.purpose).toBe("");
    expect(parsed!.requirements[0].body).toBe("內文。");
  });

  it("巢狀圍欄：收尾圍欄須同字元且不短於開頭，內層圍欄裡的需求標題不切段", () => {
    const md = [
      "### Requirement: A",
      "````markdown",
      "```",
      "### Requirement: 外層圍欄內",
      "```",
      "````",
      "~~~",
      "```",
      "### Requirement: 波浪圍欄內",
      "~~~",
      "### Requirement: B",
      "B 內文。",
    ].join("\n");
    const { requirements } = parseSpecSections(md)!;
    expect(requirements.map((r) => r.title)).toEqual(["A", "B"]);
    expect(requirements[0].body).toContain("### Requirement: 外層圍欄內");
    expect(requirements[0].body).toContain("### Requirement: 波浪圍欄內");
  });

  it("`## Requirements` 下的引言與需求段之外的 `## ` 區段不丟：分別收進 intro 與 appendix", () => {
    const md = [
      "# x Specification",
      "",
      "## Purpose",
      "",
      "目的。",
      "",
      "## Requirements",
      "",
      "這些需求適用於所有平台。",
      "",
      "### Requirement: A",
      "",
      "A 內文。",
      "",
      "## 附錄",
      "",
      "補充說明。",
    ].join("\n");
    const parsed = parseSpecSections(md)!;
    expect(parsed.purpose).toBe("目的。");
    expect(parsed.intro).toBe("這些需求適用於所有平台。");
    expect(parsed.requirements[0].body).toBe("A 內文。");
    expect(parsed.appendix).toBe("## 附錄\n\n補充說明。");
  });

  it("只有 Purpose 與需求的規格：intro 與 appendix 皆為空字串", () => {
    const parsed = parseSpecSections(SPEC)!;
    expect(parsed.intro).toBe("");
    expect(parsed.appendix).toBe("");
  });
});

/** 最小 hast 節點（測試直接構樹，不經 unified）。 */
interface Node {
  type: string;
  value?: string;
  tagName?: string;
  properties?: Record<string, unknown>;
  children?: Node[];
}

const text = (value: string): Node => ({ type: "text", value });
const el = (tagName: string, children: Node[]): Node => ({ type: "element", tagName, properties: {}, children });

/** 把子節點攤成「文字」與「[kw:…]」序列，方便比對。 */
function flat(node: Node): string[] {
  return (node.children ?? []).map((c) =>
    c.type === "text"
      ? c.value!
      : `[${c.tagName}.${(c.properties?.className as string[]).join(".")}:${c.children?.[0]?.value}]`,
  );
}

describe("rehypeSpecKeywords", () => {
  it("整詞包住 SHALL NOT／SHALL／MUST NOT／MUST", () => {
    const p = el("p", [text("系統 SHALL NOT 當機、SHALL 記錄，且 MUST 回報、MUST NOT 吞錯。")]);
    rehypeSpecKeywords()(el("root", [p]));
    expect(flat(p)).toEqual([
      "系統 ",
      "[code.spec-kw:SHALL NOT]",
      " 當機、",
      "[code.spec-kw:SHALL]",
      " 記錄，且 ",
      "[code.spec-kw:MUST]",
      " 回報、",
      "[code.spec-kw:MUST NOT]",
      " 吞錯。",
    ]);
  });

  it("不包非整詞（SHALLOW、MUSTER）", () => {
    const p = el("p", [text("SHALLOW MUSTER")]);
    rehypeSpecKeywords()(el("root", [p]));
    expect(flat(p)).toEqual(["SHALLOW MUSTER"]);
  });

  it("行內 code 與程式碼區塊內不處理", () => {
    const inline = el("code", [text("SHALL")]);
    const block = el("pre", [el("code", [text("MUST")])]);
    const p = el("p", [text("見 "), inline]);
    rehypeSpecKeywords()(el("root", [p, block]));
    expect(inline.children).toEqual([text("SHALL")]);
    expect(block.children![0].children).toEqual([text("MUST")]);
  });
});

describe("Purpose 佔位前綴跨語言對齊（引擎 ↔ UI）", () => {
  it("PURPOSE_TBD_PREFIX 與 lifecycle/model.rs 的同名常數逐字一致", () => {
    // 規格詳情頁的「Purpose 待補」提示框與引擎的 purposeTbd 同一前綴——讀原始檔逐字比對
    //（同 discussionDetailPage.test.tsx 的 STRUCTURAL_HEADERS 對齊題；repo root 由測試檔路徑推出）。
    const REPO_ROOT = resolve(dirname(expect.getState().testPath ?? ""), "../../../..");
    const source = readFileSync(resolve(REPO_ROOT, "crates/engine/speclink-core/src/lifecycle/model.rs"), "utf8");
    const m = /pub const PURPOSE_TBD_PREFIX: &str = "([^"]*)";/.exec(source);
    expect(m).not.toBeNull();
    expect(PURPOSE_TBD_PREFIX).toBe(m![1]);
  });
});

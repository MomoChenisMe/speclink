// spec「提案與設計章節以中文標籤呈現」提案側（desktop-change-doc-views design D3）。
import { describe, expect, it, vi } from "vitest";
import { fireEvent, render as rtlRender, screen } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider, translatorFor } from "../i18n";
import { changeDocSummary } from "../components/changeDocSummary";
import { DesignDoc } from "../components/DesignDoc";
import { ProposalDoc } from "../components/ProposalDoc";

const wrapper = ({ children }: { children: ReactNode }) => <I18nProvider locale="zh-TW">{children}</I18nProvider>;
const render = (ui: ReactElement) => rtlRender(ui, { wrapper });

const FEATURE_MD = [
  "## Why",
  "",
  "- 看不出層次",
  "- 散文太多",
  "",
  "## What Changes",
  "",
  "- 第一步",
  "- 第二步",
  "",
  "## Non-Goals",
  "",
  "- 不改格式",
  "",
  "## Capabilities",
  "",
  "### New Capabilities",
  "",
  "(none)",
  "",
  "### Modified Capabilities",
  "",
  "- `desktop-app`: 詳情頁改版",
  "- `server-api`: 不在 delta",
  "",
  "## Impact",
  "",
  "- Affected specs: desktop-app",
  "- Affected code:",
  "  - New: `a/one.ts`、a/two.ts",
  "  - Modified: b/three.tsx",
  "  - Removed: (none)",
].join("\n");

const headings = (container: HTMLElement, selector = "h2[data-doc-heading]") =>
  Array.from(container.querySelectorAll(selector)).map((h) => h.textContent);

// spec Example「章節對照與畫法」：一列一案（提案章節用 ProposalDoc、設計章節用 DesignDoc）。
describe("章節對照與畫法", () => {
  const CAPS = "### Modified Capabilities\n\n- `desktop-app`: 說明\n";
  const IMPACT = "- Affected specs: desktop-app\n- Affected code:\n  - New: a.ts\n  - Modified: (none)\n  - Removed: (none)\n";
  const rows: Array<[string, string, "proposal" | "design", string, string]> = [
    ["Summary", "摘要", "proposal", "一句話。", ".bg-muted"],
    ["Why", "為什麼", "proposal", "- 甲\n- 乙\n", "[data-row-list]"],
    ["What Changes", "變更內容", "proposal", "- 甲\n- 乙\n", "[data-numbered-steps]"],
    ["Proposed Solution", "解法", "proposal", "1. 甲\n2. 乙\n", "[data-numbered-steps]"],
    ["Non-Goals", "非目標", "proposal", "- 甲\n", "[data-row-list] svg"],
    ["Alternatives Considered", "曾考慮的替代案", "proposal", "- 方案——理由\n", "[data-alternative-reason]"],
    ["Capabilities", "能力", "proposal", CAPS, "[data-capability-chip]"],
    ["Impact", "影響", "proposal", IMPACT, "[data-impact-specs] + [data-impact-group]"],
    ["Context", "背景", "design", "- 甲\n- 乙\n", "[data-row-list]"],
    ["Decisions", "決策", "design", "### D1 標題\n\n- 點\n", "[data-decision]"],
    ["Implementation Contract", "實作契約", "design", "- **Behavior**：值\n", "dl dt"],
    ["Risks / Trade-offs", "風險與取捨", "design", "- [風險] → 對策\n", "[data-risk]"],
  ];
  it.each(rows)("## %s → 「%s」", (heading, label, doc, body, anchor) => {
    const md = `## ${heading}\n\n${body}`;
    const { container } = render(
      doc === "proposal" ? (
        <ProposalDoc content={md} deltaCaps={[]} />
      ) : (
        <DesignDoc content={md} jump={null} onJumped={() => {}} />
      ),
    );
    expect(container.querySelector("h2[data-doc-heading]")!.textContent).toBe(label);
    expect(container.querySelector(anchor)).toBeTruthy();
  });
});

// spec Scenario「替代方案與影響段的逐項降級」與 Example「影響段不符形狀時當一般列」的數字磚。
describe("提案數字磚「影響的檔案」", () => {
  const i18n = translatorFor("zh-TW");
  const tiles = (proposal: string) =>
    changeDocSummary(
      "proposal",
      { proposal, design: null, tasksMd: null, specDocs: {}, change: null, tickets: { review: null, verify: null } },
      i18n,
    ).tiles.map((x) => `${x.label} ${x.value}`);

  it("New 兩個、Modified 一個、Removed (none) 時為 3", () => {
    const md = [
      "## Alternatives Considered",
      "",
      "- 保留舊畫法——討論裁定改掉",
      "- 只有方案",
      "",
      "## Impact",
      "",
      "- Affected code:",
      "  - New: a/one.ts、a/two.ts",
      "  - Modified: b/three.tsx",
      "  - Removed: (none)",
    ].join("\n");
    expect(tiles(md)).toContain("影響的檔案 3");
  });

  it("影響段只有散文列點時不出「影響的檔案」格", () => {
    const md = "## Impact\n\n- 影響看板與詳情頁\n- 無 CLI 變動\n";
    expect(tiles(md).some((x) => x.startsWith("影響的檔案"))).toBe(false);
  });
});

describe("ProposalDoc", () => {
  it("模板章節成中文段標題、依序進本頁目錄，英文標題不直出", () => {
    const { container } = render(<ProposalDoc content={FEATURE_MD} deltaCaps={[]} />);
    expect(headings(container)).toEqual(["為什麼", "變更內容", "非目標", "能力", "影響"]);
    expect(headings(container, "h3[data-doc-heading]")).toEqual(["新增能力", "修改能力"]);
    expect(screen.queryByText("Why")).toBeNull();
    expect(screen.queryByText("What Changes")).toBeNull();
  });

  it("變更內容為編號步驟、非目標為帶禁止圖示的細線列，列式章節標題右端有列點數籤", () => {
    const { container } = render(<ProposalDoc content={FEATURE_MD} deltaCaps={[]} />);
    const steps = container.querySelector('[data-section="sections.whatChanges"] [data-numbered-steps]')!;
    expect(steps.querySelectorAll("li")).toHaveLength(2);
    expect(steps.textContent).toContain("1");
    const nonGoals = container.querySelector('[data-section="sections.nonGoals"]')!;
    expect(nonGoals.querySelector("[data-row-list] svg")).toBeTruthy();
    const why = container.querySelector('[data-section="sections.why"]')!;
    expect(why.querySelector("h2")!.parentElement!.textContent).toBe("為什麼2");
  });

  it("白名單外的二級標題以原文成段，不整篇退回", () => {
    const { container } = render(<ProposalDoc content={"## Why\n\n動機。\n\n## Rollout\n\n第一段。\n\n第二段。\n"} deltaCaps={[]} />);
    expect(headings(container)).toEqual(["為什麼", "Rollout"]);
    expect(container.querySelector('[data-section="custom"]')!.textContent).toContain("第二段。");
  });

  it("沒有二級標題時整篇照 Markdown，無章節段", () => {
    const { container } = render(<ProposalDoc content={"# 自由文件\n\n段落而已。\n"} deltaCaps={[]} />);
    expect(screen.getByText("段落而已。")).toBeTruthy();
    expect(container.querySelectorAll("[data-section]")).toHaveLength(0);
  });

  it("摘要成灰底引言段", () => {
    const { container } = render(<ProposalDoc content={"## Summary\n\n一句話摘要。\n"} deltaCaps={[]} />);
    const lead = container.querySelector('[data-section="sections.summary"]')!;
    expect(lead.textContent).toContain("摘要");
    expect(lead.querySelector(".bg-muted")!.textContent).toContain("一句話摘要。");
  });

  it("替代方案含「——」的列拆成方案與落敗理由，不含的整條為一般列", () => {
    const md = "## Alternatives Considered\n\n- 保留舊畫法——討論裁定改掉\n- 只有方案沒有理由\n";
    const { container } = render(<ProposalDoc content={md} deltaCaps={[]} />);
    const rows = container.querySelectorAll("[data-alternative]");
    expect(rows).toHaveLength(2);
    expect(rows[0].querySelector("[data-alternative-option]")!.textContent).toBe("保留舊畫法");
    expect(rows[0].querySelector("[data-alternative-reason]")!.textContent).toBe("討論裁定改掉");
    expect(rows[1].querySelector("[data-alternative-reason]")).toBeNull();
    expect(rows[1].textContent).toBe("只有方案沒有理由");
  });

  it("影響段成規格列與新增／修改／移除三組等寬路徑籤，(none) 顯示「無」", () => {
    const { container } = render(<ProposalDoc content={FEATURE_MD} deltaCaps={[]} />);
    const groups = Array.from(container.querySelectorAll("[data-impact-group]"));
    expect(groups.map((g) => g.firstElementChild!.textContent)).toEqual(["新增 2", "修改 1", "移除 無"]);
    expect(Array.from(container.querySelectorAll("[data-path-chip]")).map((c) => c.firstElementChild!.textContent)).toEqual([
      "a/one.ts",
      "a/two.ts",
      "b/three.tsx",
    ]);
    // hover 出的複製鈕；沒有宿主路徑動作時沒有顯示鈕（每顆籤只有複製鈕）。
    expect(container.querySelectorAll('[data-path-chip] button[aria-label="複製"]')).toHaveLength(3);
    expect(container.querySelectorAll("[data-path-chip] button")).toHaveLength(3);
  });

  it("宿主提供路徑動作時路徑籤多一顆宿主文案的顯示鈕，點擊回報路徑", () => {
    const reveal = vi.fn();
    render(<ProposalDoc content={FEATURE_MD} deltaCaps={[]} pathReveal={{ label: "在 Finder 顯示", reveal }} />);
    const buttons = screen.getAllByRole("button", { name: "在 Finder 顯示" });
    expect(buttons).toHaveLength(3);
    fireEvent.click(buttons[2]);
    expect(reveal).toHaveBeenCalledWith("b/three.tsx");
  });

  it("「移除」組的路徑籤沒有顯示鈕（檔案已不存在），只有複製鈕", () => {
    const md = "## Impact\n\n- Affected code:\n  - New: a/new.ts\n  - Modified: (none)\n  - Removed: c/gone.ts\n";
    const { container } = render(<ProposalDoc content={md} deltaCaps={[]} pathReveal={{ label: "在 Finder 顯示", reveal: vi.fn() }} />);
    const removed = container.querySelector('[data-impact-group="removed"]')!;
    expect(removed.querySelector('button[aria-label="複製"]')).toBeTruthy();
    expect(removed.querySelector('button[aria-label="在 Finder 顯示"]')).toBeNull();
    expect(container.querySelector('[data-impact-group="new"] button[aria-label="在 Finder 顯示"]')).toBeTruthy();
  });

  it("能力列說明為空時不畫「無內容」占位", () => {
    const md = "## Capabilities\n\n### Modified Capabilities\n\n- `desktop-app`:\n";
    const { container } = render(<ProposalDoc content={md} deltaCaps={[]} />);
    const section = container.querySelector('[data-section="sections.capabilities"]')!;
    expect(section.querySelector("[data-capability-chip]")!.textContent).toBe("desktop-app");
    expect(section.textContent).not.toContain("（無內容）");
  });

  it("影響段不符形狀時整段當一般細線列、無路徑籤", () => {
    const md = "## Impact\n\n- 影響看板與詳情頁\n- 無 CLI 變動\n";
    const { container } = render(<ProposalDoc content={md} deltaCaps={[]} />);
    expect(container.querySelectorAll("[data-path-chip]")).toHaveLength(0);
    expect(container.querySelectorAll('[data-section="sections.impact"] [data-row-list] li')).toHaveLength(2);
  });

  it("delta 已載入的能力籤可點並回報能力名，其餘為不可點的籤；能力段之下接頁給的提示框", () => {
    const onOpenCapability = vi.fn();
    const { container } = render(
      <ProposalDoc
        content={FEATURE_MD}
        deltaCaps={["desktop-app"]}
        onOpenCapability={onOpenCapability}
        afterCapabilities={<div data-testid="cap-note">提示</div>}
      />,
    );
    const chips = Array.from(container.querySelectorAll('[data-section="sections.capabilities"] [data-capability-chip]'));
    expect(chips.map((c) => [c.textContent, c.tagName])).toEqual([
      ["desktop-app", "BUTTON"],
      ["server-api", "SPAN"],
    ]);
    fireEvent.click(chips[0]);
    expect(onOpenCapability).toHaveBeenCalledWith("desktop-app");
    expect(container.querySelector('[data-section="sections.capabilities"]')!.textContent).toContain("無");
    expect(container.querySelector('[data-section="sections.capabilities"] [data-testid="cap-note"]')).toBeTruthy();
  });
});

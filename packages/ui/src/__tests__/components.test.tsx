import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, fireEvent } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";

// 既有中文斷言包 I18nProvider locale zh-TW 後照舊斷言（i18n 抽 key 的回歸保護）。
const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

import { DocumentTree } from "../components/DocumentTree";
import { DocumentViewer } from "../components/DocumentViewer";
import { Markdown } from "../components/Markdown";
import { SECTION_HEADING_CLS } from "../components/DocParts";
import { ProposalDoc } from "../components/ProposalDoc";
import type { ChangeItem, SpecItem } from "../adapter";

const changes: ChangeItem[] = [
  { name: "desktop-shell-and-browser", status: "in-progress", totalTasks: 17, completedTasks: 11 },
  { name: "web-server-postgres", status: "pending", totalTasks: 0, completedTasks: 0 },
];
const specs: SpecItem[] = [{ id: "verb-contract" }, { id: "desktop-app" }];

describe("DocumentTree", () => {
  it("renders changes and specs, and calls onSelect on click", () => {
    const onSelect = vi.fn();
    render(<DocumentTree changes={changes} specs={specs} onSelect={onSelect} />);
    expect(screen.getByText("verb-contract")).toBeTruthy();
    fireEvent.click(screen.getByText("web-server-postgres"));
    expect(onSelect).toHaveBeenCalledWith({ kind: "change", id: "web-server-postgres" });
  });
});

// spec「markdown 內容保留文件結構呈現」「raw HTML 不以原文呈現」（desktop-reading-experience）
describe("Markdown", () => {
  it("renders a single newline as a line break (soft break → <br>)", () => {
    const { container } = render(<Markdown content={"**Focus**: 甲\n**Position**: 乙"} />);
    expect(container.querySelector("br")).toBeTruthy();
  });

  it("does not render HTML comment text", () => {
    const { container } = render(
      <Markdown content={"<!-- secret-anchor entries appended by CLI -->\n\n正文段落"} />,
    );
    expect(container.textContent).not.toContain("secret-anchor");
    expect(container.textContent).toContain("正文段落");
  });

  it("keeps HTML inside code fences verbatim", () => {
    const { container } = render(<Markdown content={"```\n<div>x</div>\n```"} />);
    expect(container.textContent).toContain("<div>x</div>");
  });

  // 文件正文常把 `<name>` 當佔位字寫；佔位字剛好是 HTML 標籤名時，原始 HTML 若被解析，
  // <summary>／<details> 會把後文包進收合區塊、<template> 會吞掉後文。原始 HTML 一律丟棄。
  it("drops raw HTML so placeholder-like tags do not hide or swallow the following text", () => {
    const { container } = render(
      <Markdown
        content={"- 格式 speclink(<change-name>): <summary> 與欄位\n- 第二項 <template> 之後\n- 第三項 <details> 收合"}
      />,
    );
    expect(container.querySelector("summary, details, template")).toBeNull();
    for (const text of ["與欄位", "之後", "第三項", "收合"]) expect(container.textContent).toContain(text);
  });

  it("does not render raw HTML elements or their id attributes", () => {
    const { container } = render(<Markdown content={'前 <span id="manual-h-1">中</span> 後'} />);
    expect(container.querySelector("#manual-h-1")).toBeNull();
    expect(container.textContent).toContain("中");
  });

  // spec「寬表格於容器內橫捲」：表格外層容器橫向可捲、不設高度上限。
  it("wraps wide tables in a horizontally scrollable container without a height cap", () => {
    const { container } = render(<Markdown content={"| a | b |\n| - | - |\n| 1 | 2 |"} />);
    const scroller = container.querySelector('[data-streamdown="table"]')?.parentElement as HTMLElement;
    expect(scroller.className).toContain("overflow-x-auto");
    expect(scroller.style.maxHeight).toBe("");
  });

  it("renders inside the markdown hook class without typography prose (desktop-design-foundation D10)", () => {
    const { container } = render(<Markdown content={"hello"} />);
    const root = container.querySelector(".markdown");
    expect(root).toBeTruthy();
    expect(root?.className).not.toContain("prose");
  });

  // spec「markdown 文件內容行寬有上限」（design D3 文件容器）：
  // 固定行寬上限（96ch ≈ 48 全形字 @16px），抽屜全螢幕時行寬不隨之增長。
  it("caps content line width instead of stretching full-bleed (markdown 文件內容行寬有上限)", () => {
    const { container } = render(<Markdown content={"hello"} />);
    const root = container.querySelector(".markdown");
    expect(root?.className).not.toContain("max-w-none");
    expect(root?.className).toContain("max-w-[96ch]");
  });
});

// spec 需求「提案與設計章節以中文標籤呈現」（desktop-change-doc-views design D1 章節模型、D3 提案渲染器）。
describe("ProposalDoc 章節標籤", () => {
  const PROPOSAL_MD = [
    "## Why",
    "",
    "動機段落。",
    "",
    "## What Changes",
    "",
    "- 第一項變更",
    "",
    "## Non-Goals",
    "",
    "- 不做的事",
    "",
    "## Capabilities",
    "",
    "### Modified Capabilities",
    "",
    "- desktop-app: 章節呈現",
    "",
    "## Impact",
    "",
    "- Affected specs: desktop-app",
  ].join("\n");

  it("提案模板章節渲染中文標籤，英文標題不直出", () => {
    render(<ProposalDoc content={PROPOSAL_MD} deltaCaps={[]} />);
    for (const label of ["為什麼", "變更內容", "非目標", "能力", "影響"]) {
      expect(screen.getByText(label)).toBeTruthy();
    }
    expect(screen.getByText("動機段落。")).toBeTruthy();
    expect(screen.queryByText("Why")).toBeNull();
    expect(screen.queryByText("What Changes")).toBeNull();
    expect(screen.queryByText("Impact")).toBeNull();
  });

  it("設計模板章節渲染中文標籤（含斜線複合詞）", () => {
    const md = "## Context\n\n背景內文。\n\n## Decisions\n\n決策內文。\n\n## Risks / Trade-offs\n\n風險內文。\n";
    render(<ProposalDoc content={md} deltaCaps={[]} />);
    expect(screen.getByText("背景")).toBeTruthy();
    expect(screen.getByText("決策")).toBeTruthy();
    expect(screen.getByText("風險與取捨")).toBeTruthy();
    expect(screen.queryByText("Context")).toBeNull();
    expect(screen.queryByText(/Trade-offs/)).toBeNull();
  });

  it("白名單外標題以原文成段（不翻譯），內文照排", () => {
    const md = "## Decisions\n\n### D1 章節切分：白名單映射\n\n決策細節。\n\n## 自訂章節\n\n自訂內文。\n";
    render(<ProposalDoc content={md} deltaCaps={[]} />);
    expect(screen.getByText("決策")).toBeTruthy();
    // 自訂決策標題照 prose 渲染；自訂章節以原文為段標題。
    expect(screen.getByText(/D1 章節切分/)).toBeTruthy();
    expect(screen.getByText("自訂章節")).toBeTruthy();
    expect(screen.getByText("自訂內文。")).toBeTruthy();
  });

  it("模板附註 (optional) 於比對前剝除", () => {
    render(<ProposalDoc content={"## Non-Goals (optional)\n\n- 不做\n"} deltaCaps={[]} />);
    expect(screen.getByText("非目標")).toBeTruthy();
    expect(screen.queryByText(/Non-Goals/)).toBeNull();
  });

  // spec「章節標題為 16px 粗體」（desktop-change-doc-views design D2 單一常數）。
  it("章節標題為 16px 粗體款式（引用共用常數）", () => {
    render(<ProposalDoc content={PROPOSAL_MD} deltaCaps={[]} />);
    const label = screen.getByText("為什麼");
    for (const cls of SECTION_HEADING_CLS.split(" ")) expect(label.className).toContain(cls);
    expect(label.className).toContain("text-base");
    expect(label.className).toContain("font-semibold");
    for (const stale of ["text-xs", "uppercase", "tracking-wider", "text-muted-foreground"]) {
      expect(label.className).not.toContain(stale);
    }
  });

  it("沒有任何二級標題時整篇退回單一 markdown 檢視", () => {
    const { container } = render(<ProposalDoc content={"# 自由文件\n\n段落內容而已。\n"} deltaCaps={[]} />);
    expect(screen.getByText("段落內容而已。")).toBeTruthy();
    expect(container.querySelectorAll("[data-section]").length).toBe(0);
  });
});

describe("DocumentViewer", () => {
  it("renders the given document content", () => {
    render(<DocumentViewer content={"## Why\nbecause"} />);
    expect(screen.getByText(/because/)).toBeTruthy();
  });

  it("shows an empty state when content is null", () => {
    render(<DocumentViewer content={null} />);
    expect(screen.getByText(/選擇|no document|select/i)).toBeTruthy();
  });
});

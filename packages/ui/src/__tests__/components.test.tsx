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
import { LABEL_CLS, SectionedDoc } from "../components/SectionedDoc";
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

  it("renders inside a prose container with the markdown hook class", () => {
    const { container } = render(<Markdown content={"hello"} />);
    const root = container.querySelector(".markdown");
    expect(root).toBeTruthy();
    expect(root?.className).toContain("prose");
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

// spec 需求「提案與設計章節以中文標籤呈現」（design D1 白名單切分、D2 對照表）。
describe("SectionedDoc", () => {
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
    render(<SectionedDoc content={PROPOSAL_MD} />);
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
    render(<SectionedDoc content={md} />);
    expect(screen.getByText("背景")).toBeTruthy();
    expect(screen.getByText("決策")).toBeTruthy();
    expect(screen.getByText("風險與取捨")).toBeTruthy();
    expect(screen.queryByText("Context")).toBeNull();
    expect(screen.queryByText(/Trade-offs/)).toBeNull();
  });

  it("白名單外標題照 prose 排，不標籤化", () => {
    const md = "## Decisions\n\n### D1 章節切分：白名單映射\n\n決策細節。\n\n## 自訂章節\n\n自訂內文。\n";
    render(<SectionedDoc content={md} />);
    expect(screen.getByText("決策")).toBeTruthy();
    // 自訂決策標題與自訂章節照 prose 標題渲染（文字仍可見、非標籤樣式的區塊標題）。
    expect(screen.getByText(/D1 章節切分/)).toBeTruthy();
    expect(screen.getByText("自訂章節")).toBeTruthy();
    expect(screen.getByText("自訂內文。")).toBeTruthy();
  });

  it("模板附註 (optional) 於比對前剝除", () => {
    render(<SectionedDoc content={"## Non-Goals (optional)\n\n- 不做\n"} />);
    expect(screen.getByText("非目標")).toBeTruthy();
    expect(screen.queryByText(/Non-Goals/)).toBeNull();
  });

  // spec「標籤為大標題且字級大於內文」（design D6 粗體大標題、單一常數）。
  it("章節標籤為粗體大標題款式（引用共用常數）", () => {
    render(<SectionedDoc content={PROPOSAL_MD} />);
    const label = screen.getByText("為什麼");
    for (const cls of LABEL_CLS.split(" ")) expect(label.className).toContain(cls);
    expect(label.className).toContain("text-xl");
    expect(label.className).toContain("font-bold");
    for (const stale of ["text-xs", "uppercase", "tracking-wider", "text-muted-foreground"]) {
      expect(label.className).not.toContain(stale);
    }
  });

  it("無任何白名單章節整篇退回單一 markdown 檢視", () => {
    const { container } = render(<SectionedDoc content={"# 自由文件\n\n段落內容而已。\n"} />);
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

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

import { ChangeBoard } from "../components/ChangeBoard";
import { DocumentTree } from "../components/DocumentTree";
import { DocumentViewer } from "../components/DocumentViewer";
import { Markdown } from "../components/Markdown";
import type { ChangeItem, SpecItem } from "../adapter";

const changes: ChangeItem[] = [
  { name: "desktop-shell-and-browser", status: "in-progress", totalTasks: 17, completedTasks: 11 },
  { name: "web-server-postgres", status: "pending", totalTasks: 0, completedTasks: 0 },
];
const specs: SpecItem[] = [{ id: "verb-contract" }, { id: "desktop-app" }];

describe("ChangeBoard", () => {
  it("renders each change with its name and task progress", () => {
    render(<ChangeBoard changes={changes} />);
    expect(screen.getByText("desktop-shell-and-browser")).toBeTruthy();
    expect(screen.getByText(/11\s*\/\s*17/)).toBeTruthy();
  });

  it("invokes onRunVerb with the verb and change name when a verb button is clicked", () => {
    const onRunVerb = vi.fn();
    render(<ChangeBoard changes={changes} onRunVerb={onRunVerb} />);
    fireEvent.click(screen.getAllByRole("button", { name: /validate/i })[0]);
    expect(onRunVerb).toHaveBeenCalledWith("validate", "desktop-shell-and-browser");
  });
});

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

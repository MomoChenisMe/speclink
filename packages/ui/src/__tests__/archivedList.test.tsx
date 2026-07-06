// spec 需求「已封存變更可展開檢視」：列徽章、展開唯讀分頁、懶載入、
// 任務核取方塊不可互動、缺件文件顯示空狀態。
import { describe, it, expect, vi } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";

import { ArchivedList } from "../components/ArchivedList";
import type { ArchivedItem } from "../adapter";

const FULL: ArchivedItem = {
  datedName: "2026-07-05-desktop-shell-and-browser",
  date: "2026-07-05",
  name: "desktop-shell-and-browser",
  tasksTotal: 48,
  tasksDone: 48,
};

const BARE: ArchivedItem = {
  datedName: "2026-07-01-no-tasks",
  date: "2026-07-01",
  name: "no-tasks",
};

const DOCS: Record<string, string> = {
  "proposal.md": "## Why\n\n封存的提案內文。\n",
  "tasks.md": "## 1. G\n\n- [x] 1.1 完成的任務\n- [x] 1.2 另一項\n",
  "specs/desktop-app/spec.md": "## ADDED Requirements\n\n### Requirement: 桌面殼\n",
};

function makeLoaders() {
  return {
    loadDocument: vi.fn(async (_datedName: string, artifact: string) => DOCS[artifact] ?? null),
    loadCapabilities: vi.fn(async (_datedName: string) => ["desktop-app"]),
  };
}

function renderList(items: ArchivedItem[], loaders = makeLoaders()) {
  render(
    <ArchivedList
      archived={items}
      query=""
      onQuery={() => {}}
      loadDocument={loaders.loadDocument}
      loadCapabilities={loaders.loadCapabilities}
    />,
  );
  return loaders;
}

describe("ArchivedList（封存展開檢視）", () => {
  it("列顯示任務數徽章；無 tasks.md 的封存項不顯示徽章", () => {
    renderList([FULL, BARE]);
    expect(screen.getByText("48/48")).toBeTruthy();
    // BARE 列存在但沒有徽章
    expect(screen.getByText("no-tasks")).toBeTruthy();
    expect(screen.queryByText("0/0")).toBeNull();
  });

  it("內容懶載入：展開前不讀任何文件", () => {
    const loaders = renderList([FULL]);
    expect(loaders.loadDocument).not.toHaveBeenCalled();
    expect(loaders.loadCapabilities).not.toHaveBeenCalled();
  });

  it("點擊列展開唯讀分頁（提案／設計／任務／規格）並載入內容", async () => {
    const loaders = renderList([FULL]);
    fireEvent.click(screen.getByText("desktop-shell-and-browser"));
    await waitFor(() => expect(screen.getByRole("tab", { name: /提案/ })).toBeTruthy());
    expect(screen.getByRole("tab", { name: /設計/ })).toBeTruthy();
    expect(screen.getByRole("tab", { name: /任務/ })).toBeTruthy();
    expect(screen.getByRole("tab", { name: /規格/ })).toBeTruthy();
    // 提案分頁內容來自封存目錄的實體文件
    await waitFor(() => expect(screen.getByText("封存的提案內文。")).toBeTruthy());
    expect(loaders.loadDocument).toHaveBeenCalledWith(FULL.datedName, "proposal.md");
    expect(loaders.loadCapabilities).toHaveBeenCalledWith(FULL.datedName);
  });

  it("任務分頁為唯讀：核取方塊不可點擊", async () => {
    renderList([FULL]);
    fireEvent.click(screen.getByText("desktop-shell-and-browser"));
    await waitFor(() => screen.getByRole("tab", { name: /任務/ }));
    fireEvent.mouseDown(screen.getByRole("tab", { name: /任務/ }));
    await waitFor(() => expect(screen.getByLabelText("任務 1")).toBeTruthy());
    const checkbox = screen.getByLabelText("任務 1") as HTMLInputElement;
    expect(checkbox.disabled).toBe(true);
    // 唯讀模式不提供排序按鈕
    expect(screen.queryByLabelText("上移任務 1")).toBeNull();
  });

  it("缺件文件的分頁顯示空狀態而非錯誤", async () => {
    renderList([FULL]);
    fireEvent.click(screen.getByText("desktop-shell-and-browser"));
    await waitFor(() => screen.getByRole("tab", { name: /設計/ }));
    fireEvent.mouseDown(screen.getByRole("tab", { name: /設計/ }));
    // DOCS 沒有 design.md → loadDocument 回 null → 空狀態文案
    await waitFor(() => expect(screen.getByText(/無設計文件/)).toBeTruthy());
  });
});

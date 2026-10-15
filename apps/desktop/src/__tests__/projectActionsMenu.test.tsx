// 專案動作選單（desktop-app「專案層檔案系統動作」，design D5）：標題列「⋯」與
// 圖示列方塊右鍵渲染同一份項目。
import { describe, expect, it, vi } from "vitest";
import { fireEvent, render as rtlRender, screen } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuTrigger,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger,
  I18nProvider,
} from "@speclink/ui";

import { ProjectActionItems, type ProjectActions } from "../components/ProjectActionsMenu";
import { APP_MESSAGES } from "../i18n/messages";
import type { Platform } from "../platform";
import type { ProjectTab } from "../tabs";

const zh = APP_MESSAGES["zh-TW"];
const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

const LOCAL: ProjectTab = { locator: { kind: "local", root: "/work/alpha" }, name: "alpha" };
const REMOTE_NO_CHECKOUT: ProjectTab = {
  locator: { kind: "remote", connectionId: "c1", projectId: "demo", repoId: "api" },
  name: "Demo/api",
};
const REMOTE_CHECKOUT: ProjectTab = {
  locator: {
    kind: "remote",
    connectionId: "c1",
    projectId: "demo",
    repoId: "web",
    checkoutRoot: "/work/web",
  },
  name: "Demo/web",
};

function fakeActions(): ProjectActions {
  return {
    reveal: vi.fn(),
    openTerminal: vi.fn(),
    openEditor: vi.fn(),
    copyPath: vi.fn(),
    refresh: vi.fn(),
    close: vi.fn(),
  };
}

/** 受控開著的下拉選單：點項目不會關閉，可逐項點擊。 */
function renderDropdown(
  tab: ProjectTab,
  { platform = "macos", error }: { platform?: Platform; error?: string } = {},
) {
  const actions = fakeActions();
  render(
    <DropdownMenu open>
      <DropdownMenuTrigger>⋯</DropdownMenuTrigger>
      <DropdownMenuContent>
        <ProjectActionItems menu="dropdown" tab={tab} platform={platform} actions={actions} error={error} />
      </DropdownMenuContent>
    </DropdownMenu>,
  );
  return actions;
}

const itemNames = () =>
  screen.getAllByRole("menuitem").map((item) => item.querySelector("span")?.textContent ?? "");

describe("ProjectActionItems", () => {
  it("本機專案：六個動作項與一條分隔線，順序固定", () => {
    renderDropdown(LOCAL);
    expect(itemNames()).toEqual([
      "在 Finder 顯示",
      "在終端機開啟",
      "以編輯器開啟",
      "複製路徑",
      "重新整理",
      "關閉專案",
    ]);
    expect(screen.getAllByRole("separator")).toHaveLength(1);
  });

  it("各項點擊呼叫對應動作一次，帶該專案的 locator key", () => {
    const actions = renderDropdown(LOCAL);
    const key = "local:/work/alpha";
    for (const [label, fn] of [
      ["在 Finder 顯示", actions.reveal],
      ["在終端機開啟", actions.openTerminal],
      ["以編輯器開啟", actions.openEditor],
      ["複製路徑", actions.copyPath],
      ["重新整理", actions.refresh],
      ["關閉專案", actions.close],
    ] as const) {
      fireEvent.click(screen.getByRole("menuitem", { name: new RegExp(`^${label}`) }));
      expect(fn).toHaveBeenCalledTimes(1);
      expect(fn).toHaveBeenCalledWith(key);
    }
  });

  it("複製路徑右端顯示路徑尾段；快捷鍵以 <kbd> 顯示；關閉專案為紅字", () => {
    renderDropdown(LOCAL);
    expect(screen.getByRole("menuitem", { name: /^複製路徑/ }).textContent).toContain("alpha");
    expect(screen.getByText("⌘R").tagName).toBe("KBD");
    expect(screen.getByText("⌘W").tagName).toBe("KBD");
    expect(screen.getByRole("menuitem", { name: /^關閉專案/ }).className).toContain("text-destructive");
  });

  it("remote 無 checkout：只有重新整理與關閉專案", () => {
    renderDropdown(REMOTE_NO_CHECKOUT);
    expect(itemNames()).toEqual(["重新整理", "關閉專案"]);
    expect(screen.queryByRole("separator")).toBeNull();
  });

  it("remote 有 checkout：四個檔案系統項目在、路徑尾段取 checkout", () => {
    const actions = renderDropdown(REMOTE_CHECKOUT);
    expect(itemNames()).toHaveLength(6);
    expect(screen.getByRole("menuitem", { name: /^複製路徑/ }).textContent).toContain("web");
    fireEvent.click(screen.getByRole("menuitem", { name: /^以編輯器開啟/ }));
    expect(actions.openEditor).toHaveBeenCalledWith("remote:c1/demo/web");
  });

  it("錯誤變體：首列為錯誤訊息，其後只有「自專案列移除」", () => {
    const actions = renderDropdown(LOCAL, { error: "目錄不存在" });
    const label = screen.getByText("目錄不存在");
    expect(label.className).toContain("text-destructive");
    expect(itemNames()).toEqual([zh["app.removeTab"]]);
    fireEvent.click(screen.getByRole("menuitem", { name: zh["app.removeTab"] }));
    expect(actions.close).toHaveBeenCalledWith("local:/work/alpha");
  });

  it.each([
    ["macos", "在 Finder 顯示", "⌘R"],
    ["windows", "在檔案總管顯示", "Ctrl+R"],
    ["linux", "在檔案管理員顯示", "Ctrl+R"],
  ] as const)("%s：檔案管理員名稱與快捷鍵隨平台", (platform, reveal, shortcut) => {
    renderDropdown(LOCAL, { platform });
    expect(itemNames()[0]).toBe(reveal);
    expect(screen.getByText(shortcut)).toBeTruthy();
  });

  it("右鍵選單渲染同一份項目", () => {
    render(
      <ContextMenu>
        <ContextMenuTrigger>alpha</ContextMenuTrigger>
        <ContextMenuContent>
          <ProjectActionItems menu="context" tab={LOCAL} platform="macos" actions={fakeActions()} />
        </ContextMenuContent>
      </ContextMenu>,
    );
    fireEvent.contextMenu(screen.getByText("alpha"));
    expect(itemNames()).toHaveLength(6);
    expect(screen.getAllByRole("separator")).toHaveLength(1);
  });
});

// 下拉選單、右鍵選單與 Kbd 原語（design D5）：兩套選單共用同一組 class 常數。
import { describe, it, expect, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "../components/ui/dropdown-menu";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuLabel,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "../components/ui/context-menu";
import { Kbd, MenuShortcut } from "../components/ui/kbd";

const classes = (el: Element) => (el.getAttribute("class") ?? "").split(/\s+/);

describe("DropdownMenu", () => {
  it("觸發後 Content 為 12px 圓角浮層、Item 8px 圓角、destructive 紅字、Shortcut 為 <kbd>、有分隔線", async () => {
    const user = userEvent.setup();
    const onRefresh = vi.fn();
    render(
      <DropdownMenu>
        <DropdownMenuTrigger>開</DropdownMenuTrigger>
        <DropdownMenuContent>
          <DropdownMenuLabel>標籤</DropdownMenuLabel>
          <DropdownMenuItem onSelect={onRefresh}>
            重新整理
            <MenuShortcut>⌘R</MenuShortcut>
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem variant="destructive">關閉專案</DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>,
    );
    await user.click(screen.getByText("開"));

    const menu = screen.getByRole("menu");
    expect(classes(menu)).toEqual(expect.arrayContaining(["rounded-xl", "shadow-lg", "bg-card"]));
    const refresh = screen.getByRole("menuitem", { name: /重新整理/ });
    expect(classes(refresh)).toEqual(expect.arrayContaining(["rounded-lg", "h-8"]));
    expect(screen.getByText("⌘R").tagName).toBe("KBD");
    expect(classes(screen.getByRole("menuitem", { name: "關閉專案" }))).toContain("text-destructive");
    expect(classes(refresh)).not.toContain("text-destructive");
    expect(screen.getByRole("separator")).toBeTruthy();

    await user.click(refresh);
    expect(onRefresh).toHaveBeenCalledTimes(1);
  });
});

describe("ContextMenu", () => {
  it("右鍵開啟，Content／Item／destructive 與下拉選單同一組 class", () => {
    render(
      <ContextMenu>
        <ContextMenuTrigger>方塊</ContextMenuTrigger>
        <ContextMenuContent>
          <ContextMenuLabel>錯誤</ContextMenuLabel>
          <ContextMenuItem>在 Finder 顯示</ContextMenuItem>
          <ContextMenuSeparator />
          <ContextMenuItem variant="destructive">
            關閉專案
            <MenuShortcut>⌘W</MenuShortcut>
          </ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>,
    );
    fireEvent.contextMenu(screen.getByText("方塊"));

    expect(classes(screen.getByRole("menu"))).toEqual(
      expect.arrayContaining(["rounded-xl", "shadow-lg", "bg-card"]),
    );
    expect(classes(screen.getByRole("menuitem", { name: "在 Finder 顯示" }))).toEqual(
      expect.arrayContaining(["rounded-lg", "h-8"]),
    );
    expect(classes(screen.getByRole("menuitem", { name: /^關閉專案/ }))).toContain("text-destructive");
    expect(screen.getByRole("separator")).toBeTruthy();
    // 快捷鍵槽兩套選單共用同一個元件：右端對齊的 <kbd>。
    const shortcut = screen.getByText("⌘W");
    expect(shortcut.tagName).toBe("KBD");
    expect(classes(shortcut)).toContain("ml-auto");
  });
});

describe("Kbd", () => {
  it("等寬 11px 灰字", () => {
    render(<Kbd>Ctrl+W</Kbd>);
    const kbd = screen.getByText("Ctrl+W");
    expect(kbd.tagName).toBe("KBD");
    expect(classes(kbd)).toEqual(
      expect.arrayContaining(["font-mono", "text-[11px]", "text-muted-foreground"]),
    );
  });
});

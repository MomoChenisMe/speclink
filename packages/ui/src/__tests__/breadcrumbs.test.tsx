// 共用麵包屑（server-web-shell-alignment design D4）：desktop 主區頂列與 server 後台共用一份。
import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { Breadcrumbs } from "../components/Breadcrumbs";

const classes = (el: Element) => (el.getAttribute("class") ?? "").split(/\s+/);

describe("Breadcrumbs", () => {
  it("項與項之間是「/」，最後一項為前景色，外層 nav 標為頁面路徑", () => {
    render(<Breadcrumbs crumbs={[{ label: "專案" }, { label: "變更" }, { label: "add-auth" }]} />);
    const nav = screen.getByRole("navigation", { name: "頁面路徑" });
    expect(nav.textContent).toBe("專案/變更/add-auth");
    expect(screen.getAllByText("/")).toHaveLength(2);
    expect(classes(screen.getByText("add-auth"))).toContain("text-foreground");
    expect(classes(screen.getByText("專案"))).not.toContain("text-foreground");
  });

  it("有 onClick 的項是按鈕且點擊呼叫回呼，沒有的項不是按鈕", async () => {
    const user = userEvent.setup();
    const onHome = vi.fn();
    render(<Breadcrumbs crumbs={[{ label: "後台", onClick: onHome }, { label: "使用者" }]} />);
    await user.click(screen.getByRole("button", { name: "後台" }));
    expect(onHome).toHaveBeenCalledTimes(1);
    expect(screen.queryByRole("button", { name: "使用者" })).toBeNull();
    expect(screen.getAllByRole("button")).toHaveLength(1);
    // 鍵盤聚焦時顯示 2px focus ring（共用 ring token），不靠瀏覽器預設外框。
    expect(classes(screen.getByRole("button", { name: "後台" }))).toEqual(
      expect.arrayContaining(["focus-visible:ring-2", "focus-visible:ring-ring"]),
    );
  });

  it("mono 項用等寬字型", () => {
    render(<Breadcrumbs crumbs={[{ label: "變更" }, { label: "add-auth", mono: true }]} />);
    expect(classes(screen.getByText("add-auth"))).toContain("font-mono");
    expect(classes(screen.getByText("變更"))).not.toContain("font-mono");
  });
});

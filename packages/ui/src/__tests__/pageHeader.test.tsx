// 頁標題區原語（design D6）：24px 一般字重標題＋灰字說明＋右側動作槽。
import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";

import { PageHeader } from "../components/ui/page-header";

const classes = (el: Element) => (el.getAttribute("class") ?? "").split(/\s+/);

describe("PageHeader", () => {
  it("標題為 2xl 一般字重的 h2、說明為 13px 灰字、動作槽在右側", () => {
    render(
      <PageHeader title="變更" description="依生命週期分欄。" actions={<button>搜尋</button>} />,
    );
    const title = screen.getByRole("heading", { level: 2, name: "變更" });
    expect(classes(title)).toEqual(expect.arrayContaining(["text-2xl", "font-normal"]));
    expect(classes(screen.getByText("依生命週期分欄。"))).toEqual(
      expect.arrayContaining(["text-[13px]", "text-muted-foreground"]),
    );
    const slot = screen.getByRole("button", { name: "搜尋" }).parentElement as HTMLElement;
    expect(classes(slot)).toEqual(expect.arrayContaining(["flex", "items-center", "shrink-0"]));
    const root = title.closest("[data-page-header]") as HTMLElement;
    expect(classes(root)).toEqual(expect.arrayContaining(["flex", "items-end", "justify-between"]));
  });

  it("level={1} 渲染 h1、未給時仍為 h2，樣式相同", () => {
    const { unmount } = render(<PageHeader level={1} title="使用者" />);
    const h1 = screen.getByRole("heading", { level: 1, name: "使用者" });
    expect(classes(h1)).toEqual(expect.arrayContaining(["text-2xl", "font-normal"]));
    unmount();
    render(<PageHeader title="使用者" />);
    expect(screen.getByRole("heading", { level: 2, name: "使用者" })).toBeTruthy();
    expect(screen.queryByRole("heading", { level: 1 })).toBeNull();
  });

  it("說明與動作槽皆選配", () => {
    render(<PageHeader title="規格" />);
    const root = screen.getByRole("heading", { name: "規格" }).closest("[data-page-header]") as HTMLElement;
    expect(root.querySelector("p")).toBeNull();
    expect(root.children).toHaveLength(1);
  });

  // server-web-detail-pages design D8：詳情頁的狀態籤接在標題文字後面，標題仍是同一個 heading。
  it("給 badge 時狀態籤與標題同一列、標題仍是唯一 heading 且字級不變", () => {
    render(<PageHeader level={1} title="Member" badge={<span>有效</span>} />);
    const headings = screen.getAllByRole("heading");
    expect(headings).toHaveLength(1);
    const title = screen.getByRole("heading", { level: 1, name: "Member" });
    expect(classes(title)).toEqual(expect.arrayContaining(["text-2xl", "font-normal"]));
    const row = screen.getByText("有效").parentElement as HTMLElement;
    expect(title.parentElement).toBe(row);
    expect(classes(row)).toEqual(expect.arrayContaining(["flex", "items-center", "gap-2"]));
  });

  it("不給 badge 時標題直接在標題欄內（DOM 與加槽前相同）", () => {
    render(<PageHeader level={1} title="使用者" description="說明" />);
    const title = screen.getByRole("heading", { level: 1, name: "使用者" });
    const column = title.parentElement as HTMLElement;
    expect(classes(column)).toEqual(expect.arrayContaining(["min-w-0", "grow", "basis-64"]));
    expect(column.children).toHaveLength(2);
  });

  // server-web-list-pages design D1：放不下時動作槽換到標題下方一列，槽內控件再各自換行。
  it("根節點與動作槽帶 flex-wrap，動作槽寬度不超過整列", () => {
    render(<PageHeader title="使用者" actions={<button>邀請使用者</button>} />);
    const slot = screen.getByRole("button", { name: "邀請使用者" }).parentElement as HTMLElement;
    const root = slot.closest("[data-page-header]") as HTMLElement;
    expect(classes(root)).toContain("flex-wrap");
    expect(classes(slot)).toEqual(expect.arrayContaining(["flex-wrap", "max-w-full"]));
  });
});

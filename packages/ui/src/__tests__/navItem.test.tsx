// 共用導覽項（design D4）：desktop 專案欄與 server-web 管理面導覽同一份。
import { describe, it, expect, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { Archive } from "lucide-react";

import { NavItem } from "../components/NavItem";

const classes = (el: Element) => (el.getAttribute("class") ?? "").split(/\s+/);

describe("NavItem", () => {
  it("32px 高、8px 圓角；作用中為主色淡底主色字", () => {
    render(<NavItem icon={<Archive />} label="變更" active onClick={() => {}} />);
    const item = screen.getByRole("button", { name: "變更" });
    expect(classes(item)).toEqual(
      expect.arrayContaining(["h-8", "rounded-lg", "bg-primary/12", "text-primary", "font-medium"]),
    );
    expect(item.getAttribute("aria-current")).toBe("page");
  });

  it("非作用中為一般字色、無主色淡底", () => {
    render(<NavItem icon={<Archive />} label="規格" />);
    const item = screen.getByRole("button", { name: "規格" });
    expect(classes(item)).toContain("text-foreground");
    expect(classes(item)).not.toContain("bg-primary/12");
    expect(item.getAttribute("aria-current")).toBeNull();
  });

  it("計數徽章渲染在右端；ariaLabel 讓無障礙名稱不含數字", () => {
    const onClick = vi.fn();
    render(<NavItem icon={<Archive />} label="已封存" ariaLabel="已封存" count={7} onClick={onClick} />);
    const item = screen.getByRole("button", { name: "已封存" });
    const badge = screen.getByText("7");
    expect(item.contains(badge)).toBe(true);
    expect(classes(badge)).toEqual(expect.arrayContaining(["ml-auto", "rounded-full", "text-[11px]"]));
    fireEvent.click(item);
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it("count 為 0 仍顯示徽章（數字本身是資訊）", () => {
    render(<NavItem icon={<Archive />} label="已封存" count={0} />);
    expect(screen.getByText("0")).toBeTruthy();
  });

  it("asChild：渲染成子元素（如路由連結），圖示與標籤落在子元素內", () => {
    render(
      <NavItem asChild icon={<Archive data-testid="icon" />} label="成員" active>
        <a href="/admin/members" />
      </NavItem>,
    );
    const link = screen.getByRole("link", { name: "成員" });
    expect(link.getAttribute("href")).toBe("/admin/members");
    expect(classes(link)).toEqual(expect.arrayContaining(["h-8", "text-primary"]));
    expect(link.contains(screen.getByTestId("icon"))).toBe(true);
    expect(screen.queryByRole("button")).toBeNull();
  });
});

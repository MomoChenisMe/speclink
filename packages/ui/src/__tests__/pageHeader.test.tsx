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

  it("說明與動作槽皆選配", () => {
    render(<PageHeader title="規格" />);
    const root = screen.getByRole("heading", { name: "規格" }).closest("[data-page-header]") as HTMLElement;
    expect(root.querySelector("p")).toBeNull();
    expect(root.children).toHaveLength(1);
  });
});

// shadcn Table 原語（跨 Desktop／Server Web 共用）：語意 <table> 結構、themed 邊框、
// 窄容器橫向捲動。server-web 管理頁的多個表格改用此原語，不再手刻原生 <table>。
import { describe, it, expect } from "vitest";
import { render, screen, within } from "@testing-library/react";

import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../components/ui/table";

function sample() {
  return render(
    <Table>
      <TableCaption>使用者清單</TableCaption>
      <TableHeader>
        <TableRow>
          <TableHead>名稱</TableHead>
          <TableHead>狀態</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        <TableRow>
          <TableCell>alice</TableCell>
          <TableCell>啟用</TableCell>
        </TableRow>
      </TableBody>
    </Table>,
  );
}

describe("Table 原語", () => {
  it("渲染語意 table／columnheader／cell 角色", () => {
    sample();
    const table = screen.getByRole("table");
    expect(table).toBeTruthy();
    // <caption> 供螢幕閱讀器辨識表格用途。
    expect(within(table).getByText("使用者清單").tagName).toBe("CAPTION");
    expect(screen.getByRole("columnheader", { name: "名稱" })).toBeTruthy();
    expect(screen.getByRole("columnheader", { name: "狀態" })).toBeTruthy();
    expect(screen.getByRole("cell", { name: "alice" })).toBeTruthy();
    expect(screen.getAllByRole("row")).toHaveLength(2);
  });

  it("以 themed border-border 畫列邊框（非 Tailwind v4 預設的 currentColor 黑框）", () => {
    sample();
    // 資料列帶 border-border——手刻裸 border 會落到 currentColor（≈黑），這正是要避免的。
    const bodyRow = screen.getByRole("cell", { name: "alice" }).closest("tr");
    expect(bodyRow?.className).toContain("border-border");
  });

  it("外層容器可橫向捲動，窄視窗表格不撐破版面", () => {
    sample();
    const wrapper = screen.getByRole("table").parentElement;
    expect(wrapper?.className).toContain("overflow-x-auto");
  });

  // server-web-list-pages design D1、D7：呼叫端自己就是捲動區（列表卡）時不包外層容器——外層的
  // overflow-x-auto 會自成捲動容器，表頭的 sticky 就黏在那層、隨列表卡捲走。
  it("wrapped={false} 時不包外層捲動容器，表格直接在呼叫端的容器內", () => {
    const { container } = render(
      <div data-host>
        <Table wrapped={false}>
          <TableBody>
            <TableRow>
              <TableCell>x</TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </div>,
    );
    const table = screen.getByRole("table");
    expect(table.parentElement).toBe(container.querySelector("[data-host]"));
    expect(table.className).toContain("w-full");
  });

  // 固定表頭由元件提供（呼叫端不以 className 補底色）：捲動時表頭留在頂端、帶卡片底色蓋住底下的列。
  it("TableHeader sticky 時表頭固定在頂端並帶卡片底色；未給時沒有", () => {
    const { unmount } = render(
      <Table wrapped={false}>
        <TableHeader sticky>
          <TableRow>
            <TableHead>名稱</TableHead>
          </TableRow>
        </TableHeader>
      </Table>,
    );
    const head = screen.getByRole("table").querySelector("thead") as HTMLElement;
    expect(head.className.split(/\s+/)).toEqual(expect.arrayContaining(["sticky", "top-0", "bg-card"]));
    unmount();
    sample();
    expect((screen.getByRole("table").querySelector("thead") as HTMLElement).className).not.toContain("sticky");
  });

  it("className 併入 cn，不覆蓋基底樣式", () => {
    render(
      <Table className="custom-table">
        <TableBody>
          <TableRow>
            <TableCell>x</TableCell>
          </TableRow>
        </TableBody>
      </Table>,
    );
    const table = screen.getByRole("table");
    expect(table.className).toContain("custom-table");
    expect(table.className).toContain("w-full");
  });
});

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

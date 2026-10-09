// spec 需求「清單最新在前與換頁瀏覽」的換頁工具列（desktop-list-pages-reskin design D2：
// 自建受控元件）：左側筆數範圍＋每頁筆數下拉、右側 ‹ 頁碼 › 與跳頁輸入（總頁數 > 1
// 才出現）、pageWindow 純函式的省略號規則、total 為 0 不渲染。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, fireEvent, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { ListPager, PAGE_SIZE, pageWindow } from "../components/ListPager";

// 既有中文斷言包 I18nProvider locale zh-TW（與 specList.test 同型）。
const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

const noop = () => {};
const pageButtons = () =>
  screen.queryAllByRole("button", { name: /^第 \d+ 頁$/ }).map((b) => b.textContent);

describe("pageWindow（頁碼視窗）", () => {
  // 總頁數 ≤ 7 全列；否則恆列 1 與 M、中段為 page−1..page+1，與兩端不相鄰處插入 …。
  it.each<[number, number, Array<number | "…">]>([
    [1, 1, [1]],
    [3, 5, [1, 2, 3, 4, 5]],
    [4, 7, [1, 2, 3, 4, 5, 6, 7]],
    [1, 8, [1, 2, "…", 8]],
    [2, 8, [1, 2, 3, "…", 8]],
    [4, 8, [1, "…", 3, 4, 5, "…", 8]],
    [8, 8, [1, "…", 7, 8]],
  ])("page %i / %i → %j", (page, pageCount, expected) => {
    expect(pageWindow(page, pageCount)).toEqual(expected);
  });
});

describe("ListPager（換頁工具列）", () => {
  it("PAGE_SIZE 共用常數為每頁 20 筆", () => {
    expect(PAGE_SIZE).toBe(20);
  });

  it("total 為 0 時不渲染任何內容", () => {
    const { container } = render(
      <ListPager page={1} pageCount={1} total={0} pageSize={20} onPage={noop} onPageSize={noop} />,
    );
    expect(container.innerHTML).toBe("");
  });

  it("左側顯示筆數範圍「第 21–40 筆，共 45 筆」與每頁筆數下拉", () => {
    render(<ListPager page={2} pageCount={3} total={45} pageSize={20} onPage={noop} onPageSize={noop} />);
    expect(screen.getByText("第 21–40 筆，共 45 筆")).toBeTruthy();
    const perPage = screen.getByRole("combobox", { name: "每頁 20 個" });
    expect(perPage.textContent).toContain("每頁 20 個");
  });

  it("末頁範圍以總筆數收尾：第 41–45 筆", () => {
    render(<ListPager page={3} pageCount={3} total={45} pageSize={20} onPage={noop} onPageSize={noop} />);
    expect(screen.getByText("第 41–45 筆，共 45 筆")).toBeTruthy();
  });

  it("每頁下拉選 50 呼叫 onPageSize(50)；選項為 20／50／100", async () => {
    const user = userEvent.setup();
    const onPageSize = vi.fn();
    render(<ListPager page={1} pageCount={3} total={45} pageSize={20} onPage={noop} onPageSize={onPageSize} />);
    await user.click(screen.getByRole("combobox", { name: "每頁 20 個" }));
    const options = await screen.findAllByRole("option");
    expect(options.map((o) => o.textContent)).toEqual(["每頁 20 個", "每頁 50 個", "每頁 100 個"]);
    await user.click(screen.getByRole("option", { name: "每頁 50 個" }));
    expect(onPageSize).toHaveBeenCalledWith(50);
  });

  it("總頁數 > 1 時右側有 ‹ 頁碼 › 與跳頁輸入；作用中頁碼主色淡底、其餘無", () => {
    render(<ListPager page={2} pageCount={3} total={45} pageSize={20} onPage={noop} onPageSize={noop} />);
    expect(pageButtons()).toEqual(["1", "2", "3"]);
    const active = screen.getByRole("button", { name: "第 2 頁" });
    expect(active.className).toContain("bg-primary/12");
    expect(active.getAttribute("aria-current")).toBe("page");
    expect(screen.getByRole("button", { name: "第 1 頁" }).className).not.toContain("bg-primary/12");
    expect(screen.getByRole("spinbutton", { name: "跳到第幾頁" })).toBeTruthy();
  });

  it("第 1 頁上一頁鈕 disabled、末頁下一頁鈕 disabled；點擊以正確頁碼呼叫 onPage", () => {
    const onPage = vi.fn();
    const { rerender } = render(
      <ListPager page={1} pageCount={3} total={45} pageSize={20} onPage={onPage} onPageSize={noop} />,
    );
    const prev = () => screen.getByRole("button", { name: "上一頁" }) as HTMLButtonElement;
    const next = () => screen.getByRole("button", { name: "下一頁" }) as HTMLButtonElement;
    expect(prev().disabled).toBe(true);
    expect(next().disabled).toBe(false);
    fireEvent.click(next());
    expect(onPage).toHaveBeenCalledWith(2);
    fireEvent.click(screen.getByRole("button", { name: "第 3 頁" }));
    expect(onPage).toHaveBeenCalledWith(3);
    rerender(<ListPager page={3} pageCount={3} total={45} pageSize={20} onPage={onPage} onPageSize={noop} />);
    expect(prev().disabled).toBe(false);
    expect(next().disabled).toBe(true);
    fireEvent.click(prev());
    expect(onPage).toHaveBeenLastCalledWith(2);
  });

  it("跳頁輸入 9 按 Enter 於 3 頁時呼叫 onPage(3)（越界鉗制）；0 鉗制至 1；非數字不呼叫", () => {
    const onPage = vi.fn();
    render(<ListPager page={1} pageCount={3} total={45} pageSize={20} onPage={onPage} onPageSize={noop} />);
    const jump = screen.getByRole("spinbutton", { name: "跳到第幾頁" }) as HTMLInputElement;
    fireEvent.change(jump, { target: { value: "9" } });
    fireEvent.keyDown(jump, { key: "Enter" });
    expect(onPage).toHaveBeenLastCalledWith(3);
    fireEvent.change(jump, { target: { value: "0" } });
    fireEvent.keyDown(jump, { key: "Enter" });
    expect(onPage).toHaveBeenLastCalledWith(1);
    onPage.mockClear();
    fireEvent.change(jump, { target: { value: "abc" } });
    fireEvent.keyDown(jump, { key: "Enter" });
    fireEvent.change(jump, { target: { value: "" } });
    fireEvent.keyDown(jump, { key: "Enter" });
    expect(onPage).not.toHaveBeenCalled();
  });

  it("跳頁輸入失焦也提交：輸入 2 後 blur 呼叫 onPage(2)", () => {
    const onPage = vi.fn();
    render(<ListPager page={1} pageCount={3} total={45} pageSize={20} onPage={onPage} onPageSize={noop} />);
    const jump = screen.getByRole("spinbutton", { name: "跳到第幾頁" });
    fireEvent.change(jump, { target: { value: "2" } });
    fireEvent.blur(jump);
    expect(onPage).toHaveBeenCalledWith(2);
  });

  it("8 頁時以省略號收攏中段，省略號不是按鈕", () => {
    render(<ListPager page={4} pageCount={8} total={160} pageSize={20} onPage={noop} onPageSize={noop} />);
    expect(pageButtons()).toEqual(["1", "3", "4", "5", "8"]);
    const bar = screen.getByRole("navigation");
    expect(within(bar).getAllByText("…")).toHaveLength(2);
  });

  it("單頁（總頁數 1）時只有左側：無頁碼鈕、無 ‹ ›、無跳頁輸入", () => {
    render(<ListPager page={1} pageCount={1} total={13} pageSize={20} onPage={noop} onPageSize={noop} />);
    expect(screen.getByText("第 1–13 筆，共 13 筆")).toBeTruthy();
    expect(screen.getByRole("combobox", { name: "每頁 20 個" })).toBeTruthy();
    expect(pageButtons()).toEqual([]);
    expect(screen.queryByRole("button", { name: "上一頁" })).toBeNull();
    expect(screen.queryByRole("button", { name: "下一頁" })).toBeNull();
    expect(screen.queryByRole("spinbutton")).toBeNull();
  });
});

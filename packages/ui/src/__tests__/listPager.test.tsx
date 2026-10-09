// spec 需求「清單最新在前與換頁瀏覽」的換頁工具列（desktop-list-pages-reskin design D2：
// 自建受控元件）：左側筆數範圍＋每頁筆數下拉、右側 ‹ 頁碼 › 與跳頁輸入（總頁數 > 1
// 才出現）、pageWindow 純函式的省略號規則、total 為 0 不渲染。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, renderHook, act, screen, fireEvent, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { ListPager, PAGE_SIZE, pageWindow, usePaging } from "../components/ListPager";

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

  it("工具列自帶上緣分隔線（列表卡 footer 槽不畫線，工具列不渲染時卡底不留空線）", () => {
    const { container } = render(
      <ListPager page={1} pageCount={1} total={13} pageSize={20} onPage={noop} onPageSize={noop} />,
    );
    expect((container.firstElementChild as HTMLElement).className).toContain("border-t");
  });

  it("跳頁輸入只認 Enter：打了數字後焦點移到頁碼鈕時不提交並清空，點頁碼 2 只換到第 2 頁", () => {
    const onPage = vi.fn();
    render(<ListPager page={1} pageCount={3} total={45} pageSize={20} onPage={onPage} onPageSize={noop} />);
    const jump = screen.getByRole("spinbutton", { name: "跳到第幾頁" }) as HTMLInputElement;
    const page2 = screen.getByRole("button", { name: "第 2 頁" });
    fireEvent.change(jump, { target: { value: "3" } });
    // 滑鼠按下頁碼鈕的那一刻輸入框先失焦（relatedTarget＝頁碼鈕），點擊隨後才到。
    fireEvent.blur(jump, { relatedTarget: page2 });
    fireEvent.click(page2);
    expect(onPage.mock.calls).toEqual([[2]]);
    expect(jump.value).toBe("");
  });

  it("跳頁輸入失焦時 relatedTarget 為 null 也不提交並清空（WebKit 點按鈕不給焦點、點空白處皆同）", () => {
    const onPage = vi.fn();
    render(<ListPager page={1} pageCount={3} total={45} pageSize={20} onPage={onPage} onPageSize={noop} />);
    const jump = screen.getByRole("spinbutton", { name: "跳到第幾頁" }) as HTMLInputElement;
    fireEvent.change(jump, { target: { value: "2" } });
    fireEvent.blur(jump);
    expect(onPage).not.toHaveBeenCalled();
    expect(jump.value).toBe("");
  });

  it("8 頁時以省略號收攏中段，省略號不是按鈕", () => {
    render(<ListPager page={4} pageCount={8} total={160} pageSize={20} onPage={noop} onPageSize={noop} />);
    expect(pageButtons()).toEqual(["1", "3", "4", "5", "8"]);
    const bar = screen.getByRole("navigation");
    expect(within(bar).getAllByText("…")).toHaveLength(2);
  });

  it("單頁（總頁數 1）時只有左側：無頁碼鈕、無 « ‹ › »、無跳頁輸入", () => {
    render(<ListPager page={1} pageCount={1} total={13} pageSize={20} onPage={noop} onPageSize={noop} />);
    expect(screen.getByText("第 1–13 筆，共 13 筆")).toBeTruthy();
    expect(screen.getByRole("combobox", { name: "每頁 20 個" })).toBeTruthy();
    expect(pageButtons()).toEqual([]);
    expect(screen.queryByRole("button", { name: "第一頁" })).toBeNull();
    expect(screen.queryByRole("button", { name: "上一頁" })).toBeNull();
    expect(screen.queryByRole("button", { name: "下一頁" })).toBeNull();
    expect(screen.queryByRole("button", { name: "最後一頁" })).toBeNull();
    expect(screen.queryByRole("spinbutton")).toBeNull();
  });

  // spec scenario「第一頁與最後一頁鈕」：« 在 ‹ 左側、» 在 › 右側，首末頁時停用。
  it("« 第一頁鈕與 » 最後一頁鈕：位於 ‹ › 外側；第 1 頁時 « 停用、末頁時 » 停用", () => {
    const { rerender } = render(
      <ListPager page={1} pageCount={3} total={45} pageSize={20} onPage={noop} onPageSize={noop} />,
    );
    const first = () => screen.getByRole("button", { name: "第一頁" }) as HTMLButtonElement;
    const last = () => screen.getByRole("button", { name: "最後一頁" }) as HTMLButtonElement;
    const nav = screen.getByRole("navigation");
    const order = within(nav)
      .getAllByRole("button")
      .map((b) => b.getAttribute("aria-label"));
    expect(order).toEqual(["第一頁", "上一頁", "第 1 頁", "第 2 頁", "第 3 頁", "下一頁", "最後一頁"]);
    expect(first().disabled).toBe(true);
    expect(last().disabled).toBe(false);
    rerender(<ListPager page={3} pageCount={3} total={45} pageSize={20} onPage={noop} onPageSize={noop} />);
    expect(first().disabled).toBe(false);
    expect(last().disabled).toBe(true);
  });

  it("第 2 頁點 » 呼叫 onPage(3)、點 « 呼叫 onPage(1)", () => {
    const onPage = vi.fn();
    render(<ListPager page={2} pageCount={3} total={45} pageSize={20} onPage={onPage} onPageSize={noop} />);
    fireEvent.click(screen.getByRole("button", { name: "最後一頁" }));
    expect(onPage).toHaveBeenLastCalledWith(3);
    fireEvent.click(screen.getByRole("button", { name: "第一頁" }));
    expect(onPage).toHaveBeenLastCalledWith(1);
  });
});

describe("usePaging（清單換頁狀態）", () => {
  const ITEMS = Array.from({ length: 45 }, (_, i) => i);

  it("切出目前頁；換頁與改每頁筆數呼叫 onMove；改每頁筆數把頁碼鉗制寫回；清單縮短時頁碼派生鉗制", () => {
    const onMove = vi.fn();
    const { result, rerender } = renderHook(({ list }) => usePaging(list, { onMove }), {
      initialProps: { list: ITEMS },
    });
    expect(result.current.pager).toMatchObject({ page: 1, pageCount: 3, total: 45, pageSize: PAGE_SIZE });
    act(() => result.current.pager.onPage(3));
    expect(result.current.pageItems).toEqual(ITEMS.slice(40, 45));
    expect(onMove).toHaveBeenCalledTimes(1);
    act(() => result.current.pager.onPageSize(50));
    expect(result.current.pager).toMatchObject({ page: 1, pageCount: 1, pageSize: 50 });
    expect(onMove).toHaveBeenCalledTimes(2);
    // 鉗制已寫回：改回 20 不彈回舊的第 3 頁。
    act(() => result.current.pager.onPageSize(20));
    expect(result.current.pager.page).toBe(1);
    // 清單縮短：停在第 3 頁時剩 25 筆 → 派生為第 2 頁。
    act(() => result.current.pager.onPage(3));
    rerender({ list: ITEMS.slice(0, 25) });
    expect(result.current.pager).toMatchObject({ page: 2, pageCount: 2, total: 25 });
    // setPage 只改頁碼、不捲動（搜尋重設與聚焦列用）。
    onMove.mockClear();
    act(() => result.current.setPage(1));
    expect(result.current.pager.page).toBe(1);
    expect(onMove).not.toHaveBeenCalled();
  });

  it("值與回呼都給時受控：以 prop 為準，改每頁筆數呼叫回呼", () => {
    const onPageSizeChange = vi.fn();
    const { result } = renderHook(() =>
      usePaging(ITEMS, { pageSize: 50, onPageSizeChange, onMove: () => {} }),
    );
    expect(result.current.pager.pageSize).toBe(50);
    expect(result.current.pageItems).toHaveLength(45);
    act(() => result.current.pager.onPageSize(100));
    expect(onPageSizeChange).toHaveBeenCalledWith(100);
  });
});

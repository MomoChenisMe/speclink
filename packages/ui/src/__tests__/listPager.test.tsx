// spec 需求「清單最新在前與換頁瀏覽」的換頁控制列行為（design D2：自建 ListPager
// 受控元件，不採 shadcn Pagination）：pageCount ≤ 1 不渲染、頁界 disabled、
// 點按以正確頁碼呼叫 onPage、「第 N／M 頁」文案。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, fireEvent } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { ListPager, PAGE_SIZE } from "../components/ListPager";

// 既有中文斷言包 I18nProvider locale zh-TW（與 specList.test 同型）。
const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

describe("ListPager（換頁控制列）", () => {
  it("PAGE_SIZE 共用常數為每頁 20 筆", () => {
    expect(PAGE_SIZE).toBe(20);
  });

  it("pageCount ≤ 1 時不渲染任何內容", () => {
    const single = render(<ListPager page={1} pageCount={1} onPage={() => {}} />);
    expect(single.container.innerHTML).toBe("");
    const zero = render(<ListPager page={1} pageCount={0} onPage={() => {}} />);
    expect(zero.container.innerHTML).toBe("");
  });

  it("顯示「第 N／M 頁」；第 1 頁上一頁鈕 disabled、末頁下一頁鈕 disabled", () => {
    const { rerender } = render(<ListPager page={1} pageCount={3} onPage={() => {}} />);
    expect(screen.getByText("第 1／3 頁")).toBeTruthy();
    expect((screen.getByRole("button", { name: "上一頁" }) as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByRole("button", { name: "下一頁" }) as HTMLButtonElement).disabled).toBe(false);
    // 末頁：下一頁 disabled、上一頁可用。
    rerender(<ListPager page={3} pageCount={3} onPage={() => {}} />);
    expect(screen.getByText("第 3／3 頁")).toBeTruthy();
    expect((screen.getByRole("button", { name: "上一頁" }) as HTMLButtonElement).disabled).toBe(false);
    expect((screen.getByRole("button", { name: "下一頁" }) as HTMLButtonElement).disabled).toBe(true);
  });

  it("點上一頁／下一頁以正確頁碼呼叫 onPage（受控：自身不持頁碼）", () => {
    const onPage = vi.fn();
    render(<ListPager page={2} pageCount={3} onPage={onPage} />);
    fireEvent.click(screen.getByRole("button", { name: "上一頁" }));
    expect(onPage).toHaveBeenCalledWith(1);
    fireEvent.click(screen.getByRole("button", { name: "下一頁" }));
    expect(onPage).toHaveBeenCalledWith(3);
    expect(onPage).toHaveBeenCalledTimes(2);
  });
});

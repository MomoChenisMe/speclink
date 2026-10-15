// 詳情頁頂列右端的 ‹ n／m ›（server-web-detail-pages design D3：由 desktop 搬進共用元件庫，
// desktop 與 server 後台共用）。字串改由內建字典提供，所以這裡不帶 app 字典。
import { describe, expect, it, vi, afterEach } from "vitest";
import { cleanup, fireEvent, render as rtlRender, screen } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { DetailNav } from "../components/DetailNav";
import { I18nProvider } from "../i18n";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

afterEach(cleanup);

const prev = () => screen.getByRole("button", { name: "上一個" }) as HTMLButtonElement;
const next = () => screen.getByRole("button", { name: "下一個" }) as HTMLButtonElement;

describe("DetailNav", () => {
  it("2/3：兩鈕可按、序位文字為「2 / 3」且為等寬 11px", () => {
    render(<DetailNav index={1} total={3} onStep={() => {}} />);
    expect(prev().disabled).toBe(false);
    expect(next().disabled).toBe(false);
    const pos = screen.getByText("2 / 3");
    expect(pos.className).toContain("font-mono");
    expect(pos.className).toContain("text-[11px]");
    expect(pos.getAttribute("aria-label")).toBe("第 2 個，共 3 個");
  });

  it("1/3：‹ 停用、› 可按", () => {
    render(<DetailNav index={0} total={3} onStep={() => {}} />);
    expect(prev().disabled).toBe(true);
    expect(next().disabled).toBe(false);
  });

  it("3/3：› 停用、‹ 可按（不循環）", () => {
    render(<DetailNav index={2} total={3} onStep={() => {}} />);
    expect(prev().disabled).toBe(false);
    expect(next().disabled).toBe(true);
  });

  it("目前項目不在序列（index -1）或序列為空時整組缺席", () => {
    const { container, unmount } = render(<DetailNav index={-1} total={3} onStep={() => {}} />);
    expect(container.innerHTML).toBe("");
    unmount();
    const second = render(<DetailNav index={0} total={0} onStep={() => {}} />);
    expect(second.container.innerHTML).toBe("");
  });

  it("點 ‹ 呼叫 onStep(-1)、點 › 呼叫 onStep(1)", () => {
    const onStep = vi.fn();
    render(<DetailNav index={1} total={3} onStep={onStep} />);
    fireEvent.click(prev());
    fireEvent.click(next());
    expect(onStep.mock.calls).toEqual([[-1], [1]]);
  });
});

// 共用列、列表卡與搜尋框（desktop-list-pages-reskin design D1、D3）：規格頁與已封存
// 頁三種列共用 ListRow（整列可點、標題＋hover 複製鈕、描述列、meta 槽、› 收尾、
// Enter／Space 觸發、複製不冒泡）；ListCard 的 header／footer 落點與內容捲動容器；
// SearchField 為全圓 280px 搜尋框。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, fireEvent, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { ListRow } from "../components/ListRow";
import { ListCard } from "../components/ListCard";
import { SearchField } from "../components/SearchField";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

const row = () => document.querySelector("[data-row]") as HTMLElement;

describe("ListRow（共用列）", () => {
  it("渲染等寬標題、描述列、meta 槽、leading 槽與 › 收尾；整列可點且可聚焦", () => {
    const onClick = vi.fn();
    render(
      <ListRow
        data-row="a"
        title="desktop-app"
        description="桌面 app 的行為契約。"
        leading={<span data-lead>2026-07-08</span>}
        meta={<span data-meta>7</span>}
        onClick={onClick}
      />,
    );
    const el = row();
    expect(el.getAttribute("role")).toBe("button");
    expect(el.getAttribute("tabindex")).toBe("0");
    // 標題等寬強調、一行截斷；與複製鈕同屬標題群組。
    const title = within(el).getByText("desktop-app");
    expect(title.className).toContain("font-mono");
    expect(title.className).toContain("truncate");
    expect(el.querySelector("[data-title-group]")!.contains(title)).toBe(true);
    // 描述列一行截斷。
    const desc = el.querySelector("[data-desc]") as HTMLElement;
    expect(desc.textContent).toBe("桌面 app 的行為契約。");
    expect(desc.className).toContain("truncate");
    // leading 在最左、meta 在右端、› 收尾。
    expect(el.firstElementChild!.querySelector("[data-lead]")).toBeTruthy();
    expect(el.querySelector("[data-meta]")).toBeTruthy();
    expect(el.lastElementChild!.classList.contains("lucide-chevron-right")).toBe(true);
    fireEvent.click(el);
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it("無描述時描述列缺席、列退回單行", () => {
    render(<ListRow data-row="b" title="bare" onClick={() => {}} />);
    expect(row().querySelector("[data-desc]")).toBeNull();
  });

  it("Enter 與 Space 觸發 onClick；其他鍵不觸發", () => {
    const onClick = vi.fn();
    render(<ListRow data-row="c" title="k" onClick={onClick} />);
    fireEvent.keyDown(row(), { key: "Enter" });
    fireEvent.keyDown(row(), { key: " " });
    fireEvent.keyDown(row(), { key: "a" });
    expect(onClick).toHaveBeenCalledTimes(2);
  });

  it("複製鈕 hover 才顯現，點擊寫入 copyValue 且不觸發 onClick；Enter 在複製鈕上也不開列", () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", { value: { writeText }, configurable: true });
    const onClick = vi.fn();
    render(
      <ListRow data-row="d" title="old-topic" copyValue="old-topic" copyLabel="複製 slug" onClick={onClick} />,
    );
    const copyBtn = within(row()).getByLabelText("複製 slug");
    expect(copyBtn.className).toContain("opacity-0");
    expect(copyBtn.className).toContain("group-hover:opacity-100");
    fireEvent.click(copyBtn);
    expect(writeText).toHaveBeenCalledWith("old-topic");
    fireEvent.keyDown(copyBtn, { key: "Enter" });
    expect(onClick).not.toHaveBeenCalled();
  });

  it("未給 copyValue 時無複製鈕；children 落在標題旁", () => {
    render(
      <ListRow data-row="e" title="t" onClick={() => {}}>
        <span data-stamp>章</span>
      </ListRow>,
    );
    expect(within(row()).queryByRole("button", { name: /複製/ })).toBeNull();
    expect(row().querySelector("[data-title-group] [data-stamp]")).toBeTruthy();
  });
});

describe("ListCard（列表卡）", () => {
  it("header 在頂部、children 在 data-list-scroll 捲動容器內、footer 在捲動容器外的底部", () => {
    render(
      <ListCard header={<div data-head>頭</div>} footer={<div data-foot>腳</div>}>
        <div data-body>列</div>
      </ListCard>,
    );
    const scroll = document.querySelector("[data-list-scroll]") as HTMLElement;
    expect(scroll.className).toContain("overflow-y-auto");
    expect(scroll.className).toContain("flex-1");
    expect(scroll.className).toContain("min-h-0");
    expect(scroll.querySelector("[data-body]")).toBeTruthy();
    const head = document.querySelector("[data-head]") as HTMLElement;
    const foot = document.querySelector("[data-foot]") as HTMLElement;
    expect(scroll.contains(head)).toBe(false);
    expect(scroll.contains(foot)).toBe(false);
    // 文件順序：header → 捲動容器 → footer。
    expect(head.compareDocumentPosition(scroll) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(scroll.compareDocumentPosition(foot) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    // 有 header 時內容卡以 rounded-t-none border-t-0 接在分頁列下（Tabs card variant 接法）。
    const card = document.querySelector("[data-list-card]") as HTMLElement;
    expect(card.className).toContain("rounded-t-none");
    expect(card.className).toContain("border-t-0");
    expect(card.contains(head)).toBe(false);
    expect(card.contains(foot)).toBe(true);
  });

  it("無 header 時內容卡維持完整圓角；無 footer 時底列缺席", () => {
    render(
      <ListCard>
        <div>列</div>
      </ListCard>,
    );
    const card = document.querySelector("[data-list-card]") as HTMLElement;
    expect(card.className).toContain("rounded-2xl");
    expect(card.className).not.toContain("rounded-t-none");
    expect(document.querySelector("[data-list-footer]")).toBeNull();
  });

  it("scrollRef 指向捲動容器", () => {
    const ref = { current: null as HTMLDivElement | null };
    render(
      <ListCard scrollRef={ref}>
        <div>列</div>
      </ListCard>,
    );
    expect(ref.current).toBe(document.querySelector("[data-list-scroll]"));
  });
});

describe("SearchField（全圓搜尋框）", () => {
  it("輸入框 class 含 rounded-full 與 280px 寬，帶搜尋圖示；輸入以字串回呼 onChange", () => {
    const onChange = vi.fn();
    render(<SearchField value="" onChange={onChange} placeholder="搜尋規格…" />);
    const input = screen.getByPlaceholderText("搜尋規格…") as HTMLInputElement;
    expect(input.className).toContain("rounded-full");
    expect(input.className).toContain("w-[280px]");
    expect(input.parentElement!.querySelector(".lucide-search")).toBeTruthy();
    fireEvent.change(input, { target: { value: "desk" } });
    expect(onChange).toHaveBeenCalledWith("desk");
  });
});

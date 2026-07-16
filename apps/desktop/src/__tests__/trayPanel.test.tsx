// 面板樣式呈現層（spec「面板樣式（macOS）」；design D5）：以主視窗推送的
// TraySnapshot 薄渲染——分區與內容和原生選單同源；jsdom 只測結構與回呼，
// 原生質感（不搶焦點、貼齊、失焦收合）由 4.4 真視窗手動驗證。
import { describe, it, expect, vi } from "vitest";
import { act, render as rtlRender, screen, fireEvent, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider, type ChangeItem } from "@speclink/ui";

import { TrayPanel } from "../panel/TrayPanel";
import { APP_MESSAGES } from "../i18n/messages";
import type { TraySnapshot } from "../tray";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);
const render = (ui: ReactElement) => rtlRender(ui, { wrapper: zhWrapper });

function change(over: Partial<ChangeItem> & { name: string }): ChangeItem {
  return { status: "", totalTasks: 0, completedTasks: 0, ...over };
}

function snapshot(over: Partial<TraySnapshot> = {}): TraySnapshot {
  return {
    tabs: [
      { root: "/proj/one", name: "one" },
      { root: "/proj/two", name: "two" },
    ],
    activeRoot: "/proj/one",
    changes: [
      change({ name: "prop", totalTasks: 5, completedTasks: 0 }), // proposed
      change({ name: "inprog", totalTasks: 12, completedTasks: 3 }), // in-progress
      change({ name: "rdy", totalTasks: 4, completedTasks: 4 }), // ready
    ],
    discussions: [{ slug: "d1", topic: "討論一", promoted: false }],
    ...over,
  };
}

function renderPanel(over: Partial<Parameters<typeof TrayPanel>[0]> = {}) {
  const handlers = {
    onOpenChange: vi.fn(),
    onOpenDiscussion: vi.fn(),
    onOpenProject: vi.fn(),
    onOpenApp: vi.fn(),
    onCopy: vi.fn(),
  };
  render(<TrayPanel snapshot={snapshot()} {...handlers} {...over} />);
  return handlers;
}

describe("TrayPanel 渲染（與原生選單同源的分區內容）", () => {
  it("依生命週期分區渲染變更（含進度數）與討論清單（slug 為題、topic 為描述）", () => {
    renderPanel();
    // 生命週期分區 header
    for (const label of ["提案中", "進行中", "已就緒", "討論"]) {
      expect(screen.getByText(label)).toBeTruthy();
    }
    // 變更列：名稱與 n/m 進度數
    const inprog = screen.getByTestId("panel-change-inprog");
    expect(within(inprog).getByText("inprog")).toBeTruthy();
    expect(within(inprog).getByText("3/12")).toBeTruthy();
    // 討論列：slug 為題、topic 為描述（識別錨點慣例）
    const disc = screen.getByTestId("panel-discussion-d1");
    expect(within(disc).getByText("d1")).toBeTruthy();
    expect(within(disc).getByText("討論一")).toBeTruthy();
  });

  it("專案區列出分頁且作用中標示、點非作用中專案回呼切換", () => {
    const h = renderPanel();
    const one = screen.getByTestId("panel-project-/proj/one");
    const two = screen.getByTestId("panel-project-/proj/two");
    expect(one.getAttribute("data-active")).toBe("true");
    expect(two.getAttribute("data-active")).toBe("false");
    fireEvent.click(two);
    expect(h.onOpenProject).toHaveBeenCalledWith("/proj/two");
  });

  it("無變更顯示空狀態、無討論顯示「討論 0」", () => {
    renderPanel({ snapshot: snapshot({ changes: [], discussions: [] }) });
    expect(screen.getByText("尚無進行中變更")).toBeTruthy();
    expect(screen.getByText("討論 0")).toBeTruthy();
  });

  it("討論分流：已轉出討論列於「已轉出」分區（討論中列於「討論」分區）", () => {
    renderPanel({
      snapshot: snapshot({
        discussions: [
          { slug: "open-d", topic: "討論中的", promoted: false },
          { slug: "prom-d", topic: "已轉出的", promoted: true },
        ],
      }),
    });
    expect(screen.getByText("討論")).toBeTruthy();
    expect(screen.getByText("已轉出")).toBeTruthy();
    expect(screen.getByTestId("panel-discussion-open-d")).toBeTruthy();
    expect(screen.getByTestId("panel-discussion-prom-d")).toBeTruthy();
    // 已轉出分區在討論分區之後
    const html = document.body.innerHTML;
    expect(html.indexOf("open-d")).toBeLessThan(html.indexOf("已轉出"));
  });

  it("無已轉出討論時不出現「已轉出」分區", () => {
    renderPanel();
    expect(screen.queryByText("已轉出")).toBeNull();
  });

  it("分區逾 5 筆：面板顯示前 5＋「還有 N 個…」，點擊展開、再點收合", () => {
    const eight = Array.from({ length: 8 }, (_, i) =>
      change({ name: `c${i}`, totalTasks: 2, completedTasks: 1 }),
    );
    renderPanel({ snapshot: snapshot({ changes: eight, discussions: [] }) });
    expect(screen.getByTestId("panel-change-c4")).toBeTruthy();
    expect(screen.queryByTestId("panel-change-c5")).toBeNull();
    fireEvent.click(screen.getByText("還有 3 個…"));
    expect(screen.getByTestId("panel-change-c7")).toBeTruthy();
    fireEvent.click(screen.getByText("收合"));
    expect(screen.queryByTestId("panel-change-c5")).toBeNull();
  });

  it("五筆以下不出現溢出列", () => {
    renderPanel();
    expect(screen.queryByText(/還有 \d+ 個…/)).toBeNull();
  });
});

describe("TrayPanel 互動（開啟與 hover 複製）", () => {
  it("點擊變更列本體發出開啟事件（攜帶 change name）", () => {
    const h = renderPanel();
    fireEvent.click(screen.getByTestId("panel-change-inprog"));
    expect(h.onOpenChange).toHaveBeenCalledWith("inprog");
  });

  it("點擊討論列本體發出開啟事件（攜帶 slug）", () => {
    const h = renderPanel();
    fireEvent.click(screen.getByTestId("panel-discussion-d1"));
    expect(h.onOpenDiscussion).toHaveBeenCalledWith("d1");
  });

  it("變更列複製鈕回呼 name（不含進度字元）且不觸發開啟", () => {
    const h = renderPanel();
    const row = screen.getByTestId("panel-change-inprog");
    fireEvent.click(within(row).getByRole("button", { name: "複製名稱" }));
    expect(h.onCopy).toHaveBeenCalledWith("inprog");
    expect(h.onOpenChange).not.toHaveBeenCalled();
  });

  it("討論列複製鈕回呼 slug 且不觸發開啟", () => {
    const h = renderPanel();
    const row = screen.getByTestId("panel-discussion-d1");
    fireEvent.click(within(row).getByRole("button", { name: "複製 slug" }));
    expect(h.onCopy).toHaveBeenCalledWith("d1");
    expect(h.onOpenDiscussion).not.toHaveBeenCalled();
  });

  it("底部「開啟 Speclink」回呼開啟主視窗", () => {
    const h = renderPanel();
    fireEvent.click(screen.getByText("開啟 Speclink"));
    expect(h.onOpenApp).toHaveBeenCalled();
  });

  it("複製回饋：點擊後圖示短暫轉勾號、約 1.2 秒後復原（看板 copied 同模式）", () => {
    vi.useFakeTimers();
    const h = renderPanel();
    const row = screen.getByTestId("panel-discussion-d1");
    const btn = within(row).getByRole("button", { name: "複製 slug" });
    expect(btn.querySelector("svg.lucide-check")).toBeNull();
    fireEvent.click(btn);
    expect(h.onCopy).toHaveBeenCalledWith("d1");
    expect(btn.querySelector("svg.lucide-check")).toBeTruthy();
    expect(btn.querySelector("svg.lucide-copy")).toBeNull();
    act(() => {
      vi.advanceTimersByTime(1300);
    });
    expect(btn.querySelector("svg.lucide-check")).toBeNull();
    expect(btn.querySelector("svg.lucide-copy")).toBeTruthy();
    vi.useRealTimers();
  });
});

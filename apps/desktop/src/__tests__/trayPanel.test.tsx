// 面板樣式呈現層（spec「面板樣式（macOS）」；design D5）：以主視窗推送的
// TraySnapshot 薄渲染——分區與內容和原生選單同源；jsdom 只測結構與回呼，
// 原生質感（不搶焦點、貼齊、失焦收合）由 4.4 真視窗手動驗證。
import { describe, it, expect, vi } from "vitest";
import { act, render as rtlRender, screen, fireEvent, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider, STAGE_BADGE, STAGE_BAR, STAGE_ICON, type ChangeItem } from "@speclink/ui";

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
      { key: "local:/proj/one", root: "/proj/one", name: "one" },
      { key: "local:/proj/two", root: "/proj/two", name: "two" },
    ],
    activeKey: "local:/proj/one",
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
    onOpenSettings: vi.fn(),
    onQuit: vi.fn(),
    onCopy: vi.fn(),
    onAddProject: vi.fn(),
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
    const one = screen.getByTestId("panel-project-local:/proj/one");
    const two = screen.getByTestId("panel-project-local:/proj/two");
    expect(one.getAttribute("data-active")).toBe("true");
    expect(two.getAttribute("data-active")).toBe("false");
    fireEvent.click(two);
    expect(h.onOpenProject).toHaveBeenCalledWith("/proj/two");
  });

  it("全無變更時三個生命週期分區常駐：各帶計數 0 空狀態卡、無佔位卡（D8 同構）", () => {
    renderPanel({ snapshot: snapshot({ changes: [], discussions: [] }) });
    // 佔位卡不再存在（spec「面板樣式（macOS）」常駐分區行為）
    expect(screen.queryByTestId("panel-empty-changes")).toBeNull();
    expect(screen.queryByText("尚無進行中變更")).toBeNull();
    for (const id of ["panel-section-proposed", "panel-section-in-progress", "panel-section-ready"]) {
      const card = screen.getByTestId(id);
      expect(card.className.split(/\s+/)).toEqual(expect.arrayContaining(["min-h-12", "justify-center"]));
      expect(within(card).getByTestId("panel-section-count").textContent).toBe("0");
    }
    const disc = screen.getByTestId("panel-section-discussions");
    expect(disc.className.split(/\s+/)).toEqual(expect.arrayContaining(["min-h-12", "justify-center"]));
    expect(within(disc).getByText("討論")).toBeTruthy();
    expect(within(disc).getByTestId("panel-section-count").textContent).toBe("0");
  });

  it("部分有資料時空階段分區仍常駐：計數 0/1/0、順序固定提案中→進行中→已就緒", () => {
    renderPanel({
      snapshot: snapshot({
        changes: [change({ name: "solo", totalTasks: 12, completedTasks: 3 })], // in-progress
        discussions: [],
      }),
    });
    const ids = ["panel-section-proposed", "panel-section-in-progress", "panel-section-ready"];
    const counts = ids.map(
      (id) => within(screen.getByTestId(id)).getByTestId("panel-section-count").textContent,
    );
    expect(counts).toEqual(["0", "1", "0"]);
    expect(within(screen.getByTestId("panel-section-in-progress")).getByTestId("panel-change-solo")).toBeTruthy();
    const [proposed, inProgress, ready] = ids.map((id) => screen.getByTestId(id));
    expect(proposed.compareDocumentPosition(inProgress) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(inProgress.compareDocumentPosition(ready) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
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

  it("區塊順序：討論（＋已轉出）區塊位於生命週期分區之前、「開啟 Speclink」最末", () => {
    renderPanel({
      snapshot: snapshot({
        discussions: [
          { slug: "open-d", topic: "討論中的", promoted: false },
          { slug: "prom-d", topic: "已轉出的", promoted: true },
        ],
      }),
    });
    const follows = (a: Element, b: Element) =>
      (a.compareDocumentPosition(b) & Node.DOCUMENT_POSITION_FOLLOWING) !== 0;
    const discussions = screen.getByTestId("panel-section-discussions");
    const promoted = screen.getByTestId("panel-section-promoted");
    const proposed = screen.getByTestId("panel-section-proposed");
    const ready = screen.getByTestId("panel-section-ready");
    const open = screen.getByText("開啟 Speclink");
    expect(follows(discussions, promoted)).toBe(true);
    expect(follows(promoted, proposed)).toBe(true);
    expect(follows(ready, open)).toBe(true);
  });

  it("分割線恰三條：tab 條後、討論區塊後、生命週期區塊後；分區卡之間無分割線", () => {
    renderPanel();
    const dividers = screen.getAllByTestId("panel-divider");
    expect(dividers.length).toBe(3);
    const follows = (a: Element, b: Element) =>
      (a.compareDocumentPosition(b) & Node.DOCUMENT_POSITION_FOLLOWING) !== 0;
    const tabs = screen.getByTestId("panel-project-tabs");
    const discussions = screen.getByTestId("panel-section-discussions");
    const proposed = screen.getByTestId("panel-section-proposed");
    const ready = screen.getByTestId("panel-section-ready");
    const open = screen.getByText("開啟 Speclink");
    expect(follows(tabs, dividers[0])).toBe(true);
    expect(follows(dividers[0], discussions)).toBe(true);
    expect(follows(discussions, dividers[1])).toBe(true);
    expect(follows(dividers[1], proposed)).toBe(true);
    expect(follows(ready, dividers[2])).toBe(true);
    expect(follows(dividers[2], open)).toBe(true);
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

describe("TrayPanel 專案 tab 條（spec「面板樣式（macOS）」；design D1）", () => {
  it("每個 tab 含專案名首字母 avatar 與專案名", () => {
    renderPanel();
    const one = screen.getByTestId("panel-project-local:/proj/one");
    expect(within(one).getByText("O")).toBeTruthy();
    expect(within(one).getByText("one")).toBeTruthy();
    const two = screen.getByTestId("panel-project-local:/proj/two");
    expect(within(two).getByText("T")).toBeTruthy();
    expect(within(two).getByText("two")).toBeTruthy();
  });

  it("作用中 tab 實心主色底、非作用中無實心底", () => {
    renderPanel();
    const one = screen.getByTestId("panel-project-local:/proj/one");
    const two = screen.getByTestId("panel-project-local:/proj/two");
    expect(one.className.split(/\s+/)).toContain("bg-primary");
    expect(two.className.split(/\s+/)).not.toContain("bg-primary");
  });

  it("tab 條尾端有「加入專案」動作項，點擊觸發 onAddProject 而非 onOpenProject（D7）", () => {
    const h = renderPanel();
    const strip = screen.getByTestId("panel-project-tabs");
    const add = within(strip).getByTestId("panel-add-project");
    expect(add.getAttribute("title")).toBe("加入專案");
    fireEvent.click(add);
    expect(h.onAddProject).toHaveBeenCalled();
    expect(h.onOpenProject).not.toHaveBeenCalled();
  });

  it("tab 容器橫向捲動且隱藏捲軸", () => {
    renderPanel();
    const strip = screen.getByTestId("panel-project-tabs");
    const classes = strip.className.split(/\s+/);
    expect(classes).toContain("overflow-x-auto");
    expect(classes).toContain("[scrollbar-width:none]");
    expect(classes).toContain("[&::-webkit-scrollbar]:hidden");
  });
});

describe("TrayPanel 分區卡片化與主色（spec「面板樣式（macOS）」；design D2／D3）", () => {
  const discBoth = [
    { slug: "open-d", topic: "討論中的", promoted: false },
    { slug: "prom-d", topic: "已轉出的", promoted: true },
  ];

  it("生命週期與討論分區各自為圓角半透明卡片、無 hr 分隔線", () => {
    renderPanel({ snapshot: snapshot({ discussions: discBoth }) });
    for (const id of [
      "panel-section-proposed",
      "panel-section-in-progress",
      "panel-section-ready",
      "panel-section-discussions",
      "panel-section-promoted",
    ]) {
      const classes = screen.getByTestId(id).className.split(/\s+/);
      expect(classes).toContain("rounded-lg");
      expect(classes).toContain("bg-foreground/5");
    }
    expect(document.querySelector("hr")).toBeNull();
  });

  it("分區標題圖示帶主色（生命週期依階段階梯、討論分區主色）", () => {
    renderPanel({ snapshot: snapshot({ discussions: discBoth }) });
    const iconClasses = (id: string) =>
      (screen.getByTestId(id).querySelector("svg")?.getAttribute("class") ?? "").split(/\s+/);
    expect(iconClasses("panel-section-proposed")).toContain(STAGE_ICON.proposed);
    expect(iconClasses("panel-section-in-progress")).toContain(STAGE_ICON["in-progress"]);
    expect(iconClasses("panel-section-ready")).toContain(STAGE_ICON.ready);
    expect(iconClasses("panel-section-discussions")).toContain("text-primary");
    expect(iconClasses("panel-section-promoted")).toContain("text-primary");
  });

  it("根容器以毛玻璃同半徑圓角裁切（wash 不得畫出 vibrancy 圓角外）", () => {
    renderPanel();
    const root = screen.getByTestId("panel-root");
    expect(root.className.split(/\s+/)).toContain("rounded-[13px]");
  });

  it("進度條填色依階段套用共用色階（STAGE_BAR）", () => {
    renderPanel();
    const fillClasses = (name: string) =>
      (
        screen
          .getByTestId(`panel-change-${name}`)
          .querySelector('[data-testid="panel-progress-fill"]')?.className ?? ""
      ).split(/\s+/);
    expect(fillClasses("prop")).toContain(STAGE_BAR.proposed);
    expect(fillClasses("inprog")).toContain(STAGE_BAR["in-progress"]);
    expect(fillClasses("rdy")).toContain(STAGE_BAR.ready);
  });
});

describe("TrayPanel 分區計數（spec「面板樣式（macOS）」；design D8）", () => {
  it("各分區標題帶項目計數徽章（STAGE_BADGE 同語彙；討論分區取看板討論欄同款）", () => {
    renderPanel({
      snapshot: snapshot({
        discussions: [
          { slug: "open-d", topic: "討論中的", promoted: false },
          { slug: "prom-d", topic: "已轉出的", promoted: true },
        ],
      }),
    });
    const count = (id: string) =>
      screen.getByTestId(id).querySelector('[data-testid="panel-section-count"]')!;
    for (const id of [
      "panel-section-proposed",
      "panel-section-in-progress",
      "panel-section-ready",
      "panel-section-discussions",
      "panel-section-promoted",
    ]) {
      expect(count(id).textContent).toBe("1");
    }
    expect(count("panel-section-proposed").className.split(/\s+/)).toEqual(
      expect.arrayContaining(STAGE_BADGE.proposed.split(/\s+/)),
    );
    expect(count("panel-section-ready").className.split(/\s+/)).toEqual(
      expect.arrayContaining(STAGE_BADGE.ready.split(/\s+/)),
    );
    expect(count("panel-section-discussions").className.split(/\s+/)).toEqual(
      expect.arrayContaining(STAGE_BADGE.proposed.split(/\s+/)),
    );
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

  it("動作區塊依序渲染「開啟 Speclink」「設定」「結束」，點擊各列觸發對應回呼", () => {
    const h = renderPanel();
    const follows = (a: Element, b: Element) =>
      (a.compareDocumentPosition(b) & Node.DOCUMENT_POSITION_FOLLOWING) !== 0;
    const open = screen.getByText("開啟 Speclink");
    const settings = screen.getByText("設定");
    const quit = screen.getByText("結束");
    expect(follows(open, settings)).toBe(true);
    expect(follows(settings, quit)).toBe(true);
    fireEvent.click(settings);
    expect(h.onOpenSettings).toHaveBeenCalled();
    expect(h.onOpenApp).not.toHaveBeenCalled();
    fireEvent.click(quit);
    expect(h.onQuit).toHaveBeenCalled();
  });

  it("複製鈕退出 tab 順序（tabIndex=-1）且點擊仍複製（design D4 焦點修復）", () => {
    const h = renderPanel();
    const discBtn = within(screen.getByTestId("panel-discussion-d1")).getByRole("button", {
      name: "複製 slug",
    });
    const changeBtn = within(screen.getByTestId("panel-change-inprog")).getByRole("button", {
      name: "複製名稱",
    });
    expect(discBtn.getAttribute("tabindex")).toBe("-1");
    expect(changeBtn.getAttribute("tabindex")).toBe("-1");
    fireEvent.click(discBtn);
    expect(h.onCopy).toHaveBeenCalledWith("d1");
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

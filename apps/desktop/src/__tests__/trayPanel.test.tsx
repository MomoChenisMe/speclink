// 面板樣式呈現層（spec「面板樣式（macOS）」；design D5）：以主視窗推送的
// TraySnapshot 薄渲染——分區與內容和原生選單同源；jsdom 只測結構與回呼，
// 原生質感（不搶焦點、貼齊、失焦收合）由 4.4 真視窗手動驗證。
import { describe, it, expect, vi } from "vitest";
import { act, cleanup, render as rtlRender, screen, fireEvent, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import {
  I18nProvider,
  REVIEW_TONE,
  SEMANTIC_TONE,
  STAGE_BADGE,
  STAGE_BAR,
  STAGE_ICON,
  VERIFY_TONE,
  type ChangeItem,
} from "@speclink/ui";

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
      { key: "local:/proj/one", name: "one", source: "local", status: "ready" },
      { key: "local:/proj/two", name: "two", source: "local", status: "ready" },
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
    onOpenProjectSettings: vi.fn(),
    onOpenSettings: vi.fn(),
    onQuit: vi.fn(),
    onCopy: vi.fn(),
    onAddProject: vi.fn(),
    onRetryWorkspace: vi.fn(),
    onOpenRecovery: vi.fn(),
    onOpenServerSettings: vi.fn(),
    onReauthenticate: vi.fn(),
  };
  render(<TrayPanel snapshot={snapshot()} {...handlers} {...over} />);
  return handlers;
}

describe("TrayPanel 渲染（與原生選單同源的分區內容）", () => {
  it("active no-session error 以精簡復原卡取代舊資料，retry 原地而詳情／登入走明確回呼", () => {
    const key = "remote:c1/demo/backend";
    const h = renderPanel({
      snapshot: snapshot({
        tabs: [
          {
            key,
            name: "Demo/backend",
            source: "remote",
            status: "error",
            failureKind: "needs-reauth",
            connectionId: "c1",
            serverLabel: "Team Server",
            serverOrigin: "https://spec.example.test",
          },
        ],
        activeKey: key,
        changes: [change({ name: "previous-workspace-change", totalTasks: 2, completedTasks: 1 })],
        discussions: [{ slug: "previous-workspace-discussion", topic: "舊資料", promoted: false }],
      } as unknown as Partial<TraySnapshot>),
    });

    const card = screen.getByTestId("panel-recovery-card");
    expect(card.textContent).toContain("需要重新登入");
    expect(card.textContent).toContain("Demo/backend");
    expect(card.textContent).toContain("Team Server");
    expect(screen.queryByText("previous-workspace-change")).toBeNull();
    expect(screen.queryByText("previous-workspace-discussion")).toBeNull();
    expect(screen.queryByTestId("panel-section-proposed")).toBeNull();

    fireEvent.click(within(card).getByRole("button", { name: "重新登入" }));
    fireEvent.click(within(card).getByRole("button", { name: "查看問題" }));
    expect(h.onReauthenticate).toHaveBeenCalledWith("c1");
    expect(h.onOpenRecovery).toHaveBeenCalledWith(key);
    for (const button of within(card).getAllByRole("button")) {
      expect(button.getAttribute("tabindex")).toBe("-1");
    }
  });

  it("active restoring 顯示進度且不可重複 retry，offline session 則保留 stale 內容", () => {
    const key = "remote:c1/demo/backend";
    const { rerender } = rtlRender(
      <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
        <TrayPanel
          snapshot={snapshot({
            tabs: [
              {
                key,
                name: "Demo/backend",
                source: "remote",
                status: "restoring",
                connectionId: "c1",
                serverLabel: "Team Server",
              },
            ],
            activeKey: key,
          } as unknown as Partial<TraySnapshot>)}
          onOpenProject={vi.fn()}
          onOpenChange={vi.fn()}
          onOpenDiscussion={vi.fn()}
          onOpenApp={vi.fn()}
          onOpenProjectSettings={vi.fn()}
          onOpenSettings={vi.fn()}
          onQuit={vi.fn()}
          onCopy={vi.fn()}
          onAddProject={vi.fn()}
          onRetryWorkspace={vi.fn()}
          onOpenRecovery={vi.fn()}
          onOpenServerSettings={vi.fn()}
          onReauthenticate={vi.fn()}
        />
      </I18nProvider>,
    );
    expect(screen.getByTestId("panel-recovery-card").textContent).toContain("正在連線");
    expect(screen.queryByRole("button", { name: "重新連線" })).toBeNull();

    rerender(
      <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
        <TrayPanel
          snapshot={snapshot({
            tabs: [
              {
                key,
                name: "Demo/backend",
                source: "remote",
                status: "offline",
                connectionId: "c1",
                serverLabel: "Team Server",
              },
            ],
            activeKey: key,
            changes: [change({ name: "offline-stale-change", totalTasks: 2, completedTasks: 1 })],
          } as unknown as Partial<TraySnapshot>)}
          onOpenProject={vi.fn()}
          onOpenChange={vi.fn()}
          onOpenDiscussion={vi.fn()}
          onOpenApp={vi.fn()}
          onOpenProjectSettings={vi.fn()}
          onOpenSettings={vi.fn()}
          onQuit={vi.fn()}
          onCopy={vi.fn()}
          onAddProject={vi.fn()}
          onRetryWorkspace={vi.fn()}
          onOpenRecovery={vi.fn()}
          onOpenServerSettings={vi.fn()}
          onReauthenticate={vi.fn()}
        />
      </I18nProvider>,
    );
    expect(screen.getByTestId("panel-stale-status").textContent).toContain("離線");
    expect(screen.getByText("offline-stale-change")).toBeTruthy();
    expect(screen.queryByTestId("panel-recovery-card")).toBeNull();
  });

  it("active unreachable 的 Panel retry 只走原地回呼，不誤觸主視窗詳情", () => {
    const key = "remote:c1/demo/backend";
    const h = renderPanel({
      snapshot: snapshot({
        tabs: [
          {
            key,
            name: "Demo/backend",
            source: "remote",
            status: "error",
            failureKind: "unreachable",
            connectionId: "c1",
            serverLabel: "Team Server",
          },
        ],
        activeKey: key,
      } as unknown as Partial<TraySnapshot>),
    });
    fireEvent.click(
      within(screen.getByTestId("panel-recovery-card")).getByRole("button", {
        name: "重新連線",
      }),
    );
    expect(h.onRetryWorkspace).toHaveBeenCalledWith(key);
    expect(h.onOpenRecovery).not.toHaveBeenCalled();
    expect(h.onOpenApp).not.toHaveBeenCalled();
  });

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
    expect(h.onOpenProject).toHaveBeenCalledWith("local:/proj/two");
  });

  it("點 remote 專案 tab 以 locator key 回呼切換，不因空 root 靜默", () => {
    const remoteKey = "remote:conn-1/demo/backend";
    const h = renderPanel({
      snapshot: snapshot({
        tabs: [
          { key: "local:/proj/one", name: "one", source: "local", status: "ready" },
          {
            key: remoteKey,
            name: "Demo/backend",
            source: "remote",
            status: "ready",
            connectionId: "conn-1",
            serverLabel: "Team Server",
          },
        ],
      }),
    });
    fireEvent.click(screen.getByTestId(`panel-project-${remoteKey}`));
    expect(h.onOpenProject).toHaveBeenCalledWith(remoteKey);
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

  it("生命週期分區圖示維持主色階梯，討論／已轉出分區圖示為中性", () => {
    // spec「生命週期階梯與互動回饋豁免」：主色階梯是三個生命週期分區的語彙；
    // 討論與已轉出不在該階梯上，借穿階梯樣式會讓「顏色＝階段」的讀法失準。
    renderPanel({ snapshot: snapshot({ discussions: discBoth }) });
    const iconClasses = (id: string) =>
      (screen.getByTestId(id).querySelector("svg")?.getAttribute("class") ?? "").split(/\s+/);
    expect(iconClasses("panel-section-proposed")).toContain(STAGE_ICON.proposed);
    expect(iconClasses("panel-section-in-progress")).toContain(STAGE_ICON["in-progress"]);
    expect(iconClasses("panel-section-ready")).toContain(STAGE_ICON.ready);
    for (const id of ["panel-section-discussions", "panel-section-promoted"]) {
      expect(iconClasses(id).join(" ")).not.toContain("primary");
      expect(iconClasses(id).join(" ")).toContain("muted-foreground");
    }
  });

  it("根容器以毛玻璃同半徑圓角裁切（wash 不得畫出 vibrancy 圓角外）", () => {
    renderPanel();
    const root = screen.getByTestId("panel-root");
    expect(root.className.split(/\s+/)).toContain("rounded-[13px]");
  });

  it("根層裝飾漸層為中性，不以主色鋪底", () => {
    // 裝飾面屬靜態層，主色留給連結／互動／進度。
    renderPanel();
    const root = screen.getByTestId("panel-root");
    expect(root.className).not.toContain("from-primary");
    expect(root.className).toContain("from-foreground/5");
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
    // 討論／已轉出不在生命週期階梯上：計數徽章轉中性（與欄頭圖示同一判斷）。
    for (const id of ["panel-section-discussions", "panel-section-promoted"]) {
      const cls = count(id).className;
      expect(cls).toContain("bg-muted");
      expect(cls).not.toContain("primary");
    }
  });
});

describe("TrayPanel 狀態語意色（spec「介面狀態語意色分層」）", () => {
  const remoteTab = (over: Record<string, unknown>) => ({
    key: "remote:c1/demo/backend",
    name: "Demo/backend",
    source: "remote",
    connectionId: "c1",
    serverLabel: "Team Server",
    ...over,
  });

  const renderTab = (over: Record<string, unknown>) =>
    renderPanel({
      snapshot: snapshot({
        tabs: [remoteTab(over)],
        activeKey: "remote:c1/demo/backend",
      } as unknown as Partial<TraySnapshot>),
    });

  it("復原卡依狀態分色：還原中為藍、錯誤為紅、需重新登入維持琥珀", () => {
    const iconWrap = () =>
      screen.getByTestId("panel-recovery-card").querySelector("span") as HTMLElement;

    renderTab({ status: "restoring" });
    expect(iconWrap().className).toContain(SEMANTIC_TONE.inProgress);
    cleanup();

    renderTab({ status: "error", failureKind: "unreachable" });
    expect(iconWrap().className).toContain("destructive");
    cleanup();

    renderTab({ status: "error", failureKind: "needs-reauth" });
    expect(iconWrap().className).toContain("amber");
  });

  it("作用中非 ready 分頁：選取以主色外框表達，狀態由列內語意色承載", () => {
    renderTab({ status: "restoring" });
    const tab = screen.getByTestId("panel-project-remote:c1/demo/backend");
    // 選取＝主色外框（不是琥珀底，琥珀是警示語意，兩者混用會誤讀為「這個分頁有問題」）。
    expect(tab.className).toContain("border-primary");
    expect(tab.className).not.toContain("amber");
    // 狀態文字自己帶語意色：還原中＝藍。
    const status = within(tab).getByText("正在連線");
    expect(status.className).toContain(SEMANTIC_TONE.inProgress);
  });

  it("stale 列的「重新登入」鈕為中性 outline，不與琥珀警示搶注意力", () => {
    renderTab({ status: "needs-reauth" });
    const stale = screen.getByTestId("panel-stale-status");
    const button = within(stale).getByRole("button", { name: "重新登入" });
    expect(button.className).toContain("border");
    expect(button.className).not.toContain("amber");
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

  it("動作區塊依序渲染「開啟 Speclink」「專案設定」「設定」「結束」，點擊各列觸發對應回呼", () => {
    const h = renderPanel();
    const follows = (a: Element, b: Element) =>
      (a.compareDocumentPosition(b) & Node.DOCUMENT_POSITION_FOLLOWING) !== 0;
    const open = screen.getByText("開啟 Speclink");
    const projectSettings = screen.getByText("專案設定");
    const settings = screen.getByText("設定");
    const quit = screen.getByText("結束");
    expect(follows(open, projectSettings)).toBe(true);
    expect(follows(projectSettings, settings)).toBe(true);
    expect(follows(settings, quit)).toBe(true);
    fireEvent.click(projectSettings);
    expect(h.onOpenProjectSettings).toHaveBeenCalled();
    expect(h.onOpenSettings).not.toHaveBeenCalled();
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

describe("三段式版面（spec「面板樣式（macOS）」：固定頁首／可捲中段／固定頁尾）", () => {
  it("頁首含 tab 條、中段含分區卡並帶縱向捲動樣式、頁尾含動作四列；分割線歸屬頁首 1／中段 1／頁尾 1", () => {
    renderPanel();
    const header = screen.getByTestId("panel-header");
    const scroll = screen.getByTestId("panel-scroll");
    const footer = screen.getByTestId("panel-footer");
    // 頁首＝tab 條；中段＝討論與生命週期分區卡；頁尾＝動作四列
    expect(within(header).getByTestId("panel-project-tabs")).toBeTruthy();
    expect(within(scroll).getByTestId("panel-section-discussions")).toBeTruthy();
    expect(within(scroll).getByTestId("panel-section-proposed")).toBeTruthy();
    expect(within(scroll).getByTestId("panel-section-ready")).toBeTruthy();
    expect(within(footer).getByText("開啟 Speclink")).toBeTruthy();
    expect(within(footer).getByText("專案設定")).toBeTruthy();
    expect(within(footer).getByText("設定")).toBeTruthy();
    expect(within(footer).getByText("結束")).toBeTruthy();
    // 捲動樣式只掛中段
    expect(scroll.className).toContain("overflow-y-auto");
    expect(header.className).not.toContain("overflow-y-auto");
    expect(footer.className).not.toContain("overflow-y-auto");
    // 分割線歸屬：tab 條之下隨頁首、動作區之上隨頁尾、討論與生命週期之間隨中段捲動
    expect(within(header).getAllByTestId("panel-divider")).toHaveLength(1);
    expect(within(scroll).getAllByTestId("panel-divider")).toHaveLength(1);
    expect(within(footer).getAllByTestId("panel-divider")).toHaveLength(1);
  });

  it("中段隱藏原生捲軸（不佔版面寬度）——原生捲軸在此環境常駐且佔寬，改由指示條自繪", () => {
    renderPanel();
    const scroll = screen.getByTestId("panel-scroll");
    expect(scroll.className).toContain("[scrollbar-width:none]");
    expect(scroll.className).toContain("[&::-webkit-scrollbar]:hidden");
  });

  it("捲動指示條：內容未溢出不渲染；捲動時浮現，停止約 0.8 秒後淡出", () => {
    vi.useFakeTimers();
    renderPanel();
    const scroll = screen.getByTestId("panel-scroll");
    // jsdom 無版面計算：以量測值模擬「內容溢出」，指示條的幾何與顯隱皆由此推導
    expect(screen.queryByTestId("panel-scroll-indicator")).toBeNull();
    for (const [prop, value] of [
      ["clientHeight", 200],
      ["scrollHeight", 600],
      ["scrollTop", 0],
    ] as const) {
      Object.defineProperty(scroll, prop, { configurable: true, value });
    }
    act(() => {
      fireEvent.scroll(scroll);
    });
    const indicator = screen.getByTestId("panel-scroll-indicator");
    expect(indicator.getAttribute("data-active")).toBe("true");
    expect(indicator.className).toContain("opacity-100");
    act(() => {
      vi.advanceTimersByTime(900);
    });
    expect(screen.getByTestId("panel-scroll-indicator").getAttribute("data-active")).toBe("false");
    expect(screen.getByTestId("panel-scroll-indicator").className).toContain("opacity-0");
    vi.useRealTimers();
  });

  it("毛玻璃補光層：root 帶主題背景色半透明基底（隨深淺模式，亮度錨定主題色）", () => {
    renderPanel();
    // 濃度值由真實視窗調參定案——只斷言補光層存在（bg-background/<n>），不釘數值
    expect(screen.getByTestId("panel-root").className).toMatch(/bg-background\/\d+/);
  });

  it("recovery 排列下復原卡歸中段、頁首頁尾結構不分支", () => {
    const key = "remote:c1/demo/backend";
    renderPanel({
      snapshot: snapshot({
        tabs: [
          {
            key,
            name: "Demo/backend",
            source: "remote",
            status: "error",
            connectionId: "c1",
            serverLabel: "Team Server",
          },
        ],
        activeKey: key,
      } as unknown as Partial<TraySnapshot>),
    });
    const header = screen.getByTestId("panel-header");
    const scroll = screen.getByTestId("panel-scroll");
    const footer = screen.getByTestId("panel-footer");
    expect(within(header).getByTestId("panel-project-tabs")).toBeTruthy();
    expect(within(scroll).getByTestId("panel-recovery-card")).toBeTruthy();
    expect(within(scroll).queryAllByTestId("panel-divider")).toHaveLength(0);
    expect(within(footer).getByText("結束")).toBeTruthy();
    expect(within(header).getAllByTestId("panel-divider")).toHaveLength(1);
    expect(within(footer).getAllByTestId("panel-divider")).toHaveLength(1);
  });
});

// --- 面板變更列的品質站章（spec tray-status-menu「面板變更列的品質站章」；
// design D7）：兩章並排、順序固定，圖示／色調／tooltip 與看板卡片共用同一組
// 樣式表與 i18n 詞條；非站章的行內符號不進 tray。
describe("面板變更列的品質站章", () => {
  function rowOf(name: string): HTMLElement {
    return screen.getByTestId(`panel-change-${name}`);
  }

  it("兩章並排且順序固定（審查章在前、驗證章在後）", () => {
    // spec Scenario「面板兩章並排」。
    renderPanel({
      snapshot: snapshot({
        changes: [
          change({
            name: "inprog",
            totalTasks: 12,
            completedTasks: 3,
            reviewStatus: "reviewed",
            verifyStatus: "verified",
          }),
        ],
      }),
    });
    const row = rowOf("inprog");
    const review = within(row).getByLabelText("已審查");
    const verify = within(row).getByLabelText("已驗證");
    expect(review.compareDocumentPosition(verify) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    // 章落在名稱與任務數之間。
    const count = within(row).getByText("3/12");
    expect(verify.compareDocumentPosition(count) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  });

  it("單章情境只渲染該站的章", () => {
    // spec Scenario「僅驗證章」。
    renderPanel({
      snapshot: snapshot({
        changes: [
          change({
            name: "inprog",
            totalTasks: 12,
            completedTasks: 3,
            reviewStatus: "none",
            verifyStatus: "inVerify",
          }),
        ],
      }),
    });
    const row = rowOf("inprog");
    expect(within(row).getByLabelText("驗證中")).toBeTruthy();
    for (const label of ["審查中", "已審查", "已審查·其後有變動"]) {
      expect(within(row).queryByLabelText(label)).toBeNull();
    }
  });

  it("兩站皆 none 時零站章，且列不長出其他行內符號", () => {
    // spec Scenario「無章時列組成不變」＋design D7：頭像／討論泡／restale／
    // metaError 明文不進 tray——那是看板的閱讀脈絡，一瞥介面上是雜訊。
    renderPanel({
      snapshot: snapshot({
        changes: [
          change({
            name: "inprog",
            totalTasks: 12,
            completedTasks: 3,
            reviewStatus: "none",
            verifyStatus: "none",
            createdBy: "Someone <s@example.com>",
            fromDiscussions: ["alpha"],
            restaleFrom: ["alpha"],
            metaError: "bad yaml",
          }),
        ],
      }),
    });
    const row = rowOf("inprog");
    for (const label of ["審查中", "已審查", "已審查·其後有變動", "驗證中", "已驗證", "已驗證·其後有變動"]) {
      expect(within(row).queryByLabelText(label)).toBeNull();
    }
    expect(row.textContent).not.toContain("Someone");
    expect(row.textContent).not.toContain("alpha");
    expect(row.textContent).not.toContain("bad yaml");
    expect(within(row).queryByLabelText("S")).toBeNull();
  });

  it("章的樣式與詞條與看板卡片同源（不另建第二份對照）", () => {
    renderPanel({
      snapshot: snapshot({
        changes: [
          change({
            name: "inprog",
            totalTasks: 12,
            completedTasks: 3,
            reviewStatus: "reviewedStale",
            verifyStatus: "verifiedStale",
          }),
        ],
      }),
    });
    const row = rowOf("inprog");
    const review = within(row).getByLabelText("已審查·其後有變動");
    const verify = within(row).getByLabelText("已驗證·其後有變動");
    expect(review.className).toContain(REVIEW_TONE.reviewedStale);
    expect(verify.className).toContain(VERIFY_TONE.verifiedStale);
    expect(VERIFY_TONE.verifiedStale).toBe(REVIEW_TONE.reviewedStale);
  });
});

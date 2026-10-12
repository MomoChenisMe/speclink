// 面板樣式呈現層（spec「面板樣式（macOS）」；design D5）：以主視窗推送的
// TraySnapshot 薄渲染——分區與內容和原生選單同源；jsdom 只測結構與回呼，
// 原生質感（不搶焦點、貼齊、失焦收合）由 4.4 真視窗手動驗證。
import { beforeEach, describe, it, expect, vi } from "vitest";
import { act, cleanup, render as rtlRender, screen, fireEvent, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import {
  I18nProvider,
  DISCUSSION_TONE,
  REVIEW_TONE,
  SEMANTIC_SURFACE,
  SEMANTIC_TONE,
  STAGE_BADGE,
  STAGE_BAR,
  STAGE_ICON,
  VERIFY_TONE,
  type ChangeItem,
} from "@speclink/ui";

import { TrayPanel } from "../panel/TrayPanel";
import { RAIL_SQUARE } from "../components/projectTile";
import {
  TRAY_PROMOTED_EXPANDED_STORAGE_KEY,
  readTrayPromotedExpanded,
  writeTrayPromotedExpanded,
} from "../panel/promotedExpanded";
import { APP_MESSAGES } from "../i18n/messages";
import type { TraySnapshot } from "../tray";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);
const render = (ui: ReactElement) => rtlRender(ui, { wrapper: zhWrapper });

// 已轉出收合列的展開記憶存 localStorage：每題從「未記憶＝收合」起跑，前一題的展開不外溢。
beforeEach(() => localStorage.removeItem(TRAY_PROMOTED_EXPANDED_STORAGE_KEY));

/** b 在 a 之後（文件順序）。 */
const follows = (a: Element, b: Element) =>
  (a.compareDocumentPosition(b) & Node.DOCUMENT_POSITION_FOLLOWING) !== 0;
/** 分區卡的標頭（卡的第一個子節點）。 */
const sectionHeader = (id: string) => screen.getByTestId(id).firstElementChild as HTMLElement;

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
    pendingTabKey: null,
    workspaceLoading: false,
    workspaceLoadFailed: false,
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
      expect(within(card).getByTestId("section-count").textContent).toBe("0");
    }
    const disc = screen.getByTestId("panel-section-discussions");
    expect(disc.className.split(/\s+/)).toEqual(expect.arrayContaining(["min-h-12", "justify-center"]));
    expect(within(disc).getByText("討論")).toBeTruthy();
    expect(within(disc).getByTestId("section-count").textContent).toBe("0");
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
      (id) => within(screen.getByTestId(id)).getByTestId("section-count").textContent,
    );
    expect(counts).toEqual(["0", "1", "0"]);
    expect(within(screen.getByTestId("panel-section-in-progress")).getByTestId("panel-change-solo")).toBeTruthy();
    const [proposed, inProgress, ready] = ids.map((id) => screen.getByTestId(id));
    expect(proposed.compareDocumentPosition(inProgress) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(inProgress.compareDocumentPosition(ready) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  });

  it("討論分流：已轉出討論收於討論卡底部收合列，展開後列於收合列上方（無獨立已轉出分區）", () => {
    renderPanel({
      snapshot: snapshot({
        discussions: [
          { slug: "open-d", topic: "討論中的", promoted: false },
          { slug: "prom-d", topic: "已轉出的", promoted: true },
        ],
      }),
    });
    expect(screen.queryByTestId("panel-section-promoted")).toBeNull();
    const card = screen.getByTestId("panel-section-discussions");
    expect(within(card).getByTestId("panel-discussion-open-d")).toBeTruthy();
    const toggle = within(card).getByTestId("panel-promoted-toggle");
    expect(toggle.textContent).toContain("已轉出 1");
    fireEvent.click(toggle);
    const rows = within(card).getByTestId("panel-promoted-rows");
    expect(within(rows).getByTestId("panel-discussion-prom-d")).toBeTruthy();
    expect(within(rows).queryByTestId("panel-discussion-open-d")).toBeNull();
    // 討論中列 → 展開區 → 收合列
    expect(follows(screen.getByTestId("panel-discussion-open-d"), rows)).toBe(true);
    expect(follows(rows, toggle)).toBe(true);
  });

  it("零已轉出時無收合列", () => {
    renderPanel();
    expect(screen.queryByTestId("panel-promoted-toggle")).toBeNull();
    expect(screen.queryByText(/已轉出/)).toBeNull();
  });

  it("區塊順序：討論卡（底部帶已轉出收合列）位於生命週期分區之前、「開啟 Speclink」最末", () => {
    renderPanel({
      snapshot: snapshot({
        discussions: [
          { slug: "open-d", topic: "討論中的", promoted: false },
          { slug: "prom-d", topic: "已轉出的", promoted: true },
        ],
      }),
    });
    const discussions = screen.getByTestId("panel-section-discussions");
    const proposed = screen.getByTestId("panel-section-proposed");
    const ready = screen.getByTestId("panel-section-ready");
    const open = screen.getByText("開啟 Speclink");
    expect(within(discussions).getByTestId("panel-promoted-toggle")).toBeTruthy();
    expect(follows(discussions, proposed)).toBe(true);
    expect(follows(ready, open)).toBe(true);
  });

  it("區塊之間只以間距分隔：無任何分割線", () => {
    renderPanel();
    expect(screen.queryAllByTestId("panel-divider")).toHaveLength(0);
    expect(document.querySelector("hr")).toBeNull();
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

  const tile = (el: HTMLElement) => el.querySelector("[data-project-tile]") as HTMLElement;

  it("作用中 tab 的方塊實心主色、非作用中方塊灰底；tab 本體不塗主色、hover 為灰底", () => {
    renderPanel();
    const one = screen.getByTestId("panel-project-local:/proj/one");
    const two = screen.getByTestId("panel-project-local:/proj/two");
    expect(tile(one).className.split(/\s+/)).toEqual(expect.arrayContaining(["bg-primary", "text-primary-foreground"]));
    expect(tile(two).className.split(/\s+/)).toEqual(expect.arrayContaining(["bg-muted", "text-muted-foreground"]));
    expect(tile(two).className.split(/\s+/)).not.toContain("bg-primary");
    for (const tab of [one, two]) {
      expect(tab.className).not.toContain("primary");
      expect(tab.className.split(/\s+/)).toContain("hover:bg-foreground/5");
    }
  });

  it("tab 方塊與圖示列同形：classList 含 RAIL_SQUARE 全部 token，專案名列於方塊之下", () => {
    renderPanel();
    const one = screen.getByTestId("panel-project-local:/proj/one");
    for (const token of RAIL_SQUARE.split(" ")) expect(tile(one).classList.contains(token)).toBe(true);
    const name = within(one).getByText("one");
    expect(tile(one).compareDocumentPosition(name) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  });

  it("tab 以 title 帶出專案全名（名稱在方塊下會截斷）", () => {
    renderPanel({
      snapshot: snapshot({
        tabs: [{ key: "local:/proj/long", name: "speclink-desktop", source: "local", status: "ready" }],
        activeKey: "local:/proj/long",
      }),
    });
    expect(screen.getByTestId("panel-project-local:/proj/long").getAttribute("title")).toBe("speclink-desktop");
  });

  it("首字母與圖示列同一規則：去頭尾空白後取第一個字", () => {
    renderPanel({
      snapshot: snapshot({
        tabs: [{ key: "local:/proj/sp", name: " 規格", source: "local", status: "ready" }],
        activeKey: "local:/proj/sp",
      }),
    });
    expect(tile(screen.getByTestId("panel-project-local:/proj/sp")).textContent).toBe("規");
  });

  it("「加入專案」為虛線框方塊＋文字（與圖示列的「＋」同形）", () => {
    renderPanel();
    const add = screen.getByTestId("panel-add-project");
    const square = add.querySelector("[data-project-tile]") as HTMLElement;
    for (const token of RAIL_SQUARE.split(" ")) expect(square.classList.contains(token)).toBe(true);
    expect(square.className.split(/\s+/)).toEqual(expect.arrayContaining(["border-dashed", "border-border"]));
    expect(within(add).getByText("加入專案")).toBeTruthy();
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

  it("tab 條、各分區與動作區各自為白底細框 12px 圓角卡", () => {
    renderPanel({ snapshot: snapshot({ discussions: discBoth }) });
    for (const id of [
      "panel-project-card",
      "panel-section-proposed",
      "panel-section-in-progress",
      "panel-section-ready",
      "panel-section-discussions",
      "panel-actions-card",
    ]) {
      const classes = screen.getByTestId(id).className.split(/\s+/);
      expect(classes).toEqual(expect.arrayContaining(["bg-card", "border", "border-border", "rounded-xl"]));
    }
  });

  it("生命週期與討論分區圖示各取所屬色相，已轉出收合列為中性", () => {
    // spec「生命週期四欄各一色相」：系統匣分區與看板四欄同色，同一來源（stage.ts）；
    // 已轉出不是生命週期階段，收合列維持中性、計數在文字裡不給色相徽章。
    renderPanel({ snapshot: snapshot({ discussions: discBoth }) });
    const iconClasses = (id: string) =>
      (screen.getByTestId(id).querySelector("svg")?.getAttribute("class") ?? "").split(/\s+/);
    expect(iconClasses("panel-section-proposed")).toContain(STAGE_ICON.proposed);
    expect(iconClasses("panel-section-in-progress")).toContain(STAGE_ICON["in-progress"]);
    expect(iconClasses("panel-section-ready")).toContain(STAGE_ICON.ready);
    expect(STAGE_ICON.proposed).toBe("text-stage-proposed");
    expect(STAGE_ICON["in-progress"]).toBe("text-stage-in-progress");
    expect(STAGE_ICON.ready).toBe("text-stage-ready");
    expect(iconClasses("panel-section-discussions")).toContain(DISCUSSION_TONE.icon);
    expect(DISCUSSION_TONE.icon).toBe("text-stage-discussion");
    const toggle = screen.getByTestId("panel-promoted-toggle");
    expect(toggle.className).toContain("text-muted-foreground");
    expect(toggle.innerHTML).not.toContain("stage-");
    expect(within(toggle).queryByTestId("section-count")).toBeNull();
  });

  it("根容器以毛玻璃同半徑圓角裁切（wash 不得畫出 vibrancy 圓角外）", () => {
    renderPanel();
    const root = screen.getByTestId("panel-root");
    expect(root.className.split(/\s+/)).toContain("rounded-[13px]");
  });

  it("根層不疊漸層、以側欄面色約九成補光", () => {
    // 主色留給選取與主要操作；漸層 wash 隨卡片化移除。
    renderPanel();
    const root = screen.getByTestId("panel-root");
    expect(root.className).not.toContain("bg-linear-to-b");
    expect(root.className).not.toContain("from-");
    expect(root.className.split(/\s+/)).toContain("bg-sidebar/90");
  });

  it("卡內的列以細線分隔、hover 為半透明前景色灰底，不以主色實心反白", () => {
    const eight = Array.from({ length: 8 }, (_, i) =>
      change({ name: `c${i}`, totalTasks: 2, completedTasks: 1 }),
    );
    renderPanel({ snapshot: snapshot({ changes: eight, discussions: discBoth }) });
    const rows = [
      screen.getByTestId("panel-change-c0"),
      screen.getByTestId("panel-discussion-open-d"),
      screen.getByText("還有 3 個…").parentElement!,
      screen.getByText("開啟 Speclink"),
    ];
    for (const row of rows) {
      const classes = row.className.split(/\s+/);
      expect(classes).toEqual(expect.arrayContaining(["hover:bg-foreground/5", "border-b", "last:border-b-0"]));
      // 列內已無 group-hover 配色，列不必是 group。
      expect(classes).not.toContain("group");
      expect(row.className).not.toContain("hover:bg-primary");
      expect(row.className).not.toMatch(/rounded/);
    }
    // 列內任何元素都不掛 hover 反白配色（站章、波次、任務數、topic、進度條、動作圖示、複製鈕）。
    expect(document.body.innerHTML).not.toContain("group-hover:text-primary-foreground");
    expect(document.body.innerHTML).not.toContain("group-hover:bg-primary-foreground");
  });

  it("有內容的卡標頭下有細線，空狀態卡標頭無", () => {
    renderPanel({ snapshot: snapshot({ changes: [change({ name: "solo", totalTasks: 2, completedTasks: 1 })] }) });
    // 進行中有一列、討論有一則：標頭下有線。
    for (const id of ["panel-section-in-progress", "panel-section-discussions"]) {
      expect(sectionHeader(id).className.split(/\s+/)).toEqual(expect.arrayContaining(["border-b", "px-3", "py-2"]));
    }
    // 提案中、已就緒為空狀態卡：標頭下無線。
    for (const id of ["panel-section-proposed", "panel-section-ready"]) {
      expect(sectionHeader(id).className.split(/\s+/)).not.toContain("border-b");
    }
  });

  it("載入中與載入失敗的卡標頭下有細線（其下有骨架或失敗提示）", () => {
    renderPanel({ snapshot: snapshot({ workspaceLoading: true, changes: [], discussions: [] }) });
    for (const id of ["panel-section-proposed", "panel-section-discussions"]) {
      expect(sectionHeader(id).className.split(/\s+/)).toContain("border-b");
    }
    cleanup();
    renderPanel({ snapshot: snapshot({ workspaceLoadFailed: true, changes: [], discussions: [] }) });
    for (const id of ["panel-section-proposed", "panel-section-discussions"]) {
      expect(sectionHeader(id).className.split(/\s+/)).toContain("border-b");
    }
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
    // 三階各一色相（與看板同源），互不相同。
    expect(new Set([STAGE_BAR.proposed, STAGE_BAR["in-progress"], STAGE_BAR.ready]).size).toBe(3);
  });
});

describe("TrayPanel 分區計數（spec「面板樣式（macOS）」；design D8）", () => {
  it("各分區標題帶項目計數徽章（STAGE_BADGE 同語彙；討論分區取看板討論欄同款，只計討論中）", () => {
    renderPanel({
      snapshot: snapshot({
        discussions: [
          { slug: "open-d", topic: "討論中的", promoted: false },
          { slug: "prom-d", topic: "已轉出的", promoted: true },
        ],
      }),
    });
    const count = (id: string) =>
      screen.getByTestId(id).querySelector('[data-testid="section-count"]')!;
    for (const id of [
      "panel-section-proposed",
      "panel-section-in-progress",
      "panel-section-ready",
      "panel-section-discussions",
    ]) {
      expect(count(id).textContent).toBe("1");
    }
    expect(count("panel-section-proposed").className.split(/\s+/)).toEqual(
      expect.arrayContaining(STAGE_BADGE.proposed.split(/\s+/)),
    );
    expect(count("panel-section-ready").className.split(/\s+/)).toEqual(
      expect.arrayContaining(STAGE_BADGE.ready.split(/\s+/)),
    );
    // 討論分區計數取桃紫（與看板討論欄同款）；已轉出不是生命週期階段，維持中性。
    expect(count("panel-section-discussions").className.split(/\s+/)).toEqual(
      expect.arrayContaining(DISCUSSION_TONE.badge.split(/\s+/)),
    );
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
    expect(iconWrap().className).toContain("status-warning");
  });

  it.each([
    { status: "restoring", failureKind: undefined, label: "正在連線", tone: SEMANTIC_TONE.inProgress },
    { status: "error", failureKind: "unreachable", label: "無法連線", tone: SEMANTIC_TONE.danger },
  ])("作用中 $status 分頁：方塊同為實心主色標示選取，狀態由狀態字的語意色承載", ({ status, failureKind, label, tone }) => {
    renderTab({ status, failureKind });
    const tab = screen.getByTestId("panel-project-remote:c1/demo/backend");
    // 選取只由方塊承載（與圖示列一致），tab 本體不畫主色外框、不塗琥珀。
    const square = tab.querySelector("[data-project-tile]") as HTMLElement;
    expect(square.className.split(/\s+/)).toContain("bg-primary");
    expect(tab.className).not.toContain("border-primary");
    expect(tab.className).not.toContain("status-warning");
    // 狀態文字自己帶語意色：還原中＝藍、連不上＝紅。
    expect(within(tab).getByText(label).className).toContain(tone);
  });

  it("stale 列為琥珀淡底卡，「重新登入」鈕為中性 outline、不與琥珀警示搶注意力", () => {
    renderTab({ status: "needs-reauth" });
    const stale = screen.getByTestId("panel-stale-status");
    expect(stale.className.split(/\s+/)).toEqual(
      expect.arrayContaining(["rounded-xl", "border", ...SEMANTIC_SURFACE.warning.split(" ")]),
    );
    const button = within(stale).getByRole("button", { name: "重新登入" });
    expect(button.className).toContain("border");
    expect(button.className).not.toContain("status-warning");
    expect(button.getAttribute("tabindex")).toBe("-1");
  });

  it("復原卡為白底卡＋語意色邊框，資訊區淡灰底，動作鈕為共用 Button（主要實心、次要 outline）", () => {
    renderTab({ status: "error", failureKind: "unreachable" });
    const card = screen.getByTestId("panel-recovery-card");
    const classes = card.className.split(/\s+/);
    expect(classes).toEqual(expect.arrayContaining(["rounded-xl", "border", "bg-card", "border-destructive/40"]));
    // 語意色只留邊框：淡紅底被白卡蓋掉。
    expect(classes).not.toContain("bg-destructive/10");
    expect(card.querySelector("dl")!.className.split(/\s+/)).toContain("bg-muted/50");
    const retry = within(card).getByRole("button", { name: "重新連線" });
    expect(retry.className.split(/\s+/)).toEqual(expect.arrayContaining(["bg-primary", "col-span-2", "h-7"]));
    for (const name of ["查看問題", "伺服器設定"]) {
      const button = within(card).getByRole("button", { name });
      expect(button.className.split(/\s+/)).toEqual(expect.arrayContaining(["border", "border-input", "h-7"]));
    }
    for (const button of within(card).getAllByRole("button")) {
      expect(button.getAttribute("tabindex")).toBe("-1");
    }
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
    // 列 hover 為灰底、不反白：複製鈕維持本色，不帶反白配色。
    for (const b of [discBtn, changeBtn]) expect(b.className).not.toContain("group-hover:text-primary-foreground");
    fireEvent.click(discBtn);
    expect(h.onCopy).toHaveBeenCalledWith("d1");
  });

  it("複製回饋：點擊後圖示短暫轉勾號、1.5 秒後復原（共用 CopyButton）", () => {
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
      vi.advanceTimersByTime(1400);
    });
    expect(btn.querySelector("svg.lucide-check")).toBeTruthy();
    act(() => {
      vi.advanceTimersByTime(200);
    });
    expect(btn.querySelector("svg.lucide-check")).toBeNull();
    expect(btn.querySelector("svg.lucide-copy")).toBeTruthy();
    vi.useRealTimers();
  });
});

describe("三段式版面（spec「面板樣式（macOS）」：固定頁首／可捲中段／固定頁尾）", () => {
  it("頁首為 tab 條卡、中段含分區卡並帶縱向捲動樣式、頁尾為動作四列卡；無分割線", () => {
    renderPanel();
    const header = screen.getByTestId("panel-header");
    const scroll = screen.getByTestId("panel-scroll");
    const footer = screen.getByTestId("panel-footer");
    // 頁首＝tab 條卡；中段＝討論與生命週期分區卡；頁尾＝動作四列卡
    expect(within(within(header).getByTestId("panel-project-card")).getByTestId("panel-project-tabs")).toBeTruthy();
    expect(within(scroll).getByTestId("panel-section-discussions")).toBeTruthy();
    expect(within(scroll).getByTestId("panel-section-proposed")).toBeTruthy();
    expect(within(scroll).getByTestId("panel-section-ready")).toBeTruthy();
    const actions = within(footer).getByTestId("panel-actions-card");
    expect(within(actions).getByText("開啟 Speclink")).toBeTruthy();
    expect(within(actions).getByText("專案設定")).toBeTruthy();
    expect(within(actions).getByText("設定")).toBeTruthy();
    expect(within(actions).getByText("結束")).toBeTruthy();
    // 捲動樣式只掛中段
    expect(scroll.className).toContain("overflow-y-auto");
    expect(header.className).not.toContain("overflow-y-auto");
    expect(footer.className).not.toContain("overflow-y-auto");
    expect(screen.queryAllByTestId("panel-divider")).toHaveLength(0);
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

  it("毛玻璃補光層：root 帶側欄面色半透明基底（隨深淺模式，亮度錨定側欄面色）", () => {
    renderPanel();
    // 濃度約九成、真機可微調——只斷言補光層存在（bg-sidebar/<n>）
    expect(screen.getByTestId("panel-root").className).toMatch(/bg-sidebar\/\d+/);
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
    expect(within(header).getByTestId("panel-project-card")).toBeTruthy();
    expect(within(scroll).getByTestId("panel-recovery-card")).toBeTruthy();
    expect(within(within(footer).getByTestId("panel-actions-card")).getByText("結束")).toBeTruthy();
    expect(screen.queryAllByTestId("panel-divider")).toHaveLength(0);
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

  it("hover 灰底時章色調不變：只取共用色調表、不掛 hover 改色", () => {
    // spec Scenario「hover 灰底時章色調不變」：列 hover 為半透明前景色灰底，
    // 章的對比不受影響，hover 期間維持與看板卡片共用的色調。
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
    for (const badge of [review, verify]) {
      expect(badge.className).not.toContain("group-hover:");
    }
    expect(review.className).toContain(REVIEW_TONE.reviewed);
    expect(verify.className).toContain(VERIFY_TONE.verified);
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

  it("worktree 掛著時渲染 worktree 標記，樣式與詞條與看板卡片同源", () => {
    // worktree 掛著＝工作正於副本進行中，與站章同屬影響收尾動作的狀態；
    // 色調與 tooltip 沿看板卡片（SEMANTIC_TONE.inProgress＋card.worktree 詞條）。
    renderPanel({
      snapshot: snapshot({
        changes: [
          change({
            name: "inprog",
            totalTasks: 12,
            completedTasks: 3,
            reviewStatus: "reviewed",
            verifyStatus: "verified",
            worktree: { branch: "speclink/inprog", path: "/w/inprog" },
          }),
        ],
      }),
    });
    const row = rowOf("inprog");
    const badge = within(row).getByLabelText("worktree");
    expect(badge.getAttribute("title")).toBe("在 worktree speclink/inprog 中進行");
    expect(badge.className).toContain(SEMANTIC_TONE.inProgress);
    // hover 灰底期間維持本色，與站章同規則。
    expect(badge.className).not.toContain("group-hover:");
    // 位置固定：站章之後、任務數之前。
    const verify = within(row).getByLabelText("已驗證");
    const count = within(row).getByText("3/12");
    expect(verify.compareDocumentPosition(badge) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(badge.compareDocumentPosition(count) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  });

  it("無 worktree 時不渲染 worktree 標記", () => {
    renderPanel({
      snapshot: snapshot({
        changes: [
          change({ name: "inprog", totalTasks: 12, completedTasks: 3, worktree: null }),
        ],
      }),
    });
    expect(within(rowOf("inprog")).queryByLabelText("worktree")).toBeNull();
  });
});

// spec「面板分頁切換中回饋」「面板分區首訪 skeleton」（design D5）：
// 兩個狀態自 TraySnapshot 導出，面板純渲染。
describe("面板載入回饋", () => {
  it("切換中 → 目標分頁出 spinner，分區維持原 workspace 內容", () => {
    renderPanel({ snapshot: snapshot({ pendingTabKey: "local:/proj/two" }) });
    const target = screen.getByTestId("panel-project-local:/proj/two");
    expect(target.querySelector('[data-tab-pending="true"]')).toBeTruthy();
    // 分區照常顯示既有資料——切換尚未發生。
    expect(screen.getByText("inprog")).toBeTruthy();
  });

  it("切換中 → 非目標分頁不出 spinner", () => {
    renderPanel({ snapshot: snapshot({ pendingTabKey: "local:/proj/two" }) });
    const other = screen.getByTestId("panel-project-local:/proj/one");
    expect(other.querySelector('[data-tab-pending="true"]')).toBeNull();
  });

  it("無切換 → 全部分頁無 spinner", () => {
    renderPanel();
    expect(document.querySelector('[data-tab-pending="true"]')).toBeNull();
  });

  it("首訪未載入 → 分區標題照常、內容為佔位列", () => {
    renderPanel({ snapshot: snapshot({ workspaceLoading: true, changes: [], discussions: [] }) });
    expect(screen.getByText("進行中")).toBeTruthy();
    expect(screen.getByText("討論")).toBeTruthy();
    expect(document.querySelectorAll('[aria-busy="true"]').length).toBeGreaterThan(0);
  });

  it("首訪未載入 → 不顯示分區計數（不謊報 0 筆）", () => {
    renderPanel({ snapshot: snapshot({ workspaceLoading: true, changes: [], discussions: [] }) });
    expect(document.querySelector('[data-testid="section-count"]')).toBeNull();
  });

  it("首訪未載入 → 不顯示空狀態文案", () => {
    renderPanel({ snapshot: snapshot({ workspaceLoading: true, changes: [], discussions: [] }) });
    expect(screen.queryByText("尚無進行中變更")).toBeNull();
  });

  it("已載入 → 照常顯示資料，無佔位列", () => {
    renderPanel();
    expect(screen.getByText("inprog")).toBeTruthy();
    expect(document.querySelector('[aria-busy="true"]')).toBeNull();
  });

  // 骨架條件整條由 store 導出（design D3）：零分頁、探測失敗、讀取失敗等
  // 「不會有載入發生」的情境一律為 false，面板不必自行辨識——舊版面板自組
  // 三欄旗標時，正是這幾條讓骨架永久掛著。
  it("零分頁 → 不出佔位列，走既有空狀態卡", () => {
    renderPanel({
      snapshot: snapshot({
        tabs: [],
        activeKey: null,
        changes: [],
        discussions: [],
      }),
    });
    expect(document.querySelector('[aria-busy="true"]')).toBeNull();
    expect(screen.getByTestId("panel-section-discussions")).toBeTruthy();
  });

  it("有分頁但無活躍 workspace → 不出佔位列", () => {
    renderPanel({
      snapshot: snapshot({
        activeKey: null,
        changes: [],
        discussions: [],
      }),
    });
    expect(document.querySelector('[aria-busy="true"]')).toBeNull();
  });

  it("remote 復原態遮蔽資料時 → 不出佔位列（復原呈現優先）", () => {
    const key = "remote:c1/demo/backend";
    renderPanel({
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
        workspaceLoading: true,
        changes: [],
        discussions: [],
      }),
    });
    expect(document.querySelector('[aria-busy="true"]')).toBeNull();
  });

  // spec「面板分區載入失敗終態」：讀不到 ≠ 確認是空的，兩者不得同貌。
  it("首訪載入失敗 → 分區顯示失敗提示，非骨架、非空態", () => {
    renderPanel({
      snapshot: snapshot({ workspaceLoadFailed: true, changes: [], discussions: [] }),
    });
    expect(document.querySelector('[aria-busy="true"]')).toBeNull();
    expect(screen.queryByText("尚無進行中變更")).toBeNull();
    expect(screen.getAllByTestId("panel-section-load-failed").length).toBeGreaterThan(0);
  });

  it("載入中優先於失敗提示（重試在途時不留舊提示）", () => {
    renderPanel({
      snapshot: snapshot({
        workspaceLoading: true,
        workspaceLoadFailed: true,
        changes: [],
        discussions: [],
      }),
    });
    expect(document.querySelectorAll('[aria-busy="true"]').length).toBeGreaterThan(0);
    expect(screen.queryByTestId("panel-section-load-failed")).toBeNull();
  });

  it("remote 復原態遮蔽資料時 → 不顯示失敗提示（復原呈現優先）", () => {
    const key = "remote:c1/demo/backend";
    renderPanel({
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
        workspaceLoadFailed: true,
        changes: [],
        discussions: [],
      }),
    });
    expect(screen.queryByTestId("panel-section-load-failed")).toBeNull();
    expect(screen.getByTestId("panel-recovery-card")).toBeTruthy();
  });
});

// --- 變更列的波次標示與順序同源（spec tray-status-menu「變更列的波次標示與順序
// 同源」；add-change-plan-desktop design D7）：列首波次數字（tooltip 與看板卡片
// 共用 card.wave）、被擋列整列降透明度且 tooltip 列前置、缺 wave 時列不變。
describe("面板變更列的波次標示", () => {
  const rowOf = (name: string) => screen.getByTestId(`panel-change-${name}`);
  const planned = [
    change({ name: "add-a", totalTasks: 4, completedTasks: 1, wave: 1, blockedBy: [], dependsOn: [], overlaps: [] }),
    change({ name: "add-b", totalTasks: 4, completedTasks: 1, wave: 2, blockedBy: ["add-a"], dependsOn: ["add-a"], overlaps: [] }),
  ];

  it("列依 payload 順序、列首為波次數字（名稱之前），被擋列降透明度且 tooltip 列出前置", () => {
    // spec Scenario「面板列首波次與被擋變淡」。
    renderPanel({ snapshot: snapshot({ changes: planned, discussions: [] }) });
    const section = screen.getByTestId("panel-section-in-progress");
    const rows = within(section).getAllByTestId(/^panel-change-/).map((el) => el.dataset.testid);
    expect(rows).toEqual(["panel-change-add-a", "panel-change-add-b"]);

    const a = rowOf("add-a");
    const waveA = within(a).getByLabelText("第 1 波");
    expect(waveA.textContent).toBe("1");
    expect(waveA.compareDocumentPosition(within(a).getByText("add-a")) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(a.className).not.toMatch(/opacity-/);
    expect(a.getAttribute("title")).toBeNull();

    const b = rowOf("add-b");
    expect(within(b).getByLabelText("第 2 波").textContent).toBe("2");
    expect(b.className).toMatch(/opacity-/);
    // 淡化落在列的內容上：列本身不降透明度，底部分隔線與其他列同色。
    expect(b.className.split(/\s+/)).toContain("[&>*]:opacity-60");
    expect(b.className.split(/\s+/)).not.toContain("opacity-60");
    expect(b.getAttribute("title")).toBe("等待：add-a");
  });

  it("hover 灰底期間波次數字維持主色、不改色", () => {
    renderPanel({ snapshot: snapshot({ changes: planned, discussions: [] }) });
    const wave = within(rowOf("add-a")).getByLabelText("第 1 波");
    expect(wave.className).toContain("text-primary");
    expect(wave.className).not.toContain("group-hover:");
  });

  it("缺 wave（remote 或 plan 成環）時列無波次數字、不變淡、無 tooltip", () => {
    // spec Scenario「缺欄位不變」。
    renderPanel();
    const row = rowOf("inprog");
    expect(within(row).queryByLabelText(/^第 \d+ 波$/)).toBeNull();
    expect(row.className).not.toMatch(/opacity-/);
    expect(row.getAttribute("title")).toBeNull();
  });
});

// spec「討論列表」面板的已轉出收合列記憶（design D4）：app 本機一鍵、與看板的記憶分開；
// 缺席或壞值視為收合，localStorage 不可用時讀寫都不拋錯。
describe("面板已轉出收合列的展開記憶", () => {
  it("鍵缺席 → 收合", () => {
    localStorage.removeItem(TRAY_PROMOTED_EXPANDED_STORAGE_KEY);
    expect(readTrayPromotedExpanded()).toBe(false);
  });

  it("壞值（\"yes\"）→ 收合", () => {
    localStorage.setItem(TRAY_PROMOTED_EXPANDED_STORAGE_KEY, "yes");
    expect(readTrayPromotedExpanded()).toBe(false);
    localStorage.removeItem(TRAY_PROMOTED_EXPANDED_STORAGE_KEY);
  });

  it("寫入 true 後讀回 true，鍵與看板的記憶鍵分開", () => {
    writeTrayPromotedExpanded(true);
    expect(TRAY_PROMOTED_EXPANDED_STORAGE_KEY).toBe("speclink.tray.promotedExpanded");
    expect(localStorage.getItem(TRAY_PROMOTED_EXPANDED_STORAGE_KEY)).toBe("true");
    expect(readTrayPromotedExpanded()).toBe(true);
    localStorage.removeItem(TRAY_PROMOTED_EXPANDED_STORAGE_KEY);
  });

  it("localStorage 不可用時讀回收合、寫入不拋錯", () => {
    // vitest.setup 掛的 localStorage 來自另一個 jsdom window，全域 Storage.prototype 攔不到；
    // spy 要落在該實例的 prototype（同 App.test 的做法）。
    const proto = Object.getPrototypeOf(localStorage) as Storage;
    localStorage.setItem(TRAY_PROMOTED_EXPANDED_STORAGE_KEY, "true");
    const get = vi.spyOn(proto, "getItem").mockImplementation(() => {
      throw new Error("denied");
    });
    const set = vi.spyOn(proto, "setItem").mockImplementation(() => {
      throw new Error("denied");
    });
    try {
      // 鍵裡明明是 "true"：讀回 false 只可能來自 catch。
      expect(readTrayPromotedExpanded()).toBe(false);
      expect(() => writeTrayPromotedExpanded(true)).not.toThrow();
      expect(get).toHaveBeenCalled();
      expect(set).toHaveBeenCalled();
    } finally {
      get.mockRestore();
      set.mockRestore();
    }
  });
});

// spec「討論列表」面板的已轉出收合列（design D4）：預設收合、點按就地展開並記憶，
// 展開區與其他分區同受溢出摺疊約束。
describe("面板的已轉出收合列", () => {
  const promoted = (n: number) =>
    Array.from({ length: n }, (_, i) => ({ slug: `prom-${i}`, topic: `已轉出 ${i}`, promoted: true }));
  const open2 = [
    { slug: "open-a", topic: "討論中 a", promoted: false },
    { slug: "open-b", topic: "討論中 b", promoted: false },
  ];

  it("預設收合：只列討論中，底部為「已轉出 3 ›」、aria-expanded 為 false", () => {
    renderPanel({ snapshot: snapshot({ discussions: [...open2, ...promoted(3)] }) });
    const card = screen.getByTestId("panel-section-discussions");
    expect(within(card).getByTestId("section-count").textContent).toBe("2");
    const toggle = within(card).getByTestId("panel-promoted-toggle");
    expect(toggle.getAttribute("role")).toBe("button");
    expect(toggle.getAttribute("aria-expanded")).toBe("false");
    expect(toggle.getAttribute("tabindex")).toBe("-1");
    expect(toggle.textContent).toBe("已轉出 3");
    expect(toggle.querySelector("svg.lucide-chevron-right")?.getAttribute("class")).not.toContain("rotate-90");
    expect(screen.queryByTestId("panel-promoted-rows")).toBeNull();
    expect(screen.queryByTestId("panel-discussion-prom-0")).toBeNull();
  });

  it("點按就地展開並寫入記憶，再點收合", () => {
    renderPanel({ snapshot: snapshot({ discussions: [...open2, ...promoted(3)] }) });
    const toggle = screen.getByTestId("panel-promoted-toggle");
    fireEvent.click(toggle);
    expect(toggle.getAttribute("aria-expanded")).toBe("true");
    expect(toggle.querySelector("svg.lucide-chevron-right")?.getAttribute("class")).toContain("rotate-90");
    const rows = screen.getByTestId("panel-promoted-rows");
    expect(within(rows).getAllByTestId(/^panel-discussion-/)).toHaveLength(3);
    expect(localStorage.getItem(TRAY_PROMOTED_EXPANDED_STORAGE_KEY)).toBe("true");
    fireEvent.click(toggle);
    expect(screen.queryByTestId("panel-promoted-rows")).toBeNull();
    expect(localStorage.getItem(TRAY_PROMOTED_EXPANDED_STORAGE_KEY)).toBe("false");
  });

  it("收合列可用 Enter／Space 開合（與 role=button 一致）", () => {
    renderPanel({ snapshot: snapshot({ discussions: promoted(2) }) });
    const toggle = screen.getByTestId("panel-promoted-toggle");
    fireEvent.keyDown(toggle, { key: "Enter" });
    expect(toggle.getAttribute("aria-expanded")).toBe("true");
    fireEvent.keyDown(toggle, { key: " " });
    expect(toggle.getAttribute("aria-expanded")).toBe("false");
    fireEvent.keyDown(toggle, { key: "a" });
    expect(toggle.getAttribute("aria-expanded")).toBe("false");
  });

  it("重新掛載（面板重開、app 重啟）讀回展開", () => {
    renderPanel({ snapshot: snapshot({ discussions: [...open2, ...promoted(3)] }) });
    fireEvent.click(screen.getByTestId("panel-promoted-toggle"));
    cleanup();
    renderPanel({ snapshot: snapshot({ discussions: [...open2, ...promoted(3)] }) });
    expect(screen.getByTestId("panel-promoted-toggle").getAttribute("aria-expanded")).toBe("true");
    expect(screen.getByTestId("panel-promoted-rows")).toBeTruthy();
  });

  it("記憶壞值視為收合", () => {
    localStorage.setItem(TRAY_PROMOTED_EXPANDED_STORAGE_KEY, "yes");
    renderPanel({ snapshot: snapshot({ discussions: promoted(2) }) });
    expect(screen.getByTestId("panel-promoted-toggle").getAttribute("aria-expanded")).toBe("false");
  });

  it("討論中 0、已轉出 2：「討論 0」標頭＋收合列，不走空狀態卡", () => {
    renderPanel({ snapshot: snapshot({ discussions: promoted(2) }) });
    const card = screen.getByTestId("panel-section-discussions");
    expect(within(card).getByTestId("section-count").textContent).toBe("0");
    expect(within(card).getByTestId("panel-promoted-toggle").textContent).toBe("已轉出 2");
    expect(card.className.split(/\s+/)).not.toContain("min-h-12");
    expect(sectionHeader("panel-section-discussions").className.split(/\s+/)).toContain("border-b");
  });

  it("展開區 7 則：直列前 5、尾端「還有 2 個…」", () => {
    renderPanel({ snapshot: snapshot({ discussions: promoted(7) }) });
    fireEvent.click(screen.getByTestId("panel-promoted-toggle"));
    const rows = screen.getByTestId("panel-promoted-rows");
    expect(within(rows).getAllByTestId(/^panel-discussion-/)).toHaveLength(5);
    fireEvent.click(within(rows).getByText("還有 2 個…"));
    expect(within(rows).getAllByTestId(/^panel-discussion-/)).toHaveLength(7);
  });

  it("載入中與載入失敗不渲染收合列", () => {
    renderPanel({ snapshot: snapshot({ workspaceLoading: true, discussions: promoted(2) }) });
    expect(screen.queryByTestId("panel-promoted-toggle")).toBeNull();
    cleanup();
    renderPanel({ snapshot: snapshot({ workspaceLoadFailed: true, discussions: promoted(2) }) });
    expect(screen.queryByTestId("panel-promoted-toggle")).toBeNull();
  });
});

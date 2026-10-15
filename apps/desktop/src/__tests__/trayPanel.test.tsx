// 面板樣式呈現層（spec「面板樣式」；design D5）：以主視窗推送的
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
import {
  TRAY_COLLAPSED_SECTIONS_STORAGE_KEY,
  readTrayCollapsedSections,
  writeTrayCollapsedSections,
} from "../panel/sectionCollapse";
import { tabRevealScrollLeft } from "../panel/tabStrip";
import { EDGE_FADE_PX, edgeFade, edgeFadeMask } from "../panel/edgeFade";
import { APP_MESSAGES } from "../i18n/messages";
import type { TraySnapshot } from "../tray";
import { PanelRoot } from "../panel/main";
import { emit, listen } from "@tauri-apps/api/event";

vi.mock("@tauri-apps/api/event", () => ({
  emit: vi.fn().mockResolvedValue(undefined),
  listen: vi.fn().mockResolvedValue(() => {}),
}));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ setSize: vi.fn().mockResolvedValue(undefined), setTheme: vi.fn().mockResolvedValue(undefined) }),
  LogicalSize: class { constructor(public width: number, public height: number) {} },
}));
vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({ writeText: vi.fn() }));

describe("面板入口配色握手", () => {
  it("先等訂閱才 ready，首次快照套色後呈現，重開與後續快照皆使用最新值", async () => {
    vi.mocked(emit).mockClear();
    let callback!: (event: { payload: TraySnapshot }) => void;
    let finish!: (unlisten: () => void) => void;
    const stop = vi.fn();
    vi.mocked(listen).mockImplementationOnce((_event, handler) => {
      callback = handler as typeof callback;
      return new Promise((resolve) => { finish = resolve; });
    });
    const view = render(<PanelRoot />);
    view.container.id = "root";
    expect(emit).not.toHaveBeenCalled();
    expect(screen.queryByTestId("panel-header")).toBeNull();
    await act(async () => { finish(stop); });
    expect(emit).toHaveBeenCalledWith("tray-panel-ready", {});
    act(() => callback({ payload: snapshot({ themePreference: "light" }) }));
    expect(document.documentElement.dataset.theme).toBe("light");
    expect(screen.getByTestId("panel-header")).toBeTruthy();
    act(() => callback({ payload: snapshot({ themePreference: "dark" }) }));
    expect(document.documentElement.dataset.theme).toBe("dark");
    view.unmount();
    expect(stop).toHaveBeenCalledOnce();
    vi.mocked(listen).mockImplementationOnce(async (_event, handler) => {
      callback = handler as typeof callback;
      return stop;
    });
    const second = render(<PanelRoot />);
    await act(async () => {});
    act(() => callback({ payload: snapshot({ themePreference: "dark" }) }));
    expect(document.documentElement.dataset.theme).toBe("dark");
    expect(screen.getByTestId("panel-header")).toBeTruthy();
    second.unmount();
  });

  it("訂閱尚未完成就卸載時不送 ready，並清理延遲的監聽", async () => {
    vi.mocked(emit).mockClear();
    let finish!: (unlisten: () => void) => void;
    const stop = vi.fn();
    vi.mocked(listen).mockImplementationOnce(() => new Promise((resolve) => { finish = resolve; }));
    const view = render(<PanelRoot />);
    view.unmount();
    await act(async () => { finish(stop); });
    expect(stop).toHaveBeenCalledOnce();
    expect(emit).not.toHaveBeenCalled();
  });
});

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);
const render = (ui: ReactElement) => rtlRender(ui, { wrapper: zhWrapper });

// 已轉出卡的展開記憶與內容卡收合記憶都存 localStorage：每題從「未記憶」起跑，
// 前一題的展開／收合不外溢。
beforeEach(() => {
  localStorage.removeItem(TRAY_PROMOTED_EXPANDED_STORAGE_KEY);
  localStorage.removeItem(TRAY_COLLAPSED_SECTIONS_STORAGE_KEY);
});

/** b 在 a 之後（文件順序）。 */
const follows = (a: Element, b: Element) =>
  (a.compareDocumentPosition(b) & Node.DOCUMENT_POSITION_FOLLOWING) !== 0;
/** 分區卡（panel-section-<id>，id 為 proposed、discussions、promoted 等）。 */
const card = (id: string) => screen.getByTestId(`panel-section-${id}`);
/** 內容卡標題的收合觸發區（包住整條分區標題）。 */
const sectionToggle = (id: string) => within(screen.getByTestId(id)).getByTestId("panel-section-toggle");
/** 分區卡的標頭（收合觸發區內的 SectionHeader）。 */
const sectionHeader = (id: string) => sectionToggle(id).firstElementChild as HTMLElement;
/** 收合觸發內箭頭的 class token（以空白切開，才分得出 rotate-90 與 -rotate-90）。 */
const chevronTokens = (toggle: HTMLElement) =>
  (toggle.querySelector("svg.lucide-chevron-right")?.getAttribute("class") ?? "").split(/\s+/);
/** jsdom 沒有版面：注入可捲區的捲動量測（scrollTop／scrollHeight…）。 */
function injectMetrics(el: HTMLElement, metrics: Record<string, number>) {
  for (const [prop, value] of Object.entries(metrics)) {
    Object.defineProperty(el, prop, { configurable: true, value });
  }
}
/** 注入捲動量測後觸發捲動事件。 */
function scrollWith(el: HTMLElement, metrics: Record<string, number>) {
  injectMetrics(el, metrics);
  act(() => {
    fireEvent.scroll(el);
  });
}

function change(over: Partial<ChangeItem> & { name: string }): ChangeItem {
  return { status: "", totalTasks: 0, completedTasks: 0, ...over };
}

function snapshot(over: Partial<TraySnapshot> = {}): TraySnapshot {
  return {
    themePreference: "light",
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

/** 6 個本機專案 tab（p1…p6）：tab 條總寬超出可捲區的情境。 */
const six = Array.from({ length: 6 }, (_, i) => ({
  key: `local:/proj/p${i + 1}`,
  name: `p${i + 1}`,
  source: "local" as const,
  status: "ready" as const,
}));

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
  const props = { snapshot: snapshot(), platform: "macos" as const, ...handlers, ...over };
  const { rerender } = render(<TrayPanel {...props} />);
  return {
    ...handlers,
    /** 送入新快照（模擬主視窗推送），其餘 props 不變。 */
    pushSnapshot: (next: TraySnapshot) => rerender(<TrayPanel {...props} snapshot={next} />),
  };
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
          platform="macos"
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
          platform="macos"
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
    // 佔位卡不再存在（spec「面板樣式」常駐分區行為）
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

  it("討論分流：已轉出討論列於「討論」卡正下方的獨立「已轉出」卡（不在討論卡內、無舊收合列）", () => {
    renderPanel({
      snapshot: snapshot({
        discussions: [
          { slug: "open-d", topic: "討論中的", promoted: false },
          { slug: "prom-d", topic: "已轉出的", promoted: true },
        ],
      }),
    });
    const discussions = screen.getByTestId("panel-section-discussions");
    const promoted = screen.getByTestId("panel-section-promoted");
    expect(within(discussions).getByTestId("panel-discussion-open-d")).toBeTruthy();
    // 討論卡 → 已轉出卡 → 頁尾；已轉出卡是兄弟卡，不是討論卡的子節點。
    expect(discussions.contains(promoted)).toBe(false);
    expect(follows(discussions, promoted)).toBe(true);
    expect(follows(promoted, screen.getByTestId("panel-footer"))).toBe(true);
    expect(screen.queryByTestId("panel-promoted-toggle")).toBeNull();
    expect(screen.queryByTestId("panel-promoted-rows")).toBeNull();
    expect(within(promoted).getByTestId("section-count").textContent).toBe("1");
    fireEvent.click(sectionToggle("panel-section-promoted"));
    expect(within(promoted).getByTestId("panel-discussion-prom-d")).toBeTruthy();
    expect(within(promoted).queryByTestId("panel-discussion-open-d")).toBeNull();
    expect(within(discussions).queryByTestId("panel-discussion-prom-d")).toBeNull();
    expect(screen.queryByTestId("panel-promoted-rows")).toBeNull();
  });

  it("零已轉出時無已轉出卡", () => {
    renderPanel();
    expect(screen.queryByTestId("panel-section-promoted")).toBeNull();
    expect(screen.queryByText(/已轉出/)).toBeNull();
  });

  it("區塊順序：提案中→進行中→已就緒→討論→已轉出五張卡、動作區塊卡最末", () => {
    renderPanel({
      snapshot: snapshot({
        discussions: [
          { slug: "open-d", topic: "討論中的", promoted: false },
          { slug: "prom-d", topic: "已轉出的", promoted: true },
        ],
      }),
    });
    const proposed = screen.getByTestId("panel-section-proposed");
    const inProgress = screen.getByTestId("panel-section-in-progress");
    const ready = screen.getByTestId("panel-section-ready");
    const discussions = screen.getByTestId("panel-section-discussions");
    const promoted = screen.getByTestId("panel-section-promoted");
    const actions = screen.getByTestId("panel-actions-card");
    expect(follows(proposed, inProgress)).toBe(true);
    expect(follows(inProgress, ready)).toBe(true);
    expect(follows(ready, discussions)).toBe(true);
    expect(follows(discussions, promoted)).toBe(true);
    expect(follows(promoted, actions)).toBe(true);
    expect(within(actions).getByText("開啟 Speclink")).toBeTruthy();
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

describe("TrayPanel 專案 tab 條（spec「面板樣式」；design D1）", () => {
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

  it("「加入專案」與專案 tab 同構：虛線框「＋」方塊（與圖示列的「＋」同形）其下為「加入專案」文字", () => {
    renderPanel();
    const add = screen.getByTestId("panel-add-project");
    const square = add.querySelector("[data-project-tile]") as HTMLElement;
    for (const token of RAIL_SQUARE.split(" ")) expect(square.classList.contains(token)).toBe(true);
    expect(square.className.split(/\s+/)).toEqual(expect.arrayContaining(["border-dashed", "border-border"]));
    const label = within(add).getByText("加入專案");
    expect(follows(square, label)).toBe(true);
    // 文字不截斷（英文「Add project」比專案 tab 寬），寬度至少與專案 tab 對齊。
    expect(label.className.split(/\s+/)).toContain("whitespace-nowrap");
    expect(add.className.split(/\s+/)).toContain("min-w-16");
  });

  it("「加入專案」固定在 tab 條卡右端、不在可捲區內，帶名稱且點擊觸發 onAddProject 而非 onOpenProject（D7）", () => {
    const h = renderPanel();
    const card = screen.getByTestId("panel-project-card");
    const strip = screen.getByTestId("panel-project-tabs");
    const add = within(card).getByTestId("panel-add-project");
    expect(strip.contains(add)).toBe(false);
    expect(follows(strip, add)).toBe(true);
    expect(add.getAttribute("aria-label")).toBe("加入專案");
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

// spec「面板樣式」的邊緣淡出（design「專案 tab 條：「＋」固定右端、邊緣淡出、捲到作用中」
// 「中段上下緣淡出：與 tab 條共用邊緣淡出」）：淡出判斷與遮罩是 edgeFade.ts 的純函式、tab 條與中段
// 共用——start＝左／上、end＝右／下。
describe("面板可捲區的邊緣淡出", () => {
  // spec Example「淡出的側邊」（tab 條）：可捲區寬 250、tab 總寬 500（未超出那列為 250）。
  it.each([
    { position: "最左端", scrollLeft: 0, scrollWidth: 500, fade: "end" },
    { position: "中間", scrollLeft: 100, scrollWidth: 500, fade: "both" },
    { position: "最右端", scrollLeft: 250, scrollWidth: 500, fade: "start" },
    { position: "tab 總寬未超出", scrollLeft: 0, scrollWidth: 250, fade: "none" },
  ])("tab 條：$position → $fade", ({ scrollLeft, scrollWidth, fade }) => {
    expect(edgeFade({ position: scrollLeft, scrollSize: scrollWidth, viewportSize: 250 })).toBe(fade);
  });

  // spec Example「中段淡出的邊緣」：中段可視高 400、內容高 900（未超限那列為 400）。
  it.each([
    { position: "最上方", scrollTop: 0, scrollHeight: 900, fade: "end" },
    { position: "中間", scrollTop: 250, scrollHeight: 900, fade: "both" },
    { position: "最下方", scrollTop: 500, scrollHeight: 900, fade: "start" },
    { position: "內容未超限", scrollTop: 0, scrollHeight: 400, fade: "none" },
  ])("中段：$position → $fade", ({ scrollTop, scrollHeight, fade }) => {
    expect(edgeFade({ position: scrollTop, scrollSize: scrollHeight, viewportSize: 400 })).toBe(fade);
  });

  it("容差 1px：小數捲動位置差不到 1px 視為已到盡頭", () => {
    expect(edgeFade({ position: 0.6, scrollSize: 500, viewportSize: 250 })).toBe("end");
    expect(edgeFade({ position: 249.5, scrollSize: 500, viewportSize: 250 })).toBe("start");
  });

  it("遮罩：不淡出時不掛遮罩；橫向沿 to right、縱向沿 to bottom，淡出側漸到透明", () => {
    expect(edgeFadeMask("x", "none")).toBeUndefined();
    expect(edgeFadeMask("y", "none")).toBeUndefined();
    expect(edgeFadeMask("x", "start")).toBe(`linear-gradient(to right, transparent, black ${EDGE_FADE_PX}px, black)`);
    expect(edgeFadeMask("y", "end")).toBe(
      `linear-gradient(to bottom, black, black calc(100% - ${EDGE_FADE_PX}px), transparent)`,
    );
    expect(edgeFadeMask("y", "both")).toBe(
      `linear-gradient(to bottom, transparent, black ${EDGE_FADE_PX}px, black calc(100% - ${EDGE_FADE_PX}px), transparent)`,
    );
  });

  it("中段捲動後上下緣淡出隨是否還有內容改變；固定頁首與頁尾不帶淡出", () => {
    renderPanel();
    const scroll = screen.getByTestId("panel-scroll");
    const scrollTo = (scrollTop: number, scrollHeight = 900) =>
      scrollWith(scroll, { scrollTop, scrollHeight, clientHeight: 400 });
    expect(scroll.getAttribute("data-fade")).toBe("none");
    scrollTo(0);
    expect(scroll.getAttribute("data-fade")).toBe("end");
    scrollTo(250);
    expect(scroll.getAttribute("data-fade")).toBe("both");
    scrollTo(500);
    expect(scroll.getAttribute("data-fade")).toBe("start");
    scrollTo(0, 400);
    expect(scroll.getAttribute("data-fade")).toBe("none");
    for (const id of ["panel-header", "panel-footer"]) {
      expect(screen.getByTestId(id).getAttribute("data-fade")).toBeNull();
      expect(screen.getByTestId(id).getAttribute("style")).toBeNull();
    }
  });

  it("尺寸改變時重算淡出：中段內容 wrapper 變高、tab 清單改變，不必等捲動事件", () => {
    // 可手動觸發的 ResizeObserver 替身（全域 stub 是 no-op）：記下每個觀察者與它觀察的元素。
    const observers: Array<{ callback: () => void; targets: Element[] }> = [];
    vi.stubGlobal(
      "ResizeObserver",
      class {
        private entry: { callback: () => void; targets: Element[] };
        constructor(callback: () => void) {
          this.entry = { callback, targets: [] };
          observers.push(this.entry);
        }
        observe(el: Element) {
          this.entry.targets.push(el);
        }
        unobserve() {}
        disconnect() {
          this.entry.targets = [];
        }
      },
    );
    try {
      const h = renderPanel({ snapshot: snapshot({ tabs: six.slice(0, 3), activeKey: six[0].key }) });
      // 中段：內容 wrapper 變高到超出可視高（收合展開、快照增減）→ 下緣淡出。
      const scroll = screen.getByTestId("panel-scroll");
      const content = screen.getByTestId("panel-scroll-content");
      injectMetrics(scroll, { scrollTop: 0, scrollHeight: 900, clientHeight: 400 });
      expect(scroll.getAttribute("data-fade")).toBe("none");
      act(() => {
        for (const o of observers) if (o.targets.includes(content)) o.callback();
      });
      expect(scroll.getAttribute("data-fade")).toBe("end");
      // tab 條：tab 清單由 3 個變 6 個、總寬超出可捲區 → 右側淡出。
      const strip = screen.getByTestId("panel-project-tabs");
      injectMetrics(strip, { scrollLeft: 0, scrollWidth: 416, clientWidth: 250 });
      // 注入尺寸本身不觸發重算（jsdom 掛載時的值沿用）；tab 清單改變才重算。
      expect(strip.getAttribute("data-fade")).not.toBe("end");
      h.pushSnapshot(snapshot({ tabs: six, activeKey: six[0].key }));
      expect(strip.getAttribute("data-fade")).toBe("end");
    } finally {
      vi.unstubAllGlobals();
    }
  });
});

// spec「面板樣式」的專案 tab 條捲動（design「專案 tab 條：「＋」固定右端、邊緣淡出、
// 捲到作用中」）：露出目標是 tabStrip.ts 的純函式；jsdom 沒有版面，元件測試以注入尺寸驗證接線。
describe("面板專案 tab 條的淡出與捲到作用中", () => {

  // 露出目標：tab 寬 64、可捲區寬 250。結果要讓 tab 完整落在可視範圍內。
  const fullyVisible = (tabLeft: number, scrollLeft: number) =>
    tabLeft >= scrollLeft && tabLeft + 64 <= scrollLeft + 250;

  it("露出目標：已完整可見 → 不動", () => {
    expect(tabRevealScrollLeft({ tabLeft: 74, tabWidth: 64, scrollLeft: 0, viewportWidth: 250 })).toBe(0);
    expect(tabRevealScrollLeft({ tabLeft: 142, tabWidth: 64, scrollLeft: 60, viewportWidth: 250 })).toBe(60);
  });

  it("露出目標：在右側外 → 往右捲到它完整可見", () => {
    const next = tabRevealScrollLeft({ tabLeft: 346, tabWidth: 64, scrollLeft: 0, viewportWidth: 250 });
    expect(next).toBeGreaterThan(0);
    expect(fullyVisible(346, next)).toBe(true);
  });

  it("露出目標：在左側外 → 往左捲到它完整可見，且不小於 0", () => {
    const next = tabRevealScrollLeft({ tabLeft: 74, tabWidth: 64, scrollLeft: 150, viewportWidth: 250 });
    expect(next).toBeLessThan(150);
    expect(next).toBeGreaterThanOrEqual(0);
    expect(fullyVisible(74, next)).toBe(true);
    expect(tabRevealScrollLeft({ tabLeft: 6, tabWidth: 64, scrollLeft: 150, viewportWidth: 250 })).toBe(0);
  });

  it("捲動事件後可捲區的遮罩隨左右是否還有隱藏的 tab 改變", () => {
    renderPanel({ snapshot: snapshot({ tabs: six, activeKey: six[0].key }) });
    const strip = screen.getByTestId("panel-project-tabs");
    const scrollTo = (scrollLeft: number, scrollWidth = 500) =>
      scrollWith(strip, { scrollLeft, scrollWidth, clientWidth: 250 });
    scrollTo(0);
    expect(strip.getAttribute("data-fade")).toBe("end");
    scrollTo(100);
    expect(strip.getAttribute("data-fade")).toBe("both");
    scrollTo(250);
    expect(strip.getAttribute("data-fade")).toBe("start");
    scrollTo(0, 250);
    expect(strip.getAttribute("data-fade")).toBe("none");
  });

  it("6 個專案、作用中為第 6 個：開啟時捲到它完整可見、左側淡出；面板再次取得焦點時重新捲回", () => {
    // 版面注入：tab 寬 64、間距 4、可捲區內距 6、可捲區寬 250（第 4 個起超出）。
    const tabLeft = (el: Element) => {
      const n = /^panel-project-local:\/proj\/p(\d)$/.exec(el.getAttribute("data-testid") ?? "")?.[1];
      return n === undefined ? null : 6 + (Number(n) - 1) * 68;
    };
    const isStrip = (el: Element) => el.getAttribute("data-testid") === "panel-project-tabs";
    const scrollLefts = new WeakMap<Element, number>();
    const stubs: Array<[object, string, PropertyDescriptor]> = [
      [HTMLElement.prototype, "offsetLeft", { get(this: Element) { return tabLeft(this) ?? 0; } }],
      [HTMLElement.prototype, "offsetWidth", { get(this: Element) { return tabLeft(this) === null ? 0 : 64; } }],
      [Element.prototype, "clientWidth", { get(this: Element) { return isStrip(this) ? 250 : 0; } }],
      [Element.prototype, "scrollWidth", { get(this: Element) { return isStrip(this) ? 416 : 0; } }],
      [
        Element.prototype,
        "scrollLeft",
        {
          get(this: Element) { return scrollLefts.get(this) ?? 0; },
          set(this: Element, v: number) { scrollLefts.set(this, v); },
        },
      ],
    ];
    const originals = stubs.map(([proto, name]) => [proto, name, Object.getOwnPropertyDescriptor(proto, name)] as const);
    for (const [proto, name, desc] of stubs) Object.defineProperty(proto, name, { configurable: true, ...desc });
    try {
      renderPanel({ snapshot: snapshot({ tabs: six, activeKey: six[5].key }) });
      const strip = screen.getByTestId("panel-project-tabs");
      const sixthLeft = 6 + 5 * 68;
      expect(strip.scrollLeft).toBeGreaterThan(0);
      expect(fullyVisible(sixthLeft, strip.scrollLeft)).toBe(true);
      // 捲到作用中 tab 後左側有隱藏的 tab、右側已到盡頭：只淡左（spec 情境「開啟面板時捲到作用中專案」）。
      expect(strip.getAttribute("data-fade")).toBe("start");
      // 使用者把 tab 條捲回最左後關閉面板；再次開啟（面板視窗取得焦點）時捲回作用中 tab。
      strip.scrollLeft = 0;
      act(() => {
        fireEvent.focus(window);
      });
      expect(fullyVisible(sixthLeft, strip.scrollLeft)).toBe(true);
    } finally {
      for (const [proto, name, desc] of originals) {
        if (desc) Object.defineProperty(proto, name, desc);
        else delete (proto as Record<string, unknown>)[name];
      }
    }
  });
});

describe("TrayPanel 分區卡片化與主色（spec「面板樣式」；design D2／D3）", () => {
  const discBoth = [
    { slug: "open-d", topic: "討論中的", promoted: false },
    { slug: "prom-d", topic: "已轉出的", promoted: true },
  ];

  it("tab 條、各分區（含已轉出卡）與動作區各自為白底細框 12px 圓角卡", () => {
    renderPanel({ snapshot: snapshot({ discussions: discBoth }) });
    for (const id of [
      "panel-project-card",
      "panel-section-proposed",
      "panel-section-in-progress",
      "panel-section-ready",
      "panel-section-discussions",
      "panel-section-promoted",
      "panel-actions-card",
    ]) {
      const classes = screen.getByTestId(id).className.split(/\s+/);
      expect(classes).toEqual(expect.arrayContaining(["bg-card", "border", "border-border", "rounded-xl"]));
    }
  });

  it("生命週期與討論分區圖示各取所屬色相，已轉出卡標頭為中性灰的討論圖示", () => {
    // spec「生命週期四欄各一色相」：系統匣分區與看板四欄同色，同一來源（stage.ts）；
    // 已轉出不是生命週期階段：圖示與計數徽章維持中性（desktop-app「介面狀態語意色分層」）。
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
    // 已轉出卡：與「討論」卡同款圖示，但灰色；不用頁尾「開啟 Speclink」的箭頭圖示。
    const promoted = screen.getByTestId("panel-section-promoted");
    expect(iconClasses("panel-section-discussions")).toContain("lucide-message-square-text");
    expect(iconClasses("panel-section-promoted")).toContain("lucide-message-square-text");
    expect(iconClasses("panel-section-promoted")).toContain("text-muted-foreground");
    expect(iconClasses("panel-section-promoted")).not.toContain(DISCUSSION_TONE.icon);
    // 計數徽章沿用 Badge 的 secondary 變體（中性灰底），呼叫端不以 className 覆寫顏色。
    const badge = within(promoted).getByTestId("section-count").className.split(/\s+/);
    expect(badge).toEqual(expect.arrayContaining(["bg-muted", "text-foreground"]));
    expect(badge).not.toContain("text-muted-foreground");
    for (const token of DISCUSSION_TONE.badge.split(/\s+/)) expect(badge).not.toContain(token);
    expect(promoted.querySelector("svg.lucide-arrow-up-right")).toBeNull();
    // 展開後已轉出列不另加底色：列的祖先節點都沒有舊收合列的 bg-muted/40。
    fireEvent.click(sectionToggle("panel-section-promoted"));
    for (let el = screen.getByTestId("panel-discussion-prom-d").parentElement; el; el = el.parentElement) {
      expect((el.getAttribute("class") ?? "").split(/\s+/)).not.toContain("bg-muted/40");
    }
  });

  it("根容器以毛玻璃同半徑圓角裁切（wash 不得畫出 vibrancy 圓角外）", () => {
    renderPanel();
    const root = screen.getByTestId("panel-root");
    expect(root.className.split(/\s+/)).toContain("rounded-[13px]");
  });

  it("根層不疊漸層、以側欄面色約六成補光，白卡維持不透明", () => {
    // 主色留給選取與主要操作；漸層 wash 隨卡片化移除。補光層調回約六成，毛玻璃才從
    // 外框與卡片縫隙透得出來；卡片本身維持不透明白底（卡內亮度錨定卡片面色）。
    renderPanel({ snapshot: snapshot({ discussions: discBoth }) });
    const root = screen.getByTestId("panel-root");
    expect(root.className).not.toContain("bg-linear-to-b");
    expect(root.className).not.toContain("from-");
    expect(root.className.split(/\s+/)).toContain("bg-sidebar/60");
    expect(root.className.split(/\s+/)).not.toContain("bg-sidebar/90");
    for (const id of [
      "panel-project-card",
      "panel-section-proposed",
      "panel-section-in-progress",
      "panel-section-ready",
      "panel-section-discussions",
      "panel-actions-card",
    ]) {
      expect(screen.getByTestId(id).className.split(/\s+/)).toContain("bg-card");
    }
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

describe("面板根層的平台外觀（spec「面板樣式」平台差異；tray-panel-windows design D2）", () => {
  it("Windows：不透明側欄底色、不畫圓角、不鋪半透明補光", () => {
    renderPanel({ platform: "windows" });
    const classes = screen.getByTestId("panel-root").className.split(/\s+/);
    expect(classes).toContain("bg-sidebar");
    expect(classes).not.toContain("rounded-[13px]");
    expect(classes).not.toContain("bg-sidebar/60");
  });

  it("macOS：維持 13px 圓角與約六成補光層", () => {
    renderPanel({ platform: "macos" });
    const classes = screen.getByTestId("panel-root").className.split(/\s+/);
    expect(classes).toContain("rounded-[13px]");
    expect(classes).toContain("bg-sidebar/60");
    expect(classes).not.toContain("bg-sidebar");
  });
});

describe("TrayPanel 分區計數（spec「面板樣式」；design D8）", () => {
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

  it("動作區塊卡為單列：「開啟 Speclink」文字鈕在前，其後依序為「專案設定」「設定」「結束」三顆圖示鈕", () => {
    const h = renderPanel();
    const actions = screen.getByTestId("panel-actions-card");
    const open = within(actions).getByText("開啟 Speclink");
    const iconButtons = within(actions)
      .getAllByRole("button")
      .filter((b) => b.textContent === "");
    expect(iconButtons.map((b) => b.getAttribute("aria-label"))).toEqual(["專案設定", "設定", "結束"]);
    const [projectSettings, settings, quit] = iconButtons;
    for (const b of iconButtons) {
      // 名稱提示走原生 title（面板無焦點，不掛 Radix Tooltip）；圖示鈕退出 Tab 順序。
      expect(b.getAttribute("title")).toBe(b.getAttribute("aria-label"));
      expect(b.getAttribute("tabindex")).toBe("-1");
      expect(b.getAttribute("type")).toBe("button");
      expect(follows(open, b)).toBe(true);
    }
    // 文字鈕與圖示鈕的 hover 為半透明前景色灰底，不以主色反白。
    for (const b of [open.closest("button")!, ...iconButtons]) {
      expect(b.className.split(/\s+/)).toContain("hover:bg-foreground/5");
      expect(b.className).not.toContain("hover:bg-primary");
    }
    // 「設定」與「結束」之間以細直線分隔符隔開防誤按：間距由分隔符兩側外距承擔，鈕本身不帶左外距。
    const separator = within(actions).getByTestId("panel-action-separator");
    expect(separator.getAttribute("aria-hidden")).toBe("true");
    expect(follows(settings, separator)).toBe(true);
    expect(follows(separator, quit)).toBe(true);
    expect(follows(projectSettings, separator)).toBe(true);
    for (const b of [open.closest("button")!, ...iconButtons]) {
      expect(b.className).not.toMatch(/(^|\s)ml-\d/);
    }
    // 四顆鈕同在一列（同一個父節點）。
    expect(new Set([open.closest("button")!, ...iconButtons].map((b) => b.parentElement)).size).toBe(1);
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

describe("三段式版面（spec「面板樣式」：固定頁首／可捲中段／固定頁尾）", () => {
  it("頁首為 tab 條卡、中段含分區卡並帶縱向捲動樣式、頁尾為動作單列卡；無分割線", () => {
    renderPanel();
    const header = screen.getByTestId("panel-header");
    const scroll = screen.getByTestId("panel-scroll");
    const footer = screen.getByTestId("panel-footer");
    // 頁首＝tab 條卡；中段＝生命週期與討論分區卡；頁尾＝動作單列卡
    expect(within(within(header).getByTestId("panel-project-card")).getByTestId("panel-project-tabs")).toBeTruthy();
    expect(within(scroll).getByTestId("panel-section-discussions")).toBeTruthy();
    expect(within(scroll).getByTestId("panel-section-proposed")).toBeTruthy();
    expect(within(scroll).getByTestId("panel-section-ready")).toBeTruthy();
    const actions = within(footer).getByTestId("panel-actions-card");
    expect(within(actions).getByText("開啟 Speclink")).toBeTruthy();
    for (const name of ["專案設定", "設定", "結束"]) {
      expect(within(actions).getByRole("button", { name })).toBeTruthy();
    }
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
    // 濃度約六成、真機可微調——只斷言補光層存在（bg-sidebar/<n>）
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
    expect(within(within(footer).getByTestId("panel-actions-card")).getByRole("button", { name: "結束" })).toBeTruthy();
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

// spec「討論列表」面板的已轉出卡展開記憶（design D4）：app 本機一鍵、與看板的記憶分開；
// 缺席或壞值視為收合，localStorage 不可用時讀寫都不拋錯。
describe("面板已轉出卡的展開記憶", () => {
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

// spec「面板內容卡收合」（design「收合觸發沿用已轉出收合列的模式」「收合中的卡只留標題一行、
// 不自動展開」）：四張內容卡的整條標題是收合觸發區；收合後只剩標題一行，計數照常更新。
describe("面板內容卡收合", () => {
  const SECTIONS = ["proposed", "in-progress", "ready", "discussions"] as const;
  const toggle = (id: string) => sectionToggle(`panel-section-${id}`);
  const threeInProgress = [
    change({ name: "ip1", totalTasks: 4, completedTasks: 1 }),
    change({ name: "ip2", totalTasks: 4, completedTasks: 2 }),
    change({ name: "ip3", totalTasks: 4, completedTasks: 3 }),
  ];

  it("(a) 點「進行中」標題收合：列消失、aria-expanded 為 false、計數仍為 3、箭頭朝右；再點恢復", () => {
    renderPanel({ snapshot: snapshot({ changes: threeInProgress }) });
    expect(toggle("in-progress").getAttribute("aria-expanded")).toBe("true");
    expect(chevronTokens(toggle("in-progress"))).toContain("rotate-90");
    fireEvent.click(toggle("in-progress"));
    expect(toggle("in-progress").getAttribute("aria-expanded")).toBe("false");
    expect(within(card("in-progress")).queryByTestId(/^panel-change-/)).toBeNull();
    expect(within(card("in-progress")).getByTestId("section-count").textContent).toBe("3");
    expect(within(card("in-progress")).getByText("進行中")).toBeTruthy();
    expect(chevronTokens(toggle("in-progress"))).not.toContain("rotate-90");
    // 其他卡不受影響。
    expect(toggle("proposed").getAttribute("aria-expanded")).toBe("true");
    fireEvent.click(toggle("in-progress"));
    expect(toggle("in-progress").getAttribute("aria-expanded")).toBe("true");
    expect(within(card("in-progress")).getAllByTestId(/^panel-change-/)).toHaveLength(3);
    expect(chevronTokens(toggle("in-progress"))).toContain("rotate-90");
  });

  it("(b) 收合「討論」後重開面板、再切到另一個專案仍收合；記憶值為含 discussions 的 JSON 陣列", () => {
    renderPanel({
      snapshot: snapshot({
        discussions: [
          { slug: "w1", topic: "一", promoted: false },
          { slug: "w2", topic: "二", promoted: false },
          { slug: "w3", topic: "三", promoted: false },
        ],
      }),
    });
    fireEvent.click(toggle("discussions"));
    expect(JSON.parse(localStorage.getItem(TRAY_COLLAPSED_SECTIONS_STORAGE_KEY)!)).toContain("discussions");
    // 重新掛載＝關閉再開啟面板、重啟 app。
    cleanup();
    const h = renderPanel({
      snapshot: snapshot({ discussions: [{ slug: "w1", topic: "一", promoted: false }] }),
    });
    expect(toggle("discussions").getAttribute("aria-expanded")).toBe("false");
    // 主視窗推送另一個作用中專案的快照（2 則討論）：仍收合、計數換成新專案的。
    h.pushSnapshot(
      snapshot({
        activeKey: "local:/proj/two",
        discussions: [
          { slug: "s1", topic: "甲", promoted: false },
          { slug: "s2", topic: "乙", promoted: false },
        ],
      }),
    );
    expect(toggle("discussions").getAttribute("aria-expanded")).toBe("false");
    expect(within(card("discussions")).getByTestId("section-count").textContent).toBe("2");
    expect(within(card("discussions")).queryByTestId(/^panel-discussion-/)).toBeNull();
  });

  it("spec Example「收起討論後切換專案」：wadpilot 8 則收起後點 speclink（2 則）→ 只剩一行「討論」與計數 2，箭頭朝右", () => {
    const wadpilot = { key: "local:/proj/wadpilot", name: "wadpilot", source: "local" as const, status: "ready" as const };
    const speclink = { key: "local:/proj/speclink", name: "speclink", source: "local" as const, status: "ready" as const };
    const topics = (prefix: string, n: number) =>
      Array.from({ length: n }, (_, i) => ({ slug: `${prefix}-${i}`, topic: `${prefix} ${i}`, promoted: false }));
    const h = renderPanel({
      snapshot: snapshot({ tabs: [wadpilot, speclink], activeKey: wadpilot.key, discussions: topics("w", 8) }),
    });
    fireEvent.click(toggle("discussions"));
    // 點 speclink 的專案 tab：面板回呼切換，主視窗推送 speclink 的快照。
    fireEvent.click(screen.getByTestId(`panel-project-${speclink.key}`));
    expect(h.onOpenProject).toHaveBeenCalledWith(speclink.key);
    h.pushSnapshot(snapshot({ tabs: [wadpilot, speclink], activeKey: speclink.key, discussions: topics("s", 2) }));
    expect(toggle("discussions").getAttribute("aria-expanded")).toBe("false");
    expect(card("discussions").children).toHaveLength(1);
    expect(within(card("discussions")).getByText("討論")).toBeTruthy();
    expect(within(card("discussions")).getByTestId("section-count").textContent).toBe("2");
    expect(chevronTokens(toggle("discussions"))).not.toContain("rotate-90");
    expect(chevronTokens(toggle("discussions"))).not.toContain("-rotate-90");
  });

  it("(c) 記憶存壞值時四張卡全部展開、不顯示錯誤", () => {
    for (const raw of ["not json", '{"proposed":true}']) {
      localStorage.setItem(TRAY_COLLAPSED_SECTIONS_STORAGE_KEY, raw);
      renderPanel();
      for (const id of SECTIONS) expect(toggle(id).getAttribute("aria-expanded")).toBe("true");
      expect(screen.queryByRole("alert")).toBeNull();
      cleanup();
    }
  });

  it("(d) 收合「已就緒」後送入多一筆已就緒變更的快照：卡仍收合、計數由 0 變 1", () => {
    const base = [change({ name: "prop", totalTasks: 5, completedTasks: 0 })];
    const h = renderPanel({ snapshot: snapshot({ changes: base }) });
    expect(within(card("ready")).getByTestId("section-count").textContent).toBe("0");
    fireEvent.click(toggle("ready"));
    h.pushSnapshot(snapshot({ changes: [...base, change({ name: "rdy", totalTasks: 4, completedTasks: 4 })] }));
    expect(toggle("ready").getAttribute("aria-expanded")).toBe("false");
    expect(within(card("ready")).getByTestId("section-count").textContent).toBe("1");
    expect(screen.queryByTestId("panel-change-rdy")).toBeNull();
  });

  it("(e) 收合的卡只剩標題一行：空狀態卡不帶最小高度、標題下沒有細線", () => {
    renderPanel({ snapshot: snapshot({ changes: threeInProgress }) });
    fireEvent.click(toggle("ready")); // 計數 0 的空狀態卡
    fireEvent.click(toggle("in-progress")); // 有 3 列的卡
    for (const id of ["ready", "in-progress"]) {
      expect(card(id).className.split(/\s+/)).not.toContain("min-h-12");
      expect(sectionHeader(`panel-section-${id}`).className.split(/\s+/)).not.toContain("border-b");
      // 卡內只剩收合觸發區一個子節點。
      expect(card(id).children).toHaveLength(1);
    }
  });

  it("(e) 收合的卡在載入中與載入失敗時也只剩標題（不渲染骨架、失敗提示）", () => {
    renderPanel();
    fireEvent.click(toggle("in-progress"));
    fireEvent.click(toggle("discussions"));
    cleanup();
    renderPanel({ snapshot: snapshot({ workspaceLoading: true, changes: [], discussions: [] }) });
    for (const id of ["in-progress", "discussions"]) {
      expect(card(id).querySelector('[aria-busy="true"]')).toBeNull();
      expect(card(id).children).toHaveLength(1);
    }
    expect(card("proposed").querySelector('[aria-busy="true"]')).toBeTruthy();
    cleanup();
    renderPanel({ snapshot: snapshot({ workspaceLoadFailed: true, changes: [], discussions: [] }) });
    for (const id of ["in-progress", "discussions"]) {
      expect(within(card(id)).queryByTestId("panel-section-load-failed")).toBeNull();
    }
    expect(within(card("proposed")).getByTestId("panel-section-load-failed")).toBeTruthy();
  });

  it("(f) 討論卡收起再展開：「還有 N 個…」溢出列回到未展開，「已轉出」卡全程照常呈現且維持展開", () => {
    const openEight = Array.from({ length: 8 }, (_, i) => ({ slug: `open-${i}`, topic: `討論 ${i}`, promoted: false }));
    const promotedThree = Array.from({ length: 3 }, (_, i) => ({ slug: `prom-${i}`, topic: `已轉出 ${i}`, promoted: true }));
    renderPanel({ snapshot: snapshot({ discussions: [...openEight, ...promotedThree] }) });
    const promotedRows = () => within(card("promoted")).getAllByTestId(/^panel-discussion-/);
    fireEvent.click(screen.getByText("還有 3 個…"));
    fireEvent.click(toggle("promoted"));
    expect(screen.getByTestId("panel-discussion-open-7")).toBeTruthy();
    fireEvent.click(toggle("discussions"));
    expect(screen.queryByText("還有 3 個…")).toBeNull();
    expect(toggle("promoted").getAttribute("aria-expanded")).toBe("true");
    expect(promotedRows()).toHaveLength(3);
    fireEvent.click(toggle("discussions"));
    expect(screen.getByText("還有 3 個…")).toBeTruthy();
    expect(screen.queryByTestId("panel-discussion-open-5")).toBeNull();
    expect(toggle("promoted").getAttribute("aria-expanded")).toBe("true");
    expect(promotedRows()).toHaveLength(3);
  });

  it("收起討論卡時已轉出卡照常呈現：仍在其下、收合狀態與計數不變", () => {
    const promotedThree = Array.from({ length: 3 }, (_, i) => ({ slug: `prom-${i}`, topic: `已轉出 ${i}`, promoted: true }));
    renderPanel({ snapshot: snapshot({ discussions: [{ slug: "d1", topic: "討論一", promoted: false }, ...promotedThree] }) });
    expect(toggle("promoted").getAttribute("aria-expanded")).toBe("false");
    fireEvent.click(toggle("discussions"));
    expect(card("discussions").children).toHaveLength(1);
    expect(toggle("promoted").getAttribute("aria-expanded")).toBe("false");
    expect(within(card("promoted")).getByTestId("section-count").textContent).toBe("3");
    expect(follows(card("discussions"), card("promoted"))).toBe(true);
  });

  it("沒有收合記憶時四張卡展開、已轉出卡收合（收合集合壞值、已轉出記憶缺席）", () => {
    localStorage.setItem(TRAY_COLLAPSED_SECTIONS_STORAGE_KEY, "not json");
    const promotedTwo = Array.from({ length: 2 }, (_, i) => ({ slug: `prom-${i}`, topic: `已轉出 ${i}`, promoted: true }));
    renderPanel({ snapshot: snapshot({ discussions: promotedTwo }) });
    for (const id of SECTIONS) expect(toggle(id).getAttribute("aria-expanded")).toBe("true");
    expect(toggle("promoted").getAttribute("aria-expanded")).toBe("false");
    expect(within(card("promoted")).getByTestId("section-count").textContent).toBe("2");
    expect(chevronTokens(toggle("promoted"))).not.toContain("rotate-90");
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("切換已轉出卡不寫入收合集合（展開狀態只存已轉出自己的記憶）", () => {
    const promotedTwo = Array.from({ length: 2 }, (_, i) => ({ slug: `prom-${i}`, topic: `已轉出 ${i}`, promoted: true }));
    renderPanel({ snapshot: snapshot({ discussions: promotedTwo }) });
    // 寫入端會濾掉未知 id，看集合內容驗不出來；改驗收合集合的記憶鍵根本沒被寫入。
    fireEvent.click(toggle("promoted"));
    fireEvent.click(toggle("promoted"));
    expect(localStorage.getItem(TRAY_PROMOTED_EXPANDED_STORAGE_KEY)).toBe("false");
    expect(localStorage.getItem(TRAY_COLLAPSED_SECTIONS_STORAGE_KEY)).toBeNull();
    // 收合集合已有內容時也一樣：收起「提案中」後再切換已轉出，不再寫收合集合。
    fireEvent.click(toggle("proposed"));
    const set = vi.spyOn(Object.getPrototypeOf(localStorage) as Storage, "setItem");
    try {
      fireEvent.click(toggle("promoted"));
      expect(set).toHaveBeenCalledWith(TRAY_PROMOTED_EXPANDED_STORAGE_KEY, "true");
      expect(set.mock.calls.map(([key]) => key)).not.toContain(TRAY_COLLAPSED_SECTIONS_STORAGE_KEY);
    } finally {
      set.mockRestore();
    }
  });

  it("(g) 專案 tab 條卡、動作區塊卡、復原卡、stale 狀態列沒有收合觸發；點其空白處不收起內容", () => {
    renderPanel();
    for (const id of ["panel-project-card", "panel-actions-card"]) {
      const el = screen.getByTestId(id);
      expect(within(el).queryByTestId("panel-section-toggle")).toBeNull();
      expect(el.querySelector("[aria-expanded]")).toBeNull();
      fireEvent.click(el);
    }
    for (const id of SECTIONS) expect(toggle(id).getAttribute("aria-expanded")).toBe("true");
    cleanup();

    const remote = (status: string) =>
      snapshot({
        tabs: [
          {
            key: "remote:c1/demo/backend",
            name: "Demo/backend",
            source: "remote",
            status,
            failureKind: "unreachable",
            connectionId: "c1",
            serverLabel: "Team Server",
          },
        ],
        activeKey: "remote:c1/demo/backend",
      } as unknown as Partial<TraySnapshot>);
    renderPanel({ snapshot: remote("error") });
    expect(screen.getByTestId("panel-recovery-card").querySelector("[aria-expanded]")).toBeNull();
    expect(screen.queryAllByTestId("panel-section-toggle")).toHaveLength(0);
    cleanup();
    renderPanel({ snapshot: remote("offline") });
    const stale = screen.getByTestId("panel-stale-status");
    expect(stale.querySelector("[aria-expanded]")).toBeNull();
    expect(stale.getAttribute("aria-expanded")).toBeNull();
  });

  it("(h) 收合觸發為 role=button、tabIndex 為 -1，Enter／Space 可切換", () => {
    renderPanel();
    for (const id of SECTIONS) {
      expect(toggle(id).getAttribute("role")).toBe("button");
      expect(toggle(id).getAttribute("tabindex")).toBe("-1");
    }
    fireEvent.keyDown(toggle("proposed"), { key: "Enter" });
    expect(toggle("proposed").getAttribute("aria-expanded")).toBe("false");
    fireEvent.keyDown(toggle("proposed"), { key: " " });
    expect(toggle("proposed").getAttribute("aria-expanded")).toBe("true");
    fireEvent.keyDown(toggle("proposed"), { key: "a" });
    expect(toggle("proposed").getAttribute("aria-expanded")).toBe("true");
    // 按住不放的自動重複不再切換（原生 button 按住也只觸發一次）。
    fireEvent.keyDown(toggle("proposed"), { key: "Enter" });
    for (const key of ["Enter", " "]) {
      fireEvent.keyDown(toggle("proposed"), { key, repeat: true });
      expect(toggle("proposed").getAttribute("aria-expanded")).toBe("false");
    }
    expect(JSON.parse(localStorage.getItem(TRAY_COLLAPSED_SECTIONS_STORAGE_KEY)!)).toEqual(["proposed"]);
  });
});

// spec「面板內容卡收合」的記憶（design「收合狀態記在面板本機、所有專案共用一份」）：
// 單一鍵、值為收合中分區 id 的 JSON 陣列；讀不到或讀到壞值一律全部展開，寫入失敗靜默。
describe("面板內容卡的收合記憶", () => {
  it("鍵缺席 → 空集合（全部展開）", () => {
    expect(TRAY_COLLAPSED_SECTIONS_STORAGE_KEY).toBe("speclink.tray.collapsedSections");
    expect(readTrayCollapsedSections().size).toBe(0);
  });

  it.each([
    ["非 JSON", "not json"],
    ["非陣列（物件）", '{"proposed":true}'],
    ["非陣列（字串）", '"proposed"'],
    ["null", "null"],
  ])("壞值（%s）→ 空集合", (_label, raw) => {
    localStorage.setItem(TRAY_COLLAPSED_SECTIONS_STORAGE_KEY, raw);
    expect(readTrayCollapsedSections().size).toBe(0);
  });

  it("陣列中的未知 id 與非字串元素被忽略", () => {
    localStorage.setItem(
      TRAY_COLLAPSED_SECTIONS_STORAGE_KEY,
      JSON.stringify(["ready", "bogus", 3, null, "discussions"]),
    );
    expect([...readTrayCollapsedSections()].sort()).toEqual(["discussions", "ready"]);
  });

  it("寫入以 JSON 陣列整份覆寫，讀回同一集合", () => {
    writeTrayCollapsedSections(new Set(["proposed", "in-progress"]));
    writeTrayCollapsedSections(new Set(["discussions"]));
    const raw = localStorage.getItem(TRAY_COLLAPSED_SECTIONS_STORAGE_KEY)!;
    expect(JSON.parse(raw)).toEqual(["discussions"]);
    expect([...readTrayCollapsedSections()]).toEqual(["discussions"]);
    writeTrayCollapsedSections(new Set());
    expect(JSON.parse(localStorage.getItem(TRAY_COLLAPSED_SECTIONS_STORAGE_KEY)!)).toEqual([]);
  });

  it("localStorage 不可用時讀回空集合、寫入不拋錯", () => {
    // spy 落在 localStorage 實例的 prototype（同「已轉出卡的展開記憶」的做法）。
    const proto = Object.getPrototypeOf(localStorage) as Storage;
    localStorage.setItem(TRAY_COLLAPSED_SECTIONS_STORAGE_KEY, JSON.stringify(["ready"]));
    const get = vi.spyOn(proto, "getItem").mockImplementation(() => {
      throw new Error("denied");
    });
    const set = vi.spyOn(proto, "setItem").mockImplementation(() => {
      throw new Error("denied");
    });
    try {
      // 鍵裡明明有 ready：讀回空集合只可能來自 catch。
      expect(readTrayCollapsedSections().size).toBe(0);
      expect(() => writeTrayCollapsedSections(new Set(["ready"]))).not.toThrow();
      expect(get).toHaveBeenCalled();
      expect(set).toHaveBeenCalled();
    } finally {
      get.mockRestore();
      set.mockRestore();
    }
  });
});

// spec「討論列表」面板的已轉出卡：「討論」卡正下方的獨立卡，收合外觀與互動同其他內容卡
// （同一個 SectionCard）；預設收合、展開狀態沿用既有本機記憶，有料才現。
describe("面板的已轉出卡", () => {
  const promoted = (n: number) =>
    Array.from({ length: n }, (_, i) => ({ slug: `prom-${i}`, topic: `已轉出 ${i}`, promoted: true }));
  const openN = (n: number) =>
    Array.from({ length: n }, (_, i) => ({ slug: `open-${i}`, topic: `討論中 ${i}`, promoted: false }));
  const toggle = () => sectionToggle("panel-section-promoted");

  it("預設收合：「討論」卡只列討論中，「已轉出」卡只剩標頭（「已轉出」、計數 3、箭頭朝右）", () => {
    renderPanel({ snapshot: snapshot({ discussions: [...openN(2), ...promoted(3)] }) });
    expect(within(card("discussions")).getByTestId("section-count").textContent).toBe("2");
    expect(toggle().getAttribute("role")).toBe("button");
    expect(toggle().getAttribute("aria-expanded")).toBe("false");
    expect(toggle().getAttribute("tabindex")).toBe("-1");
    expect(within(toggle()).getByText("已轉出")).toBeTruthy();
    expect(within(card("promoted")).getByTestId("section-count").textContent).toBe("3");
    expect(chevronTokens(toggle())).not.toContain("rotate-90");
    expect(chevronTokens(toggle())).not.toContain("-rotate-90");
    // 卡內只剩標頭一個子節點，已轉出列不在 DOM。
    expect(card("promoted").children).toHaveLength(1);
    expect(screen.queryByTestId("panel-discussion-prom-0")).toBeNull();
  });

  it("點標頭在標頭下方展開（箭頭朝下）並寫入記憶，再點收合", () => {
    renderPanel({ snapshot: snapshot({ discussions: [...openN(2), ...promoted(3)] }) });
    fireEvent.click(toggle());
    expect(toggle().getAttribute("aria-expanded")).toBe("true");
    // 與其他內容卡同一規則：展開朝下（rotate-90），不再沿用舊收合列的朝上（-rotate-90）。
    expect(chevronTokens(toggle())).toContain("rotate-90");
    expect(chevronTokens(toggle())).not.toContain("-rotate-90");
    const rows = within(card("promoted")).getAllByTestId(/^panel-discussion-/);
    expect(rows).toHaveLength(3);
    expect(follows(toggle(), rows[0])).toBe(true);
    expect(localStorage.getItem(TRAY_PROMOTED_EXPANDED_STORAGE_KEY)).toBe("true");
    fireEvent.click(toggle());
    expect(toggle().getAttribute("aria-expanded")).toBe("false");
    expect(within(card("promoted")).queryByTestId(/^panel-discussion-/)).toBeNull();
    expect(chevronTokens(toggle())).not.toContain("rotate-90");
    expect(localStorage.getItem(TRAY_PROMOTED_EXPANDED_STORAGE_KEY)).toBe("false");
  });

  it("標頭可用 Enter／Space 開合（與 role=button 一致）", () => {
    renderPanel({ snapshot: snapshot({ discussions: promoted(2) }) });
    fireEvent.keyDown(toggle(), { key: "Enter" });
    expect(toggle().getAttribute("aria-expanded")).toBe("true");
    fireEvent.keyDown(toggle(), { key: " " });
    expect(toggle().getAttribute("aria-expanded")).toBe("false");
    fireEvent.keyDown(toggle(), { key: "a" });
    expect(toggle().getAttribute("aria-expanded")).toBe("false");
    // 按住不放的自動重複不再切換。
    fireEvent.keyDown(toggle(), { key: " ", repeat: true });
    expect(toggle().getAttribute("aria-expanded")).toBe("false");
  });

  it("重新掛載（面板重開、app 重啟）讀回展開", () => {
    renderPanel({ snapshot: snapshot({ discussions: [...openN(2), ...promoted(3)] }) });
    fireEvent.click(toggle());
    cleanup();
    renderPanel({ snapshot: snapshot({ discussions: [...openN(2), ...promoted(3)] }) });
    expect(toggle().getAttribute("aria-expanded")).toBe("true");
    expect(within(card("promoted")).getAllByTestId(/^panel-discussion-/)).toHaveLength(3);
  });

  it("記憶壞值視為收合", () => {
    localStorage.setItem(TRAY_PROMOTED_EXPANDED_STORAGE_KEY, "yes");
    renderPanel({ snapshot: snapshot({ discussions: promoted(2) }) });
    expect(toggle().getAttribute("aria-expanded")).toBe("false");
  });

  it("討論中 0、已轉出 2：「討論」卡為計數 0 的空狀態卡，其下為計數 2 的「已轉出」卡", () => {
    renderPanel({ snapshot: snapshot({ discussions: promoted(2) }) });
    expect(within(card("discussions")).getByTestId("section-count").textContent).toBe("0");
    expect(card("discussions").className.split(/\s+/)).toContain("min-h-12");
    expect(sectionHeader("panel-section-discussions").className.split(/\s+/)).not.toContain("border-b");
    expect(within(card("promoted")).getByTestId("section-count").textContent).toBe("2");
    expect(follows(card("discussions"), card("promoted"))).toBe(true);
  });

  it("已轉出 7 則展開：卡內直列前 5、尾端「還有 2 個…」", () => {
    renderPanel({ snapshot: snapshot({ discussions: promoted(7) }) });
    fireEvent.click(toggle());
    expect(within(card("promoted")).getAllByTestId(/^panel-discussion-/)).toHaveLength(5);
    fireEvent.click(within(card("promoted")).getByText("還有 2 個…"));
    expect(within(card("promoted")).getAllByTestId(/^panel-discussion-/)).toHaveLength(7);
  });

  it("spec「已轉出卡與討論卡以間距分開」：討論中 11、已轉出 3 → 討論卡最後一列為「還有 6 個…」、卡內無已轉出列", () => {
    renderPanel({ snapshot: snapshot({ discussions: [...openN(11), ...promoted(3)] }) });
    const discussions = card("discussions");
    expect(within(discussions).getAllByTestId(/^panel-discussion-open-/)).toHaveLength(5);
    expect(discussions.lastElementChild?.textContent).toBe("還有 6 個…");
    expect(within(discussions).queryByTestId(/^panel-discussion-prom-/)).toBeNull();
    fireEvent.click(toggle());
    expect(within(discussions).queryByTestId(/^panel-discussion-prom-/)).toBeNull();
    expect(within(card("promoted")).getAllByTestId(/^panel-discussion-prom-/)).toHaveLength(3);
  });

  it("零已轉出、載入中、載入失敗都不渲染已轉出卡；後兩者由「討論」卡呈現佔位列或失敗提示", () => {
    renderPanel({ snapshot: snapshot({ discussions: openN(2) }) });
    expect(screen.queryByTestId("panel-section-promoted")).toBeNull();
    cleanup();
    renderPanel({ snapshot: snapshot({ workspaceLoading: true, discussions: promoted(2) }) });
    expect(screen.queryByTestId("panel-section-promoted")).toBeNull();
    expect(card("discussions").querySelector('[aria-busy="true"]')).toBeTruthy();
    cleanup();
    renderPanel({ snapshot: snapshot({ workspaceLoadFailed: true, discussions: promoted(2) }) });
    expect(screen.queryByTestId("panel-section-promoted")).toBeNull();
    expect(screen.queryByTestId("panel-promoted-toggle")).toBeNull();
    expect(within(card("discussions")).getByTestId("panel-section-load-failed")).toBeTruthy();
  });

  it("同一面板內已轉出卡消失再出現（載入中、零筆）時保留展開狀態", () => {
    const h = renderPanel({ snapshot: snapshot({ discussions: [...openN(2), ...promoted(3)] }) });
    fireEvent.click(toggle());
    expect(toggle().getAttribute("aria-expanded")).toBe("true");
    h.pushSnapshot(snapshot({ workspaceLoading: true, discussions: [] }));
    expect(screen.queryByTestId("panel-section-promoted")).toBeNull();
    h.pushSnapshot(snapshot({ discussions: openN(2) }));
    expect(screen.queryByTestId("panel-section-promoted")).toBeNull();
    h.pushSnapshot(snapshot({ discussions: [...openN(2), ...promoted(3)] }));
    expect(toggle().getAttribute("aria-expanded")).toBe("true");
    expect(within(card("promoted")).getAllByTestId(/^panel-discussion-prom-/)).toHaveLength(3);
  });
});

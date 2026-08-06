import { describe, it, expect, vi, beforeEach } from "vitest";
import type { ChangeItem } from "@speclink/ui";
import { TrayIcon } from "@tauri-apps/api/tray";
import { Menu } from "@tauri-apps/api/menu";
import { Image } from "@tauri-apps/api/image";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { emit as tauriEmit, listen as tauriListen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";

import {
  buildTrayModel,
  buildTraySnapshot,
  progressBar,
  initTray,
  type TraySnapshot,
  type TrayMenuItem,
  type TrayStoreApi,
} from "../tray";
import type { WorkspaceLocator } from "../session";
import type {
  RemoteWorkspaceRecoveryState,
  WorkspaceSession,
} from "../session";
import type { ConnectionView } from "../adapter/connections";
import { trayIconBytes } from "../trayIcon";

// tray.ts 於頂層 import @tauri-apps/api 的 tray/menu/window/image（＋面板模式的 webviewWindow/dpi）；
// 於 jsdom 下以樁替，使模組載入安全並讓接線測試可攔截呼叫。純函式測試不觸及這些。
vi.mock("@tauri-apps/api/tray", () => ({
  TrayIcon: { new: vi.fn(), removeById: vi.fn() },
}));
vi.mock("@tauri-apps/api/menu", () => ({ Menu: { new: vi.fn() } }));
vi.mock("@tauri-apps/api/image", () => ({ Image: { fromBytes: vi.fn() } }));
vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({
  emit: vi.fn().mockResolvedValue(undefined),
  listen: vi.fn().mockResolvedValue(() => {}),
}));
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn().mockResolvedValue(undefined) }));
vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({ writeText: vi.fn() }));
vi.mock("@tauri-apps/plugin-positioner", () => ({
  handleIconState: vi.fn().mockResolvedValue(undefined),
}));

const fakeT = (key: string): string =>
  (({
    "tray.open": "開啟 Speclink",
    "tray.settings": "設定",
    "tray.quit": "結束",
    "app.navProjectSettings": "專案設定",
    "tray.discussions": "討論 {n}",
    "tray.discussionsHeader": "討論",
    "tray.promotedHeader": "已轉出",
    "tray.more": "還有 {n} 個…",
    "tray.openChange": "開啟此變更",
    "tray.copyName": "複製名稱",
    "tray.openDiscussion": "開啟此討論",
    "tray.copySlug": "複製 slug",
    "tray.noChanges": "尚無進行中變更",
    "tray.recovery.active": "作用中",
    "tray.recovery.restoring": "正在連線",
    "tray.recovery.offline": "離線（最後資料）",
    "tray.recovery.needsReauth": "需要重新登入",
    "tray.recovery.unreachable": "無法連線",
    "tray.recovery.retry": "重新連線",
    "tray.recovery.open": "查看問題",
    "tray.recovery.settings": "伺服器設定",
    "tray.recovery.reauthenticate": "重新登入",
    "stage.proposed": "提案中",
    "stage.in-progress": "進行中",
    "stage.ready": "已就緒",
  } as Record<string, string>)[key] ?? key);

function change(over: Partial<ChangeItem> & { name: string }): ChangeItem {
  return { status: "", totalTasks: 0, completedTasks: 0, ...over };
}

/** 典型快照：兩分頁（p1 作用中）、一提案／一進行中／一已就緒、兩討論。 */
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
    discussions: [
      { slug: "d1", topic: "討論一", promoted: false },
      { slug: "d2", topic: "討論二", promoted: false },
    ],
    ...over,
  };
}

const kinds = (items: TrayMenuItem[]) => items.map((i) => i.kind);
const byKind = <K extends TrayMenuItem["kind"]>(items: TrayMenuItem[], kind: K) =>
  items.filter((i): i is Extract<TrayMenuItem, { kind: K }> => i.kind === kind);

describe("trayIconBytes", () => {
  it("解碼內嵌 base64 為有效 PNG（magic bytes）", () => {
    const bytes = trayIconBytes();
    expect(Array.from(bytes.slice(0, 8))).toEqual([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
    expect(bytes.length).toBeGreaterThan(100);
  });
});

describe("progressBar", () => {
  it("依比例填滿 unicode 方塊", () => {
    expect(progressBar(0, 5, 8)).toBe("░░░░░░░░");
    expect(progressBar(4, 4, 8)).toBe("▓▓▓▓▓▓▓▓");
    expect(progressBar(3, 12, 8)).toBe("▓▓░░░░░░"); // round(3/12*8)=2
  });
  it("total<=0 回空字串（無任務不畫）", () => {
    expect(progressBar(0, 0)).toBe("");
    expect(progressBar(5, 0)).toBe("");
  });
});

describe("buildTrayModel", () => {
  it("專案項依分頁順序、作用中打勾", () => {
    const projects = byKind(buildTrayModel(snapshot(), fakeT).items, "project");
    expect(projects.map((p) => p.key)).toEqual(["local:/proj/one", "local:/proj/two"]);
    expect(projects.map((p) => p.checked)).toEqual([true, false]);
  });

  it("每個非空階段一個分區標題（header），依 proposed/in-progress/ready 序", () => {
    const headers = byKind(buildTrayModel(snapshot(), fakeT).items, "header");
    // 三階段皆非空＋討論 header
    expect(headers.map((h) => h.label)).toEqual(["提案中", "進行中", "已就緒", "討論"]);
  });

  it("變更列帶進度條標籤，子選單動作依序為「開啟此變更」「複製名稱」", () => {
    const changes = byKind(buildTrayModel(snapshot(), fakeT).items, "change");
    expect(changes.map((c) => c.name)).toEqual(["prop", "inprog", "rdy"]);
    // 進行中 3/12 → 進度條＋n/m
    const inprog = changes.find((c) => c.name === "inprog")!;
    expect(inprog.label).toBe("inprog  ▓▓░░░░░░ 3/12");
    // spec tray-status-menu Scenario「原生選單不受影響」：站章不進純文字 label
    // ——單一字元承載四態、無 tooltip 無色彩，不可辨識（design D7 已否決）。
    const stamped = buildTrayModel(
      snapshot({
        changes: [
          change({
            name: "inprog",
            totalTasks: 12,
            completedTasks: 3,
            reviewStatus: "reviewed",
            verifyStatus: "verifiedStale",
          }),
        ],
      }),
      fakeT,
    );
    const stampedRow = byKind(stamped.items, "change").find((c) => c.name === "inprog")!;
    expect(stampedRow.label).toBe("inprog  ▓▓░░░░░░ 3/12");
    expect(inprog.actions).toEqual([
      { kind: "open-change", label: "開啟此變更" },
      { kind: "copy-name", label: "複製名稱" },
    ]);
    // ready 全滿
    expect(changes.find((c) => c.name === "rdy")!.label).toBe("rdy  ▓▓▓▓▓▓▓▓ 4/4");
  });

  it("複製來源為純名稱：標籤含進度條字元、name 不含（spec 例：0/10）", () => {
    const items = buildTrayModel(
      snapshot({
        changes: [change({ name: "phase2-e2e-chain", totalTasks: 10, completedTasks: 0 })],
      }),
      fakeT,
    ).items;
    const c = byKind(items, "change")[0];
    expect(c.label).toBe("phase2-e2e-chain  ░░░░░░░░ 0/10");
    expect(c.name).toBe("phase2-e2e-chain");
  });

  it("無任務的變更僅顯示名稱（不畫進度條）", () => {
    const items = buildTrayModel(
      snapshot({ changes: [change({ name: "empty-tasks", totalTasks: 0, completedTasks: 0 })] }),
      fakeT,
    ).items;
    expect(byKind(items, "change")[0].label).toBe("empty-tasks");
  });

  it("討論項以 slug 為標籤、攜帶 topic，子選單動作依序為「開啟此討論」「複製 slug」", () => {
    const items = buildTrayModel(snapshot(), fakeT).items;
    const disc = byKind(items, "discussion");
    expect(disc.map((d) => [d.slug, d.label, d.topic])).toEqual([
      ["d1", "d1", "討論一"],
      ["d2", "d2", "討論二"],
    ]);
    expect(disc[0].actions).toEqual([
      { kind: "open-discussion", label: "開啟此討論" },
      { kind: "copy-slug", label: "複製 slug" },
    ]);
  });

  it("無變更時顯示空狀態", () => {
    const items = buildTrayModel(snapshot({ changes: [] }), fakeT).items;
    expect(byKind(items, "change")).toHaveLength(0);
    expect(byKind(items, "empty").some((e) => e.label === "尚無進行中變更")).toBe(true);
  });

  it("無討論時顯示「討論 0」", () => {
    const items = buildTrayModel(snapshot({ discussions: [] }), fakeT).items;
    expect(byKind(items, "discussion")).toHaveLength(0);
    expect(byKind(items, "empty").some((e) => e.label === "討論 0")).toBe(true);
  });

  it("討論分流：已轉出討論列於「已轉出」分區、討論中列於「討論」分區", () => {
    const items = buildTrayModel(
      snapshot({
        discussions: [
          { slug: "open-d", topic: "討論中的", promoted: false },
          { slug: "prom-d", topic: "已轉出的", promoted: true },
        ],
      }),
      fakeT,
    ).items;
    const headers = byKind(items, "header").map((h) => h.label);
    expect(headers).toContain("討論");
    expect(headers).toContain("已轉出");
    // 順序：討論 header → open-d → 已轉出 header → prom-d
    const labels = items.map((i) => ("label" in i ? i.label : i.kind));
    const iDisc = labels.indexOf("討論");
    const iOpen = labels.indexOf("open-d");
    const iProm = labels.indexOf("已轉出");
    const iPromD = labels.indexOf("prom-d");
    expect(iDisc).toBeLessThan(iOpen);
    expect(iOpen).toBeLessThan(iProm);
    expect(iProm).toBeLessThan(iPromD);
    // 兩分區子選單結構相同（同為 discussion 項、帶完整動作）
    const disc = byKind(items, "discussion");
    expect(disc.map((d) => d.actions.length)).toEqual([2, 2]);
  });

  it("無已轉出討論時不出現「已轉出」分區；無討論中仍顯示「討論 0」且已轉出照列", () => {
    const none = buildTrayModel(snapshot(), fakeT).items;
    expect(byKind(none, "header").map((h) => h.label)).not.toContain("已轉出");

    const onlyPromoted = buildTrayModel(
      snapshot({ discussions: [{ slug: "prom-d", topic: "已轉出的", promoted: true }] }),
      fakeT,
    ).items;
    expect(byKind(onlyPromoted, "empty").some((e) => e.label === "討論 0")).toBe(true);
    expect(byKind(onlyPromoted, "header").map((h) => h.label)).toContain("已轉出");
    expect(byKind(onlyPromoted, "discussion").map((d) => d.slug)).toEqual(["prom-d"]);
  });

  it("分區逾 5 筆：直列前 5、尾端「還有 N 個…」內嵌其餘（spec Example 門檻邊界 5/6/20）", () => {
    const mk = (n: number) =>
      Array.from({ length: n }, (_, i) => change({ name: `c${i}`, totalTasks: 2, completedTasks: 1 }));
    // 5 → 無溢出節點
    let items = buildTrayModel(snapshot({ changes: mk(5), discussions: [] }), fakeT).items;
    expect(byKind(items, "change")).toHaveLength(5);
    expect(byKind(items, "overflow")).toHaveLength(0);
    // 6 → 5＋還有 1 個
    items = buildTrayModel(snapshot({ changes: mk(6), discussions: [] }), fakeT).items;
    expect(byKind(items, "change")).toHaveLength(5);
    let overflow = byKind(items, "overflow")[0];
    expect(overflow.label).toBe("還有 1 個…");
    expect(overflow.items).toHaveLength(1);
    // 20 → 5＋還有 15 個，內嵌項目保有完整動作（同構 change 模型）
    items = buildTrayModel(snapshot({ changes: mk(20), discussions: [] }), fakeT).items;
    expect(byKind(items, "change")).toHaveLength(5);
    overflow = byKind(items, "overflow")[0];
    expect(overflow.label).toBe("還有 15 個…");
    expect(overflow.items).toHaveLength(15);
    const nested = overflow.items[0] as Extract<TrayMenuItem, { kind: "change" }>;
    expect(nested.kind).toBe("change");
    expect(nested.actions).toHaveLength(2);
  });

  it("討論分區同樣適用溢出門檻", () => {
    const ds = Array.from({ length: 7 }, (_, i) => ({ slug: `d${i}`, topic: `t${i}`, promoted: false }));
    const items = buildTrayModel(snapshot({ discussions: ds }), fakeT).items;
    expect(byKind(items, "discussion")).toHaveLength(5);
    expect(byKind(items, "overflow")[0].label).toBe("還有 2 個…");
  });

  it("模型不含數字徽章欄位（系統匣不顯示數字）", () => {
    expect("badge" in buildTrayModel(snapshot(), fakeT)).toBe(false);
  });

  it("區段順序：專案 → 分隔 → 各階段(header+change) → 分隔 → 討論(header+項) → 分隔 → 動作", () => {
    expect(kinds(buildTrayModel(snapshot(), fakeT).items)).toEqual([
      "project",
      "project",
      "separator",
      "header",
      "change",
      "header",
      "change",
      "header",
      "change",
      "separator",
      "header",
      "discussion",
      "discussion",
      "separator",
      "open",
      "project-settings",
      "settings",
      "quit",
    ]);
  });

  it("動作區含專案設定項：位於開啟 Speclink 之後、label 沿用 app.navProjectSettings", () => {
    const items = buildTrayModel(snapshot(), fakeT).items;
    const ps = byKind(items, "project-settings");
    expect(ps).toHaveLength(1);
    expect(ps[0].label).toBe("專案設定");
  });

  it("共用狀態投影讓 restoring／error／needs-reauth 成為原生復原項，且 active error 不顯示舊資料", () => {
    const errorKey = "remote:c1/demo/error";
    const model = buildTrayModel(
      snapshot({
        tabs: [
          { key: "local:/proj/one", name: "one", source: "local", status: "ready" },
          {
            key: "remote:c1/demo/ready",
            name: "Demo/ready",
            source: "remote",
            status: "ready",
            connectionId: "c1",
            serverLabel: "Team Server",
          },
          {
            key: "remote:c1/demo/restoring",
            name: "Demo/restoring",
            source: "remote",
            status: "restoring",
            connectionId: "c1",
            serverLabel: "Team Server",
          },
          {
            key: errorKey,
            name: "Demo/error",
            source: "remote",
            status: "error",
            failureKind: "unreachable",
            connectionId: "c1",
            serverLabel: "Team Server",
            serverOrigin: "https://spec.example.test",
          },
          {
            key: "remote:c1/demo/auth",
            name: "Demo/auth",
            source: "remote",
            status: "needs-reauth",
            connectionId: "c1",
            serverLabel: "Team Server",
          },
        ],
        activeKey: errorKey,
        changes: [change({ name: "previous-workspace-change", totalTasks: 2, completedTasks: 1 })],
        discussions: [{ slug: "previous-workspace-discussion", topic: "舊資料", promoted: false }],
      } as unknown as Partial<TraySnapshot>),
      fakeT,
    );

    const recoveries = byKind(model.items, "recovery");
    expect(recoveries.map((item) => item.status)).toEqual([
      "restoring",
      "error",
      "needs-reauth",
    ]);
    expect(recoveries.find((item) => item.status === "restoring")?.actions).toEqual([]);
    expect(recoveries.find((item) => item.status === "error")?.actions.map((a) => a.kind)).toEqual([
      "retry",
      "open-recovery",
      "open-settings",
    ]);
    expect(recoveries.find((item) => item.status === "needs-reauth")?.actions[0]?.kind).toBe(
      "reauthenticate",
    );
    expect(byKind(model.items, "change")).toHaveLength(0);
    expect(byKind(model.items, "discussion")).toHaveLength(0);
  });
});

// ---- 接線層（Tauri tray/menu/window API）----

/** 可觀察的假 store（tabs＋changes＋discussions＋動作），可推變更通知訂閱者。 */
function makeStore(
  activateTab = vi.fn(),
  openProjectAt = vi.fn(),
  openDetail = vi.fn(),
  openDiscussion = vi.fn(),
  openProjectViaDialog = vi.fn(),
  setBoardView = vi.fn(),
) {
  let state = {
    tabs: [
      { locator: { kind: "local", root: "/proj/one" } as const, name: "one" },
      { locator: { kind: "local", root: "/proj/two" } as const, name: "two" },
    ] as Array<{ locator: WorkspaceLocator; name: string }>,
    activeKey: "local:/proj/one",
    changes: [
      change({ name: "alpha", totalTasks: 12, completedTasks: 3 }),
      change({ name: "gamma", totalTasks: 5, completedTasks: 0 }),
    ],
    discussions: {
      active: [
        { slug: "d1", topic: "t1", promotedTo: [] as string[] },
        { slug: "d2", topic: "t2", promotedTo: [] as string[] },
      ],
    },
    trayStyle: "native-menu" as "native-menu" | "panel",
    activateTab,
    openProjectAt,
    openDetail,
    openDiscussion,
    openProjectViaDialog,
    setBoardView,
    sessions: {} as Record<string, WorkspaceSession>,
    remoteRecovery: {} as Record<string, RemoteWorkspaceRecoveryState>,
    connections: [] as ConnectionView[],
    retryRemoteWorkspace: vi.fn(),
    showRemoteWorkspaceRecovery: vi.fn(),
    openConnectionReauth: vi.fn(),
  };
  const listeners = new Set<() => void>();
  return {
    store: {
      getState: () => state,
      subscribe: (fn: () => void) => {
        listeners.add(fn);
        return () => listeners.delete(fn);
      },
    } as unknown as TrayStoreApi,
    emit(next: Partial<typeof state>) {
      state = { ...state, ...next };
      listeners.forEach((l) => l());
    },
    activateTab,
    openProjectAt,
    openDetail,
    openDiscussion,
    openProjectViaDialog,
    setBoardView,
    retryRemoteWorkspace: state.retryRemoteWorkspace,
    showRemoteWorkspaceRecovery: state.showRemoteWorkspaceRecovery,
    openConnectionReauth: state.openConnectionReauth,
  };
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type AnyItem = any;

describe("initTray 接線（選單）", () => {
  const trayObj = {
    setMenu: vi.fn(),
    setTitle: vi.fn(),
    setShowMenuOnLeftClick: vi.fn(),
    close: vi.fn(),
  };
  const win = { show: vi.fn(), setFocus: vi.fn(), unminimize: vi.fn() };
  let lastItems: AnyItem[] = [];

  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(writeText).mockResolvedValue(undefined as never);
    vi.mocked(Image.fromBytes).mockResolvedValue({} as never);
    vi.mocked(Menu.new).mockImplementation(async (opts?: { items?: AnyItem[] }) => {
      lastItems = opts?.items ?? [];
      return { id: "menu" } as never;
    });
    vi.mocked(TrayIcon.new).mockResolvedValue(trayObj as never);
    vi.mocked(getCurrentWindow).mockReturnValue(win as never);
  });

  it("建立 tray 圖示並掛上由當前快照組出的選單（含兩專案 check 項）", async () => {
    const { store } = makeStore();
    await initTray(store, { isMacOS: true });
    const opts = vi.mocked(TrayIcon.new).mock.calls[0][0] as AnyItem;
    expect(opts.menu).toBeDefined();
    expect(opts.iconAsTemplate).toBe(true);
    const projects = lastItems.filter((i) => "checked" in i);
    expect(projects).toHaveLength(2);
    expect(projects.map((p: AnyItem) => p.checked)).toEqual([true, false]);
  });

  it("變更列為子選單（有 items）、標籤帶進度條；store 變動後重建選單", async () => {
    const bag = makeStore();
    await initTray(bag.store, { isMacOS: true, debounceMs: 100 });
    vi.mocked(Menu.new).mockClear();
    trayObj.setMenu.mockClear();
    trayObj.setTitle.mockClear();

    vi.useFakeTimers();
    bag.emit({
      changes: [
        change({ name: "alpha", totalTasks: 12, completedTasks: 6 }),
        change({ name: "beta", totalTasks: 8, completedTasks: 2 }),
      ],
    });
    await vi.advanceTimersByTimeAsync(100);
    vi.useRealTimers();

    expect(trayObj.setMenu).toHaveBeenCalledTimes(1);
    const submenus = lastItems.filter((i) => Array.isArray(i.items) && /\d+\/\d+$/.test(i.text ?? ""));
    expect(submenus.map((s: AnyItem) => s.text)).toEqual(["alpha  ▓▓▓▓░░░░ 6/12", "beta  ▓▓░░░░░░ 2/8"]);
  });

  it.each([
    ["local", { kind: "local", root: "/proj/two" } as const, "two", "local:/proj/two"],
    [
      "remote",
      { kind: "remote", connectionId: "conn-1", projectId: "demo", repoId: "backend" } as const,
      "Demo/backend",
      "remote:conn-1/demo/backend",
    ],
  ])("點非作用中 %s 專案以 locator key 呼叫 activateTab", async (_kind, locator, name, key) => {
    const bag = makeStore();
    bag.emit({
      tabs: [
        { locator: { kind: "local", root: "/proj/one" }, name: "one" },
        { locator, name },
      ],
      sessions:
        _kind === "remote"
          ? {
              [key]: {
                locator,
                connectionState: {
                  connectionId: "conn-1",
                  state: "online",
                  message: null,
                },
              } as WorkspaceSession,
            }
          : {},
    });
    await initTray(bag.store, { isMacOS: true });
    const projects = lastItems.filter((i) => "checked" in i);
    projects[1].action();
    expect(bag.activateTab).toHaveBeenCalledWith(key);
    expect(bag.openProjectAt).not.toHaveBeenCalled();
  });

  it("變更子選單「開啟此變更」開主視窗並開啟該變更詳情", async () => {
    const bag = makeStore();
    await initTray(bag.store, { isMacOS: true });
    // alpha 為進行中，是子選單（有 items）
    const alpha = lastItems.find((i) => Array.isArray(i.items) && (i.text ?? "").startsWith("alpha"));
    expect(alpha).toBeDefined();
    await alpha.items[0].action();
    expect(win.show).toHaveBeenCalled();
    expect(bag.openDetail).toHaveBeenCalledWith("alpha");
  });

  it("原生 needs-reauth submenu 的顯式詳情與登入才聚焦主視窗", async () => {
    const bag = makeStore();
    const key = "remote:c1/demo/backend";
    bag.emit({
      tabs: [
        {
          locator: { kind: "remote", connectionId: "c1", projectId: "demo", repoId: "backend" },
          name: "Demo/backend",
        },
      ],
      activeKey: key,
      remoteRecovery: {
        [key]: {
          status: "error",
          failure: {
            kind: "needs-reauth",
            message: "sensitive technical detail must not enter tray snapshot",
            reason: "needs_reauth",
            status: 401,
          },
        },
      },
      connections: [
        {
          id: "c1",
          origin: "https://spec.example.test",
          name: "Team Server",
          loggedIn: false,
        },
      ],
    });
    await initTray(bag.store, { isMacOS: true });

    const recovery = lastItems.find((item) =>
      Array.isArray(item.items) && item.text.includes("Demo/backend"),
    );
    expect(recovery.items.map((item: AnyItem) => item.text)).toEqual([
      "作用中",
      "需要重新登入",
      "重新登入",
      "查看問題",
      "伺服器設定",
    ]);

    recovery.items[2].action();
    expect(bag.openConnectionReauth).toHaveBeenCalledWith("c1");
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(win.show).toHaveBeenCalledTimes(1);

    win.show.mockClear();
    recovery.items[3].action();
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(bag.showRemoteWorkspaceRecovery).toHaveBeenCalledWith(key);
    expect(win.show).toHaveBeenCalledTimes(1);
  });

  it("原生 error submenu 直接 retry 不顯示或聚焦主視窗，且 snapshot 不攜帶 technical detail", async () => {
    const bag = makeStore();
    const key = "remote:c1/demo/backend";
    bag.emit({
      tabs: [
        {
          locator: { kind: "remote", connectionId: "c1", projectId: "demo", repoId: "backend" },
          name: "Demo/backend",
        },
      ],
      activeKey: key,
      remoteRecovery: {
        [key]: {
          status: "error",
          failure: {
            kind: "unreachable",
            message: "authorization: Bearer must-never-enter-tray",
            reason: "offline",
            status: null,
          },
        },
      },
      connections: [
        {
          id: "c1",
          origin: "https://spec.example.test",
          name: "Team Server",
          loggedIn: true,
        },
      ],
    });

    const projected = buildTraySnapshot(bag.store.getState());
    expect(JSON.stringify(projected)).not.toContain("must-never-enter-tray");
    expect(projected.tabs[0]).toEqual(
      expect.objectContaining({
        key,
        status: "error",
        failureKind: "unreachable",
        serverLabel: "Team Server",
      }),
    );

    await initTray(bag.store, { isMacOS: true });
    const recovery = lastItems.find(
      (item) => Array.isArray(item.items) && item.text.includes("Demo/backend"),
    );
    recovery.items[2].action();
    expect(bag.retryRemoteWorkspace).toHaveBeenCalledWith(key);
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(win.show).not.toHaveBeenCalled();
    expect(win.setFocus).not.toHaveBeenCalled();
  });

  it("變更子選單「複製名稱」寫入剪貼簿且不開主視窗", async () => {
    const bag = makeStore();
    await initTray(bag.store, { isMacOS: true });
    const alpha = lastItems.find((i) => Array.isArray(i.items) && (i.text ?? "").startsWith("alpha"));
    expect(alpha.items.map((a: AnyItem) => a.text)).toEqual(["開啟此變更", "複製名稱"]);
    await alpha.items[1].action();
    expect(writeText).toHaveBeenCalledWith("alpha");
    expect(win.show).not.toHaveBeenCalled();
  });

  it("討論為子選單：父標籤 slug、首行 disabled topic、開啟與複製 slug 動作", async () => {
    const bag = makeStore();
    await initTray(bag.store, { isMacOS: true });
    const disc = lastItems.find((i) => Array.isArray(i.items) && i.text === "d1");
    expect(disc).toBeDefined();
    // 首行為 topic 描述（disabled、不可選取）
    expect(disc.items[0].text).toBe("t1");
    expect(disc.items[0].enabled).toBe(false);
    // 開啟此討論 → 主視窗＋openDiscussion
    expect(disc.items[1].text).toBe("開啟此討論");
    await disc.items[1].action();
    expect(win.show).toHaveBeenCalled();
    expect(bag.openDiscussion).toHaveBeenCalledWith("d1");
    // 複製 slug → 剪貼簿為 slug
    expect(disc.items[2].text).toBe("複製 slug");
    await disc.items[2].action();
    expect(writeText).toHaveBeenCalledWith("d1");
  });

  it("剪貼簿寫入失敗時靜默（不拋出、不彈窗）", async () => {
    vi.mocked(writeText).mockRejectedValue(new Error("denied"));
    const bag = makeStore();
    await initTray(bag.store, { isMacOS: true });
    const alpha = lastItems.find((i) => Array.isArray(i.items) && (i.text ?? "").startsWith("alpha"));
    await expect(Promise.resolve(alpha.items[1].action())).resolves.not.toThrow();
    // 沖洗微任務：內部 rejection 須被吞掉，否則 vitest 以 unhandled rejection 失敗
    await new Promise((r) => setTimeout(r, 0));
  });

  it("點「開啟 Speclink」顯示視窗並取得焦點", async () => {
    const { store } = makeStore();
    await initTray(store, { isMacOS: true });
    const open = lastItems.find((i) => i.text === "開啟 Speclink");
    await open.action();
    expect(win.show).toHaveBeenCalled();
    expect(win.setFocus).toHaveBeenCalled();
  });

  it("點「設定」開主視窗、取得焦點並切換至設定頁", async () => {
    const bag = makeStore();
    await initTray(bag.store, { isMacOS: true });
    const settings = lastItems.find((i) => i.text === "設定");
    expect(settings).toBeDefined();
    settings.action();
    // openMainWindow 為 async（unminimize → show → setFocus）：沖洗微任務後再斷言
    await new Promise((r) => setTimeout(r, 0));
    expect(win.show).toHaveBeenCalled();
    expect(win.setFocus).toHaveBeenCalled();
    expect(bag.setBoardView).toHaveBeenCalledWith("settings");
  });

  it("點「專案設定」開主視窗、取得焦點並切換至專案設定頁", async () => {
    const bag = makeStore();
    await initTray(bag.store, { isMacOS: true });
    const ps = lastItems.find((i) => i.text === "專案設定");
    expect(ps).toBeDefined();
    ps.action();
    // openMainWindow 為 async（unminimize → show → setFocus）：沖洗微任務後再斷言
    await new Promise((r) => setTimeout(r, 0));
    expect(win.show).toHaveBeenCalled();
    expect(win.setFocus).toHaveBeenCalled();
    expect(bag.setBoardView).toHaveBeenCalledWith("project-settings");
  });

  it("「結束」映射為原生 predefined Quit", async () => {
    const { store } = makeStore();
    await initTray(store, { isMacOS: true });
    expect(lastItems.find((i) => i.item === "Quit")).toBeDefined();
  });

  it("panel 樣式：建立時不掛選單、不開左鍵選單，點擊圖示觸發 onPanelToggle", async () => {
    const bag = makeStore();
    bag.emit({ trayStyle: "panel" });
    const onPanelToggle = vi.fn();
    await initTray(bag.store, { isMacOS: true, onPanelToggle });
    const opts = vi.mocked(TrayIcon.new).mock.calls[0][0] as AnyItem;
    expect(opts.menu).toBeUndefined();
    expect(opts.showMenuOnLeftClick).toBe(false);
    opts.action({ type: "Click", button: "Left", buttonState: "Up" });
    expect(onPanelToggle).toHaveBeenCalledTimes(1);
  });

  it("panel 樣式：右鍵點擊圖示同樣觸發 onPanelToggle（與左鍵等價），Down 不觸發", async () => {
    const bag = makeStore();
    bag.emit({ trayStyle: "panel" });
    const onPanelToggle = vi.fn();
    await initTray(bag.store, { isMacOS: true, onPanelToggle });
    const opts = vi.mocked(TrayIcon.new).mock.calls[0][0] as AnyItem;
    opts.action({ type: "Click", button: "Right", buttonState: "Up" });
    expect(onPanelToggle).toHaveBeenCalledTimes(1);
    // 放開才開閉：Down 事件不觸發（與左鍵同規則）
    opts.action({ type: "Click", button: "Right", buttonState: "Down" });
    expect(onPanelToggle).toHaveBeenCalledTimes(1);
  });

  it("native 樣式：點擊事件不觸發 onPanelToggle（選單接管互動）", async () => {
    const bag = makeStore();
    const onPanelToggle = vi.fn();
    await initTray(bag.store, { isMacOS: true, onPanelToggle });
    const opts = vi.mocked(TrayIcon.new).mock.calls[0][0] as AnyItem;
    opts.action?.({ type: "Click", button: "Left", buttonState: "Up" });
    opts.action?.({ type: "Click", button: "Right", buttonState: "Up" });
    expect(onPanelToggle).not.toHaveBeenCalled();
  });

  it("樣式切換即時分流：panel 卸選單且關左鍵開選單、切回 native 雙雙復原（無需重啟）", async () => {
    const bag = makeStore();
    await initTray(bag.store, { isMacOS: true, debounceMs: 50 });
    trayObj.setMenu.mockClear();
    vi.useFakeTimers();
    bag.emit({ trayStyle: "panel" });
    await vi.advanceTimersByTimeAsync(50);
    expect(trayObj.setMenu).toHaveBeenLastCalledWith(null);
    // 關鍵：僅卸選單不夠——左鍵仍被「開選單」路徑吃掉，點擊事件到不了 action。
    expect(trayObj.setShowMenuOnLeftClick).toHaveBeenLastCalledWith(false);
    bag.emit({ trayStyle: "native-menu" });
    await vi.advanceTimersByTimeAsync(50);
    vi.useRealTimers();
    expect(trayObj.setMenu).toHaveBeenLastCalledWith({ id: "menu" });
    expect(trayObj.setShowMenuOnLeftClick).toHaveBeenLastCalledWith(true);
  });

  it("tray 以固定 id 建立，建立前先移除同 id 孤兒（webview 重建不殘留殭屍徽章）", async () => {
    const { store } = makeStore();
    await initTray(store, { isMacOS: true });
    expect(TrayIcon.removeById).toHaveBeenCalledWith("speclink-tray");
    const opts = vi.mocked(TrayIcon.new).mock.calls[0][0] as AnyItem;
    expect(opts.id).toBe("speclink-tray");
    expect(vi.mocked(TrayIcon.removeById).mock.invocationCallOrder[0]).toBeLessThan(
      vi.mocked(TrayIcon.new).mock.invocationCallOrder[0],
    );
  });

  it("不顯示數字徽章：建立不帶 title，資料變動與點擊圖示皆不呼叫 setTitle", async () => {
    const bag = makeStore();
    bag.emit({ trayStyle: "panel" });
    const onPanelToggle = vi.fn();
    await initTray(bag.store, { isMacOS: true, debounceMs: 50, onPanelToggle });
    const opts = vi.mocked(TrayIcon.new).mock.calls[0][0] as AnyItem;
    expect(opts.title).toBeUndefined();
    vi.useFakeTimers();
    bag.emit({
      changes: [change({ name: "alpha", totalTasks: 12, completedTasks: 6 })],
    });
    await vi.advanceTimersByTimeAsync(50);
    vi.useRealTimers();
    opts.action({ type: "Click", button: "Left", buttonState: "Up" });
    await Promise.resolve();
    expect(trayObj.setTitle).not.toHaveBeenCalled();
  });

  it("溢出節點轉為原生子選單（內含其餘變更、各自保有動作子選單）", async () => {
    const bag = makeStore();
    await initTray(bag.store, { isMacOS: true, debounceMs: 50 });
    vi.useFakeTimers();
    bag.emit({
      changes: Array.from({ length: 7 }, (_, i) =>
        change({ name: `c${i}`, totalTasks: 2, completedTasks: 1 }),
      ),
    });
    await vi.advanceTimersByTimeAsync(50);
    vi.useRealTimers();
    const more = lastItems.find((i) => i.text === "還有 2 個…");
    expect(more).toBeDefined();
    expect(more.items).toHaveLength(2);
    // 內嵌項目仍是完整變更子選單（開啟此變更＋複製名稱）
    expect(more.items[0].items.map((a: AnyItem) => a.text)).toEqual(["開啟此變更", "複製名稱"]);
  });

  it("面板事件接線：訂閱 ready/action、去抖推送快照、動作回流 store", async () => {
    const bag = makeStore();
    await initTray(bag.store, { isMacOS: true, debounceMs: 50 });
    // 訂閱面板的 ready 與 action 事件
    const topics = vi.mocked(tauriListen).mock.calls.map((c) => c[0]);
    expect(topics).toContain("tray-panel-ready");
    expect(topics).toContain("tray-panel-action");
    // 資料變動去抖後推送快照給面板
    vi.mocked(tauriEmit).mockClear();
    vi.useFakeTimers();
    bag.emit({ activeKey: "local:/proj/two" });
    await vi.advanceTimersByTimeAsync(50);
    vi.useRealTimers();
    expect(vi.mocked(tauriEmit)).toHaveBeenCalledWith(
      "tray-snapshot",
      expect.objectContaining({ activeKey: "local:/proj/two" }),
    );
    // 動作回流：open-change 開主視窗並開啟該變更詳情
    const actionCall = vi.mocked(tauriListen).mock.calls.find((c) => c[0] === "tray-panel-action")!;
    (actionCall[1] as (e: AnyItem) => void)({ payload: { kind: "open-change", id: "alpha" } });
    // openMainWindow 為 async（unminimize → show）：沖洗微任務後再斷言
    await new Promise((r) => setTimeout(r, 0));
    expect(win.show).toHaveBeenCalled();
    expect(bag.openDetail).toHaveBeenCalledWith("alpha");
  });

  it("面板動作 open-project 以 remote locator key 呼叫 activateTab", async () => {
    const bag = makeStore();
    await initTray(bag.store, { isMacOS: true, debounceMs: 50 });
    const actionCall = vi.mocked(tauriListen).mock.calls.find((c) => c[0] === "tray-panel-action")!;
    const remoteKey = "remote:conn-1/demo/backend";
    (actionCall[1] as (e: AnyItem) => void)({ payload: { kind: "open-project", id: remoteKey } });
    expect(bag.activateTab).toHaveBeenCalledWith(remoteKey);
    expect(bag.openProjectAt).not.toHaveBeenCalled();
  });

  it("面板 recovery action 的 retry 留在 Tray，詳情／設定／登入才顯示並聚焦主視窗", async () => {
    const bag = makeStore();
    await initTray(bag.store, { isMacOS: true, debounceMs: 50 });
    const actionCall = vi.mocked(tauriListen).mock.calls.find((c) => c[0] === "tray-panel-action")!;
    const dispatch = actionCall[1] as (event: AnyItem) => void;
    const key = "remote:c1/demo/backend";

    dispatch({ payload: { kind: "retry-workspace", id: key } });
    expect(bag.retryRemoteWorkspace).toHaveBeenCalledWith(key);
    expect(win.show).not.toHaveBeenCalled();

    dispatch({ payload: { kind: "open-recovery", id: key } });
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(bag.showRemoteWorkspaceRecovery).toHaveBeenCalledWith(key);
    expect(win.show).toHaveBeenCalledTimes(1);
    expect(win.setFocus).toHaveBeenCalledTimes(1);

    win.show.mockClear();
    win.setFocus.mockClear();
    dispatch({ payload: { kind: "open-server-settings", id: "c1" } });
    dispatch({ payload: { kind: "reauthenticate", id: "c1" } });
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(bag.openConnectionReauth).toHaveBeenCalledTimes(2);
    expect(win.show).toHaveBeenCalledTimes(2);
    expect(win.setFocus).toHaveBeenCalledTimes(2);
  });

  it("面板動作 open-settings 喚起主視窗並切換至設定頁", async () => {
    const bag = makeStore();
    await initTray(bag.store, { isMacOS: true, debounceMs: 50 });
    const actionCall = vi.mocked(tauriListen).mock.calls.find((c) => c[0] === "tray-panel-action")!;
    (actionCall[1] as (e: AnyItem) => void)({ payload: { kind: "open-settings" } });
    // openMainWindow 為 async：沖洗微任務後再斷言
    await new Promise((r) => setTimeout(r, 0));
    expect(win.show).toHaveBeenCalled();
    expect(bag.setBoardView).toHaveBeenCalledWith("settings");
  });

  it("面板動作 open-project-settings 喚起主視窗並切換至專案設定頁", async () => {
    const bag = makeStore();
    await initTray(bag.store, { isMacOS: true, debounceMs: 50 });
    const actionCall = vi.mocked(tauriListen).mock.calls.find((c) => c[0] === "tray-panel-action")!;
    (actionCall[1] as (e: AnyItem) => void)({ payload: { kind: "open-project-settings" } });
    // openMainWindow 為 async：沖洗微任務後再斷言
    await new Promise((r) => setTimeout(r, 0));
    expect(win.show).toHaveBeenCalled();
    expect(bag.setBoardView).toHaveBeenCalledWith("project-settings");
  });

  it("面板動作 quit 呼叫結束 app 的命令", async () => {
    const bag = makeStore();
    await initTray(bag.store, { isMacOS: true, debounceMs: 50 });
    const actionCall = vi.mocked(tauriListen).mock.calls.find((c) => c[0] === "tray-panel-action")!;
    (actionCall[1] as (e: AnyItem) => void)({ payload: { kind: "quit" } });
    expect(invoke).toHaveBeenCalledWith("quit_app");
  });

  it("面板動作 add-project 先喚起主視窗再轉呼資料夾選擇流程（D7 快速加入專案）", async () => {
    const bag = makeStore();
    await initTray(bag.store, { isMacOS: true, debounceMs: 50 });
    const actionCall = vi.mocked(tauriListen).mock.calls.find((c) => c[0] === "tray-panel-action")!;
    (actionCall[1] as (e: AnyItem) => void)({ payload: { kind: "add-project" } });
    // 喚起主視窗（openIn 路徑）：主視窗在另一桌面時 macOS 才會切 Space，
    // 選擇器才於前景可見（D7 實測修訂）。openMainWindow 為 async，沖洗微任務。
    await new Promise((r) => setTimeout(r, 0));
    expect(win.show).toHaveBeenCalled();
    expect(bag.openProjectViaDialog).toHaveBeenCalled();
  });

  it("dispose 取消訂閱並關閉 tray", async () => {
    const bag = makeStore();
    const controller = await initTray(bag.store, { isMacOS: true, debounceMs: 50 });
    controller.dispose();
    expect(trayObj.close).toHaveBeenCalled();
    trayObj.setMenu.mockClear();
    vi.useFakeTimers();
    bag.emit({ activeKey: "local:/proj/two" });
    await vi.advanceTimersByTimeAsync(50);
    vi.useRealTimers();
    expect(trayObj.setMenu).not.toHaveBeenCalled();
  });
});

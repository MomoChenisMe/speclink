import { describe, it, expect, vi, beforeEach } from "vitest";
import type { ChangeItem } from "@speclink/ui";
import { TrayIcon } from "@tauri-apps/api/tray";
import { Menu } from "@tauri-apps/api/menu";
import { Image } from "@tauri-apps/api/image";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { emit as tauriEmit, listen as tauriListen } from "@tauri-apps/api/event";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";

import {
  buildTrayModel,
  progressBar,
  initTray,
  type TraySnapshot,
  type TrayMenuItem,
  type TrayStoreApi,
} from "../tray";
import { trayIconBytes } from "../trayIcon";

// tray.ts 於頂層 import @tauri-apps/api 的 tray/menu/window/image（＋面板模式的 webviewWindow/dpi）；
// 於 jsdom 下以樁替，使模組載入安全並讓接線測試可攔截呼叫。純函式測試不觸及這些。
vi.mock("@tauri-apps/api/tray", () => ({ TrayIcon: { new: vi.fn() } }));
vi.mock("@tauri-apps/api/menu", () => ({ Menu: { new: vi.fn() } }));
vi.mock("@tauri-apps/api/image", () => ({ Image: { fromBytes: vi.fn() } }));
vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({
  emit: vi.fn().mockResolvedValue(undefined),
  listen: vi.fn().mockResolvedValue(() => {}),
}));
vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({ writeText: vi.fn() }));
vi.mock("@tauri-apps/plugin-positioner", () => ({
  handleIconState: vi.fn().mockResolvedValue(undefined),
}));

const fakeT = (key: string): string =>
  (({
    "tray.open": "開啟 Speclink",
    "tray.quit": "結束",
    "tray.discussions": "討論 {n}",
    "tray.discussionsHeader": "討論",
    "tray.promotedHeader": "已轉出",
    "tray.more": "還有 {n} 個…",
    "tray.openChange": "開啟此變更",
    "tray.copyName": "複製名稱",
    "tray.openDiscussion": "開啟此討論",
    "tray.copySlug": "複製 slug",
    "tray.noChanges": "尚無進行中變更",
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
      { root: "/proj/one", name: "one" },
      { root: "/proj/two", name: "two" },
    ],
    activeRoot: "/proj/one",
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
    expect(projects.map((p) => p.root)).toEqual(["/proj/one", "/proj/two"]);
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

  it("徽章＝進行中變更數", () => {
    expect(buildTrayModel(snapshot(), fakeT).badge).toBe("1");
    expect(buildTrayModel(snapshot({ changes: [] }), fakeT).badge).toBe("");
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
      "quit",
    ]);
  });
});

// ---- 接線層（Tauri tray/menu/window API）----

/** 可觀察的假 store（tabs＋changes＋discussions＋動作），可推變更通知訂閱者。 */
function makeStore(
  openProjectAt = vi.fn(),
  openDetail = vi.fn(),
  openDiscussion = vi.fn(),
  openProjectViaDialog = vi.fn(),
) {
  let state = {
    tabs: [
      { root: "/proj/one", name: "one" },
      { root: "/proj/two", name: "two" },
    ],
    activeRoot: "/proj/one",
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
    openProjectAt,
    openDetail,
    openDiscussion,
    openProjectViaDialog,
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
    openProjectAt,
    openDetail,
    openDiscussion,
    openProjectViaDialog,
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

  it("變更列為子選單（有 items）、標籤帶進度條；store 變動後重建並更新徽章", async () => {
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
    expect(trayObj.setTitle).toHaveBeenLastCalledWith("2");
  });

  it("點非作用中專案呼叫 openProjectAt", async () => {
    const bag = makeStore();
    await initTray(bag.store, { isMacOS: true });
    const projects = lastItems.filter((i) => "checked" in i);
    projects[1].action();
    expect(bag.openProjectAt).toHaveBeenCalledWith("/proj/two");
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

  it("native 樣式：點擊事件不觸發 onPanelToggle（選單接管互動）", async () => {
    const bag = makeStore();
    const onPanelToggle = vi.fn();
    await initTray(bag.store, { isMacOS: true, onPanelToggle });
    const opts = vi.mocked(TrayIcon.new).mock.calls[0][0] as AnyItem;
    opts.action?.({ type: "Click", button: "Left", buttonState: "Up" });
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
    bag.emit({ activeRoot: "/proj/two" });
    await vi.advanceTimersByTimeAsync(50);
    vi.useRealTimers();
    expect(vi.mocked(tauriEmit)).toHaveBeenCalledWith(
      "tray-snapshot",
      expect.objectContaining({ activeRoot: "/proj/two" }),
    );
    // 動作回流：open-change 開主視窗並開啟該變更詳情
    const actionCall = vi.mocked(tauriListen).mock.calls.find((c) => c[0] === "tray-panel-action")!;
    (actionCall[1] as (e: AnyItem) => void)({ payload: { kind: "open-change", id: "alpha" } });
    // openMainWindow 為 async（unminimize → show）：沖洗微任務後再斷言
    await new Promise((r) => setTimeout(r, 0));
    expect(win.show).toHaveBeenCalled();
    expect(bag.openDetail).toHaveBeenCalledWith("alpha");
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
    bag.emit({ activeRoot: "/proj/two" });
    await vi.advanceTimersByTimeAsync(50);
    vi.useRealTimers();
    expect(trayObj.setMenu).not.toHaveBeenCalled();
  });
});

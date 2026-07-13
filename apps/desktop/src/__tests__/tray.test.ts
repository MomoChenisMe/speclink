import { describe, it, expect, vi, beforeEach } from "vitest";
import type { ChangeItem } from "@speclink/ui";
import { TrayIcon } from "@tauri-apps/api/tray";
import { Menu } from "@tauri-apps/api/menu";
import { Image } from "@tauri-apps/api/image";
import { getCurrentWindow } from "@tauri-apps/api/window";

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

const fakeT = (key: string): string =>
  (({
    "tray.open": "開啟 Speclink",
    "tray.quit": "結束",
    "tray.discussions": "討論 {n}",
    "tray.discussionsHeader": "討論",
    "tray.openChange": "開啟此變更",
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
      { slug: "d1", topic: "討論一" },
      { slug: "d2", topic: "討論二" },
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

  it("變更列帶進度條標籤與「開啟此變更」子選單動作", () => {
    const changes = byKind(buildTrayModel(snapshot(), fakeT).items, "change");
    expect(changes.map((c) => c.name)).toEqual(["prop", "inprog", "rdy"]);
    // 進行中 3/12 → 進度條＋n/m
    const inprog = changes.find((c) => c.name === "inprog")!;
    expect(inprog.label).toBe("inprog  ▓▓░░░░░░ 3/12");
    expect(inprog.actions).toEqual([{ kind: "open-change", label: "開啟此變更" }]);
    // ready 全滿
    expect(changes.find((c) => c.name === "rdy")!.label).toBe("rdy  ▓▓▓▓▓▓▓▓ 4/4");
  });

  it("無任務的變更僅顯示名稱（不畫進度條）", () => {
    const items = buildTrayModel(
      snapshot({ changes: [change({ name: "empty-tasks", totalTasks: 0, completedTasks: 0 })] }),
      fakeT,
    ).items;
    expect(byKind(items, "change")[0].label).toBe("empty-tasks");
  });

  it("列出討論（header＋各討論項，帶 slug）", () => {
    const items = buildTrayModel(snapshot(), fakeT).items;
    const disc = byKind(items, "discussion");
    expect(disc.map((d) => [d.slug, d.label])).toEqual([
      ["d1", "討論一"],
      ["d2", "討論二"],
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
    discussions: { active: [{ slug: "d1", topic: "t1" }, { slug: "d2", topic: "t2" }] },
    openProjectAt,
    openDetail,
    openDiscussion,
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
  };
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type AnyItem = any;

describe("initTray 接線（選單）", () => {
  const trayObj = { setMenu: vi.fn(), setTitle: vi.fn(), close: vi.fn() };
  const win = { show: vi.fn(), setFocus: vi.fn(), unminimize: vi.fn() };
  let lastItems: AnyItem[] = [];

  beforeEach(() => {
    vi.clearAllMocks();
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

  it("點討論項開主視窗並開啟該討論", async () => {
    const bag = makeStore();
    await initTray(bag.store, { isMacOS: true });
    const disc = lastItems.find((i) => i.text === "t1");
    expect(disc).toBeDefined();
    await disc.action();
    expect(win.show).toHaveBeenCalled();
    expect(bag.openDiscussion).toHaveBeenCalledWith("d1");
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

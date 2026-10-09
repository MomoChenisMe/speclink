import { describe, it, expect, vi, afterEach } from "vitest";
import { Menu } from "@tauri-apps/api/menu";

import {
  buildAppMenuModel,
  buildShortcutList,
  installAppMenu,
  type AppMenuEntry,
  type AppMenuModel,
} from "../appMenu";
import { APP_MESSAGES } from "../i18n/messages";
import type { Platform } from "../platform";

vi.mock("@tauri-apps/api/menu", () => ({ Menu: { new: vi.fn() } }));

const tOf = (locale: "zh-TW" | "en") => (key: string): string => APP_MESSAGES[locale][key] ?? key;

/** 項目的識別：自訂項取動作 id、預設項取系統項名、分隔線為 "-"。 */
const entryId = (e: AppMenuEntry): string =>
  e.kind === "item" ? e.id : e.kind === "predefined" ? e.item : "-";

const customItems = (model: AppMenuModel) =>
  model.flatMap((g) => g.items).filter((e) => e.kind === "item");

describe("buildAppMenuModel（design D2）", () => {
  const model = buildAppMenuModel({ t: tOf("zh-TW"), hasProject: true });

  it("六組依序為 Speclink、檔案、編輯、檢視、視窗、說明，各組項目依序排列", () => {
    expect(model.map((g) => g.id)).toEqual(["app", "file", "edit", "view", "window", "help"]);
    expect(Object.fromEntries(model.map((g) => [g.id, g.items.map(entryId)]))).toEqual({
      app: [
        "about",
        "checkUpdates",
        "-",
        "settings",
        "-",
        "Services",
        "-",
        "Hide",
        "HideOthers",
        "ShowAll",
        "-",
        "Quit",
      ],
      file: ["openProject", "closeProject", "-", "installCli"],
      edit: ["Undo", "Redo", "-", "Cut", "Copy", "Paste", "SelectAll"],
      view: [
        "viewBoard",
        "viewArchived",
        "viewSpecs",
        "viewManual",
        "viewProjectSettings",
        "-",
        "focusSearch",
        "refresh",
        "-",
        "nextProject",
      ],
      window: ["Minimize", "Maximize", "-", "bringAllToFront"],
      help: ["helpManual", "releaseNotes", "-", "github", "reportIssue"],
    });
  });

  it("自訂項目的快捷鍵字串", () => {
    const accelerators = Object.fromEntries(
      customItems(model)
        .filter((e) => e.accelerator !== undefined)
        .map((e) => [e.id, e.accelerator]),
    );
    expect(accelerators).toEqual({
      settings: "CmdOrCtrl+,",
      openProject: "Cmd+O",
      closeProject: "Cmd+W",
      viewBoard: "Cmd+1",
      viewArchived: "Cmd+2",
      viewSpecs: "Cmd+3",
      viewManual: "Cmd+4",
      focusSearch: "Cmd+F",
      refresh: "Cmd+R",
      nextProject: "Ctrl+Tab",
    });
  });

  it("隱藏、隱藏其他、結束、縮到最小是系統預設項（⌘H、⌥⌘H、⌘Q、⌘M 由系統自帶）", () => {
    const predefined = model
      .flatMap((g) => g.items)
      .filter((e) => e.kind === "predefined")
      .map((e) => e.item);
    expect(predefined).toEqual(
      expect.arrayContaining(["Hide", "HideOthers", "Quit", "Minimize"]),
    );
  });

  it("有作用中專案時所有自訂項目可用", () => {
    expect(customItems(model).filter((e) => !e.enabled)).toEqual([]);
  });

  it("沒有作用中專案時停用集合恰為關閉專案、檢視組八項與說明的手冊", () => {
    const empty = buildAppMenuModel({ t: tOf("zh-TW"), hasProject: false });
    expect(
      customItems(empty)
        .filter((e) => !e.enabled)
        .map((e) => e.id)
        .sort(),
    ).toEqual(
      [
        "closeProject",
        "viewBoard",
        "viewArchived",
        "viewSpecs",
        "viewManual",
        "viewProjectSettings",
        "focusSearch",
        "refresh",
        "nextProject",
        "helpManual",
      ].sort(),
    );
  });

  it("zh-TW 與 en 字典各建一次：文案皆取自字典（非 key 本身），兩語系的組標題不同", () => {
    const texts = (m: AppMenuModel) =>
      m.flatMap((g) => [
        g.text,
        ...g.items.flatMap((e) => (e.kind === "separator" || e.text === undefined ? [] : [e.text])),
      ]);
    const missing = (locale: "zh-TW" | "en") => {
      const keys: string[] = [];
      const recording = (key: string) => {
        keys.push(key);
        return tOf(locale)(key);
      };
      buildAppMenuModel({ t: recording, hasProject: true });
      return keys.filter((key) => !(key in APP_MESSAGES[locale]));
    };
    expect(missing("zh-TW")).toEqual([]);
    expect(missing("en")).toEqual([]);

    const zh = buildAppMenuModel({ t: tOf("zh-TW"), hasProject: true });
    const en = buildAppMenuModel({ t: tOf("en"), hasProject: true });
    expect(texts(zh).some((text) => text.startsWith("menu."))).toBe(false);
    expect(texts(en).some((text) => text.startsWith("menu."))).toBe(false);
    expect(zh.map((g) => g.text)).toEqual(["Speclink", "檔案", "編輯", "檢視", "視窗", "說明"]);
    expect(en.map((g) => g.text)).toEqual(["Speclink", "File", "Edit", "View", "Window", "Help"]);
    expect(texts(zh)).not.toEqual(texts(en));
  });
});

type MenuOptionsTree = { text?: string; item?: unknown; action?: () => void; items?: MenuOptionsTree[] };

/** 假的 Tauri 選單：記下每次 Menu.new 收到的整棵選項物件；setAsAppMenu 回傳被換下的舊選單。 */
function fakeMenus() {
  const built: MenuOptionsTree[][] = [];
  const menus: Array<{ setAsAppMenu: ReturnType<typeof vi.fn>; close: ReturnType<typeof vi.fn> }> = [];
  const replaced = { close: vi.fn().mockResolvedValue(undefined) };
  vi.mocked(Menu.new).mockImplementation(async (opts) => {
    built.push((opts?.items ?? []) as MenuOptionsTree[]);
    const menu = {
      setAsAppMenu: vi.fn().mockResolvedValue(replaced),
      close: vi.fn().mockResolvedValue(undefined),
    };
    menus.push(menu);
    return menu as never;
  });
  return { built, menus, replaced };
}

const findItem = (tree: MenuOptionsTree[], text: string): MenuOptionsTree | undefined => {
  for (const node of tree) {
    if (node.text === text && node.action) return node;
    const hit = node.items ? findItem(node.items, text) : undefined;
    if (hit) return hit;
  }
  return undefined;
};

describe("installAppMenu（design D3）", () => {
  afterEach(() => {
    vi.mocked(Menu.new).mockReset();
    vi.restoreAllMocks();
  });

  const install = (overrides: Partial<Parameters<typeof installAppMenu>[0]> = {}) => {
    const dispatch = vi.fn();
    const cancel = installAppMenu({
      isMacOS: true,
      t: tOf("zh-TW"),
      hasProject: true,
      dispatch,
      ...overrides,
    });
    return { dispatch, cancel };
  };

  it("macOS 建一次並設為 app 選單，再關掉被換下的舊選單", async () => {
    const { menus, replaced } = fakeMenus();
    install();
    await vi.waitFor(() => expect(replaced.close).toHaveBeenCalledTimes(1));
    expect(Menu.new).toHaveBeenCalledTimes(1);
    expect(menus[0].setAsAppMenu).toHaveBeenCalledTimes(1);
    expect(menus[0].close).not.toHaveBeenCalled();
  });

  it("依傳入的語言建出六組子選單", async () => {
    const { built } = fakeMenus();
    install({ t: tOf("en") });
    await vi.waitFor(() => expect(built).toHaveLength(1));
    expect(built[0].map((group) => group.text)).toEqual([
      "Speclink",
      "File",
      "Edit",
      "View",
      "Window",
      "Help",
    ]);
  });

  it("觸發選單項的 action 以對應動作 id 呼叫 dispatch", async () => {
    const { built } = fakeMenus();
    const { dispatch } = install();
    await vi.waitFor(() => expect(built).toHaveLength(1));
    findItem(built[0], "規格")?.action?.();
    findItem(built[0], "回報問題")?.action?.();
    expect(dispatch.mock.calls).toEqual([["viewSpecs"], ["reportIssue"]]);
  });

  it("非 macOS 不呼叫任何選單 API", () => {
    fakeMenus();
    const { cancel } = install({ isMacOS: false });
    cancel();
    expect(Menu.new).not.toHaveBeenCalled();
  });

  it("取消後才建好的選單不設為 app 選單，並關掉它", async () => {
    const { menus } = fakeMenus();
    const { cancel } = install();
    cancel();
    await vi.waitFor(() => expect(menus[0]?.close).toHaveBeenCalledTimes(1));
    expect(menus[0].setAsAppMenu).not.toHaveBeenCalled();
  });

  it("Menu.new 拋錯時記 console.error、不向外拋", async () => {
    const error = vi.spyOn(console, "error").mockImplementation(() => {});
    vi.mocked(Menu.new).mockRejectedValue(new Error("menu unavailable"));
    install();
    await vi.waitFor(() => expect(error).toHaveBeenCalledTimes(1));
  });
});

describe("buildShortcutList（design D5）", () => {
  const rows = (platform: Platform, locale: "zh-TW" | "en" = "zh-TW") =>
    buildShortcutList({ t: tOf(locale), platform }).map((row) => [row.text, row.keys]);

  it("macOS 回 11 列，逐列等於 spec Example「macOS 的卡片內容」", () => {
    expect(rows("macos")).toEqual([
      ["設定…", "⌘,"],
      ["開啟專案…", "⌘O"],
      ["關閉專案", "⌘W"],
      ["變更", "⌘1"],
      ["已封存", "⌘2"],
      ["規格", "⌘3"],
      ["手冊", "⌘4"],
      ["搜尋看板", "⌘F"],
      ["重新整理", "⌘R"],
      ["下一個專案", "⌃Tab"],
      ["跳到第 1–9 個專案", "⌃1–9"],
    ]);
  });

  it.each(["windows", "linux"] as const)(
    "%s 回 5 列，逐列等於 spec Example「Windows 與 Linux 的卡片內容」",
    (platform) => {
      expect(rows(platform)).toEqual([
        ["關閉專案", "Ctrl+W"],
        ["搜尋看板", "Ctrl+F"],
        ["重新整理", "Ctrl+R"],
        ["下一個專案", "Ctrl+Tab"],
        ["跳到第 1–9 個專案", "Ctrl+1–9"],
      ]);
    },
  );

  it("以 en 字典建時動作名稱為英文、按鍵不變", () => {
    expect(rows("macos", "en")).toEqual([
      ["Settings…", "⌘,"],
      ["Open Project…", "⌘O"],
      ["Close Project", "⌘W"],
      ["Changes", "⌘1"],
      ["Archived", "⌘2"],
      ["Specs", "⌘3"],
      ["Manual", "⌘4"],
      ["Search Board", "⌘F"],
      ["Refresh", "⌘R"],
      ["Next Project", "⌃Tab"],
      ["Go to Project 1–9", "⌃1–9"],
    ]);
  });
});

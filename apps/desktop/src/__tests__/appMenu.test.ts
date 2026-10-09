import { describe, it, expect, vi, afterEach } from "vitest";

import {
  buildAppMenuModel,
  installAppMenu,
  type AppMenuEntry,
  type AppMenuModel,
} from "../appMenu";
import { APP_MESSAGES } from "../i18n/messages";

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

/** 假的 Tauri 選單 API：記下每次 Menu.new 收到的整棵選項物件與 setAsAppMenu 次數。 */
function fakeMenuApi() {
  const setAsAppMenu = vi.fn().mockResolvedValue(null);
  const built: MenuOptionsTree[][] = [];
  const Menu = {
    new: vi.fn(async (opts: { items: MenuOptionsTree[] }) => {
      built.push(opts.items);
      return { setAsAppMenu };
    }),
  };
  return { menuApi: { Menu } as never, Menu, setAsAppMenu, built };
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
  afterEach(() => vi.restoreAllMocks());

  const base = (overrides: Partial<Parameters<typeof installAppMenu>[0]> = {}) => {
    const api = fakeMenuApi();
    const dispatch = vi.fn();
    const controller = installAppMenu({
      isMacOS: () => true,
      t: tOf("zh-TW"),
      hasProject: true,
      dispatch,
      menuApi: api.menuApi,
      ...overrides,
    });
    return { ...api, dispatch, controller };
  };

  it("macOS 安裝即建一次並設為 app 選單；rebuild 後各兩次", async () => {
    const { Menu, setAsAppMenu, controller } = base();
    await vi.waitFor(() => expect(setAsAppMenu).toHaveBeenCalledTimes(1));
    expect(Menu.new).toHaveBeenCalledTimes(1);

    await controller.rebuild({ t: tOf("en"), hasProject: true });
    expect(Menu.new).toHaveBeenCalledTimes(2);
    expect(setAsAppMenu).toHaveBeenCalledTimes(2);
  });

  it("rebuild 依新語言建出六組子選單", async () => {
    const { built, controller } = base();
    await controller.rebuild({ t: tOf("en"), hasProject: true });
    expect(built[built.length - 1].map((group) => group.text)).toEqual([
      "Speclink",
      "File",
      "Edit",
      "View",
      "Window",
      "Help",
    ]);
  });

  it("rebuild 的輸入與上次相同時不重建", async () => {
    const t = tOf("zh-TW");
    const { Menu, controller } = base({ t });
    await controller.rebuild({ t, hasProject: true });
    expect(Menu.new).toHaveBeenCalledTimes(1);
  });

  it("觸發選單項的 action 以對應動作 id 呼叫 dispatch", async () => {
    const { built, dispatch, setAsAppMenu } = base();
    await vi.waitFor(() => expect(setAsAppMenu).toHaveBeenCalledTimes(1));
    findItem(built[0], "規格")?.action?.();
    findItem(built[0], "回報問題")?.action?.();
    expect(dispatch.mock.calls).toEqual([["viewSpecs"], ["reportIssue"]]);
  });

  it("非 macOS 不呼叫任何選單 API，rebuild 與 dispose 為 no-op", async () => {
    const { Menu, setAsAppMenu, controller } = base({ isMacOS: () => false });
    await controller.rebuild({ t: tOf("en"), hasProject: false });
    controller.dispose();
    expect(Menu.new).not.toHaveBeenCalled();
    expect(setAsAppMenu).not.toHaveBeenCalled();
  });

  it("dispose 後的 rebuild 不再建選單", async () => {
    const { Menu, setAsAppMenu, controller } = base();
    await vi.waitFor(() => expect(setAsAppMenu).toHaveBeenCalledTimes(1));
    controller.dispose();
    await controller.rebuild({ t: tOf("en"), hasProject: true });
    expect(Menu.new).toHaveBeenCalledTimes(1);
  });

  it("Menu.new 拋錯時記 console.error、不向外拋", async () => {
    const error = vi.spyOn(console, "error").mockImplementation(() => {});
    const api = fakeMenuApi();
    api.Menu.new.mockRejectedValue(new Error("menu unavailable"));
    const controller = installAppMenu({
      isMacOS: () => true,
      t: tOf("zh-TW"),
      hasProject: true,
      dispatch: vi.fn(),
      menuApi: api.menuApi,
    });
    await expect(controller.rebuild({ t: tOf("en"), hasProject: true })).resolves.toBeUndefined();
    expect(error).toHaveBeenCalledTimes(2);
    expect(api.setAsAppMenu).not.toHaveBeenCalled();
  });
});

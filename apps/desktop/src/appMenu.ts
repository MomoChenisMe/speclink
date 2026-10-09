// macOS 原生選單列（desktop-native-menu design D1–D3）：與系統匣同一機制——文案取自
// i18n 字典、語言或「有無專案」改變時整個重建。分兩層：buildAppMenuModel 是純函式
// （直測）；installAppMenu 以 Tauri 選單 API 建出並設為 app 選單。
import {
  Menu,
  type MenuItemOptions,
  type PredefinedMenuItemOptions,
  type SubmenuOptions,
} from "@tauri-apps/api/menu";

import type { Platform } from "./platform";

/** 自訂選單項的動作 id：App 依此派發到 store 既有動作或 window 事件。 */
export type AppMenuAction =
  | "about"
  | "checkUpdates"
  | "settings"
  | "openProject"
  | "closeProject"
  | "installCli"
  | "viewBoard"
  | "viewArchived"
  | "viewSpecs"
  | "viewManual"
  | "viewProjectSettings"
  | "focusSearch"
  | "refresh"
  | "nextProject"
  | "bringAllToFront"
  | "helpManual"
  | "releaseNotes"
  | "github"
  | "reportIssue";

/** 系統預設項：行為與快捷鍵由系統提供（Tauri 的 predefined 項不接受 accelerator）。 */
export type AppMenuPredefined =
  | "Services"
  | "Hide"
  | "HideOthers"
  | "ShowAll"
  | "Quit"
  | "Undo"
  | "Redo"
  | "Cut"
  | "Copy"
  | "Paste"
  | "SelectAll"
  | "Minimize"
  | "Maximize";

export type AppMenuEntry =
  | { kind: "item"; id: AppMenuAction; text: string; accelerator?: string; enabled: boolean }
  | { kind: "predefined"; item: AppMenuPredefined; text?: string }
  | { kind: "separator" };

export interface AppMenuGroup {
  id: "app" | "file" | "edit" | "view" | "window" | "help";
  text: string;
  items: AppMenuEntry[];
}

export type AppMenuModel = AppMenuGroup[];

/** 選單模型（design D2）：沒有作用中專案時，關閉專案、檢視組全部與說明的手冊停用。 */
export function buildAppMenuModel({
  t,
  hasProject,
}: {
  t: (key: string) => string;
  hasProject: boolean;
}): AppMenuModel {
  const item = (
    id: AppMenuAction,
    key: string,
    accelerator?: string,
    enabled = true,
  ): AppMenuEntry => ({ kind: "item", id, text: t(key), accelerator, enabled });
  const predefined = (name: AppMenuPredefined, key?: string): AppMenuEntry => ({
    kind: "predefined",
    item: name,
    text: key === undefined ? undefined : t(key),
  });
  const separator: AppMenuEntry = { kind: "separator" };
  return [
    {
      id: "app",
      text: "Speclink",
      items: [
        item("about", "menu.app.about"),
        item("checkUpdates", "menu.app.checkUpdates"),
        separator,
        item("settings", "menu.app.settings", "CmdOrCtrl+,"),
        separator,
        predefined("Services"),
        separator,
        predefined("Hide", "menu.app.hide"),
        predefined("HideOthers", "menu.app.hideOthers"),
        predefined("ShowAll", "menu.app.showAll"),
        separator,
        predefined("Quit", "menu.app.quit"),
      ],
    },
    {
      id: "file",
      text: t("menu.file.title"),
      items: [
        item("openProject", "menu.file.openProject", "Cmd+O"),
        item("closeProject", "menu.file.closeProject", "Cmd+W", hasProject),
        separator,
        item("installCli", "menu.file.installCli"),
      ],
    },
    {
      id: "edit",
      text: t("menu.edit.title"),
      items: [
        predefined("Undo", "menu.edit.undo"),
        predefined("Redo", "menu.edit.redo"),
        separator,
        predefined("Cut", "menu.edit.cut"),
        predefined("Copy", "menu.edit.copy"),
        predefined("Paste", "menu.edit.paste"),
        predefined("SelectAll", "menu.edit.selectAll"),
      ],
    },
    {
      id: "view",
      text: t("menu.view.title"),
      items: [
        item("viewBoard", "menu.view.board", "Cmd+1", hasProject),
        item("viewArchived", "menu.view.archived", "Cmd+2", hasProject),
        item("viewSpecs", "menu.view.specs", "Cmd+3", hasProject),
        item("viewManual", "menu.view.manual", "Cmd+4", hasProject),
        item("viewProjectSettings", "menu.view.projectSettings", undefined, hasProject),
        separator,
        item("focusSearch", "menu.view.search", "Cmd+F", hasProject),
        item("refresh", "menu.view.refresh", "Cmd+R", hasProject),
        separator,
        item("nextProject", "menu.view.nextProject", "Ctrl+Tab", hasProject),
      ],
    },
    {
      id: "window",
      text: t("menu.window.title"),
      items: [
        predefined("Minimize", "menu.window.minimize"),
        predefined("Maximize", "menu.window.zoom"),
        separator,
        item("bringAllToFront", "menu.window.bringAllToFront"),
      ],
    },
    {
      id: "help",
      text: t("menu.help.title"),
      items: [
        item("helpManual", "menu.help.manual", undefined, hasProject),
        item("releaseNotes", "menu.help.releaseNotes"),
        separator,
        item("github", "menu.help.github"),
        item("reportIssue", "menu.help.reportIssue"),
      ],
    },
  ];
}

/** 設定頁「鍵盤快捷鍵」卡的一列（design D5）。 */
export interface ShortcutRow {
  id: AppMenuAction | "gotoProject";
  text: string;
  keys: string;
}

const MODIFIER_SYMBOLS: Record<string, string> = { Cmd: "⌘", CmdOrCtrl: "⌘", Ctrl: "⌃" };

/** 選單 accelerator 轉成 macOS 按鍵符號：`CmdOrCtrl+,` → `⌘,`、`Ctrl+Tab` → `⌃Tab`。 */
function formatAccelerator(accelerator: string): string {
  return accelerator
    .split("+")
    .map((part) => MODIFIER_SYMBOLS[part] ?? part)
    .join("");
}

/**
 * 設定頁「鍵盤快捷鍵」卡的列表（design D5），只列實際生效的快捷鍵。macOS 由選單模型導出，
 * 卡片與選單只有一份定義；Windows／Linux 沒有選單列，五列對應 App.tsx 的 keydown（Ctrl+W、
 * Ctrl+R、Ctrl+Tab、Ctrl+1–9）與 BoardSearchBar 的 Ctrl+F——改那兩處的按鍵時要同批改這裡。
 */
export function buildShortcutList({
  t,
  platform,
}: {
  t: (key: string) => string;
  platform: Platform;
}): ShortcutRow[] {
  const gotoProject = (keys: string): ShortcutRow => ({
    id: "gotoProject",
    text: t("settings.shortcutsGotoProject"),
    keys,
  });
  if (platform === "macos") {
    const menuRows = buildAppMenuModel({ t, hasProject: true })
      .flatMap((group) => group.items)
      .flatMap((entry): ShortcutRow[] =>
        entry.kind === "item" && entry.accelerator
          ? [{ id: entry.id, text: entry.text, keys: formatAccelerator(entry.accelerator) }]
          : [],
      );
    return [...menuRows, gotoProject("⌃1–9")];
  }
  return [
    { id: "closeProject", text: t("menu.file.closeProject"), keys: "Ctrl+W" },
    { id: "focusSearch", text: t("menu.view.search"), keys: "Ctrl+F" },
    { id: "refresh", text: t("menu.view.refresh"), keys: "Ctrl+R" },
    { id: "nextProject", text: t("menu.view.nextProject"), keys: "Ctrl+Tab" },
    gotoProject("Ctrl+1–9"),
  ];
}

export interface AppMenuDeps {
  isMacOS: boolean;
  t: (key: string) => string;
  hasProject: boolean;
  dispatch: (action: AppMenuAction) => void;
}

/**
 * 安裝 macOS 原生選單列（design D3）：與系統匣同一做法，把模型轉成整棵選項物件一次交給
 * `Menu.new` 後 `setAsAppMenu()`，再關掉被換下的舊選單（釋放 Rust 端資源）。非 macOS 不呼叫
 * 任何選單 API。回傳取消函式：語言或專案有無改變時 App 以新的一次取代，取消後才建好的
 * 選單不套用並關掉。建立失敗只記 console.error——app 照常、僅無選單列。
 */
export function installAppMenu(deps: AppMenuDeps): () => void {
  if (!deps.isMacOS) return () => {};

  const toOptions = (entry: AppMenuEntry): MenuItemOptions | PredefinedMenuItemOptions => {
    switch (entry.kind) {
      case "item":
        return {
          text: entry.text,
          accelerator: entry.accelerator,
          enabled: entry.enabled,
          action: () => deps.dispatch(entry.id),
        };
      case "predefined":
        return { item: entry.item, text: entry.text };
      case "separator":
        return { item: "Separator" };
    }
  };
  const items = buildAppMenuModel(deps).map(
    (group): SubmenuOptions => ({ text: group.text, items: group.items.map(toOptions) }),
  );

  let cancelled = false;
  void (async () => {
    try {
      const menu = await Menu.new({ items });
      if (cancelled) return void (await menu.close());
      const replaced = await menu.setAsAppMenu();
      await replaced?.close();
    } catch (e) {
      console.error("app menu build failed", e);
    }
  })();
  return () => {
    cancelled = true;
  };
}

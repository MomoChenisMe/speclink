// macOS 原生選單列（desktop-native-menu design D1–D3）：與系統匣同一機制——文案取自
// i18n 字典、語言或「有無專案」改變時整個重建。分兩層：buildAppMenuModel 是純函式
// （直測）；installAppMenu 以注入的 Tauri 選單 API 建出並設為 app 選單。
import type {
  Menu,
  MenuItemOptions,
  PredefinedMenuItemOptions,
  SubmenuOptions,
} from "@tauri-apps/api/menu";

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

export interface AppMenuDeps {
  isMacOS: () => boolean;
  t: (key: string) => string;
  hasProject: boolean;
  dispatch: (action: AppMenuAction) => void;
  menuApi: { Menu: Pick<typeof Menu, "new"> };
}

export interface AppMenuController {
  /** 以新語言或專案有無重建；輸入與上次相同時不重建。 */
  rebuild: (next: { t: (key: string) => string; hasProject: boolean }) => Promise<void>;
  dispose: () => void;
}

/**
 * 安裝 macOS 原生選單列（design D3）：與系統匣同一做法，把模型轉成整棵選項物件一次交給
 * `Menu.new` 後 `setAsAppMenu()`。非 macOS 不呼叫任何選單 API。重建依序執行，較舊的一次
 * 不會蓋掉較新的選單；建立失敗只記 console.error——app 照常、僅無選單列。
 */
export function installAppMenu(deps: AppMenuDeps): AppMenuController {
  if (!deps.isMacOS()) return { rebuild: async () => {}, dispose: () => {} };

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

  let disposed = false;
  let last = { t: deps.t, hasProject: deps.hasProject };
  let queue = Promise.resolve();
  const build = (input: typeof last) => {
    const items = buildAppMenuModel(input).map(
      (group): SubmenuOptions => ({ text: group.text, items: group.items.map(toOptions) }),
    );
    queue = queue.then(async () => {
      if (disposed) return;
      try {
        const menu = await deps.menuApi.Menu.new({ items });
        if (!disposed) await menu.setAsAppMenu();
      } catch (e) {
        console.error("app menu build failed", e);
      }
    });
    return queue;
  };

  void build(last);
  return {
    rebuild: (next) => {
      if (next.t === last.t && next.hasProject === last.hasProject) return queue;
      last = next;
      return build(next);
    },
    dispose: () => {
      disposed = true;
    },
  };
}

// 系統匣狀態選單（design D1/D3/D5）：前端擁有的呈現面。分兩層——
// (1) buildTrayModel：store 快照 → 選單模型的純函式（本檔上半，無 Tauri 依賴、直測）；
// (2) 接線層：訂閱 store、去抖重建、掛點擊 handler（本檔下半，以 Tauri JS tray/menu API）。
import { changeStage, STAGES, type ChangeItem } from "@speclink/ui";
import { TrayIcon } from "@tauri-apps/api/tray";
import {
  Menu,
  type MenuItemOptions,
  type CheckMenuItemOptions,
  type PredefinedMenuItemOptions,
  type SubmenuOptions,
} from "@tauri-apps/api/menu";
import { Image } from "@tauri-apps/api/image";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { appT } from "./i18n/runtime";
import { trayIconBytes } from "./trayIcon";

/** 組模型所需的 store 快照（自 AppState 收斂而來——與看板同源，design D3）。 */
export interface TraySnapshot {
  tabs: Array<{ root: string; name: string }>;
  activeRoot: string | null;
  changes: ChangeItem[];
  /** 進看板的 active 討論（列出於討論區；slug 供點擊開啟）。 */
  discussions: Array<{ slug: string; topic: string }>;
}

/** 變更子選單的動作項。 */
export type TrayChangeAction = { kind: "open-change"; label: string };

/** 選單項目模型（純資料；接線層據此建原生 menu item 並掛 action）。 */
export type TrayMenuItem =
  | { kind: "project"; root: string; label: string; checked: boolean }
  | { kind: "header"; label: string }
  | { kind: "change"; name: string; label: string; actions: TrayChangeAction[] }
  | { kind: "discussion"; slug: string; label: string }
  | { kind: "empty"; label: string }
  | { kind: "separator" }
  | { kind: "open"; label: string }
  | { kind: "quit"; label: string };

export interface TrayModel {
  items: TrayMenuItem[];
  /** macOS 標題徽章：進行中變更數；0 時為空字串（非 macOS 平台不套用）。 */
  badge: string;
}

/** unicode 方塊進度條（width 格）；total<=0 回空字串（無任務不畫）。 */
export function progressBar(completed: number, total: number, width = 8): string {
  if (total <= 0) return "";
  const filled = Math.max(0, Math.min(width, Math.round((completed / total) * width)));
  return "▓".repeat(filled) + "░".repeat(width - filled);
}

/** 變更列標籤：有任務時「名稱  ▓▓░ n/m」，無任務時僅名稱。 */
function changeLabel(c: ChangeItem): string {
  if (c.totalTasks <= 0) return c.name;
  const bar = progressBar(c.completedTasks, c.totalTasks);
  return `${c.name}  ${bar} ${c.completedTasks}/${c.totalTasks}`;
}

/**
 * 由 store 快照組出選單模型。純函式、無平台分支。區段順序：
 * 專案區（作用中打勾）→ 分隔 → 生命週期分區（提案中/進行中/已就緒各一 header＋變更子選單，
 * 變更列帶進度條，子選單含「開啟此變更」；全無變更則空狀態）→ 分隔 → 討論區（header＋各討論項，
 * 無則「討論 0」）→ 分隔 → 動作區（開啟 Speclink、結束）。徽章＝進行中變更數。
 */
export function buildTrayModel(
  snapshot: TraySnapshot,
  t: (key: string) => string,
): TrayModel {
  const items: TrayMenuItem[] = [];

  // 專案區
  for (const tab of snapshot.tabs) {
    items.push({
      kind: "project",
      root: tab.root,
      label: tab.name,
      checked: tab.root === snapshot.activeRoot,
    });
  }
  if (snapshot.tabs.length > 0) items.push({ kind: "separator" });

  // 生命週期分區：每個非空階段一個 header＋變更子選單
  let anyChange = false;
  for (const stage of STAGES) {
    const staged = snapshot.changes.filter((c) => changeStage(c) === stage);
    if (staged.length === 0) continue;
    anyChange = true;
    items.push({ kind: "header", label: t(`stage.${stage}`) });
    for (const c of staged) {
      items.push({
        kind: "change",
        name: c.name,
        label: changeLabel(c),
        actions: [{ kind: "open-change", label: t("tray.openChange") }],
      });
    }
  }
  if (!anyChange) items.push({ kind: "empty", label: t("tray.noChanges") });

  items.push({ kind: "separator" });

  // 討論區
  if (snapshot.discussions.length > 0) {
    items.push({ kind: "header", label: t("tray.discussionsHeader") });
    for (const d of snapshot.discussions) {
      items.push({ kind: "discussion", slug: d.slug, label: d.topic });
    }
  } else {
    items.push({ kind: "empty", label: t("tray.discussions").replace("{n}", "0") });
  }

  items.push({ kind: "separator" });
  items.push({ kind: "open", label: t("tray.open") });
  items.push({ kind: "quit", label: t("tray.quit") });

  const inProgress = snapshot.changes.filter((c) => changeStage(c) === "in-progress").length;
  return { items, badge: inProgress > 0 ? String(inProgress) : "" };
}

// ---- 接線層：訂閱 store、去抖重建、掛點擊 handler ----

/** tray 訂閱所需的最小 store 介面（Zustand bound store 相容——AppState 的結構子集）。 */
export interface TrayStoreApi {
  getState: () => {
    tabs: Array<{ root: string; name: string }>;
    activeRoot: string | null;
    changes: ChangeItem[];
    discussions: { active: Array<{ slug: string; topic: string }> };
    openProjectAt: (root: string) => void | Promise<void>;
    /** 開啟變更詳情抽屜（子選單「開啟此變更」用）。 */
    openDetail: (name: string) => void;
    /** 開啟討論抽屜（討論項點擊用）。 */
    openDiscussion: (slug: string) => void;
  };
  subscribe: (listener: () => void) => () => void;
}

export interface TrayDeps {
  /** macOS 才套用文字徽章與 template 圖示（可注入以利測試；預設由 navigator 偵測）。 */
  isMacOS?: boolean;
  /** 選單重建去抖延遲（ms）；預設與看板刷新同量級（400ms）。 */
  debounceMs?: number;
}

export interface TrayController {
  /** 卸載時呼叫：取消訂閱、清去抖 timer、關閉 tray 圖示。 */
  dispose: () => void;
}

/** 平台偵測：navigator.userAgent 判 macOS——無 os plugin，前端自足且可測。 */
function detectMacOS(): boolean {
  if (typeof navigator === "undefined") return false;
  const ua = navigator.userAgent || (navigator as { platform?: string }).platform || "";
  return /Macintosh|Mac OS/i.test(ua);
}

/** store 狀態 → 模型快照（與看板同源）。 */
function toSnapshot(state: ReturnType<TrayStoreApi["getState"]>): TraySnapshot {
  return {
    tabs: state.tabs.map((t) => ({ root: t.root, name: t.name })),
    activeRoot: state.activeRoot,
    changes: state.changes,
    discussions: state.discussions.active.map((d) => ({ slug: d.slug, topic: d.topic })),
  };
}

/**
 * 初始化系統匣原生選單：建圖示與首份選單、訂閱 store（去抖重建選單與 macOS 徽章）。
 * 回傳 controller，其 dispose 於 app 卸載時清理。專案切換走 store 既有 openProjectAt
 * 且不動視窗焦點；變更子選單「開啟此變更」與討論項開主視窗並跳詳情；「結束」為原生 predefined Quit。
 */
export async function initTray(store: TrayStoreApi, deps: TrayDeps = {}): Promise<TrayController> {
  const isMac = deps.isMacOS ?? detectMacOS();
  const debounceMs = deps.debounceMs ?? 400;
  const icon = await Image.fromBytes(trayIconBytes());

  type TrayItemOptions =
    | MenuItemOptions
    | CheckMenuItemOptions
    | PredefinedMenuItemOptions
    | SubmenuOptions;
  /** 開主視窗並執行 store 動作（子選單/討論項用）——tray 於主視窗 context，直接呼叫 store。 */
  const openIn = (fn: () => void) => {
    void openMainWindow();
    fn();
  };
  const toOptions = (item: TrayMenuItem): TrayItemOptions => {
    switch (item.kind) {
      case "project":
        return {
          text: item.label,
          checked: item.checked,
          action: () => {
            void store.getState().openProjectAt(item.root);
          },
        };
      case "header":
      case "empty":
        return { text: item.label, enabled: false };
      case "change":
        return {
          text: item.label,
          items: item.actions.map((a) => ({
            text: a.label,
            action: () => openIn(() => store.getState().openDetail(item.name)),
          })),
        };
      case "discussion":
        return {
          text: item.label,
          action: () => openIn(() => store.getState().openDiscussion(item.slug)),
        };
      case "separator":
        return { item: "Separator" };
      case "open":
        return { text: item.label, action: () => openMainWindow() };
      case "quit":
        return { item: "Quit", text: item.label };
    }
  };
  const render = async () => {
    const model = buildTrayModel(toSnapshot(store.getState()), appT);
    const menu = await Menu.new({ items: model.items.map(toOptions) });
    return { menu, badge: model.badge };
  };

  const first = await render();
  const tray = await TrayIcon.new({
    icon,
    iconAsTemplate: isMac,
    tooltip: "Speclink",
    menu: first.menu,
    title: isMac ? first.badge || undefined : undefined,
  });

  // 去抖訂閱：資料變動後整份重建選單並更新 macOS 徽章。
  let timer: ReturnType<typeof setTimeout> | null = null;
  const unsubscribe = store.subscribe(() => {
    if (timer !== null) clearTimeout(timer);
    timer = setTimeout(() => {
      void (async () => {
        const { menu, badge } = await render();
        await tray.setMenu(menu);
        if (isMac) await tray.setTitle(badge || null);
      })();
    }, debounceMs);
  });

  return {
    dispose: () => {
      if (timer !== null) clearTimeout(timer);
      unsubscribe();
      void tray.close();
    },
  };
}

/** 顯示主視窗並取得焦點（自最小化亦還原）——選單「開啟 Speclink」項用。 */
async function openMainWindow(): Promise<void> {
  const win = getCurrentWindow();
  await win.unminimize();
  await win.show();
  await win.setFocus();
}

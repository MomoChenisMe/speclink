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
import { emit, listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { handleIconState } from "@tauri-apps/plugin-positioner";

import { appT } from "./i18n/runtime";
import type { ConnectionView } from "./adapter/connections";
import {
  locatorKey,
  remoteWorkspaceStatus,
  type RemoteRecoveryKind,
  type RemoteWorkspaceRecoveryState,
  type RemoteWorkspaceStatus,
  type WorkspaceLocator,
  type WorkspaceSession,
} from "./session";
import { trayIconBytes } from "./trayIcon";

/** 系統匣互動樣式：由平台決定（macOS＝panel、其餘＝native-menu），panel 建立失敗時
    退回 native-menu——執行期狀態、不持久化（tray-macos-panel-only 拆除偏好）。 */
export type TrayStyle = "native-menu" | "panel";

export type TrayTabSnapshot =
  | {
      key: string;
      name: string;
      source: "local";
      status: "ready";
    }
  | {
      key: string;
      name: string;
      source: "remote";
      status: RemoteWorkspaceStatus;
      /** 僅 remote error 的封閉分類；不把 raw technical detail 帶入 Tray。 */
      failureKind?: RemoteRecoveryKind;
      connectionId: string;
      serverLabel: string;
      serverOrigin?: string;
    };

/** 組模型所需的 store 快照（自 AppState 收斂而來——與看板同源，design D3）。
 * 分頁項識別與切換把手皆為 locator key（workspace-session 決策 6）。 */
export interface TraySnapshot {
  tabs: TrayTabSnapshot[];
  activeKey: string | null;
  changes: ChangeItem[];
  /** 進看板的 active 討論（slug 供點擊開啟；promoted＝已轉出變更，分流至「已轉出」分區）。 */
  discussions: Array<{ slug: string; topic: string; promoted: boolean }>;
}

/** 變更子選單的動作項。 */
export type TrayChangeAction = { kind: "open-change" | "copy-name"; label: string };

/** 討論子選單的動作項。 */
export type TrayDiscussionAction = { kind: "open-discussion" | "copy-slug"; label: string };

export type TrayRecoveryAction = {
  kind: "retry" | "open-recovery" | "open-settings" | "reauthenticate";
  label: string;
};

/** 分區溢出門檻（spec「分區溢出摺疊」）：各分區直列筆數上限，逾此收進溢出節點。 */
export const OVERFLOW_LIMIT = 5;

/** 選單項目模型（純資料；接線層據此建原生 menu item 並掛 action）。 */
export type TrayMenuItem =
  | {
      kind: "project";
      key: string;
      label: string;
      checked: boolean;
      source: "local" | "remote";
      status: "ready" | "offline";
    }
  | {
      kind: "recovery";
      key: string;
      connectionId: string;
      label: string;
      summary: string;
      active: boolean;
      status: "restoring" | "error" | "needs-reauth";
      actions: TrayRecoveryAction[];
    }
  | { kind: "header"; label: string }
  | { kind: "change"; name: string; label: string; actions: TrayChangeAction[] }
  | {
      kind: "discussion";
      slug: string;
      /** 父項標籤＝slug（識別錨點直出）；topic 降為子選單首行描述。 */
      label: string;
      topic: string;
      actions: TrayDiscussionAction[];
    }
  | {
      /** 分區溢出節點「還有 N 個…」：內嵌其餘項目的同構模型（接線層遞迴轉子選單）。 */
      kind: "overflow";
      label: string;
      items: TrayMenuItem[];
    }
  | { kind: "empty"; label: string }
  | { kind: "separator" }
  | { kind: "open"; label: string }
  | { kind: "settings"; label: string }
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
 * 變更列帶進度條，子選單含「開啟此變更」「複製名稱」；全無變更則空狀態）→ 分隔 →
 * 討論區（header＋各討論子選單：父項標籤為 slug、topic 為子選單首行描述、
 * 含「開啟此討論」「複製 slug」；無則「討論 0」）→ 分隔 → 動作區（開啟 Speclink、設定、結束）。
 * 徽章＝進行中變更數。
 */
export function buildTrayModel(
  snapshot: TraySnapshot,
  t: (key: string) => string,
): TrayModel {
  const items: TrayMenuItem[] = [];
  const activeTab = snapshot.tabs.find((tab) => tab.key === snapshot.activeKey);
  const hideWorkspaceData =
    activeTab?.status === "restoring" || activeTab?.status === "error";

  const statusSummary = (tab: TrayTabSnapshot): string => {
    if (tab.source === "local") return "";
    if (tab.status === "restoring") return t("tray.recovery.restoring");
    if (tab.status === "offline") return t("tray.recovery.offline");
    if (tab.status === "needs-reauth" || tab.failureKind === "needs-reauth") {
      return t("tray.recovery.needsReauth");
    }
    if (tab.failureKind === "access-denied") return t("tray.recovery.accessDenied");
    if (tab.failureKind === "not-found") return t("tray.recovery.notFound");
    if (tab.failureKind === "unknown") return t("tray.recovery.unknown");
    return t("tray.recovery.unreachable");
  };

  /** 分區切片：逾門檻收進「還有 N 個…」溢出節點（≤門檻原樣返回）。 */
  const withOverflow = (rows: TrayMenuItem[]): TrayMenuItem[] => {
    if (rows.length <= OVERFLOW_LIMIT) return rows;
    return [
      ...rows.slice(0, OVERFLOW_LIMIT),
      {
        kind: "overflow",
        label: t("tray.more").replace("{n}", String(rows.length - OVERFLOW_LIMIT)),
        items: rows.slice(OVERFLOW_LIMIT),
      },
    ];
  };

  // 專案區（識別＝locator key；顯示文字與行為不變）
  for (const tab of snapshot.tabs) {
    if (tab.status === "ready" || tab.status === "offline") {
      items.push({
        kind: "project",
        key: tab.key,
        label:
          tab.status === "offline" ? `${tab.name} — ${statusSummary(tab)}` : tab.name,
        checked: tab.key === snapshot.activeKey,
        source: tab.source,
        status: tab.status,
      });
    } else {
      const needsReauth =
        tab.status === "needs-reauth" || tab.failureKind === "needs-reauth";
      const summary = statusSummary(tab);
      const actions: TrayRecoveryAction[] =
        tab.status === "restoring"
          ? []
          : [
              {
                kind: needsReauth ? "reauthenticate" : "retry",
                label: t(
                  needsReauth
                    ? "tray.recovery.reauthenticate"
                    : "tray.recovery.retry",
                ),
              },
              { kind: "open-recovery", label: t("tray.recovery.open") },
              { kind: "open-settings", label: t("tray.recovery.settings") },
            ];
      items.push({
        kind: "recovery",
        key: tab.key,
        connectionId: tab.connectionId,
        label: `${tab.name} — ${summary}`,
        summary,
        active: tab.key === snapshot.activeKey,
        status: tab.status,
        actions,
      });
    }
  }
  if (snapshot.tabs.length > 0) items.push({ kind: "separator" });

  if (!hideWorkspaceData) {
    // 生命週期分區：每個非空階段一個 header＋變更子選單
    let anyChange = false;
    for (const stage of STAGES) {
      const staged = snapshot.changes.filter((c) => changeStage(c) === stage);
      if (staged.length === 0) continue;
      anyChange = true;
      items.push({ kind: "header", label: t(`stage.${stage}`) });
      items.push(
        ...withOverflow(
          staged.map((c): TrayMenuItem => ({
            kind: "change",
            name: c.name,
            label: changeLabel(c),
            actions: [
              { kind: "open-change", label: t("tray.openChange") },
              { kind: "copy-name", label: t("tray.copyName") },
            ],
          })),
        ),
      );
    }
    if (!anyChange) items.push({ kind: "empty", label: t("tray.noChanges") });

    items.push({ kind: "separator" });

    // 討論區分流（spec「討論列表」）。
    const discussionItem = (d: { slug: string; topic: string }): TrayMenuItem => ({
      kind: "discussion",
      slug: d.slug,
      label: d.slug,
      topic: d.topic,
      actions: [
        { kind: "open-discussion", label: t("tray.openDiscussion") },
        { kind: "copy-slug", label: t("tray.copySlug") },
      ],
    });
    const openDiscussions = snapshot.discussions.filter((d) => !d.promoted);
    const promotedDiscussions = snapshot.discussions.filter((d) => d.promoted);
    if (openDiscussions.length > 0) {
      items.push({ kind: "header", label: t("tray.discussionsHeader") });
      items.push(...withOverflow(openDiscussions.map(discussionItem)));
    } else {
      items.push({ kind: "empty", label: t("tray.discussions").replace("{n}", "0") });
    }
    if (promotedDiscussions.length > 0) {
      items.push({ kind: "header", label: t("tray.promotedHeader") });
      items.push(...withOverflow(promotedDiscussions.map(discussionItem)));
    }

    items.push({ kind: "separator" });
  }
  items.push({ kind: "open", label: t("tray.open") });
  items.push({ kind: "settings", label: t("tray.settings") });
  items.push({ kind: "quit", label: t("tray.quit") });

  const inProgress = hideWorkspaceData
    ? 0
    : snapshot.changes.filter((c) => changeStage(c) === "in-progress").length;
  return { items, badge: inProgress > 0 ? String(inProgress) : "" };
}

// ---- 接線層：訂閱 store、去抖重建、掛點擊 handler ----

/** tray 訂閱所需的最小 store 介面（Zustand bound store 相容——AppState 的結構子集）。 */
export interface TrayStoreApi {
  getState: () => {
    tabs: Array<{ locator: WorkspaceLocator; name: string }>;
    activeKey: string | null;
    changes: ChangeItem[];
    discussions: { active: Array<{ slug: string; topic: string; promotedTo: string[] }> };
    sessions: Record<string, WorkspaceSession>;
    remoteRecovery: Record<string, RemoteWorkspaceRecoveryState>;
    connections: ConnectionView[];
    /** 系統匣樣式偏好：native-menu 掛原生選單、panel 卸選單改走點擊事件。 */
    trayStyle: TrayStyle;
    /** 切換既有分頁；locator key 同時涵蓋 local 與 remote。 */
    activateTab: (key: string) => void | Promise<void>;
    retryRemoteWorkspace: (key: string) => void | Promise<void>;
    showRemoteWorkspaceRecovery: (key: string) => void;
    openConnectionReauth: (connectionId: string) => void;
    /** 開資料夾選擇器加入專案（面板 add-project 動作用；取消即無事）。 */
    openProjectViaDialog: () => void | Promise<void>;
    /** 開啟變更詳情抽屜（子選單「開啟此變更」用）。 */
    openDetail: (name: string) => void;
    /** 開啟討論抽屜（討論項點擊用）。 */
    openDiscussion: (slug: string) => void;
    /** 切換主頁面（動作區「設定」用；store 既有 BoardView 動作的結構子集）。 */
    setBoardView: (view: "settings") => void;
  };
  subscribe: (listener: () => void) => () => void;
}

export interface TrayDeps {
  /** macOS 才套用文字徽章與 template 圖示（可注入以利測試；預設由 navigator 偵測）。 */
  isMacOS?: boolean;
  /** 選單重建去抖延遲（ms）；預設與看板刷新同量級（400ms）。 */
  debounceMs?: number;
  /** 面板樣式下點擊圖示（不分左右鍵）的開閉回呼（面板實作由呼叫端注入；未注入即無動作）。 */
  onPanelToggle?: () => void;
}

export interface TrayController {
  /** 卸載時呼叫：取消訂閱、清去抖 timer、關閉 tray 圖示。 */
  dispose: () => void;
}

/** 平台偵測：navigator.userAgent 判 macOS——無 os plugin，前端自足且可測。 */
export function detectMacOS(): boolean {
  if (typeof navigator === "undefined") return false;
  const ua = navigator.userAgent || (navigator as { platform?: string }).platform || "";
  return /Macintosh|Mac OS/i.test(ua);
}

/** store 狀態 → 模型快照（與看板同源）。 */
export function buildTraySnapshot(state: ReturnType<TrayStoreApi["getState"]>): TraySnapshot {
  return {
    tabs: state.tabs.map((tab): TrayTabSnapshot => {
      const key = locatorKey(tab.locator);
      if (tab.locator.kind === "local") {
        return { key, name: tab.name, source: "local", status: "ready" };
      }
      const connectionId = tab.locator.connectionId;
      const recovery = state.remoteRecovery[key];
      const session = state.sessions[key];
      const connection = state.connections.find((entry) => entry.id === connectionId);
      return {
        key,
        name: tab.name,
        source: "remote",
        status: remoteWorkspaceStatus(session, recovery),
        failureKind:
          recovery?.status === "error"
            ? recovery.failure.kind
            : session?.connectionState?.state === "needs-reauth"
              ? "needs-reauth"
              : undefined,
        connectionId,
        serverLabel: connection?.name ?? connectionId,
        serverOrigin: connection?.origin,
      };
    }),
    activeKey: state.activeKey,
    changes: state.changes,
    discussions: state.discussions.active.map((d) => ({
      slug: d.slug,
      topic: d.topic,
      promoted: d.promotedTo.length > 0,
    })),
  };
}

/**
 * 初始化系統匣原生選單：建圖示與首份選單、訂閱 store（去抖重建選單與 macOS 徽章）。
 * 回傳 controller，其 dispose 於 app 卸載時清理。專案切換走 store 既有 activateTab
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
  /** 寫剪貼簿（Rust 端外掛——主視窗隱藏/無焦點仍成功）；失敗靜默、不彈窗不中斷選單。 */
  const copy = (text: string) => {
    void writeText(text).catch(() => {});
  };
  /** 動作聯集 → 子選單項：open-* 開主視窗執行 store 動作、copy-* 寫剪貼簿。 */
  const actionItems = (
    actions: Array<TrayChangeAction | TrayDiscussionAction>,
    open: () => void,
    copyText: string,
  ): MenuItemOptions[] =>
    actions.map((a) => ({
      text: a.label,
      action:
        a.kind === "open-change" || a.kind === "open-discussion"
          ? () => openIn(open)
          : () => copy(copyText),
    }));
  const toOptions = (item: TrayMenuItem): TrayItemOptions => {
    switch (item.kind) {
      case "project":
        return {
          text: item.label,
          checked: item.checked,
          action: () => {
            void store.getState().activateTab(item.key);
          },
        };
      case "recovery":
        if (item.actions.length === 0) return { text: item.label, enabled: false };
        return {
          text: item.label,
          items: [
            {
              text: appT(item.active ? "tray.recovery.active" : "tray.recovery.inactive"),
              enabled: false,
            },
            { text: item.summary, enabled: false },
            ...item.actions.map((action): MenuItemOptions => ({
              text: action.label,
              action: () => {
                if (action.kind === "retry") {
                  void store.getState().retryRemoteWorkspace(item.key);
                } else if (action.kind === "open-recovery") {
                  openIn(() => store.getState().showRemoteWorkspaceRecovery(item.key));
                } else {
                  openIn(() => store.getState().openConnectionReauth(item.connectionId));
                }
              },
            })),
          ],
        };
      case "header":
      case "empty":
        return { text: item.label, enabled: false };
      case "change":
        return {
          text: item.label,
          items: actionItems(item.actions, () => store.getState().openDetail(item.name), item.name),
        };
      case "discussion":
        return {
          text: item.label,
          items: [
            // topic 首行描述：disabled、不可選取，僅供辨識
            { text: item.topic, enabled: false },
            ...actionItems(
              item.actions,
              () => store.getState().openDiscussion(item.slug),
              item.slug,
            ),
          ],
        };
      case "overflow":
        // 溢出節點 → 原生子選單（內嵌項目遞迴轉換；macOS 選單超高原生捲動）
        return { text: item.label, items: item.items.map(toOptions) };
      case "separator":
        return { item: "Separator" };
      case "open":
        return { text: item.label, action: () => openMainWindow() };
      case "settings":
        // 喚起主視窗＋切設定頁（與「開啟此變更」同一喚起語意）
        return { text: item.label, action: () => openIn(() => store.getState().setBoardView("settings")) };
      case "quit":
        return { item: "Quit", text: item.label };
    }
  };
  const styleOf = () => store.getState().trayStyle;
  const render = async () => {
    const model = buildTrayModel(buildTraySnapshot(store.getState()), appT);
    const menu = await Menu.new({ items: model.items.map(toOptions) });
    return { menu, badge: model.badge };
  };

  const first = await render();
  const nativeAtInit = styleOf() === "native-menu";
  const tray = await TrayIcon.new({
    icon,
    iconAsTemplate: isMac,
    tooltip: "Speclink",
    // 樣式分流：native-menu 掛選單（左鍵開選單）；panel 不掛——點擊交給 action。
    menu: nativeAtInit ? first.menu : undefined,
    showMenuOnLeftClick: nativeAtInit,
    title: isMac ? first.badge || undefined : undefined,
    action: (event) => {
      // 面板樣式限定（原生選單樣式由選單接管點擊）。
      if (styleOf() !== "panel") return;
      // 先餵 tray 事件座標給 positioner——面板顯示前的 TrayBottomCenter 依此定位。
      void handleIconState(event).catch(() => {});
      const e = event as { type?: string; button?: string; buttonState?: string };
      // 點擊不分左右鍵（spec「面板樣式（macOS）」）：主鍵與次要鍵放開時皆開閉面板。
      if (
        e.type === "Click" &&
        (e.button === "Left" || e.button === "Right") &&
        e.buttonState === "Up"
      ) {
        deps.onPanelToggle?.();
      }
    },
  });

  // 面板接線（design D5）：主視窗擁有資料——面板 ready 時補推快照（lazy 建窗的
  // 首開空窗）、去抖訂閱時推送、面板動作事件回流 store 執行（與選單動作同語意）。
  const pushSnapshot = () => {
    void emit("tray-snapshot", buildTraySnapshot(store.getState()));
  };
  const unlistenReady = await listen("tray-panel-ready", pushSnapshot);
  const unlistenAction = await listen<{ kind: string; id?: string }>(
    "tray-panel-action",
    (event) => {
      const { kind, id } = event.payload ?? {};
      if (kind === "open-project" && id) void store.getState().activateTab(id);
      else if (kind === "retry-workspace" && id) void store.getState().retryRemoteWorkspace(id);
      else if (kind === "open-recovery" && id) {
        openIn(() => store.getState().showRemoteWorkspaceRecovery(id));
      } else if ((kind === "open-server-settings" || kind === "reauthenticate") && id) {
        openIn(() => store.getState().openConnectionReauth(id));
      }
      // 快速加入專案（D7）：走 openIn 先喚起主視窗——主視窗位於另一桌面時
      // macOS 隨聚焦切換 Space，資料夾選擇器才於前景可見；取消即無事。
      else if (kind === "add-project") openIn(() => void store.getState().openProjectViaDialog());
      else if (kind === "open-change" && id) openIn(() => store.getState().openDetail(id));
      else if (kind === "open-discussion" && id) openIn(() => store.getState().openDiscussion(id));
      else if (kind === "open-app") void openMainWindow();
      else if (kind === "open-settings") openIn(() => store.getState().setBoardView("settings"));
      // 結束走 Rust 命令（webview 無法自行結束行程）；失敗靜默——tray 無可浮出錯誤的介面。
      else if (kind === "quit") void invoke("quit_app").catch(() => {});
    },
  );

  // 去抖訂閱：資料或樣式變動後依樣式分流——native 重建選單＋徽章；panel 卸選單、
  // 僅更新徽章；兩樣式皆推送快照給面板（面板未開時為無害廣播）。
  let timer: ReturnType<typeof setTimeout> | null = null;
  const unsubscribe = store.subscribe(() => {
    if (timer !== null) clearTimeout(timer);
    timer = setTimeout(() => {
      void (async () => {
        pushSnapshot();
        if (styleOf() === "native-menu") {
          const { menu, badge } = await render();
          await tray.setMenu(menu);
          await tray.setShowMenuOnLeftClick(true);
          if (isMac) await tray.setTitle(badge || null);
        } else {
          // 卸選單＋關左鍵開選單缺一不可：後者不關，左鍵仍被「開選單」路徑
          // 吃掉（選單已空＝點了無事發生），點擊事件到不了 action／面板。
          await tray.setMenu(null);
          await tray.setShowMenuOnLeftClick(false);
          const model = buildTrayModel(buildTraySnapshot(store.getState()), appT);
          if (isMac) await tray.setTitle(model.badge || null);
        }
      })();
    }, debounceMs);
  });

  return {
    dispose: () => {
      if (timer !== null) clearTimeout(timer);
      unsubscribe();
      unlistenReady();
      unlistenAction();
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

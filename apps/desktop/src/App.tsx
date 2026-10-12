import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { CloudOff, MoreHorizontal } from "lucide-react";
import {
  KanbanBoard,
  ArchivedList,
  ArchivedDetailPage,
  SpecList,
  SpecDetailPage,
  sortSpecs,
  ManualPage,
  ChangeDetailPage,
  DiscussionDetailPage,
  DetailActionsSlot,
  DetailActionsSlotProvider,
  RevertBlockedDialog,
  ReviewArchiveDialog,
  Button,
  Checkbox,
  ConfirmDialog,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger,
  I18nProvider,
  SEMANTIC_SURFACE,
  applyThemePreference,
  readThemePreference,
  writeThemePreference,
  type ThemePreference,
  SEMANTIC_TONE,
  Toaster,
  cn,
  useI18n,
  siblingChangesOf,
  discussionChipStage,
  type DocumentTarget,
  type Verb,
} from "@speclink/ui";

import { createAppStore, isSettingsView, openTicketStation, selectAssetPrompt, type AppState } from "./store";
import { locatorKey, type WorkspaceSession } from "./session";
import { initTray, type TrayController } from "./tray";
import { installAppMenu, type AppMenuAction } from "./appMenu";
import { ProjectRail, tabPathHint } from "./components/ProjectRail";
import { ProjectColumn } from "./components/ProjectColumn";
import { SettingsColumn } from "./components/SettingsColumn";
import { ProjectSettingsColumn } from "./components/ProjectSettingsColumn";
import { APP_SETTINGS_CATEGORIES, PROJECT_SETTINGS_CATEGORIES } from "./settingsCatalog";
import { EmptyWorkspace } from "./components/EmptyWorkspace";
import { ProjectActionItems, type ProjectActions } from "./components/ProjectActionsMenu";
import { LeftTitleBar, MainTitleBar, type Crumb } from "./components/TitleBar";
import { DetailBack, DetailNav, isBackShortcut } from "./components/DetailNav";
import { SwipeBack } from "./components/SwipeBack";
import { DocumentActionsMenu } from "./components/DocumentActionsMenu";
import { detectPlatform } from "./platform";
import { WorkspaceChooser } from "./components/WorkspaceChooser";
import { visibleRecents } from "./recents";
import { MigrationDialog } from "./components/MigrationDialog";
import { RemoteConflictDialog } from "./components/RemoteConflictDialog";
import { RemoteWorkspaceRecovery } from "./components/RemoteWorkspaceRecovery";
import { AppSettingsView } from "./views/AppSettingsView";
import { ISSUE_URL, REPO_URL, openExternal } from "./views/settings/AboutPage";
import { AssetNoticeCard } from "./components/AssetNoticeCard";
import { AssetNoticeDialog } from "./components/AssetNoticeDialog";
import { UpdateRailButton } from "./components/UpdateRailButton";
import { appVersion, type UpdaterAdapter } from "./adapter/updater";
import { ReleaseNotesDialog } from "./components/ReleaseNotesDialog";
import { readLastSeenVersion, whatsNewDecision, writeLastSeenVersion } from "./core/whatsNew";
import { RELEASE_NOTES, type ReleaseNotesEntry } from "./release-notes/release-notes";
import type { CliInstallAdapter } from "./adapter/cliInstall";
import { projectDirOf, type FsActionsAdapter } from "./adapter/fsActions";
import type { AppInfo } from "./adapter/appInfo";
import { ProjectSettingsView } from "./views/ProjectSettingsView";
import type { ConnectionsAdapter } from "./adapter/connections";
import type { WorkspaceAdapter } from "./adapter/workspace";
import {
  createMigrationAdapter,
  type MigrationAdapter,
} from "./adapter/migration";
import { APP_MESSAGES } from "./i18n/messages";
import {
  readLocalePreference,
  resolveUiLocale,
  writeLocalePreference,
  type LocalePreference,
} from "./i18n/locale";
import { setAppT } from "./i18n/runtime";
import { BUILTIN_TOOLS } from "./builtinTools";

export interface AppProps {
  /** local session 工廠（root 綁定 dataSource／settings／events；workspace-session 決策 3/6）。 */
  createSession: (root: string, name: string) => WorkspaceSession;
  /** remote session 工廠（remote-data-source 決策 6/7）：handshake 成功回
   * session、失敗上拋；未注入時 remote 開啟入口不啟用。 */
  openRemote?: (
    connectionId: string,
    target: string,
    checkoutRoot?: string,
  ) => Promise<WorkspaceSession>;
  /** workspace 探測面（開專案／init／統計／監看重掛）；未注入時對應 UI 不啟用。 */
  workspace?: WorkspaceAdapter;
  /** server 連線面（desktop-connections）；未注入時伺服器頁籤不啟用。 */
  connections?: ConnectionsAdapter;
  /** 正式 local→remote 遷移命令；預設使用 Tauri adapter，測試可注入。 */
  migration?: MigrationAdapter;
  /** 自動更新面（desktop-app「桌面自動更新」）；未注入時更新入口不啟用。 */
  updater?: UpdaterAdapter;
  /** CLI 佈署面（desktop-app「安裝 CLI 指令到 PATH」）；未注入時 CLI 卡不啟用。 */
  cliInstall?: CliInstallAdapter;
  /** 專案層檔案系統動作面（desktop-app「專案層檔案系統動作」）；未注入時選單動作不動作。 */
  fsActions?: FsActionsAdapter;
  /** 關於頁的版本資訊來源（desktop-about-page design D1）；未注入時版本列顯示「—」。 */
  loadAppInfo?: () => Promise<AppInfo>;
}

const DEFAULT_MIGRATION_ADAPTER = createMigrationAdapter();


/** 關閉作用中專案（非 macOS 的 Ctrl+W 與原生選單「關閉專案」共用）。 */
function closeActiveProject(store: ReturnType<typeof createAppStore>) {
  const { activeKey, closeTab } = store.getState();
  if (activeKey) closeTab(activeKey);
}

/** 四種詳情頁任一開著（spec「詳情頁互斥」：同時至多一個）。 */
function detailPageOpen(
  st: Pick<AppState, "detailChange" | "detailDiscussion" | "detailSpec" | "detailArchived">,
): boolean {
  return st.detailChange !== null || st.detailDiscussion !== null || st.detailSpec !== null || st.detailArchived !== null;
}

/** 處在兩種設定模式之一（專案設定模式要有專案；零分頁時專案設定落在空狀態，不算）。 */
function inSettingsMode(st: Pick<AppState, "boardView" | "tabs">): boolean {
  return st.boardView === "settings" || (st.boardView === "project-settings" && st.tabs.length > 0);
}

/** 有對話框開著（Radix 只在開啟時掛載內容）：返回屬於底下的頁面，不得連帶收掉對話框。 */
function dialogOpen(): boolean {
  return document.querySelector('[role="dialog"], [role="alertdialog"]') !== null;
}

/** 有確認框（封存、退回、刪除、討論封存）待確認：確認框屬於目前詳情頁，返回與切換都不作用。 */
function confirmPending(
  st: Pick<AppState, "pendingArchive" | "pendingRevert" | "pendingDelete" | "pendingArchiveDiscussion">,
): boolean {
  return st.pendingArchive !== null || st.pendingRevert !== null || st.pendingDelete !== null || st.pendingArchiveDiscussion !== null;
}

/** 各清單頁回報的呈現順序（含搜尋與篩選）——詳情頁 ‹ › 序列的來源。 */
type VisibleOrder = Record<"changes" | "discussions" | "specs" | "archivedChanges" | "archivedDiscussions", string[]>;

/** 開著的詳情頁：種類、目前項目、‹ › 序列與在序列內開啟某項的動作（與開啟同一個狀態層動作）。 */
interface OpenDetail {
  kind: "change" | "discussion" | "spec" | "archived";
  current: string;
  sequence: string[];
  open: (key: string) => void;
}

/**
 * 詳情頁種類的唯一分派處（序列、麵包屑、切換、渲染共用）。規格與已封存的序列依宿主取：宿主是
 * 對應清單頁時用它回報的順序；否則（手冊頁、規格頁開已封存）用 store 清單的未過濾順序——規格
 * 依規格頁的預設排序，已封存沿清單序。
 */
function openDetailOf(s: AppState, order: VisibleOrder): OpenDetail | null {
  const archivedHost = s.boardView === "archived";
  const a = s.detailArchived;
  if (s.detailChange) return { kind: "change", current: s.detailChange.name, sequence: order.changes, open: s.openDetail };
  if (s.detailDiscussion) {
    return { kind: "discussion", current: s.detailDiscussion.slug, sequence: order.discussions, open: s.openDiscussion };
  }
  if (s.detailSpec !== null) {
    const sequence = s.boardView === "specs" ? order.specs : sortSpecs(s.specs).map((spec) => spec.id);
    return { kind: "spec", current: s.detailSpec, sequence, open: s.openSpec };
  }
  if (a?.kind === "change") {
    const sequence = archivedHost ? order.archivedChanges : s.archived.map((x) => x.datedName);
    return { kind: "archived", current: a.datedName, sequence, open: (datedName) => s.openArchived({ kind: "change", datedName }) };
  }
  if (a?.kind === "discussion") {
    const sequence = archivedHost ? order.archivedDiscussions : s.discussions.archived.map((d) => d.slug);
    return { kind: "archived", current: a.slug, sequence, open: (slug) => s.openArchived({ kind: "discussion", slug }) };
  }
  return null;
}

const DISABLED_CHOOSER_CONNECTIONS: Pick<
  ConnectionsAdapter,
  "scopes" | "inspectCheckout" | "bindCheckout"
> = {
  scopes: async () => ({ projects: [] }),
  inspectCheckout: async () => {
    throw new Error("此環境未提供 server 連線功能");
  },
  bindCheckout: async () => {
    throw new Error("此環境未提供 server 連線功能");
  },
};

/** 對話框描述文字內嵌粗體名稱：以 {name} 佔位切分渲染。 */
function BoldName({ text, name }: { text: string; name: string }) {
  const [before, after] = text.split("{name}");
  return (
    <>
      {before}
      <b>{name}</b>
      {after}
    </>
  );
}

/** 桌面 app 進入點：解析 UI 語言（偏好優先、null 跟隨系統）並掛 I18nProvider。 */
export function App({
  createSession,
  openRemote,
  workspace,
  connections,
  migration = DEFAULT_MIGRATION_ADAPTER,
  updater,
  cliInstall,
  fsActions,
  loadAppInfo,
}: AppProps) {
  const [localePref, setLocalePrefState] = useState<LocalePreference>(() => readLocalePreference());
  // 切換即時生效並持久化（設定頁的 UI 語言三選接這裡）。
  const setLocalePref = (pref: LocalePreference) => {
    writeLocalePreference(pref);
    setLocalePrefState(pref);
  };
  const uiLocale = resolveUiLocale(
    localePref,
    typeof navigator !== "undefined" ? navigator.language : undefined,
  );
  // 外觀偏好（desktop-settings-mode design D4）：啟動套用在 index.html 的開機腳本；切換時寫入、套用並通知
  // 系統匣面板 webview 重套（非 Tauri 環境靜默）。
  const [themePref, setThemePrefState] = useState<ThemePreference>(() => readThemePreference());
  const setThemePref = (pref: ThemePreference) => {
    writeThemePreference(pref);
    applyThemePreference(pref);
    setThemePrefState(pref);
    void emit("speclink:theme-changed", pref).catch(() => {});
  };
  return (
    <I18nProvider locale={uiLocale} messages={APP_MESSAGES}>
      <AppInner
        createSession={createSession}
        openRemote={openRemote}
        workspace={workspace}
        connections={connections}
        migration={migration}
        updater={updater}
        cliInstall={cliInstall}
        fsActions={fsActions}
        loadAppInfo={loadAppInfo}
        localePref={localePref}
        onLocalePrefChange={setLocalePref}
        themePref={themePref}
        onThemePrefChange={setThemePref}
      />
      <Toaster />
    </I18nProvider>
  );
}

interface AppInnerProps extends AppProps {
  /** UI 語言偏好現值（null＝跟隨系統）；設定頁三選用。 */
  localePref: LocalePreference;
  onLocalePrefChange: (pref: LocalePreference) => void;
  /** 外觀偏好現值（null＝跟隨系統）；設定頁外觀列用。 */
  themePref: ThemePreference;
  onThemePrefChange: (pref: ThemePreference) => void;
}

/** 桌面主畫面：生命週期看板（主視圖）＋已封存獨立頁＋設定頁＋富詳情抽屜。 */
function AppInner({
  createSession,
  openRemote,
  workspace,
  connections,
  migration,
  updater,
  cliInstall,
  fsActions,
  loadAppInfo,
  localePref,
  onLocalePrefChange,
  themePref,
  onThemePrefChange,
}: AppInnerProps) {
  const useStore = useMemo(
    () =>
      createAppStore({
        createSession,
        openRemote,
        workspace,
        connections,
        migration,
        updater,
        cliInstall,
        fsActions,
        loadAppInfo,
      }),
    [createSession, openRemote, workspace, connections, migration, updater, cliInstall, fsActions, loadAppInfo],
  );
  const s = useStore();
  // 活躍 session（workspace-session 決策 6）：詳情／規格／封存抽屜的文件載入
  // 與設定頁一律經它——App 不再持有全域 dataSource。
  const activeSession = s.activeKey ? s.sessions[s.activeKey] : undefined;
  const activeTab = s.activeKey
    ? s.tabs.find((tab) => locatorKey(tab.locator) === s.activeKey)
    : undefined;
  const activeRecovery = s.activeKey ? s.remoteRecovery[s.activeKey] : undefined;
  const activeRecoveryTab =
    !activeSession && activeRecovery && activeTab?.locator.kind === "remote"
      ? activeTab
      : undefined;
  const activeRecoveryConnectionId =
    activeRecoveryTab?.locator.kind === "remote"
      ? activeRecoveryTab.locator.connectionId
      : undefined;
  const activeRecoveryConnection = activeRecoveryConnectionId
    ? s.connections.find((connection) => connection.id === activeRecoveryConnectionId)
    : undefined;
  const dataSource = activeSession?.dataSource;
  // capability 驅動停用（remote-data-source 決策 2）：remote session 依 server
  // 端點覆蓋停用 affordance；本地 session 全真、同一路徑零分岐。
  const caps = activeSession?.capabilities;
  const connectionState = activeSession?.connectionState;
  const remoteArchiveScope =
    activeSession?.locator.kind === "remote" ? activeSession.descriptor.name : null;
  // 封存入口守門（spec「封存入口三選項擴及驗證工單」）：待封存 change 仍有
  // 未結工單的站 → 以該站的三選項對話框取代一般確認；雙工單並存時依序各彈
  // 一次，全部處置完才走一般封存。
  const pendingArchiveStation =
    s.pendingArchive === null
      ? null
      : openTicketStation(s.changes, s.pendingArchive, s.pendingArchiveSettled);
  const stale =
    activeSession?.locator.kind === "remote" &&
    connectionState !== undefined &&
    connectionState.state !== "online";
  const sessionConnectionStates = Object.fromEntries(
    Object.entries(s.sessions).map(([key, session]) => [key, session.connectionState]),
  );
  const servers = connections && {
    connections: s.connections,
    phases: s.connectionPhases,
    onAdd: s.addConnection,
    onLogin: s.loginConnection,
    onCancelLogin: s.cancelLogin,
    onSubmitPat: s.submitPat,
    onLogout: s.logoutConnection,
    onRemove: s.removeConnection,
    onRefresh: s.refreshConnections,
    focusConnectionId: s.reauthConnectionId,
    // 開著的 remote session 回報需重新登入的連線：伺服器頁列為登入過期。
    expiredConnectionIds: Object.values(s.sessions).flatMap((session) =>
      session.connectionState?.state === "needs-reauth" ? [session.connectionState.connectionId] : [],
    ),
    onOpenWorkspace:
      openRemote && ((id: string) => s.openWorkspaceChooser({ initialConnectionId: id })),
    defaultConnectionId: s.defaultConnectionId,
    onDefaultConnectionChange: s.setDefaultConnectionId,
  };
  // 設定頁軟體更新卡的常駐現版號（updater 注入時啟動取一次；非 Tauri 環境靜默）。
  const [currentVersion, setCurrentVersion] = useState<string | null>(null);
  // 更新日誌對話框（desktop-app「更新日誌彈窗」）：純 UI 開關，不進 store。whatsNew＝
  // 版號變更後首次啟動自動彈出（關閉即記下現版號）；browse＝設定頁按鈕（關閉不寫記錄）。
  const [releaseNotes, setReleaseNotes] = useState<
    { mode: "whatsNew"; entries: ReleaseNotesEntry[] } | { mode: "browse"; focusVersion?: string } | null
  >(null);
  useEffect(() => {
    if (updater) {
      appVersion()
        .then((version) => {
          setCurrentVersion(version);
          const decision = whatsNewDecision({
            appVersion: version,
            entries: RELEASE_NOTES,
            lastSeen: readLastSeenVersion(),
            dev: import.meta.env.DEV,
          });
          if (decision.kind === "show") {
            setReleaseNotes({ mode: "whatsNew", entries: decision.entries });
          } else if (decision.kind === "record") {
            writeLastSeenVersion(version);
          }
        })
        .catch(() => {});
    }
  }, [updater]);
  // 變更詳情或討論抽屜開啟或換開另一個時收掉更新日誌（spec「detail 抽屜互斥」）：被
  // 抽屜頂掉不走關閉鈕，whatsNew 不記為已看過、下次啟動照舊彈出。以名稱為鍵，重載
  // 換掉同一個 change 的物件不算換開。
  const openedDrawer = s.detailChange
    ? `change:${s.detailChange.name}`
    : s.detailDiscussion
      ? `discussion:${s.detailDiscussion.slug}`
      : null;
  // 技能檔提示確認框（desktop-notice-relocation design D1）：純 UI 開關，與 releaseNotes 同型；
  // 抽屜開啟時一併收掉（可取消浮層互斥），提示消失（更新成功、保留現狀、切分頁）時也關。
  const [assetDialogOpen, setAssetDialogOpen] = useState(false);
  const assetPromptNow = selectAssetPrompt(s);
  useEffect(() => {
    if (openedDrawer !== null) {
      setReleaseNotes(null);
      setAssetDialogOpen(false);
    }
  }, [openedDrawer]);
  useEffect(() => {
    if (!assetPromptNow) setAssetDialogOpen(false);
  }, [assetPromptNow]);
  // 初始化確認框的工具多選（預設勾 claude）；對話框每次開啟重設。
  const [initTools, setInitTools] = useState<string[]>(["claude"]);
  useEffect(() => {
    if (s.pendingInit) setInitTools(["claude"]);
  }, [s.pendingInit]);
  // 啟用確認框的工具多選（決策 3：與初始化框同型、獨立狀態）。
  const [adoptTools, setAdoptTools] = useState<string[]>(["claude"]);
  useEffect(() => {
    if (s.pendingAdopt) setAdoptTools(["claude"]);
  }, [s.pendingAdopt]);
  const { t } = useI18n();
  // 同步 store 層（非 React）的 t 橋——store 組使用者可見訊息時取當前語言。
  useEffect(() => {
    setAppT(t);
  }, [t]);
  // 寫入面的停用說明（server-reader-write-guard D5）：離線時是連線原因；連線
  // 正常仍停用的寫入只剩角色原因——server 對 reader 的每個寫入都回 403。
  const writeDisabledReason = t(stale ? "remote.writeUnavailable" : "remote.readerRole");

  // 看板拖曳手勢讓路（design D6、TaskList 同款）：手勢中暫緩 workspace-changed
  // 的整批 refresh（避免拖曳中卡片重排打斷手勢），放開後補跑一次。
  const boardDragActive = useRef(false);
  const pendingRefresh = useRef(false);
  // 詳情頁內的確認（封存、刪除、退回）在途時，成功／失敗由 store 決定是否關頁——
  // 這段期間 Esc 不關頁，免得確認框收合的 Esc 尾巴誤清 detailChange。
  const detailConfirmInFlight = useRef(false);
  const handleBoardDragActive = (active: boolean) => {
    boardDragActive.current = active;
    if (!active && pendingRefresh.current) {
      pendingRefresh.current = false;
      void useStore.getState().refresh();
    }
  };

  useEffect(() => {
    // 啟動：有 workspace 即還原分頁列（依持久化 activeKey 切回上次活躍專案、
    // 背景徽章快照）；否則維持既有的整批 refresh（無活躍 session 時為無事）。
    if (workspace) void useStore.getState().restoreTabs();
    else void useStore.getState().refresh();
    if (connections) void useStore.getState().refreshConnections();
    // 啟動背景檢查更新（非阻塞；失敗靜默——desktop-app「檢查失敗靜默」）。
    if (updater) void useStore.getState().checkForUpdates(false);
    // 主視窗回到前景時重檢（desktop-app「回到前景逾一小時即重檢」；design D3）：
    // 只對取得焦點反應，節流由 store 判定；訂閱失敗（非 Tauri）靜默。unlisten 走
    // Promise 鏈取消，卸載早於訂閱完成也不漏。
    const focusUnlisten = updater?.onFocusChanged
      ? updater
          .onFocusChanged((focused) => {
            if (focused) void useStore.getState().recheckOnFocus();
          })
          .catch(() => null)
      : null;
    // CLI 佈署狀態探測（含 AppImage 版本不符的啟動自我修復）。
    if (cliInstall) void useStore.getState().refreshCliInstall();
    return () => {
      void focusUnlisten?.then((unlisten) => unlisten?.());
      // 卸載時取消漏出的搜尋去抖，杜絕在途 timer 於 store 卸載後才開火。
      useStore.getState().disposeSearch();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [useStore, connections, workspace, updater, cliInstall]);

  // 檔案監看的宿主層 wiring（workspace-session 決策 5）：訂閱活躍 session 的
  // 事件來源（workspace-changed 以自身 root 過濾），觸發既有整批 refresh；
  // 切換活躍分頁即換訂閱、卸載時解除。
  useEffect(() => {
    const st = useStore.getState();
    const session = st.activeKey ? st.sessions[st.activeKey] : undefined;
    if (!session) return;
    return session.events.subscribe(
      () => {
        // 監看重掛先於刷新（worktree 增減會改變監看拓撲；目標不變時 Rust 端
        // 沿用原監看）——拖曳手勢只暫緩畫面刷新，不暫緩重掛。
        void useStore.getState().rearmWatch();
        if (boardDragActive.current) {
          pendingRefresh.current = true;
          return;
        }
        void useStore.getState().refresh();
        // 外部 speclink update 也要讓提示自然消失（決策 4：workspace-changed 重查）。
        void useStore.getState().refreshAssetPrompt();
      },
      (event) => {
        const store = useStore.getState();
        store.applyRemoteConnectionState(event);
        // Rust runtime 是連線狀態單一真相；worker 收斂回 online 時以同一事件
        // 驅動全量 Query 重查，使用者不需再等下一筆 SSE 或手動重整。
        if (event.state === "online") void store.refresh();
      },
    );
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [useStore, s.activeKey, s.sessionEpoch]);

  // 指令檔過期探測（desktop-instruction-staleness-prompt 決策 4）：本地專案分頁
  // 成為活躍時查一次；remote 分頁由 store 自行略過。
  useEffect(() => {
    if (workspace) void useStore.getState().refreshAssetPrompt();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [useStore, workspace, s.activeKey, s.sessionEpoch]);

  // 系統匣狀態選單（tray-status-menu）：訂閱同一 store 於選單列／系統匣呈現狀態並可切專案。
  // store 為本元件範圍，故於此接線（非 main.tsx）。建立失敗（如非 Tauri 環境）只靜默降級——
  // app 照常、僅無系統匣（與檔案監看不可用時的降級一致）。卸載時 dispose。
  useEffect(() => {
    let controller: TrayController | null = null;
    let disposed = false;
    // 面板樣式（macOS）：點擊圖示 toggle 面板；建立失敗退回原生選單並於設定頁浮出錯誤。
    const onPanelToggle = () => {
      void invoke("toggle_tray_panel").catch((e) => {
        useStore.getState().panelFallback(String(e));
      });
    };
    initTray(useStore, { onPanelToggle })
      .then((c) => {
        if (disposed) c.dispose();
        else controller = c;
      })
      .catch(() => {
        /* 系統匣不可用：app 照常運作 */
      });
    return () => {
      disposed = true;
      controller?.dispose();
    };
  }, [useStore]);

  const platform = useMemo(() => detectPlatform(), []);
  // 詳情頁序列（design D2；desktop-readonly-detail-pages D4）：各清單頁回報的呈現順序（含搜尋
  // 與篩選），‹ › 與 ⌥↑／⌥↓ 共用。
  const [visibleOrder, setVisibleOrder] = useState<VisibleOrder>({
    changes: [],
    discussions: [],
    specs: [],
    archivedChanges: [],
    archivedDiscussions: [],
  });
  const detail = openDetailOf(s, visibleOrder);
  const pageOpen = detail !== null;
  const detailCurrent = detail?.current ?? null;
  const detailIndex = detail ? detail.sequence.indexOf(detail.current) : -1;
  // 確認（封存、刪除、退回）框開著或在途時，切換與返回都不作用：確認框屬於目前詳情頁，
  // 會被一起收掉；在途確認成功後 store 關掉的是「目前」詳情頁，切過去的新頁會被誤關。
  // 以 getState 讀：window 事件處理器只掛一次。
  const detailActionsBlocked = () => detailConfirmInFlight.current || confirmPending(useStore.getState());
  /** 在序列內切換：越界不動、不循環；經與開啟相同的狀態層動作（互斥與浮層規則同樣適用）。 */
  const stepDetail = (delta: -1 | 1) => {
    const next = detail?.sequence[detailIndex + delta];
    if (!detail || detailIndex < 0 || next === undefined || detailActionsBlocked()) return;
    detail.open(next);
  };
  // 專案設定的未存修改（ProjectSettingsView 回報）：離開前先問要不要捨棄。
  const projectSettingsDirty = useRef(false);
  const onProjectSettingsDirty = useCallback((dirty: boolean) => {
    projectSettingsDirty.current = dirty;
  }, []);
  const [discardPrompt, setDiscardPrompt] = useState(false);
  // 全系統唯一的返回：返回鈕、Esc、⌘[（非 macOS 為 Alt+←）、滑鼠側鍵、觸控板滑動都經此決定
  // 去處——詳情頁開著時關頁回宿主頁（確認框待確認或確認在途時不作用）；設定與專案設定模式
  // 回到進入前的頁面（專案設定有未存修改時先問）；其餘頁面沒有返回（null）。
  const backTarget = (st: AppState): (() => void) | null => {
    if (detailPageOpen(st)) return detailActionsBlocked() ? null : st.closeDetailPage;
    if (!inSettingsMode(st)) return null;
    return st.boardView === "project-settings" && projectSettingsDirty.current
      ? () => setDiscardPrompt(true)
      : st.leaveSettings;
  };
  // 觸發當下才看有沒有對話框開著（渲染時讀到的 DOM 還是上一刻，不能拿來決定滑動開關）。
  const resolveBack = (st: AppState) => (dialogOpen() ? null : backTarget(st));
  // 以呼叫當下的狀態決定去處（事件處理器與手勢結束時的狀態可能已不同於渲染時）。
  const goBack = () => resolveBack(useStore.getState())?.();
  // 標題列的動作插槽（desktop-detail-doc-tree design D3）：詳情頁以 portal 把變更層動作送進來。
  const [actionsSlot, setActionsSlot] = useState<HTMLDivElement | null>(null);
  // 切換項目時頁層捲回頂端：捲動容器不重掛，否則沿用上一項的捲動位置。
  const detailLayerRef = useRef<HTMLDivElement>(null);
  useLayoutEffect(() => {
    if (detailLayerRef.current) detailLayerRef.current.scrollTop = 0;
  }, [detailCurrent]);
  // window keydown 處理器只掛一次：以 ref 讀最新的序列與切換動作。
  const stepDetailRef = useRef(stepDetail);
  stepDetailRef.current = stepDetail;
  const handleNavKey = (e: KeyboardEvent): boolean => {
    const st = useStore.getState();
    if (e.defaultPrevented) return false;
    const target = e.target as HTMLElement | null;
    if (target && (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable)) {
      return false;
    }
    // 返回（design D8）：Esc 與 ⌘[、⌘←（非 macOS 為 Alt+←）等效於返回鈕。
    const escape = e.key === "Escape" && !e.altKey && !e.ctrlKey && !e.metaKey && !e.shiftKey;
    const back = escape || isBackShortcut(e, platform) ? resolveBack(st) : null;
    if (back) {
      e.preventDefault();
      back();
      return true;
    }
    if (detailPageOpen(st) && e.altKey && (e.key === "ArrowUp" || e.key === "ArrowDown")) {
      e.preventDefault();
      stepDetailRef.current(e.key === "ArrowUp" ? -1 : 1);
      return true;
    }
    return false;
  };
  // 鍵盤切換分頁：Ctrl+Tab 循環、Ctrl+1..9 直達（spec「專案分頁列存於 app 本機」）；
  // 專案動作選單標示的 ⌘R／Ctrl+R 重新整理、Ctrl+W 關閉專案（spec「專案層檔案系統動作」）。
  // macOS 的 ⌘W 先被系統預設選單攔走（關閉視窗），由原生選單承接，這裡不接。
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      // 返回快捷鍵（Esc、⌘[／Alt+←）與詳情頁的 ⌥↑／⌥↓ 切上一個／下一個（spec「詳情頁的導覽與返回」）。
      // 浮層已處理（defaultPrevented）或焦點在文字輸入時不作用。
      if (handleNavKey(e)) return;
      const key = e.key.toLowerCase();
      if (key === "r" && (platform === "macos" ? e.metaKey : e.ctrlKey)) {
        e.preventDefault();
        void useStore.getState().refresh();
        return;
      }
      if (!e.ctrlKey) return;
      if (e.key === "Tab") {
        e.preventDefault();
        void useStore.getState().cycleTab();
      } else if (e.key >= "1" && e.key <= "9") {
        e.preventDefault();
        void useStore.getState().gotoTab(Number(e.key));
      } else if (key === "w" && platform !== "macos") {
        e.preventDefault();
        closeActiveProject(useStore);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [useStore, platform]);
  // 滑鼠側鍵「上一頁」（button 3）：WebView 是否送出此事件依平台而異，以真機驗收為準。
  useEffect(() => {
    const onMouseUp = (e: MouseEvent) => {
      const back = e.button === 3 ? resolveBack(useStore.getState()) : null;
      if (!back) return;
      e.preventDefault();
      back();
    };
    window.addEventListener("mouseup", onMouseUp);
    return () => window.removeEventListener("mouseup", onMouseUp);
  }, [useStore]);

  // macOS 原生選單列（desktop-native-menu design D3／D4）：語言或專案有無改變時重裝，cleanup
  // 取消尚未套用的上一次；非 macOS 由 installAppMenu 自行 no-op。選單動作只派發到 store 既有
  // 動作、看板搜尋列的 window 事件、更新日誌對話框，或外部瀏覽器。
  const hasProject = s.activeKey !== null;
  // 不在看板時的「搜尋看板」：先切回看板，等搜尋列掛上（子元件的 effect 先於本元件）再聚焦。
  const pendingSearchFocus = useRef(false);
  useEffect(() => {
    if (s.boardView !== "board" || !pendingSearchFocus.current) return;
    pendingSearchFocus.current = false;
    window.dispatchEvent(new CustomEvent("speclink:focus-search"));
  }, [s.boardView]);
  useEffect(() => {
    const dispatch = (action: AppMenuAction) => {
      const st = useStore.getState();
      switch (action) {
        // 原生選單各入口進設定模式的對應分類（desktop-settings-mode design D1；
        // desktop-about-page design D4）。
        case "about":
          st.setBoardView("settings");
          return st.setSettingsSection("about");
        case "settings":
          st.setBoardView("settings");
          return st.setSettingsSection("general");
        case "installCli":
          st.setBoardView("settings");
          return st.setSettingsSection("cli");
        case "checkUpdates":
          // 手動檢查的進度與結果呈現在更新頁的「最新版本」列。
          st.setBoardView("settings");
          st.setSettingsSection("updates");
          return void st.checkForUpdates(true);
        case "openProject":
          return st.openWorkspaceChooser();
        case "closeProject":
          // 系統匣面板（nonactivating NSPanel）在前景時 ⌘W 也會進到 app 選單：主視窗為焦點才關。
          return void getCurrentWindow()
            .isFocused()
            .then((focused) => focused && closeActiveProject(useStore));
        case "viewBoard":
          return st.setBoardView("board");
        case "viewArchived":
          return st.setBoardView("archived");
        case "viewSpecs":
          return st.setBoardView("specs");
        case "viewManual":
        case "helpManual":
          return st.setBoardView("manual");
        case "viewProjectSettings":
          return st.setBoardView("project-settings");
        case "focusSearch":
          if (st.boardView === "board") {
            return void window.dispatchEvent(new CustomEvent("speclink:focus-search"));
          }
          pendingSearchFocus.current = true;
          return st.setBoardView("board");
        case "refresh":
          return void st.refresh();
        case "nextProject":
          return void st.cycleTab();
        case "bringAllToFront":
          return void getCurrentWindow().setFocus();
        case "releaseNotes":
          return setReleaseNotes({ mode: "browse" });
        case "github":
          return openExternal(REPO_URL);
        case "reportIssue":
          return openExternal(ISSUE_URL);
      }
    };
    return installAppMenu({ isMacOS: platform === "macos", t, hasProject, dispatch });
  }, [useStore, platform, t, hasProject]);

  const onRunVerb = (verb: Verb, change: string) => {
    if (verb === "archive") s.requestArchive(change);
    else void s.runVerb(verb, change);
  };

  const confirmDetailAction = async (confirm: () => Promise<void>) => {
    detailConfirmInFlight.current = true;
    try {
      await confirm();
    } finally {
      // 讓 AlertDialog 的收合 callback 先完成；之後使用者仍可正常手動關閉抽屜。
      queueMicrotask(() => {
        detailConfirmInFlight.current = false;
      });
    }
  };

  // 同源連結資料：來源討論清單（記錄已不在時以 slug 充當 topic，出身討論在前）與
  // 同源刀（與此變更共享至少一份來源討論的 active 變更，可互跳）。
  const fromSlugs = s.detailChange?.fromDiscussions ?? [];
  const allDiscussions = [...s.discussions.active, ...s.discussions.archived];
  const sourceDiscussions = fromSlugs.map((slug) => ({
    slug,
    topic: allDiscussions.find((d) => d.slug === slug)?.topic ?? slug,
  }));
  const siblingChanges = siblingChangesOf(s.changes, fromSlugs, s.detailChange?.name ?? "");
  // 封存變更抽屜的來源討論（design D1 增補）：自封存清單以 datedName 查 fromDiscussions，
  // topic 同上解析、記錄已不在時退回 slug。
  const detailArchived = s.detailArchived;
  const archivedItem =
    detailArchived?.kind === "change" ? s.archived.find((a) => a.datedName === detailArchived.datedName) : undefined;
  // 封存討論的記錄 live 優先、封存後備（與載入同一序）。
  const archivedDiscussion =
    detailArchived?.kind === "discussion" ? allDiscussions.find((d) => d.slug === detailArchived.slug) : undefined;
  const archivedFromSlugs = archivedItem?.fromDiscussions ?? [];
  const archivedSourceDiscussions = archivedFromSlugs.map((slug) => ({
    slug,
    topic: allDiscussions.find((d) => d.slug === slug)?.topic ?? slug,
  }));
  // 封存討論抽屜的衍生列（drawer-provenance-links design D4）：自封存討論清單項的 promotedTo
  // 派生三態——封存清單命中→副標封存日期；活躍清單命中→副標看板階段詞；皆無→不可點「無封存記錄」。
  // 順序沿 promotedTo 原序（轉出順序、出身在前）。
  const archivedPromotedChanges = (archivedDiscussion?.promotedTo ?? []).map(
        (name) => {
          const archivedHit = s.archived.find((a) => a.name === name);
          if (archivedHit) return { slug: name, topic: archivedHit.date };
          if (s.changes.some((c) => c.name === name)) {
            return { slug: name, topic: t(discussionChipStage(name, s.changes, s.archived)) };
          }
          return { slug: name, topic: t("rdrawer.noArchiveRecord"), disabled: true };
        },
      );
  // 點衍生籤：已封存→封存抽屜（底層不切頁）；活躍→詳情抽屜（openDetail 依正典落回看板）。
  const openPromotedChange = (name: string) => {
    const archivedHit = s.archived.find((a) => a.name === name);
    if (archivedHit) s.openArchived({ kind: "change", datedName: archivedHit.datedName });
    else if (s.changes.some((c) => c.name === name)) s.openDetail(name);
  };
  // 封存抽屜出身列的三欄（desktop-archived-parity D4）：一律自封存清單項帶入，
  // 抽屜零新查詢。封存討論無封存日期欄位——該欄自然缺席。
  const archivedProvenance = archivedItem
    ? { createdBy: archivedItem.createdBy, created: archivedItem.created, archivedDate: archivedItem.date }
    : { createdBy: archivedDiscussion?.createdBy, created: archivedDiscussion?.created };

  // 殼（design D2）：標題列文字與麵包屑依模式——設定、有作用中專案、零專案（留白）。
  // projectTab＝標題列要呈現的專案（設定模式時沒有）。
  const projectTab = s.boardView === "settings" ? undefined : activeTab;
  // 設定模式（desktop-settings-mode design D1／D2）：遠端專案沒有 .speclink.yaml 頁，記住的分類
  // 落在那頁時退回專案說明。
  const remoteProject = activeTab?.locator.kind === "remote";
  const projectSettingsSection =
    remoteProject && s.projectSettingsSection === "speclink" ? "context" : s.projectSettingsSection;
  const settingsMode = s.boardView === "settings";
  const projectSettingsMode = inSettingsMode(s) && !settingsMode;
  // 專案設定分類欄的解析失敗警示點：由專案設定頁讀到設定快照後回報。
  const [projectParseErrors, setProjectParseErrors] = useState({ config: false, speclink: false });
  const connectionNames = Object.fromEntries(s.connections.map((c) => [c.id, c.name]));
  const PAGE_LABEL_KEY: Record<Exclude<typeof s.boardView, "settings">, string> = {
    board: "app.navChanges",
    archived: "app.archived",
    specs: "app.navSpecs",
    manual: "app.navManual",
    "project-settings": "app.navProjectSettings",
  };
  // 詳情頁開啟時三段（spec「詳情頁的導覽與返回」）：專案名與頁名點擊關頁回宿主，
  // 項目名等寬不可點。
  const crumbs: Crumb[] =
    s.boardView === "settings"
      ? [
          { label: t("app.settingsTitle"), onClick: () => s.setSettingsSection("general") },
          { label: t(APP_SETTINGS_CATEGORIES.find((c) => c.id === s.settingsSection)!.labelKey) },
        ]
      : projectTab && s.boardView === "project-settings"
        ? [
            { label: projectTab.name, onClick: () => s.setBoardView("board") },
            { label: t("app.navProjectSettings"), onClick: () => s.setProjectSettingsSection("context") },
            { label: t(PROJECT_SETTINGS_CATEGORIES.find((c) => c.id === projectSettingsSection)!.labelKey) },
          ]
      : projectTab && detail
        ? [
            {
              label: projectTab.name,
              onClick: () => {
                s.closeDetailPage();
                s.setBoardView("board");
              },
            },
            { label: t(PAGE_LABEL_KEY[s.boardView]), onClick: s.closeDetailPage },
            // 已封存變更以變更名呈現（序列與複製用含日期前綴的封存目錄名）。
            { label: archivedItem?.name ?? detail.current, mono: true },
          ]
        : projectTab
          ? [
              { label: projectTab.name, onClick: () => s.setBoardView("board") },
              { label: t(PAGE_LABEL_KEY[s.boardView]) },
            ]
          : [];
  // 「開啟 ▾」（design D4）：頁回報文件，位置由 core 解析；本機目錄缺席（remote 無 checkout）時只剩複製內容。
  const hasLocalDir = activeTab ? projectDirOf(activeTab.locator) !== null : false;
  const renderDocumentActions = (target: DocumentTarget | null) => (
    <DocumentActionsMenu
      target={target}
      local={hasLocalDir}
      platform={platform}
      locate={s.locateDocument}
      onOpenDefault={(location) => void s.openDocumentDefault(location)}
      onReveal={(location) => void s.revealDocument(location)}
      onCopyPath={(location) => void s.copyDocumentPath(location)}
      onCopyContent={(text) => void s.copyDocumentContent(text)}
    />
  );
  const projectActions: ProjectActions = {
    reveal: (key) => void s.revealProject(key),
    openTerminal: (key) => void s.openProjectInTerminal(key),
    openEditor: (key) => void s.openProjectInEditor(key),
    copyPath: (key) => void s.copyProjectPath(key),
    // 背景專案沒有載入中的資料可重整：切過去即重新載入。
    refresh: (key) => (key === s.activeKey ? void s.refresh() : void s.activateTab(key)),
    close: s.closeTab,
  };

  // 變更詳情頁（design D6）：props 自原抽屜接線搬移，另接「開啟 ▾」。
  const changeDetailPage = (
    <ChangeDetailPage
      change={s.detailChange}
      refreshGen={s.refreshGen}
      loadDocument={(change, artifact) => dataSource?.getDocument(change, artifact) ?? Promise.resolve(null)}
      loadStationTicket={(change, station) =>
        dataSource?.getStationTicket(change, station) ?? Promise.resolve(null)
      }
      // changeCapabilities/changeMeta 於 remote 亦直達（status payload 映射，
      // remote-read-parity）；capability 為假時仍以空集呈現——同一條停用法則，
      // 不偽造、也不讓詳情頁載入失敗。
      loadCapabilities={(change) =>
        caps && !caps.changeCapabilities
          ? Promise.resolve([])
          : (dataSource?.changeCapabilities(change) ?? Promise.resolve([]))
      }
      loadMeta={(change) =>
        caps && !caps.changeMeta
          ? Promise.resolve(null)
          : (dataSource?.changeMeta(change) ?? Promise.resolve(null))
      }
      onRunVerb={onRunVerb}
      detailVerb={s.detailVerb}
      onClearVerb={s.clearDetailVerb}
      onDelete={s.requestDelete}
      // 認領是 RemoteOnly：後端沒有這個方法（本地）就整段不接線，詳情頁因此
      // 連停用的入口都不長出來。
      onClaim={dataSource?.claim ? s.claimChange : undefined}
      onRevert={s.requestRevert}
      // 排程分頁：同波夥伴自清單派生；前置編輯只在 capability 為真時接線（remote 的
      // reader 為假，分頁唯讀）；資料源提供候選名冊時以它為準（worktree 映射時為副本）。
      changes={s.changes}
      onSetDepends={
        caps && !caps.setDepends
          ? undefined
          : (change, on, remove) => void s.setDepends(change, on, remove)
      }
      loadDependsCandidates={
        dataSource?.dependsCandidates
          ? (change) => dataSource.dependsCandidates!(change)
          : undefined
      }
      unavailable={
        caps && {
          analyze:
            caps.validate && caps.analyze ? undefined : t("remote.analyzeUnavailable"),
          archive: caps.archive ? undefined : writeDisabledReason,
          delete: caps.deleteChange ? undefined : writeDisabledReason,
          tasks: caps.setTaskDone && caps.setAllTasks ? undefined : writeDisabledReason,
          claim: caps.claim ? undefined : writeDisabledReason,
        }
      }
      onToggleTask={
        caps && !caps.setTaskDone
          ? undefined
          : async (change, task, done) => {
              await dataSource?.setTaskDone(change, task, done);
              await s.refresh();
            }
      }
      onMoveTask={
        caps && !caps.moveTask
          ? undefined
          : async (change, from, to, before) => {
              await dataSource?.moveTask(change, from, to, before);
              await s.refresh();
            }
      }
      onSetAllTasks={
        caps && !caps.setAllTasks
          ? undefined
          : async (change, done) => {
              await dataSource?.setAllTasks(change, done);
              await s.refresh();
            }
      }
      sourceDiscussions={sourceDiscussions}
      siblingChanges={siblingChanges}
      onOpenDiscussion={s.openDiscussion}
      onOpenSibling={s.openDetail}
      renderDocumentActions={renderDocumentActions}
    />
  );
  // 討論詳情頁（design D7）。
  const discussionDetailPage = (
    <DiscussionDetailPage
      discussion={s.detailDiscussion}
      refreshGen={s.refreshGen}
      loadDocument={(slug) => dataSource?.getDiscussionDocument(slug) ?? Promise.resolve(null)}
      changes={s.changes}
      archivedChanges={s.archived}
      onOpenChangeCard={s.openDetail}
      onArchiveDiscussion={
        // 已封存的討論(自來源討論 chip 開啟)與 capability 缺口不提供封存;
        // 其餘走與討論卡同一確認流程。
        (caps && !caps.archiveDiscussion) ||
        s.discussions.archived.some((d) => d.slug === s.detailDiscussion?.slug)
          ? undefined
          : s.requestArchiveDiscussion
      }
      renderDocumentActions={renderDocumentActions}
    />
  );
  // 規格詳情頁（desktop-readonly-detail-pages design D2）：宿主為規格頁或手冊頁，皆不切頁。
  const specDetailPage = (
    <SpecDetailPage
      capability={s.detailSpec}
      refreshGen={s.refreshGen}
      loadDocument={(capability) => dataSource?.getSpecDocument(capability) ?? Promise.resolve(null)}
      archivedChanges={s.archived}
      onOpenArchivedChange={(datedName) => s.openArchived({ kind: "change", datedName })}
      renderDocumentActions={renderDocumentActions}
    />
  );
  // 已封存詳情頁（design D3）：props 自原已封存抽屜的接線搬移。
  const archivedDetailPage = (
    <ArchivedDetailPage
      target={s.detailArchived}
      refreshGen={s.refreshGen}
      loadDocument={(datedName, artifact) => dataSource?.getArchivedDocument(datedName, artifact) ?? Promise.resolve(null)}
      loadStationTicket={(datedName, station) =>
        dataSource?.getArchivedStationTicket(datedName, station) ?? Promise.resolve(null)
      }
      loadCapabilities={(datedName) => dataSource?.archivedCapabilities(datedName) ?? Promise.resolve([])}
      loadDiscussionDocument={(slug) => dataSource?.getDiscussionDocument(slug) ?? Promise.resolve(null)}
      sourceDiscussions={archivedSourceDiscussions}
      onOpenDiscussion={(slug) => s.openArchived({ kind: "discussion", slug })}
      promotedChanges={archivedPromotedChanges}
      onOpenPromotedChange={openPromotedChange}
      reviewStatus={archivedItem?.reviewStatus}
      verifyStatus={archivedItem?.verifyStatus}
      discussionKind={archivedDiscussion?.kind}
      discussionTopic={archivedDiscussion?.topic}
      {...archivedProvenance}
      renderDocumentActions={renderDocumentActions}
    />
  );

  return (
    <div className="flex h-screen overflow-hidden">
      {/* 左側（design D2）：圖示列與專案欄共用一條 48px 標題列；零專案時只剩圖示列 */}
      <div
        className={cn(
          "flex shrink-0 flex-col overflow-hidden border-r border-border bg-sidebar",
          settingsMode || s.tabs.length > 0 ? "w-[256px]" : "w-14",
        )}
      >
        {/* 零專案且非設定模式時 56px 放不下文字（macOS 紅綠燈也佔住這段），標題列留白；
            設定模式左端為「← 設定」，專案設定模式左端為「← 專案設定」、右端為等寬專案名 */}
        <LeftTitleBar
          title={projectTab?.name}
          path={projectTab && tabPathHint(projectTab, connectionNames, t)}
          back={
            settingsMode || projectSettingsMode
              ? { label: t(settingsMode ? "app.settingsTitle" : "app.navProjectSettings"), onClick: goBack }
              : undefined
          }
          trailing={projectSettingsMode ? projectTab?.name : undefined}
          menu={
            projectTab && !projectSettingsMode ? (
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button
                    variant="ghost"
                    size="icon"
                    aria-label={t("app.projectActions")}
                    className="h-7 w-7 text-muted-foreground hover:text-foreground"
                  >
                    <MoreHorizontal className="h-4 w-4" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                  <ProjectActionItems
                    menu="dropdown"
                    tab={projectTab}
                    platform={platform}
                    actions={projectActions}
                  />
                </DropdownMenuContent>
              </DropdownMenu>
            ) : undefined
          }
        />
        <div className="flex min-h-0 flex-1">
          <ProjectRail
            tabs={s.tabs}
            activeKey={s.activeKey}
            tabErrors={s.tabErrors}
            recoveryStates={s.remoteRecovery}
            connectionStates={sessionConnectionStates}
            connectionNames={connectionNames}
            pendingKey={s.pendingTabKey}
            assetPrompts={s.assetPrompts}
            platform={platform}
            actions={projectActions}
            onActivate={(key) => void s.activateTab(key)}
            onOpen={() => s.openWorkspaceChooser()}
            onOpenSettings={() => s.setBoardView("settings")}
            settingsActive={s.boardView === "settings"}
            updateButton={
              updater && <UpdateRailButton state={s.updater} onClick={s.openSettingsUpdate} />
            }
          />
          {settingsMode ? (
            <SettingsColumn section={s.settingsSection} onSelect={s.setSettingsSection} />
          ) : projectSettingsMode ? (
            <ProjectSettingsColumn
              section={projectSettingsSection}
              remote={remoteProject}
              configParseError={projectParseErrors.config}
              speclinkParseError={projectParseErrors.speclink}
              onSelect={s.setProjectSettingsSection}
              notice={
                <AssetNoticeCard prompt={assetPromptNow} onOpen={() => setAssetDialogOpen(true)} />
              }
            />
          ) : (
            s.tabs.length > 0 && (
              <ProjectColumn
                boardView={s.boardView}
                archivedCount={s.archived.length}
                onNavigate={s.setBoardView}
                notice={
                  <AssetNoticeCard prompt={assetPromptNow} onOpen={() => setAssetDialogOpen(true)} />
                }
              />
            )
          )}
        </div>
      </div>

      {/* relative＋overflow-hidden：滑動返回指示定位在主區欄、裁掉左緣外那段；設定頁的 main
          會捲動，指示不能放在 main 裡 */}
      <DetailActionsSlotProvider value={actionsSlot}>
      <div className="relative flex min-w-0 flex-1 flex-col overflow-hidden">
        <MainTitleBar
          platform={platform}
          crumbs={crumbs}
          leading={
            pageOpen && s.boardView !== "settings" ? (
              <DetailBack
                page={t(PAGE_LABEL_KEY[s.boardView])}
                keys={platform === "macos" ? "⌘[" : "Alt+←"}
                onBack={goBack}
              />
            ) : undefined
          }
          trailing={
            pageOpen ? (
              <>
                <DetailNav index={detailIndex} total={detail?.sequence.length ?? 0} onStep={stepDetail} />
                <DetailActionsSlot ref={setActionsSlot} />
              </>
            ) : undefined
          }
        />

          {stale && connectionState && (
            <div
              role="status"
              data-testid="remote-stale-banner"
              data-connection-state={connectionState.state}
              className={cn(
                "flex min-h-10 shrink-0 items-center gap-2 border-b px-4 py-2",
                SEMANTIC_SURFACE.warning,
              )}
            >
              <CloudOff className={cn("h-4 w-4 shrink-0", SEMANTIC_TONE.warning)} />
              <div className="min-w-0 text-xs">
                <span className="font-semibold">
                  {connectionState.state === "needs-reauth"
                    ? t("remote.reauthTitle")
                    : t("remote.offlineTitle")}
                </span>
                <span className="ml-2 text-muted-foreground">
                  {connectionState.message ?? t("remote.staleHint")}
                </span>
              </div>
              <span className="ml-auto rounded border border-border bg-background/60 px-1.5 py-0.5 font-mono text-[10px] font-semibold uppercase tracking-[0.16em] text-muted-foreground">
                stale
              </span>
              {connectionState.state === "needs-reauth" && (
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  className="h-7 bg-background/80 text-xs"
                  onClick={() => s.openConnectionReauth(connectionState.connectionId)}
                >
                  {t("remote.reauthAction")}
                </Button>
              )}
            </div>
          )}

          {/* 主內容：看板、規格頁、已封存頁填滿高度（清單於內部容器捲動、換頁控
              制列沉底常駐）；設定頁維持整頁縱向捲動；手冊頁的三欄分隔線要貫穿
              主內容全高，padding 由 ManualPage 各欄自管。flex 直欄讓各視圖根節點
              （h-full min-h-0）填滿主區；技能檔提示住在專案欄，主區不為它扣高度 */}
          <main
            className={cn(
              "relative flex flex-1 flex-col",
              // 設定頁各頁自帶內距（desktop-settings-mode design D5）。
              s.boardView === "manual" || isSettingsView(s.boardView) ? "p-0" : "p-5",
              isSettingsView(s.boardView) ? "overflow-y-auto" : "overflow-hidden",
            )}
          >
            {/* 宿主頁（design D1）：詳情頁開啟期間保持掛載但不可見、不可互動——捲動位置、
                搜尋字串與篩選原樣保留，關頁後即回到開頁前的樣子。 */}
            <div
              data-testid="host-page"
              inert={pageOpen || undefined}
              aria-hidden={pageOpen || undefined}
              className={cn("flex min-h-0 flex-1 flex-col", pageOpen && "invisible")}
            >
            {s.boardView === "settings" ? (
              <AppSettingsView
                section={s.settingsSection}
                platform={platform}
                localePref={localePref}
                onLocalePrefChange={onLocalePrefChange}
                themePref={themePref}
                onThemePrefChange={onThemePrefChange}
                trayPanelError={s.trayPanelError}
                servers={servers}
                updater={
                  updater && {
                    state: s.updater,
                    currentVersion,
                    onCheck: () => void s.checkForUpdates(true),
                    onShowReleaseNotes: (focusVersion) => setReleaseNotes({ mode: "browse", focusVersion }),
                    onAccept: () => void s.acceptUpdate(),
                    onCancel: s.dismissUpdate,
                    onRelaunch: () => void s.relaunchToUpdate(),
                    focus: s.focusUpdater,
                    onFocusHandled: s.clearFocusUpdater,
                    autoCheck: s.updaterAutoCheck,
                    onAutoCheckChange: s.setUpdaterAutoCheck,
                  }
                }
                cliInstall={
                  s.cliInstall
                    ? { view: s.cliInstall, onInstall: () => void s.installCli() }
                    : undefined
                }
                about={{ appInfo: s.appInfo, onLoadAppInfo: s.loadAppInfo, onOpenUpdates: s.openSettingsUpdate }}
              />
            ) : workspace !== undefined && s.tabs.length === 0 ? (
              // 零分頁（首次使用）：專案範圍頁面落入空狀態引導頁（取代空看板）；應用程式設定已於上方先行處理。
              <EmptyWorkspace
                recents={visibleRecents(s.recents, s.tabs)}
                connections={s.connections}
                onRefreshConnections={s.refreshConnections}
                workspace={workspace}
                connectionAdapter={connections ?? DISABLED_CHOOSER_CONNECTIONS}
                onOpenLocal={s.openProjectAt}
                onOpenRemote={s.openRemoteWorkspace}
                onRemoveRecent={s.forgetRecent}
                onOpenChooser={s.openWorkspaceChooser}
              />
            ) : activeRecoveryTab && activeRecovery && activeRecoveryConnectionId ? (
              <RemoteWorkspaceRecovery
                tab={activeRecoveryTab}
                recovery={activeRecovery}
                connection={activeRecoveryConnection}
                onRetry={() => void s.retryRemoteWorkspace(locatorKey(activeRecoveryTab.locator))}
                onOpenSettings={() => s.openConnectionReauth(activeRecoveryConnectionId)}
                onReauthenticate={() => s.openConnectionReauth(activeRecoveryConnectionId)}
                onRemove={() => s.closeTab(locatorKey(activeRecoveryTab.locator))}
              />
            ) : s.boardView === "project-settings" && activeSession !== undefined ? (
              <ProjectSettingsView
                settings={activeSession.settings}
                section={projectSettingsSection}
                onParseErrors={setProjectParseErrors}
                onDirtyChange={onProjectSettingsDirty}
              />
            ) : s.boardView === "manual" ? (
              <ManualPage
                index={s.manual}
                loadPage={(slug) => dataSource?.getManualPage(slug) ?? Promise.resolve(null)}
                onOpenSpec={s.openSpec}
                capabilities={s.specs.map((spec) => spec.id)}
                refreshGen={s.refreshGen}
              />
            ) : s.boardView === "specs" ? (
              <SpecList
                title={t("app.navSpecs")}
                description={t("specs.pageDesc")}
                specs={s.specs}
                onOpen={s.openSpec}
                focus={s.detailSpec}
                pageSize={s.pageSizes.specs}
                onPageSizeChange={(size) => s.setPageSize("specs", size)}
                onVisibleOrderChange={(specs) => setVisibleOrder((v) => ({ ...v, specs }))}
              />
            ) : s.boardView === "board" ? (
              <KanbanBoard
                title={t("app.navChanges")}
                description={t("board.pageDesc")}
                changes={s.changes}
                onOpenChange={s.openDetail}
                onArchive={caps && !caps.archive ? undefined : s.requestArchive}
                onRevert={s.requestRevert}
                discussions={s.discussions}
                archivedChanges={s.archived}
                onOpenDiscussion={s.openDiscussion}
                onArchiveDiscussion={
                  caps && !caps.archiveDiscussion ? undefined : s.requestArchiveDiscussion
                }
                query={s.boardQuery}
                onQuery={s.setBoardQuery}
                fulltextHits={s.searchHits}
                searchUnavailableReason={
                  caps && !caps.searchWorkspace ? t("remote.searchUnavailable") : undefined
                }
                onReorder={
                  caps?.reorderCard
                    ? (kind, id, prevId, nextId) => void s.reorderCard(kind, id, prevId, nextId)
                    : undefined
                }
                reorderUnavailableReason={
                  caps && !caps.reorderCard ? writeDisabledReason : undefined
                }
                planError={s.planError}
                onDragActiveChange={handleBoardDragActive}
                onVisibleOrderChange={(order) => setVisibleOrder((v) => ({ ...v, ...order }))}
                promotedExpanded={s.promotedExpanded}
                onPromotedExpandedChange={s.setPromotedExpanded}
                // 骨架的前提是「首訪的載入正在進行」：探測中（pendingTabKey）或整批
                // 載入中（loadingActive），且尚無真值。只看 loaded 會讓探測失敗或讀取
                // 失敗的 workspace 永遠停在載入中——讀不到不等於還在讀。
                loading={(s.pendingTabKey !== null || s.loadingActive) && !s.loaded}
                // 首訪失敗的終態：讀不到 ≠ 確認是空的，故不落回空態文案。已有舊
                // 快取（loaded）時照舊靜默沿用最後成功快照，不顯示提示。
                loadFailed={!s.loaded && s.loadFailed}
              />
            ) : (
              <ArchivedList
                title={t("app.archived")}
                description={t("archived.pageDesc")}
                archived={s.archived}
                query={s.query}
                onQuery={s.setQuery}
                archivedDiscussions={s.discussions.archived}
                onOpen={s.openArchived}
                changesPageSize={s.pageSizes.archivedChanges}
                onChangesPageSizeChange={(size) => s.setPageSize("archivedChanges", size)}
                discussionsPageSize={s.pageSizes.archivedDiscussions}
                onDiscussionsPageSizeChange={(size) => s.setPageSize("archivedDiscussions", size)}
                onVisibleOrderChange={(order) =>
                  setVisibleOrder((v) => ({
                    ...v,
                    archivedChanges: order.changes,
                    archivedDiscussions: order.discussions,
                  }))
                }
              />
            )}
            </div>
            {/* 詳情頁層（design D1／D9）：覆蓋主區，150ms 淡入＋4px 位移；關閉直接移除。 */}
            {detail && (
              <div
                ref={detailLayerRef}
                data-testid="detail-page"
                className="absolute inset-0 overflow-x-hidden overflow-y-auto bg-background animate-in fade-in slide-in-from-bottom-1 duration-150 ease-out motion-reduce:animate-none"
              >
                {
                  {
                    change: changeDetailPage,
                    discussion: discussionDetailPage,
                    spec: specDetailPage,
                    archived: archivedDetailPage,
                  }[detail.kind]
                }
              </div>
            )}
          </main>
          {/* 觸控板滑動返回（design D8）：左緣「←」指示，過門檻放開即返回（去處同返回鈕）。 */}
          <SwipeBack enabled={backTarget(s) !== null} onBack={goBack} />
      </div>
      </DetailActionsSlotProvider>

      {workspace && (
        <WorkspaceChooser
          open={s.workspaceChooser !== null}
          onOpenChange={(isOpen) => !isOpen && s.closeWorkspaceChooser()}
          connections={s.connections}
          connectionAdapter={connections ?? DISABLED_CHOOSER_CONNECTIONS}
          workspace={workspace}
          onOpenLocal={s.openProjectAt}
          onRequestMigration={s.requestMigration}
          onAddServer={s.addConnection}
          phases={s.connectionPhases}
          onCancelLogin={s.cancelLogin}
          onSubmitPat={s.submitPat}
          onRefreshConnections={s.refreshConnections}
          onOpenRemote={s.openRemoteWorkspace}
          recents={visibleRecents(s.recents, s.tabs)}
          onRemoveRecent={s.forgetRecent}
          initialConnectionId={s.workspaceChooser?.initialConnectionId}
          initialServerUrl={s.workspaceChooser?.initialServerUrl}
          initialStep={s.workspaceChooser?.initialStep}
          initialScope={s.workspaceChooser?.initialScope}
          initialCheckoutPath={s.workspaceChooser?.initialCheckoutPath}
          defaultConnectionId={s.defaultConnectionId}
        />
      )}

      {workspace && migration && (
        <MigrationDialog
          open={s.migrationRoot !== null}
          root={s.migrationRoot ?? ""}
          connections={s.connections}
          connectionAdapter={connections ?? DISABLED_CHOOSER_CONNECTIONS}
          migration={migration}
          onOpenChange={(isOpen) => !isOpen && s.cancelMigration()}
          onMigrated={(connectionId, target, checkoutRoot) =>
            s.replaceLocalWorkspaceWithRemote(checkoutRoot, connectionId, target)
          }
        />
      )}

      {/* 技能檔提示確認框（spec「指令檔過期提示」）：三列資訊、依狀態的按鈕；較新態的主
          動作改開設定頁軟體更新卡。 */}
      <AssetNoticeDialog
        open={assetDialogOpen}
        onOpenChange={setAssetDialogOpen}
        prompt={assetPromptNow}
        busy={s.assetUpdating}
        error={s.assetUpdateError}
        onApply={() => void s.applyAssetUpdate()}
        onDismiss={s.dismissAssetPrompt}
        onOpenSettingsUpdate={s.openSettingsUpdate}
      />

      {updater && (
        <ReleaseNotesDialog
          open={releaseNotes !== null}
          mode={releaseNotes?.mode ?? "browse"}
          entries={releaseNotes?.mode === "whatsNew" ? releaseNotes.entries : RELEASE_NOTES}
          focusVersion={releaseNotes?.mode === "browse" ? releaseNotes.focusVersion : undefined}
          onOpenChange={(isOpen) => {
            if (isOpen) return;
            // 先關再寫：寫入失敗不能把 modal 卡在畫面上。
            const wasWhatsNew = releaseNotes?.mode === "whatsNew";
            setReleaseNotes(null);
            if (wasWhatsNew && currentVersion) writeLastSeenVersion(currentVersion);
          }}
        />
      )}

      <RemoteConflictDialog
        conflict={s.pendingRemoteConflict}
        onContinueLocal={s.continueLocalFromConflict}
        onUseServer={s.useServerFromConflict}
        onMigrateLocal={s.migrateLocalFromConflict}
      />

      {/* 初始化確認（寫入型確認框：Radix 預設焦點落在取消） */}
      <ConfirmDialog
        open={discardPrompt}
        onOpenChange={setDiscardPrompt}
        title={t("settings.discardTitle")}
        description={t("settings.discardHint")}
        confirmLabel={t("settings.discard")}
        destructive
        onConfirm={() => {
          setDiscardPrompt(false);
          useStore.getState().leaveSettings();
        }}
      />
      <ConfirmDialog
        open={s.pendingInit !== null}
        onOpenChange={(o) => !o && s.cancelInit()}
        title={t("app.initTitle")}
        description={<BoldName text={t("app.initDesc")} name={s.pendingInit ?? ""} />}
        confirmLabel={t("app.initConfirm")}
        onConfirm={() => s.confirmInit(initTools)}
      >
        <div className="flex gap-4">
          {BUILTIN_TOOLS.map((tool) => (
            <label key={tool} className="flex items-center gap-1.5 text-sm">
              <Checkbox
                checked={initTools.includes(tool)}
                onCheckedChange={(v) =>
                  setInitTools((prev) =>
                    v === true ? [...prev, tool] : prev.filter((x) => x !== tool),
                  )
                }
              />
              {tool}
            </label>
          ))}
        </div>
      </ConfirmDialog>

      {/* 啟用確認（spec「未啟用資料夾經確認後補齊啟用」；與初始化確認框同型、獨立狀態） */}
      <ConfirmDialog
        open={s.pendingAdopt !== null}
        onOpenChange={(o) => !o && s.cancelAdopt()}
        title={t("app.adoptTitle")}
        description={<BoldName text={t("app.adoptDesc")} name={s.pendingAdopt ?? ""} />}
        confirmLabel={t("app.adoptConfirm")}
        onConfirm={() => s.confirmAdopt(adoptTools)}
      >
        <div className="flex gap-4">
          {BUILTIN_TOOLS.map((tool) => (
            <label key={tool} className="flex items-center gap-1.5 text-sm">
              <Checkbox
                checked={adoptTools.includes(tool)}
                onCheckedChange={(v) =>
                  setAdoptTools((prev) =>
                    v === true ? [...prev, tool] : prev.filter((x) => x !== tool),
                  )
                }
              />
              {tool}
            </label>
          ))}
        </div>
      </ConfirmDialog>

      {/* 討論封存確認 */}
      <ConfirmDialog
        open={s.pendingArchiveDiscussion !== null}
        onOpenChange={(o) => !o && s.cancelArchiveDiscussion()}
        title={t("app.archiveDiscussionTitle")}
        description={
          <BoldName text={t("app.archiveDiscussionDesc")} name={s.pendingArchiveDiscussion ?? ""} />
        }
        confirmLabel={t("app.archiveConfirm")}
        confirmDisabled={caps !== undefined && !caps.archiveDiscussion}
        onConfirm={s.confirmArchiveDiscussion}
      />

      {/* 封存入口的未結工單三選項（spec「封存入口三選項擴及驗證工單」）：目標
          change 該站工單未結時取代一般封存確認；未選擇前不執行封存。remote
          清單項不帶 inReview／inVerify，天然走一般確認。 */}
      <ReviewArchiveDialog
        open={pendingArchiveStation !== null}
        change={s.pendingArchive}
        station={pendingArchiveStation ?? "review"}
        onOpenChange={(o) => !o && s.cancelArchive()}
        onGoStamp={() => {
          const name = s.pendingArchive;
          s.cancelArchive();
          if (name) s.openDetail(name);
        }}
        onDiscardReview={() =>
          void s.confirmArchiveDiscardTicket(pendingArchiveStation ?? "review")
        }
        onCarryReview={() => void s.confirmArchiveCarryTicket(pendingArchiveStation ?? "review")}
      />

      {/* 封存確認 */}
      <ConfirmDialog
        open={s.pendingArchive !== null && pendingArchiveStation === null}
        onOpenChange={(o) => !o && s.cancelArchive()}
        title={t("app.archiveTitle")}
        description={
          <>
            <BoldName text={t("app.archiveDesc")} name={s.pendingArchive ?? ""} />
            {remoteArchiveScope && (
              <span className="mt-2 block rounded-md border border-border bg-muted/45 px-2.5 py-2 font-mono text-xs text-foreground">
                {t("app.archiveRemoteScope").replace("{scope}", remoteArchiveScope)}
              </span>
            )}
          </>
        }
        confirmLabel={t("app.archiveConfirm")}
        confirmDisabled={caps !== undefined && !caps.archive}
        onConfirm={() => confirmDetailAction(s.confirmArchive)}
      />

      {/* 退回提案中確認：確認後直接呼叫引擎動詞,UI 不預判守門。 */}
      <ConfirmDialog
        open={s.pendingRevert !== null}
        onOpenChange={(o) => !o && s.cancelRevert()}
        title={t("app.revertTitle")}
        description={<BoldName text={t("app.revertDesc")} name={s.pendingRevert ?? ""} />}
        confirmLabel={t("app.revertConfirm")}
        onConfirm={() => confirmDetailAction(s.confirmRevert)}
      />

      {/* 退回被守門擋下：列引擎證據與出路,無清理或強制退回的機械出路。 */}
      <RevertBlockedDialog info={s.revertBlocked} onClose={s.dismissRevertBlocked} />

      {/* 刪除確認 */}
      <ConfirmDialog
        open={s.pendingDelete !== null}
        onOpenChange={(o) => !o && s.cancelDelete()}
        title={t("app.deleteTitle")}
        description={<BoldName text={t("app.deleteDesc")} name={s.pendingDelete ?? ""} />}
        confirmLabel={t("app.deleteConfirm")}
        destructive
        onConfirm={() => confirmDetailAction(s.confirmDelete)}
      />
    </div>
  );
}

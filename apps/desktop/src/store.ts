import { create, type StoreApi, type UseBoundStore } from "zustand";
import { toast } from "sonner";
import type {
  ArchivedTarget,
  CardKind,
  ChangeItem,
  SpecItem,
  ArchivedItem,
  DiscussionItem,
  DiscussionLists,
  ListView,
  SearchHit,
  SpeclinkDataSource,
  Verb,
  AnalyzeReport,
  VerbDrawerResult,
} from "@speclink/ui";
import { RevertBlockedError, type RevertBlockedInfo } from "@speclink/ui";

import { appT } from "./i18n/runtime";
import type { ConnectionsAdapter, ConnectionView } from "./adapter/connections";
import type { WorkspaceAdapter } from "./adapter/workspace";
import type { MigrationAdapter } from "./adapter/migration";
import {
  applyRemoteConnectionState,
  locatorKey,
  normalizeRemoteOpenFailure,
  type RemoteConnectionStateEvent,
  type RemoteWorkspaceRecoveryState,
  type WorkspaceSession,
} from "./session";
import {
  persistTabs,
  readPersistedTabs,
  removeTab,
  upsertTab,
  type ProjectTab,
} from "./tabs";
import {
  instructionPrompt,
  readInstructionSkips,
  writeInstructionSkip,
  type InstructionPromptState,
} from "./instructionPrompt";
import { detectMacOS, type TrayStyle } from "./tray";
import {
  initialUpdaterState,
  reduceUpdater,
  type UpdaterState,
} from "./core/updater";
import type { PendingUpdate, UpdaterAdapter } from "./adapter/updater";
import {
  cliDeployPlan,
  cliInstallStatus,
  isDirOnPath,
  needsRedeploy,
  parseCliVersion,
  type CliInstallStatus,
  type CliPlatform,
} from "./core/cliInstall";
import type { CliInstallAdapter } from "./adapter/cliInstall";

const FAILURE_TOAST_ID = "desktop-operation-failure";
type FailureMessageKey =
  | "store.deleteFailed"
  | "store.archiveFailed"
  | "store.archiveAfterDiscardFailed"
  | "store.discardReviewFailed"
  | "store.reviewActionUnsupported"
  | "store.revertFailed"
  | "store.discussionArchiveFailed"
  | "store.reorderFailed"
  | "store.openProjectFailed"
  | "store.initFailed"
  | "store.adoptFailed";

/** error 缺席時不輸出尾綴——訊息本身已完整，且錯誤細節不得夾帶工程詞給使用者。 */
function showFailureToast(subject: string, messageKey: FailureMessageKey, error?: unknown): void {
  const detail = error === undefined ? "" : ` ✗ ${String(error)}`;
  toast.error(`${subject} · ${appT(messageKey)}${detail}`, { id: FAILURE_TOAST_ID });
}

/** remote 拖排放開後的暫時可見序：只按 UI 已解析好的鄰居移動，不理解 rank。 */
function moveBetweenNeighbors<T>(
  items: T[],
  itemId: (item: T) => string,
  id: string,
  prevId: string | null,
  nextId: string | null,
): T[] {
  const moving = items.find((item) => itemId(item) === id);
  if (!moving) return items;
  const remaining = items.filter((item) => itemId(item) !== id);
  const nextIndex = nextId === null ? -1 : remaining.findIndex((item) => itemId(item) === nextId);
  if (nextIndex >= 0) {
    return [...remaining.slice(0, nextIndex), moving, ...remaining.slice(nextIndex)];
  }
  const prevIndex = prevId === null ? -1 : remaining.findIndex((item) => itemId(item) === prevId);
  const insertAt = prevIndex >= 0 ? prevIndex + 1 : nextId !== null ? 0 : remaining.length;
  return [...remaining.slice(0, insertAt), moving, ...remaining.slice(insertAt)];
}

/** 主頁面：變更看板（預設）、規格頁、已封存頁、專案設定或應用程式設定。 */
export type BoardView = "board" | "specs" | "archived" | "project-settings" | "settings";

/** 逐連線登入互動狀態（desktop-connections）：patInput＝device flow 明確
 * 不支援（404/405），就地收 PAT（規格 PAT fallback）；awaitingApproval＝device
 * 授權等待中，承載裝置碼、驗證網址與截止時刻（倒數由呈現層以截止時刻計算，
 * 不依賴輪詢節奏）；notice＝非錯誤提示（如 PAT 登出後的帳號頁撤銷提醒）。 */
export type ConnectionPhase =
  | { kind: "idle" }
  | { kind: "busy" }
  | { kind: "awaitingApproval"; userCode: string; verificationUri: string; expiresAt: number }
  | { kind: "patInput"; error: string | null }
  | { kind: "notice"; message: string }
  | { kind: "error"; message: string };

export interface WorkspaceChooserIntent {
  initialConnectionId?: string | null;
  initialServerUrl?: string | null;
  /** 既有 marker 缺工具選集時，預填 scope 讓 chooser 直達 checkout 步驟。 */
  initialScope?: { projectKey: string; repoKey: string } | null;
  /** 既有 marker 缺工具選集時，預填 checkout 資料夾路徑（chooser 直接 inspect）。 */
  initialCheckoutPath?: string | null;
}

/** 設定頁 CLI 卡的呈現視圖（core 判定的彙整結果）。 */
export interface CliInstallView {
  platform: CliPlatform;
  status: CliInstallStatus;
  /** 平台是否支援 app 內佈署動作（macOS／AppImage）。 */
  canDeploy: boolean;
  /** 已佈署但佈署目錄不在 PATH——介面提示加入方式。 */
  pathHint: boolean;
  /** 佈署目錄（提示文案用；不支援佈署的平台為 null）。 */
  deployDir: string | null;
  /** 佈署動作進行中。 */
  busy: boolean;
  /** 佈署失敗的單行錯誤（null＝無）。 */
  error: string | null;
}

export interface RemoteMarkerConflict {
  path: string;
  url: string;
  repo: string | null;
}

export interface AppState {
  changes: ChangeItem[];
  specs: SpecItem[];
  archived: ArchivedItem[];
  /** 討論兩節（active 進看板第 0 欄、archived 進已封存頁討論節）。 */
  discussions: DiscussionLists;
  loaded: boolean;
  /** 刷新世代——每次整批 refresh 完成後遞增；內容元件據此重載已載入的文件。 */
  refreshGen: number;

  boardView: BoardView;
  view: ListView;
  query: string;
  /** 看板搜尋字串——與已封存頁的 query 各自獨立；不持久化、不跨啟動保留（spec）。 */
  boardQuery: string;
  /** 看板全文查詢命中（design D6）：去抖後由 searchWorkspace 回填；空 query 恆空。 */
  searchHits: SearchHit[];
  expandedName: string | null;

  /** 詳情抽屜當前的 change（null=關閉）。 */
  detailChange: ChangeItem | null;
  /** 討論抽屜當前的討論（null=關閉）。 */
  detailDiscussion: DiscussionItem | null;
  /** 規格抽屜當前的 capability id（null=關閉；spec-archive-drawer design D2）。 */
  detailSpec: string | null;
  /** 封存抽屜當前目標（封存變更或封存討論；null=關閉）。 */
  detailArchived: ArchivedTarget | null;

  pendingArchive: string | null;
  pendingDelete: string | null;
  /** 待確認的退回提案中（change 名）。 */
  pendingRevert: string | null;
  /** 退回被守門擋下的證據（null＝無對話框）。 */
  revertBlocked: RevertBlockedInfo | null;
  /** 待確認的討論歸檔（slug）。 */
  pendingArchiveDiscussion: string | null;
  /** 詳情抽屜內呈現的 validate／analyze 結構化結果（keyed by change）；null=無。 */
  drawerVerb: VerbDrawerResult | null;

  refresh: () => Promise<void>;
  /** 監看重掛（事件驅動）：workspace-changed 後重解析監看目標——worktree 增減
   * 會改變監看拓撲；Rust 端目標集合不變時沿用原監看，故可放心每次事件都叫。 */
  rearmWatch: () => Promise<void>;
  setBoardView: (v: BoardView) => void;
  setView: (v: ListView) => void;
  setQuery: (q: string) => void;
  setBoardQuery: (q: string) => void;
  /** 生命週期清理：取消在途搜尋去抖 timer 並作廢在途回填——由擁有此 store 的
   * 元件卸載時呼叫，杜絕去抖在 store 卸載後才開火（否則漏出的 timer 會非同步觸發）。 */
  disposeSearch: () => void;
  toggleExpand: (name: string) => void;
  openDetail: (name: string) => void;
  closeDetail: () => void;
  openDiscussion: (slug: string) => void;
  closeDiscussion: () => void;
  openSpec: (capability: string) => void;
  closeSpec: () => void;
  openArchived: (target: ArchivedTarget) => void;
  closeArchived: () => void;
  requestArchive: (name: string) => void;
  confirmArchive: () => Promise<void>;
  cancelArchive: () => void;
  /** 三選項「放棄審查」：先刪工單再照常封存（spec「封存入口的未結工單三選項」）。 */
  confirmArchiveDiscardReview: () => Promise<void>;
  /** 三選項「照樣帶走」：帶未結工單封存（--carry-review，永久顯示「曾審查未通過」）。 */
  confirmArchiveCarryReview: () => Promise<void>;
  requestDelete: (name: string) => void;
  confirmDelete: () => Promise<void>;
  cancelDelete: () => void;
  requestRevert: (name: string) => void;
  confirmRevert: () => Promise<void>;
  cancelRevert: () => void;
  /** 關閉守門對話框（唯一出路——不提供清理或強制退回）。 */
  dismissRevertBlocked: () => void;
  requestArchiveDiscussion: (slug: string) => void;
  confirmArchiveDiscussion: () => Promise<void>;
  cancelArchiveDiscussion: () => void;
  runVerb: (verb: Verb, change: string) => Promise<void>;
  /** 收合詳情抽屜的分析結果（design D2：再按分析或面板關閉鈕）。 */
  clearDrawerVerb: () => void;
  /** 看板拖排寫回：把卡片排到兩鄰居之間（null＝欄頂／欄底）；失敗以 toast 呈現。 */
  reorderCard: (kind: CardKind, id: string, prevId: string | null, nextId: string | null) => Promise<void>;

  // --- workspace／專案分頁列（注入 workspace adapter 時生效；design D3/D10/D11） ---
  /** 分頁清單（開啟順序；持久化於 app 本機）。分頁身分＝locator（workspace-session 決策 1）。 */
  tabs: ProjectTab[];
  /** session 集（locatorKey 為鍵；workspace-session 決策 6）——資料載入一律經活躍 session。 */
  sessions: Record<string, WorkspaceSession>;
  /** Rust connection 狀態事件套用至同 connection 的所有 remote sessions。 */
  applyRemoteConnectionState: (event: RemoteConnectionStateEvent) => void;
  /** session 事件訂閱世代；重新認證完成後遞增，讓 active worker 解除再重掛。 */
  sessionEpoch: number;
  /** 目前 active 分頁的 locator key（null＝零分頁空狀態）。 */
  activeKey: string | null;
  /** 待確認的初始化目錄（uninitialized 判定觸發；null＝無對話框）。 */
  pendingInit: string | null;
  /** 待確認的啟用專案根（unadopted 判定觸發、錨定探測回報 root；null＝無對話框）。 */
  pendingAdopt: string | null;
  /** 失效分頁錯誤（locator key → 單行訊息）。 */
  tabErrors: Record<string, string>;
  /** 尚無 session 的 remote 分頁復原狀態；執行期限定、不持久化。 */
  remoteRecovery: Record<string, RemoteWorkspaceRecoveryState>;
  /** null＝chooser 關閉；物件承載伺服器頁或 remote marker 的預選意圖。 */
  workspaceChooser: WorkspaceChooserIntent | null;
  openWorkspaceChooser: (intent?: WorkspaceChooserIntent) => void;
  closeWorkspaceChooser: () => void;
  /** local openspec/ 與 remote marker 並存時的強制選擇。 */
  pendingRemoteConflict: RemoteMarkerConflict | null;
  continueLocalFromConflict: () => Promise<void>;
  useServerFromConflict: () => Promise<void>;
  migrateLocalFromConflict: () => Promise<void>;
  cancelRemoteConflict: () => void;
  /** 正式遷移對話框的本機來源 root。 */
  migrationRoot: string | null;
  requestMigration: (root: string) => Promise<void>;
  cancelMigration: () => void;
  /** 開啟專案（dialog 或還原路徑）：三態分流。 */
  openProjectAt: (path: string) => Promise<void>;
  /** 開啟 remote workspace（remote-data-source 決策 6）：handshake 成功才建
   * session 與分頁並切至看板；失敗上拋、由開啟表單就地呈現。 */
  openRemoteWorkspace: (
    connectionId: string,
    target: string,
    checkoutRoot?: string,
  ) => Promise<void>;
  /** 正式遷移完成後，以同 checkoutRoot 的 remote session 原地取代 local 分頁。 */
  replaceLocalWorkspaceWithRemote: (
    root: string,
    connectionId: string,
    target: string,
  ) => Promise<void>;
  /** 開資料夾選擇器再走 openProjectAt；取消即無事。 */
  openProjectViaDialog: () => Promise<void>;
  /** 點分頁（locator key）：remote 無 session 時先選取並進 restoring，失敗留在復原頁。 */
  activateTab: (key: string) => Promise<void>;
  /** 對既有 remote recovery 分頁原地重走 handshake，不改主視窗焦點。 */
  retryRemoteWorkspace: (key: string) => Promise<void>;
  /** 由 Tray 顯式開啟問題詳情：選取既有 recovery destination，但不隱含重試。 */
  showRemoteWorkspaceRecovery: (key: string) => void;
  closeTab: (key: string) => void;
  confirmInit: (tools: string[]) => Promise<void>;
  cancelInit: () => void;
  confirmAdopt: (tools: string[]) => Promise<void>;
  cancelAdopt: () => void;
  /** 啟動：還原持久化分頁、依持久化 activeKey 切回最後活躍專案、背景 local 分頁各探測一次路徑有效性。 */
  restoreTabs: () => Promise<void>;
  /** Ctrl+Tab：循環切至下一分頁（走開啟專案語意）。 */
  cycleTab: () => Promise<void>;
  /** Ctrl+1..9：直達第 N 個分頁（1-based；超界不動作）。 */
  gotoTab: (n: number) => Promise<void>;

  // --- 指令檔過期提示（desktop-instruction-staleness-prompt；決策 4/6/7） ---
  /** 活躍本地分頁的提示現值（null＝不提示：現版、無法判定或已略過同版）。 */
  instructionPrompt: InstructionPromptState | null;
  /** 更新失敗的單行訊息（呈現於提示原位、可重試）；成功或重查即清空。 */
  instructionUpdateError: string | null;
  /** 更新進行中（動作停用、避免重複觸發）。 */
  instructionUpdating: boolean;
  /** 對活躍本地分頁重跑探測並裁決是否提示；remote 分頁與無 adapter 時無動作。 */
  refreshInstructionPrompt: () => Promise<void>;
  /** 主動作（更新／安裝）：經引擎既有再生入口整套再生，成功後重查。 */
  applyInstructionUpdate: () => Promise<void>;
  /** 保留現狀：記下此專案已略過當前產物層版號並收合提示，不寫入專案任何檔案。 */
  dismissInstructionPrompt: () => void;

  // --- 自動更新（desktop-app「桌面自動更新」；design D6） ---
  /** 更新狀態機現值（core/updater reducer 驅動；執行期狀態、不持久化）。 */
  updater: UpdaterState;
  /** 檢查更新（manual＝手動入口）：自動失敗靜默、手動失敗浮出無法檢查。 */
  checkForUpdates: (manual: boolean) => Promise<void>;
  /** 使用者同意：下載並套用；簽章驗證失敗轉錯誤態、既有安裝不受影響。 */
  acceptUpdate: () => Promise<void>;
  /** 使用者稍後：回閒置、不下載。 */
  dismissUpdate: () => void;
  /** 套用完成後重啟為新版。 */
  relaunchToUpdate: () => Promise<void>;

  // --- 安裝 CLI 指令（desktop-app「安裝 CLI 指令到 PATH」；design D5） ---
  /** CLI 佈署狀態視圖（null＝尚未探測或無 adapter；執行期狀態、不持久化）。 */
  cliInstall: CliInstallView | null;
  /** 探測狀態並執行 AppImage 版本不符的啟動自我修復；App 啟動與動作後呼叫。 */
  refreshCliInstall: () => Promise<void>;
  /** 顯式佈署動作（macOS symlink／AppImage 複製；其餘平台為 no-op）。 */
  installCli: () => Promise<void>;

  // --- 系統匣樣式（平台決定；tray 接線層訂閱分流） ---
  /** 系統匣樣式現值：macOS＝panel、其餘＝native-menu；執行期狀態、不持久化。 */
  trayStyle: TrayStyle;
  /** 面板建立失敗的單行錯誤（設定頁本機設定簽以獨立警示行浮出）。 */
  trayPanelError: string | null;
  /** 面板建立失敗的退回（spec：退回原生選單並浮出單行錯誤）。 */
  panelFallback: (message: string) => void;

  // --- server 連線（desktop-connections）：app 全域、不經 session 綁定 ---
  /** saved servers 清單（registry 檢視＋由 Keychain 推導的登入狀態）。 */
  connections: ConnectionView[];
  /** 逐連線互動狀態（keyed by origin；執行期狀態、不持久化）。 */
  connectionPhases: Record<string, ConnectionPhase>;
  /** needs-reauth 導向伺服器簽時要聚焦的 connection id。 */
  reauthConnectionId: string | null;
  openConnectionReauth: (connectionId: string) => void;
  refreshConnections: () => Promise<void>;
  /** 新增（同 origin 即更新顯示名）並隨即進入登入流程（決策 7）；
   * 無效輸入上拋、由表單就地呈現。回傳正規化 origin（決策三——供發起登入的
   * 介面追蹤該連線的互動狀態）；無 adapter 時回 null。 */
  addConnection: (baseUrl: string, name: string) => Promise<string | null>;
  /** device 預設登入的啟動段；明確不支援時轉 patInput、連線錯誤浮為可讀狀態。
   * 等待授權時進 awaitingApproval 並由本 store 排程單次觀測（design 決策二）。 */
  loginConnection: (origin: string) => Promise<void>;
  /** 取消等待授權：停止排程、回未登入；授權請求留給 server 自然逾期。 */
  cancelLogin: (origin: string) => void;
  /** PAT 單次過境提交；無效 PAT 留在輸入面並就地浮錯。 */
  submitPat: (origin: string, pat: string) => Promise<void>;
  logoutConnection: (origin: string) => Promise<void>;
  /** 移除連線（Rust 側先登出再刪條目——決策 6）。 */
  removeConnection: (id: string) => Promise<void>;
}

type WorkspaceSnapshot = Pick<
  AppState,
  "changes" | "specs" | "archived" | "discussions" | "loaded"
>;

type CliProbe = Awaited<ReturnType<CliInstallAdapter["probe"]>>;

/** 探測結果 → 三態判定（版本解析與比對歸 core）。 */
function statusFromProbe(probe: CliProbe): CliInstallStatus {
  const deployed = probe.deployedVersionOutput
    ? parseCliVersion(probe.deployedVersionOutput)
    : null;
  return cliInstallStatus(deployed, probe.appVersion);
}

/** 探測＋判定 → 設定頁 CLI 卡視圖（PATH 提示只對 ~/.local/bin 佈署平台）。 */
function cliViewFrom(probe: CliProbe, status: CliInstallStatus): CliInstallView {
  const deploysToLocalBin = probe.platform === "macos" || probe.platform === "linux-appimage";
  const deployDir = deploysToLocalBin && probe.home ? `${probe.home}/.local/bin` : null;
  return {
    platform: probe.platform,
    status,
    canDeploy: deployDir !== null && probe.bundledCliPath !== null,
    pathHint:
      deployDir !== null &&
      status.kind !== "not-installed" &&
      !isDirOnPath(deployDir, probe.pathEnv, probe.pathDelimiter),
    deployDir,
    busy: false,
    error: null,
  };
}

/** createAppStore 的注入面（workspace-session 決策 6）：session 工廠取代全域
 * dataSource；workspace 為探測面（開專案／init／統計／選資料夾／監看重掛）。 */
export interface AppStoreDeps {
  /** local session 工廠（root、顯示名）；測試注入假 session。 */
  createSession: (root: string, name: string) => WorkspaceSession;
  /** remote session 工廠（remote-data-source 決策 6/7）：以 connectionId 與
   * workspace 識別（project 或 project/repo）走 handshake，成功回 session、
   * 失敗上拋——未注入時 remote 開啟入口不啟用。 */
  openRemote?: (
    connectionId: string,
    target: string,
    checkoutRoot?: string,
  ) => Promise<WorkspaceSession>;
  /** workspace 探測面；未注入時對應 UI 不啟用。 */
  workspace?: WorkspaceAdapter;
  /** server 連線面（desktop-connections）；未注入時伺服器頁籤不啟用。 */
  connections?: ConnectionsAdapter;
  /** local→remote 遷移與「採用 server」本機備份面。 */
  migration?: MigrationAdapter;
  /** 自動更新面（tauri-plugin-updater 委派）；未注入時更新入口不啟用。 */
  updater?: UpdaterAdapter;
  /** CLI 佈署面（探測與計畫執行）；未注入時安裝 CLI 卡不啟用。 */
  cliInstall?: CliInstallAdapter;
}

/**
 * 建立 app 狀態 store（Zustand）。狀態集中此處、留在 apps/desktop；共用元件
 * （packages/ui）不依賴 store，仍經 props 取資料——守住資料源解耦。資料載入
 * 一律經活躍 session 的 dataSource（單活躍載入語意不變）。
 */
export function createAppStore(deps: AppStoreDeps): UseBoundStore<StoreApi<AppState>> {
  const {
    createSession,
    openRemote,
    workspace,
    connections: connectionsAdapter,
    migration,
    updater: updaterAdapter,
    cliInstall: cliInstallAdapter,
  } = deps;
  return create<AppState>((set, get) => {
    // 全文查詢的去抖與 latest-wins 狀態（design D6）——閉包層、不進 store state。
    let searchSeq = 0;
    let searchTimer: ReturnType<typeof setTimeout> | null = null;
    // check 找到的待套用更新（承載 plugin 的下載句柄）——閉包層、不進 store state。
    let pendingUpdate: PendingUpdate | null = null;
    // 最後一次 CLI 佈署探測（installCli 取 home 與 sidecar 路徑）——閉包層。
    let lastCliProbe: Awaited<ReturnType<CliInstallAdapter["probe"]>> | null = null;
    // remote handshake 世代只屬目前 app 執行期；同 locator 僅最新結果可落地。
    const remoteOpenGeneration = new Map<string, number>();
    // 看板最後成功內容只活在本次 app 執行期，依 locator 隔離且不進 localStorage。
    const workspaceSnapshots = new Map<string, WorkspaceSnapshot>();
    const latestRefreshGeneration = new Map<string, number>();
    let nextRefreshGeneration = 0;

    function emptyWorkspaceSnapshot(): WorkspaceSnapshot {
      return {
        changes: [],
        specs: [],
        archived: [],
        discussions: { active: [], archived: [] },
        loaded: false,
      };
    }

    function visibleWorkspaceSnapshot(key: string): WorkspaceSnapshot {
      return workspaceSnapshots.get(key) ?? emptyWorkspaceSnapshot();
    }

    function resetWorkspaceTransientState(): Partial<AppState> {
      if (searchTimer !== null) {
        clearTimeout(searchTimer);
        searchTimer = null;
      }
      searchSeq++;
      return {
        searchHits: [],
        expandedName: null,
        detailChange: null,
        detailDiscussion: null,
        detailSpec: null,
        detailArchived: null,
        pendingArchive: null,
        pendingDelete: null,
        pendingRevert: null,
        revertBlocked: null,
        pendingArchiveDiscussion: null,
        drawerVerb: null,
      };
    }

    /** 活躍分頁的本地專案根；remote 分頁與零分頁回 null（指令檔探測只對本地
     * checkout 有意義——決策 4）。 */
    function activeLocalRoot(): string | null {
      const key = get().activeKey;
      const locator = key ? get().sessions[key]?.locator : undefined;
      return locator?.kind === "local" ? locator.root : null;
    }

    function workspaceActivationState(key: string): Partial<AppState> {
      return {
        ...visibleWorkspaceSnapshot(key),
        ...(get().activeKey === key ? {} : resetWorkspaceTransientState()),
      };
    }

    function beginRefresh(key: string): number {
      const generation = ++nextRefreshGeneration;
      latestRefreshGeneration.set(key, generation);
      return generation;
    }

    function isCurrentRefresh(key: string, generation: number): boolean {
      return latestRefreshGeneration.get(key) === generation;
    }

    function pruneWorkspaceSnapshots(live: Set<string>): void {
      for (const key of workspaceSnapshots.keys()) {
        if (!live.has(key)) workspaceSnapshots.delete(key);
      }
      for (const key of latestRefreshGeneration.keys()) {
        if (!live.has(key)) latestRefreshGeneration.delete(key);
      }
    }

    function bumpRemoteOpenGeneration(key: string): number {
      const generation = (remoteOpenGeneration.get(key) ?? 0) + 1;
      remoteOpenGeneration.set(key, generation);
      return generation;
    }

    function isCurrentRemoteOpen(key: string, generation: number): boolean {
      return remoteOpenGeneration.get(key) === generation;
    }

    /** 活躍 session；零分頁空狀態回 null。 */
    function activeSession(): WorkspaceSession | null {
      const { activeKey, sessions } = get();
      return (activeKey && sessions[activeKey]) || null;
    }

    /** 活躍 session 的 dataSource；零分頁空狀態回 null（資料操作一律早退）。 */
    function activeDataSource(): SpeclinkDataSource | null {
      return activeSession()?.dataSource ?? null;
    }

    /** 封存終局的共同收尾（D1）：成功由畫面表達（關抽屜、不發 toast），失敗以
     * 單槽 toast 呈現 core 的訊息；兩條路都重載。`archive` 動詞與「照樣帶走」
     * 都是封存終局，收尾只此一份。 */
    async function settleArchive(
      change: string,
      run: () => Promise<unknown>,
      failKey: FailureMessageKey = "store.archiveFailed",
    ) {
      try {
        await run();
        set({ detailChange: null });
      } catch (e) {
        showFailureToast(change, failKey, e);
      }
      await get().refresh();
    }

    /** 逐連線互動狀態的單點更新（desktop-connections）。 */
    function setConnectionPhase(origin: string, phase: ConnectionPhase) {
      set({ connectionPhases: { ...get().connectionPhases, [origin]: phase } });
    }

    /** 等待授權中的輪詢排程（keyed by origin；design 決策二——節奏歸前端）：
     * timer＝下一次單次觀測、deviceCode＝觀測把手、intervalMs＝目前間隔
     * （slow_down 即加倍）、deadline＝授權截止時刻（以時刻判逾時，睡眠醒來
     * 仍正確）。取消或終態即抹除該筆——Rust 側無長駐迴圈可中斷。 */
    const pollSchedules = new Map<
      string,
      { timer: ReturnType<typeof setTimeout>; deviceCode: string; intervalMs: number; deadline: number }
    >();

    function stopPolling(origin: string) {
      const schedule = pollSchedules.get(origin);
      if (schedule) {
        clearTimeout(schedule.timer);
        pollSchedules.delete(origin);
      }
    }

    /** 排下一次單次觀測；截止時刻已過即就地收為逾時（不再打 server）。 */
    function scheduleObservation(origin: string, deviceCode: string, intervalMs: number, deadline: number) {
      const timer = setTimeout(() => {
        void observeOnce(origin);
      }, intervalMs);
      pollSchedules.set(origin, { timer, deviceCode, intervalMs, deadline });
    }

    async function observeOnce(origin: string) {
      const schedule = pollSchedules.get(origin);
      if (!schedule || !connectionsAdapter) return;
      if (Date.now() >= schedule.deadline) {
        stopPolling(origin);
        setConnectionPhase(origin, { kind: "error", message: appT("servers.expired") });
        return;
      }
      try {
        const result = await connectionsAdapter.deviceLoginObserve(origin, schedule.deviceCode);
        // 觀測在途時使用者按了取消：結果一律作廢，不得復活等待面或登入。
        if (pollSchedules.get(origin) !== schedule) return;
        if (result.status === "loggedIn") {
          stopPolling(origin);
          await get().refreshConnections();
          await recoverRemoteSessions(origin);
          setConnectionPhase(origin, { kind: "idle" });
        } else if (result.status === "denied") {
          stopPolling(origin);
          setConnectionPhase(origin, { kind: "error", message: appT("servers.denied") });
        } else if (result.status === "expired") {
          stopPolling(origin);
          setConnectionPhase(origin, { kind: "error", message: appT("servers.expired") });
        } else {
          const intervalMs = result.slowDown ? schedule.intervalMs * 2 : schedule.intervalMs;
          scheduleObservation(origin, schedule.deviceCode, intervalMs, schedule.deadline);
        }
      } catch (e) {
        stopPolling(origin);
        setConnectionPhase(origin, { kind: "error", message: String(e) });
      }
    }

    /** 登入成功後原地復活同 connection 的全部已建立 remote sessions：逐一
     * handshake、保留失敗 session／分頁並標 tab error，最後重查與重掛 worker。 */
    async function recoverRemoteSessions(origin: string) {
      if (!openRemote) return;
      const connectionId = get().connections.find((entry) => entry.origin === origin)?.id;
      if (!connectionId) return;
      const targets = Object.entries(get().sessions).filter(
        ([, session]) =>
          session.locator.kind === "remote" && session.locator.connectionId === connectionId,
      );
      const recoveryKeys = get()
        .tabs.filter(
          (tab) =>
            tab.locator.kind === "remote" &&
            tab.locator.connectionId === connectionId &&
            !get().sessions[locatorKey(tab.locator)],
        )
        .map((tab) => locatorKey(tab.locator));
      if (targets.length === 0 && recoveryKeys.length === 0) return;

      const sessions = { ...get().sessions };
      const tabErrors = { ...get().tabErrors };
      for (const [key, session] of targets) {
        const locator = session.locator;
        if (locator.kind !== "remote") continue;
        try {
          const target = `${locator.projectId}/${locator.repoId}`;
          const fresh = locator.checkoutRoot
            ? await openRemote(connectionId, target, locator.checkoutRoot)
            : await openRemote(connectionId, target);
          if (fresh.id !== key) throw new Error("重新連線後 workspace 身分不一致");
          sessions[key] = fresh;
          delete tabErrors[key];
        } catch (error) {
          tabErrors[key] = String(error);
        }
      }
      set({ sessions, tabErrors });
      for (const key of recoveryKeys) await reconnectRemoteTab(key, false);
      await get().refresh();
      set((state) => ({
        sessionEpoch: state.sessionEpoch + 1,
        boardView: "board",
        reauthConnectionId: null,
      }));
    }

    function markerLocation(url: string): { origin: string; project: string } {
      const parsed = new URL(url);
      const marker = "/api/speclink/v1/projects/";
      const index = parsed.pathname.indexOf(marker);
      const project =
        index >= 0 ? parsed.pathname.slice(index + marker.length).split("/")[0] : "";
      if (!project) throw new Error("remote marker 的 url 未包含 project 識別");
      return { origin: parsed.origin.toLowerCase(), project: decodeURIComponent(project) };
    }

    function pathName(path: string): string {
      const trimmed = path.replace(/[\\/]+$/, "");
      return trimmed.split(/[\\/]/).pop() || path;
    }

    /** 命中專案的共同尾聲：upsert session 與分頁（去重）、設 activeKey、清對話框、
     * persist、顯式重掛監看、整批 refresh。probe 為純探測，其回報值即後端真相。 */
    async function enterProject(root: string, name: string) {
      const locator = { kind: "local", root } as const;
      const key = locatorKey(locator);
      const tabs = upsertTab(get().tabs, { locator, name });
      const sessions = { ...get().sessions };
      if (!sessions[key]) sessions[key] = createSession(root, name);
      // 淘汰出分頁列的 session 一併回收（上限丟最舊）。
      const live = new Set(tabs.map((t) => locatorKey(t.locator)));
      for (const k of Object.keys(sessions)) if (!live.has(k)) delete sessions[k];
      pruneWorkspaceSnapshots(live);
      const tabErrors = { ...get().tabErrors };
      delete tabErrors[key];
      set({
        tabs,
        sessions,
        tabErrors,
        activeKey: key,
        pendingInit: null,
        pendingAdopt: null,
        ...workspaceActivationState(key),
      });
      persistTabs(tabs, key);
      // 監看顯式跟隨活躍 session（決策 5）；不可用僅失去自動刷新、app 照常。
      try {
        await workspace?.watchWorkspace(root);
      } catch {
        /* 降級：無自動刷新 */
      }
      await get().refresh();
    }

    /** remote session 的入列尾聲（remote-data-source 決策 6）：upsert 分頁與
     * session、設 activeKey、persist、整批 refresh。無檔案監看——事件面由
     * session 的 remote-workspace-changed 訂閱承擔。 */
    async function adoptRemoteSession(session: WorkspaceSession) {
      const key = session.id;
      const tabs = upsertTab(get().tabs, {
        locator: session.locator,
        name: session.descriptor.name,
      });
      const sessions = { ...get().sessions, [key]: session };
      const live = new Set(tabs.map((t) => locatorKey(t.locator)));
      for (const k of Object.keys(sessions)) if (!live.has(k)) delete sessions[k];
      pruneWorkspaceSnapshots(live);
      const tabErrors = { ...get().tabErrors };
      delete tabErrors[key];
      const remoteRecovery = { ...get().remoteRecovery };
      delete remoteRecovery[key];
      set({
        tabs,
        sessions,
        tabErrors,
        remoteRecovery,
        activeKey: key,
        pendingInit: null,
        pendingAdopt: null,
        ...workspaceActivationState(key),
      });
      persistTabs(tabs, key);
      await get().refresh();
    }

    /** 對已存在的 remote tab 重走 handshake。activate=true 只在使用者選取分頁時
     * 同步更新 activeKey；Tray／retry 使用 false，不得搶走目前作用中分頁。 */
    async function reconnectRemoteTab(key: string, activate: boolean) {
      if (!openRemote) return;
      const tab = get().tabs.find((candidate) => locatorKey(candidate.locator) === key);
      if (!tab || tab.locator.kind !== "remote") return;

      const generation = bumpRemoteOpenGeneration(key);
      set((state) => {
        const tabErrors = { ...state.tabErrors };
        delete tabErrors[key];
        return {
          ...(activate
            ? {
                activeKey: key,
                boardView: "board" as const,
                ...workspaceActivationState(key),
              }
            : {}),
          tabErrors,
          remoteRecovery: {
            ...state.remoteRecovery,
            [key]: { status: "restoring", failure: null },
          },
        };
      });
      if (activate) persistTabs(get().tabs, key);

      const { connectionId, projectId, repoId, checkoutRoot } = tab.locator;
      try {
        const session = checkoutRoot
          ? await openRemote(connectionId, `${projectId}/${repoId}`, checkoutRoot)
          : await openRemote(connectionId, `${projectId}/${repoId}`);
        if (!isCurrentRemoteOpen(key, generation)) return;
        if (!get().tabs.some((candidate) => locatorKey(candidate.locator) === key)) return;
        if (session.id !== key) throw new Error("重新連線後 workspace 身分不一致");

        const tabs = get().tabs.map((candidate) =>
          locatorKey(candidate.locator) === key
            ? {
                ...candidate,
                locator: session.locator,
                name: session.descriptor.name,
              }
            : candidate,
        );
        const tabErrors = { ...get().tabErrors };
        delete tabErrors[key];
        const remoteRecovery = { ...get().remoteRecovery };
        delete remoteRecovery[key];
        set((state) => ({
          tabs,
          sessions: { ...state.sessions, [key]: session },
          tabErrors,
          remoteRecovery,
          // activeKey 先於 session 成立時，必須顯式觸發事件訂閱重新掛載。
          sessionEpoch: state.sessionEpoch + 1,
        }));
        persistTabs(tabs, get().activeKey);
        if (get().activeKey === key) await get().refresh();
      } catch (error) {
        if (!isCurrentRemoteOpen(key, generation)) return;
        if (!get().tabs.some((candidate) => locatorKey(candidate.locator) === key)) return;
        const failure = normalizeRemoteOpenFailure(error);
        set((state) => ({
          tabErrors: { ...state.tabErrors, [key]: failure.message },
          remoteRecovery: {
            ...state.remoteRecovery,
            [key]: { status: "error", failure },
          },
        }));
      }
    }

    return {
    changes: [],
    specs: [],
    archived: [],
    discussions: { active: [], archived: [] },
    loaded: false,
    refreshGen: 0,
    boardView: "board",
    view: "active",
    query: "",
    boardQuery: "",
    searchHits: [],
    expandedName: null,
    detailChange: null,
    detailDiscussion: null,
    detailSpec: null,
    detailArchived: null,
    pendingArchive: null,
    pendingDelete: null,
    pendingRevert: null,
    revertBlocked: null,
    pendingArchiveDiscussion: null,
    drawerVerb: null,

    async refresh() {
      const sourceKey = get().activeKey;
      const session = sourceKey ? get().sessions[sourceKey] : null;
      if (!sourceKey || !session) return;
      const generation = beginRefresh(sourceKey);
      const dataSource = session.dataSource;
      // capability 驅動（remote-data-source 決策 2）：server 未提供的讀取跳過、
      // 以空集呈現（archived 頁另有提示卡），不讓整批 refresh 失敗。
      let loaded: [ChangeItem[], SpecItem[], ArchivedItem[], DiscussionLists];
      try {
        loaded = await Promise.all([
          dataSource.listChanges(),
          dataSource.listSpecs(),
          session.capabilities.listArchived ? dataSource.listArchived() : Promise.resolve([]),
          dataSource.listDiscussions(),
        ]);
      } catch {
        // remote 壞天氣：最後一次成功 snapshot 是唯讀真值；任一 reload 失敗
        // 都不得用空集或部分結果覆蓋。連線呈現只由 Rust 狀態事件決定。
        return;
      }
      const [changes, specs, archived, discussions] = loaded;
      const snapshot: WorkspaceSnapshot = {
        changes,
        specs,
        archived,
        discussions,
        loaded: true,
      };
      if (!isCurrentRefresh(sourceKey, generation)) return;
      workspaceSnapshots.set(sourceKey, snapshot);
      set((state) => ({
        ...(state.activeKey === sourceKey ? snapshot : {}),
      }));
      if (get().activeKey !== sourceKey) return;
      // 詳情開著時同步其資料（如任務數更新）
      const cur = get().detailChange;
      if (cur) {
        set({ detailChange: changes.find((c) => c.name === cur.name) ?? null });
      }
      // 討論抽屜開著時同步（輪數更新、轉出後 promotedTo 增長；封存則關閉）
      const curD = get().detailDiscussion;
      if (curD) {
        set({ detailDiscussion: discussions.active.find((d) => d.slug === curD.slug) ?? null });
      }
      // 清單就緒後遞增刷新世代——開著的內容檢視據此重載至磁碟現況。
      set((st) => ({ refreshGen: st.refreshGen + 1 }));
    },

    async rearmWatch() {
      const st = get();
      const session = st.activeKey ? st.sessions[st.activeKey] : undefined;
      if (session?.locator.kind !== "local") return;
      try {
        await workspace?.watchWorkspace(session.locator.root);
      } catch {
        /* 降級：無自動刷新 */
      }
    },

    setBoardView(boardView) {
      set({ boardView, reauthConnectionId: null });
    },

    setView(view) {
      set({ view });
    },

    setQuery(query) {
      set({ query });
    },

    setBoardQuery(boardQuery) {
      set({ boardQuery });
      // 全文查詢（design D6）：200ms 去抖、latest-wins 序號防交錯；空 query 清
      // 命中並作廢在途；失敗靜默退回欄位比對（spec）——不彈錯、不阻斷輸入。
      if (searchTimer !== null) clearTimeout(searchTimer);
      const seq = ++searchSeq;
      if (!boardQuery.trim()) {
        set({ searchHits: [] });
        return;
      }
      searchTimer = setTimeout(() => {
        const dataSource = activeDataSource();
        if (!dataSource) return;
        void dataSource
          .searchWorkspace(boardQuery)
          .then((hits) => {
            if (seq === searchSeq) set({ searchHits: hits });
          })
          .catch(() => {
            if (seq === searchSeq) set({ searchHits: [] });
          });
      }, 200);
    },

    disposeSearch() {
      if (searchTimer !== null) {
        clearTimeout(searchTimer);
        searchTimer = null;
      }
      // 前進序號作廢任何已開火、在途的查詢回填（latest-wins 語意）。
      searchSeq++;
    },

    toggleExpand(name) {
      set({ expandedName: get().expandedName === name ? null : name });
    },

    // detail 抽屜互斥（drawer-exclusivity）：每個 open* 清除其他三個 detail 欄位，
    // 後開者取代先開者；互斥由此層保證，呼叫端入口不需自行先關再開。
    openDetail(name) {
      const c = get().changes.find((x) => x.name === name);
      // 換 change 清掉上一個 change 的動詞結果（drawerVerb keyed by change）。
      if (c)
        set({
          detailChange: c,
          drawerVerb: null,
          detailDiscussion: null,
          detailSpec: null,
          detailArchived: null,
        });
    },

    closeDetail() {
      set({ detailChange: null, drawerVerb: null });
    },

    openDiscussion(slug) {
      const lists = get().discussions;
      const d =
        lists.active.find((x) => x.slug === slug) ??
        lists.archived.find((x) => x.slug === slug);
      // 取代變更詳情抽屜時 drawerVerb 一併清空（比照 closeDetail）。
      if (d)
        set({
          detailDiscussion: d,
          detailChange: null,
          drawerVerb: null,
          detailSpec: null,
          detailArchived: null,
        });
    },

    closeDiscussion() {
      set({ detailDiscussion: null });
    },

    openSpec(capability) {
      set({
        detailSpec: capability,
        detailChange: null,
        drawerVerb: null,
        detailDiscussion: null,
        detailArchived: null,
      });
    },

    closeSpec() {
      set({ detailSpec: null });
    },

    openArchived(target) {
      set({
        detailArchived: target,
        detailChange: null,
        drawerVerb: null,
        detailDiscussion: null,
        detailSpec: null,
      });
    },

    closeArchived() {
      set({ detailArchived: null });
    },

    requestArchive(name) {
      set({ pendingArchive: name });
    },

    async confirmArchive() {
      const name = get().pendingArchive;
      set({ pendingArchive: null });
      if (name) await get().runVerb("archive", name);
    },

    cancelArchive() {
      set({ pendingArchive: null });
    },

    async confirmArchiveDiscardReview() {
      const name = get().pendingArchive;
      set({ pendingArchive: null });
      if (!name) return;
      const dataSource = activeDataSource();
      const discardReview = dataSource?.discardReview;
      if (!dataSource || !discardReview) {
        showFailureToast(name, "store.reviewActionUnsupported");
        return;
      }
      try {
        await discardReview(name);
      } catch (e) {
        showFailureToast(name, "store.discardReviewFailed", e);
        await get().refresh();
        return;
      }
      // 工單已刪：封存此後失敗要點名審查紀錄已不在，不得只說「封存失敗」。
      await settleArchive(
        name,
        () => dataSource.runVerb("archive", name),
        "store.archiveAfterDiscardFailed",
      );
    },

    async confirmArchiveCarryReview() {
      const name = get().pendingArchive;
      set({ pendingArchive: null });
      if (!name) return;
      const archiveCarryReview = activeDataSource()?.archiveCarryReview;
      if (!archiveCarryReview) {
        showFailureToast(name, "store.reviewActionUnsupported");
        return;
      }
      await settleArchive(name, () => archiveCarryReview(name));
    },

    requestDelete(name) {
      set({ pendingDelete: name });
    },

    async confirmDelete() {
      const name = get().pendingDelete;
      set({ pendingDelete: null });
      if (!name) return;
      const dataSource = activeDataSource();
      if (!dataSource) return;
      try {
        await dataSource.deleteChange(name);
        set({ detailChange: null });
      } catch (e) {
        showFailureToast(name, "store.deleteFailed", e);
      }
      await get().refresh();
    },

    cancelDelete() {
      set({ pendingDelete: null });
    },

    requestRevert(name) {
      set({ pendingRevert: name });
    },

    async confirmRevert() {
      const name = get().pendingRevert;
      set({ pendingRevert: null });
      if (!name) return;
      const dataSource = activeDataSource();
      if (!dataSource) return;
      try {
        // UI 不預判守門——引擎是唯一裁決點;成功後不手動搬卡,重載後
        // startedAt 為空、勾選為 0,卡片依派生自然回提案中欄。
        await dataSource.revertChangeToProposed(name);
      } catch (e) {
        if (e instanceof RevertBlockedError) {
          set({
            revertBlocked: {
              change: name,
              checkedTasks: e.checkedTasks,
              touchedFiles: e.touchedFiles,
            },
          });
        } else {
          showFailureToast(name, "store.revertFailed", e);
        }
      }
      await get().refresh();
    },

    cancelRevert() {
      set({ pendingRevert: null });
    },

    dismissRevertBlocked() {
      set({ revertBlocked: null });
    },

    requestArchiveDiscussion(slug) {
      set({ pendingArchiveDiscussion: slug });
    },

    async confirmArchiveDiscussion() {
      const slug = get().pendingArchiveDiscussion;
      set({ pendingArchiveDiscussion: null });
      if (!slug) return;
      const dataSource = activeDataSource();
      if (!dataSource) return;
      try {
        await dataSource.archiveDiscussion(slug);
        set({ detailDiscussion: null });
      } catch (e) {
        showFailureToast(slug, "store.discussionArchiveFailed", e);
      }
      await get().refresh();
    },

    cancelArchiveDiscussion() {
      set({ pendingArchiveDiscussion: null });
    },

    async runVerb(verb, change) {
      const dataSource = activeDataSource();
      if (!dataSource) return;
      // archive 屬看板全域操作：成功由畫面表達，失敗以單槽 toast 呈現（D1）。
      if (verb === "archive") {
        await settleArchive(change, () => dataSource.runVerb(verb, change));
        return;
      }
      // 「分析」一鍵雙動詞（design D1）：validate＋analyze 併發執行、合併為
      // 單一結構化抽屜結果，不佔頂列。任一失敗即整批以 error 呈現、不靜默。
      try {
        const [v, a] = await Promise.all([
          dataSource.runVerb("validate", change),
          dataSource.runVerb("analyze", change),
        ]);
        const o = (v ?? {}) as { valid?: boolean; errors?: string[] };
        set({
          drawerVerb: {
            change,
            validate: { valid: !!o.valid, errors: o.errors ?? [] },
            analyze: a as AnalyzeReport,
          },
        });
      } catch (e) {
        set({ drawerVerb: { change, error: String(e) } });
      }
      await get().refresh();
    },

    clearDrawerVerb() {
      set({ drawerVerb: null });
    },

    async reorderCard(kind, id, prevId, nextId) {
      const dataSource = activeDataSource();
      if (!dataSource) return;
      const sourceKey = get().activeKey;
      const session = activeSession();
      let previousSnapshot: WorkspaceSnapshot | null = null;
      if (sourceKey && session?.locator.kind === "remote") {
        const current = get();
        previousSnapshot = workspaceSnapshots.get(sourceKey) ?? {
          changes: current.changes,
          specs: current.specs,
          archived: current.archived,
          discussions: current.discussions,
          loaded: current.loaded,
        };
        const optimistic: WorkspaceSnapshot =
          kind === "change"
            ? {
                ...previousSnapshot,
                changes: moveBetweenNeighbors(
                  previousSnapshot.changes,
                  (change) => change.name,
                  id,
                  prevId,
                  nextId,
                ),
              }
            : {
                ...previousSnapshot,
                discussions: {
                  ...previousSnapshot.discussions,
                  active: moveBetweenNeighbors(
                    previousSnapshot.discussions.active,
                    (discussion) => discussion.slug,
                    id,
                    prevId,
                    nextId,
                  ),
                },
              };
        workspaceSnapshots.set(sourceKey, optimistic);
        set((state) => (state.activeKey === sourceKey ? optimistic : {}));
      }
      try {
        await dataSource.reorderCard(kind, id, prevId, nextId);
      } catch (e) {
        // 寫回失敗不留假象（spec）：先復原最後成功 snapshot，再 refresh server 現況。
        if (sourceKey && previousSnapshot) {
          workspaceSnapshots.set(sourceKey, previousSnapshot);
          set((state) => (state.activeKey === sourceKey ? previousSnapshot! : {}));
        }
        showFailureToast(id, "store.reorderFailed", e);
      }
      await get().refresh();
    },

    // --- workspace／專案分頁列 ---
    tabs: [],
    sessions: {},
    sessionEpoch: 0,
    applyRemoteConnectionState(event) {
      let changed = false;
      const sessions = Object.fromEntries(
        Object.entries(get().sessions).map(([key, session]) => {
          const next = applyRemoteConnectionState(session, event);
          if (next !== session) changed = true;
          return [key, next];
        }),
      );
      if (changed) set({ sessions });
    },
    activeKey: null,
    pendingInit: null,
    pendingAdopt: null,
    tabErrors: {},
    remoteRecovery: {},
    workspaceChooser: null,
    pendingRemoteConflict: null,
    migrationRoot: null,

    openWorkspaceChooser(intent = {}) {
      set({ workspaceChooser: intent });
    },

    closeWorkspaceChooser() {
      set({ workspaceChooser: null });
    },

    async requestMigration(root) {
      if (!workspace) throw new Error("此環境未提供本機 workspace 功能");
      const probe = await workspace.openProject(root);
      if (probe.status !== "project") {
        throw new Error(`無法準備本機遷移分頁：${root} 不再是本機 Speclink 專案`);
      }
      // 遷移確認前先建立 local session／分頁。如此 import 失敗時仍有原 local
      // 分頁可留在原位，成功時 replaceLocalWorkspaceWithRemote 才有明確取代目標。
      await enterProject(probe.root, probe.name);
      set({ migrationRoot: probe.root, workspaceChooser: null });
    },

    cancelMigration() {
      set({ migrationRoot: null });
    },

    async continueLocalFromConflict() {
      const conflict = get().pendingRemoteConflict;
      if (!conflict) return;
      set({ pendingRemoteConflict: null });
      await enterProject(conflict.path, pathName(conflict.path));
    },

    async useServerFromConflict() {
      const conflict = get().pendingRemoteConflict;
      if (!conflict) return;
      if (!openRemote) throw new Error("此環境未提供 remote workspace 功能");
      if (!migration) throw new Error("此環境未提供本機 workspace 備份功能");

      const marker = markerLocation(conflict.url);
      let available = get().connections;
      if (connectionsAdapter) {
        available = await connectionsAdapter.list();
        set({ connections: available });
      }
      const connection = available.find(
        (entry) => entry.loggedIn && entry.origin.toLowerCase() === marker.origin,
      );
      if (!connection) throw new Error(`尚未登入 ${marker.origin}，無法採用 server 內容`);
      const target = conflict.repo ? `${marker.project}/${conflict.repo}` : marker.project;

      // 資料夾轉為 checkout 前先收斂既有工具選集（spec「以 server 為準」＝備份後棄用
      // 本機、完成工具 reconciliation 後開啟）。同步在建立分頁之前完成。
      if (connectionsAdapter && conflict.repo) {
        const inspection = await connectionsAdapter.inspectCheckout(
          conflict.path,
          connection.origin,
          marker.project,
          conflict.repo,
        );
        if (inspection.tools.length > 0) {
          await connectionsAdapter.bindCheckout(
            conflict.path,
            connection.origin,
            marker.project,
            conflict.repo,
            inspection.tools,
          );
        }
      }

      // 先完成只讀 handshake，再改名本機 openspec/；任一步失敗都不建立 remote 分頁。
      const session = await openRemote(connection.id, target, conflict.path);
      await migration.adoptRemote(conflict.path);
      await adoptRemoteSession(session);
      set({ pendingRemoteConflict: null, boardView: "board" });
    },

    async migrateLocalFromConflict() {
      const conflict = get().pendingRemoteConflict;
      if (!conflict) return;
      await enterProject(conflict.path, pathName(conflict.path));
      set({ pendingRemoteConflict: null, migrationRoot: conflict.path });
    },

    cancelRemoteConflict() {
      set({ pendingRemoteConflict: null });
    },

    // --- 自動更新（desktop-app「桌面自動更新」；design D6） ---
    updater: initialUpdaterState,
    async checkForUpdates(manual) {
      if (!updaterAdapter) return;
      const before = get().updater;
      const checking = reduceUpdater(before, { type: "checkStarted", manual });
      if (checking === before) return; // 下載中／待重啟不可重檢
      set({ updater: checking });
      try {
        const update = await updaterAdapter.check();
        pendingUpdate = update;
        set({
          updater: reduceUpdater(
            get().updater,
            update ? { type: "updateFound", version: update.version } : { type: "noUpdate" },
          ),
        });
      } catch {
        // 離線／端點不可達：reducer 決定靜默（自動）或浮出（手動）。
        set({ updater: reduceUpdater(get().updater, { type: "checkFailed" }) });
      }
    },
    async acceptUpdate() {
      const pending = pendingUpdate;
      if (!pending) return;
      set({ updater: reduceUpdater(get().updater, { type: "accepted" }) });
      try {
        await pending.downloadAndInstall();
        set({ updater: reduceUpdater(get().updater, { type: "downloaded" }) });
      } catch (error) {
        // 簽章驗證失敗等：轉錯誤態、清掉待套用項；既有安裝不受影響。
        pendingUpdate = null;
        set({
          updater: reduceUpdater(get().updater, {
            type: "installFailed",
            message: error instanceof Error ? error.message : String(error),
          }),
        });
      }
    },
    dismissUpdate() {
      set({ updater: reduceUpdater(get().updater, { type: "dismissed" }) });
    },
    async relaunchToUpdate() {
      await updaterAdapter?.relaunch();
    },

    // --- 安裝 CLI 指令（desktop-app「安裝 CLI 指令到 PATH」；design D5） ---
    cliInstall: null,
    async refreshCliInstall() {
      if (!cliInstallAdapter) return;
      try {
        let probe = await cliInstallAdapter.probe();
        let status = statusFromProbe(probe);
        // AppImage 版本不符的啟動自我修復（spec：自動重新佈署，無需使用者操作）。
        if (needsRedeploy(probe.platform, status) && probe.home && probe.bundledCliPath) {
          const plan = cliDeployPlan(probe.platform, {
            home: probe.home,
            bundledCliPath: probe.bundledCliPath,
          });
          if (plan.action !== "none") {
            await cliInstallAdapter.deploy(plan);
            probe = await cliInstallAdapter.probe();
            status = statusFromProbe(probe);
          }
        }
        lastCliProbe = probe;
        set({ cliInstall: cliViewFrom(probe, status) });
      } catch {
        // 探測失敗（殼層異常）：不顯示卡——探測是最佳努力，不擋其他設定。
      }
    },
    async installCli() {
      const view = get().cliInstall;
      const probe = lastCliProbe;
      if (!cliInstallAdapter || !view?.canDeploy || view.busy) return;
      if (!probe?.home || !probe.bundledCliPath) return;
      set({ cliInstall: { ...view, busy: true, error: null } });
      try {
        const plan = cliDeployPlan(probe.platform, {
          home: probe.home,
          bundledCliPath: probe.bundledCliPath,
        });
        if (plan.action === "none") return; // canDeploy 平台不會走到；供型別收窄
        await cliInstallAdapter.deploy(plan);
        // 重探測呈現已安裝與 PATH 提示（spec：動作後偵測 PATH）。
        await get().refreshCliInstall();
      } catch (error) {
        const current = get().cliInstall;
        if (current) {
          set({
            cliInstall: {
              ...current,
              busy: false,
              error: error instanceof Error ? error.message : String(error),
            },
          });
        }
      }
    },

    // --- 系統匣樣式（平台決定、不持久化） ---
    trayStyle: detectMacOS() ? "panel" : "native-menu",
    trayPanelError: null,
    panelFallback(message) {
      set({ trayStyle: "native-menu", trayPanelError: message });
    },

    // --- server 連線（desktop-connections；決策 5/6/7） ---
    connections: [],
    connectionPhases: {},
    reauthConnectionId: null,
    openConnectionReauth(connectionId) {
      set({ boardView: "settings", reauthConnectionId: connectionId });
    },
    async refreshConnections() {
      if (!connectionsAdapter) return;
      try {
        set({ connections: await connectionsAdapter.list() });
      } catch {
        // 清單讀取失敗保留現值——registry 壞檔在 Rust 側已歸零，走到這裡
        // 多半是暫時性環境問題，不清空使用者眼前的清單。
      }
    },
    async addConnection(baseUrl, name) {
      if (!connectionsAdapter) return null;
      const entry = await connectionsAdapter.add(baseUrl, name);
      // 規格「新增後清單即時反映」：條目先上清單、再進入登入流程。
      await get().refreshConnections();
      await get().loginConnection(entry.origin);
      return entry.origin;
    },
    async loginConnection(origin) {
      if (!connectionsAdapter) return;
      // 重按登入＝重來一次：先停掉上一輪的排程，避免兩份輪詢並行。
      stopPolling(origin);
      setConnectionPhase(origin, { kind: "busy" });
      try {
        const result = await connectionsAdapter.deviceLoginStart(origin);
        if (result.status === "loggedIn") {
          await get().refreshConnections();
          await recoverRemoteSessions(origin);
          setConnectionPhase(origin, { kind: "idle" });
        } else if (result.status === "unsupported") {
          setConnectionPhase(origin, { kind: "patInput", error: null });
        } else {
          const auth = result.authorization;
          const deadline = Date.now() + auth.expiresIn * 1000;
          setConnectionPhase(origin, {
            kind: "awaitingApproval",
            userCode: auth.userCode,
            verificationUri: auth.verificationUri,
            expiresAt: deadline,
          });
          scheduleObservation(origin, auth.deviceCode, Math.max(auth.interval, 1) * 1000, deadline);
        }
      } catch (e) {
        setConnectionPhase(origin, { kind: "error", message: String(e) });
      }
    },
    cancelLogin(origin) {
      // 取消不通知 server——授權請求自然逾期，本機不留等待狀態與 credential。
      stopPolling(origin);
      setConnectionPhase(origin, { kind: "idle" });
    },
    async submitPat(origin, pat) {
      if (!connectionsAdapter) return;
      setConnectionPhase(origin, { kind: "busy" });
      try {
        await connectionsAdapter.patLogin(origin, pat);
        await get().refreshConnections();
        await recoverRemoteSessions(origin);
        setConnectionPhase(origin, { kind: "idle" });
      } catch (e) {
        // 無效 PAT：留在輸入面、就地浮出錯誤。
        setConnectionPhase(origin, { kind: "patInput", error: String(e) });
      }
    },
    async logoutConnection(origin) {
      if (!connectionsAdapter) return;
      setConnectionPhase(origin, { kind: "busy" });
      try {
        const outcome = await connectionsAdapter.logout(origin);
        setConnectionPhase(
          origin,
          outcome.patNotice
            ? { kind: "notice", message: appT("servers.patNotice") }
            : { kind: "idle" },
        );
        await get().refreshConnections();
      } catch (e) {
        setConnectionPhase(origin, { kind: "error", message: String(e) });
      }
    },
    async removeConnection(id) {
      if (!connectionsAdapter) return;
      const origin = get().connections.find((c) => c.id === id)?.origin;
      if (origin) {
        stopPolling(origin); // 條目要消失了，等待中的觀測不得再回填它的狀態
        setConnectionPhase(origin, { kind: "busy" });
      }
      try {
        await connectionsAdapter.remove(id);
        if (origin) {
          const phases = { ...get().connectionPhases };
          delete phases[origin];
          set({ connectionPhases: phases });
        }
        await get().refreshConnections();
      } catch (e) {
        if (origin) setConnectionPhase(origin, { kind: "error", message: String(e) });
      }
    },

    async openProjectAt(path) {
      if (!workspace) return;
      try {
        const probe = await workspace.openProject(path);
        if (probe.status === "project") {
          await enterProject(probe.root, probe.name);
        } else if (probe.status === "uninitialized") {
          set({ pendingInit: probe.dir });
        } else if (probe.status === "unadopted") {
          // 有規格資料但未啟用（spec「未啟用資料夾經確認後補齊啟用」）：
          // 錨定探測回報的專案根開啟用確認框，確認前零寫入。
          set({ pendingAdopt: probe.root });
        } else if (probe.hasLocalOpenspec) {
          set({
            pendingRemoteConflict: {
              path,
              url: probe.url,
              repo: probe.repo ?? null,
            },
          });
        } else {
          const marker = markerLocation(probe.url);
          let available = get().connections;
          if (connectionsAdapter) {
            available = await connectionsAdapter.list();
            set({ connections: available });
          }
          const connection = available.find(
            (entry) => entry.loggedIn && entry.origin.toLowerCase() === marker.origin,
          );
          const target = probe.repo ? `${marker.project}/${probe.repo}` : marker.project;
          if (!connection) {
            set({ workspaceChooser: { initialServerUrl: marker.origin } });
          } else if (connectionsAdapter && probe.repo) {
            // 開啟前先收斂本機 Agent 工具（spec「remote marker 資料夾的探測分流」）：
            // 有既有選集就自動 reconciliation 後才 handshake；缺選集則導回 chooser
            // checkout 步驟由使用者明示選擇，同步成功前不建立 remote session。
            const inspection = await connectionsAdapter.inspectCheckout(
              path,
              connection.origin,
              marker.project,
              probe.repo,
            );
            if (inspection.tools.length === 0) {
              set({
                workspaceChooser: {
                  initialConnectionId: connection.id,
                  initialScope: { projectKey: marker.project, repoKey: probe.repo },
                  initialCheckoutPath: path,
                },
              });
            } else {
              await connectionsAdapter.bindCheckout(
                path,
                connection.origin,
                marker.project,
                probe.repo,
                inspection.tools,
              );
              await get().openRemoteWorkspace(connection.id, target, path);
            }
          } else {
            await get().openRemoteWorkspace(connection.id, target, path);
          }
        }
      } catch (e) {
        // 開啟失敗（spec：顯示帶所選路徑的單行錯誤並維持原專案）。
        showFailureToast(path, "store.openProjectFailed", e);
      }
    },

    async openProjectViaDialog() {
      if (!workspace) return;
      const picked = await workspace.pickFolder();
      if (picked) await get().openProjectAt(picked);
    },

    async openRemoteWorkspace(connectionId, target, checkoutRoot) {
      if (!openRemote) return;
      // handshake fail-closed（決策 6）：失敗原樣上拋、不建分頁不建 session。
      const session = checkoutRoot
        ? await openRemote(connectionId, target, checkoutRoot)
        : await openRemote(connectionId, target);
      await adoptRemoteSession(session);
      // 開啟 workspace 的意圖是看板——自伺服器頁切回看板呈現 server 資料。
      set({ boardView: "board", workspaceChooser: null });
    },

    async replaceLocalWorkspaceWithRemote(root, connectionId, target) {
      if (!openRemote) throw new Error("此環境未提供 remote workspace 功能");
      const localKey = locatorKey({ kind: "local", root });
      if (!get().tabs.some((tab) => locatorKey(tab.locator) === localKey)) {
        throw new Error(`找不到待轉換的本機分頁：${root}`);
      }

      // handshake 成功前不改 store；若開啟 remote 失敗，原 local 分頁與 session 保留。
      const session = await openRemote(connectionId, target, root);
      const remoteKey = session.id;
      const replacement = {
        locator: session.locator,
        name: session.descriptor.name,
      };
      const tabs: ProjectTab[] = [];
      for (const tab of get().tabs) {
        const key = locatorKey(tab.locator);
        if (key === localKey) tabs.push(replacement);
        else if (key !== remoteKey) tabs.push(tab);
      }

      const sessions = { ...get().sessions, [remoteKey]: session };
      delete sessions[localKey];
      const live = new Set(tabs.map((tab) => locatorKey(tab.locator)));
      for (const key of Object.keys(sessions)) if (!live.has(key)) delete sessions[key];
      workspaceSnapshots.delete(localKey);
      pruneWorkspaceSnapshots(live);
      const tabErrors = { ...get().tabErrors };
      delete tabErrors[localKey];
      delete tabErrors[remoteKey];
      set({
        tabs,
        sessions,
        tabErrors,
        activeKey: remoteKey,
        boardView: "board",
        workspaceChooser: null,
        ...workspaceActivationState(remoteKey),
      });
      persistTabs(tabs, remoteKey);
      await get().refresh();
    },

    async activateTab(key) {
      if (!workspace) return;
      const tab = get().tabs.find((t) => locatorKey(t.locator) === key);
      if (!tab) return;
      if (tab.locator.kind === "remote") {
        const existing = get().sessions[key];
        if (existing) {
          bumpRemoteOpenGeneration(key);
          const tabErrors = { ...get().tabErrors };
          delete tabErrors[key];
          const remoteRecovery = { ...get().remoteRecovery };
          delete remoteRecovery[key];
          set({
            activeKey: key,
            tabErrors,
            remoteRecovery,
            ...workspaceActivationState(key),
          });
          persistTabs(get().tabs, key);
          await get().refresh();
        } else {
          await reconnectRemoteTab(key, true);
        }
        return;
      }
      try {
        const probe = await workspace.openProject(tab.locator.root);
        if (probe.status === "project") {
          await enterProject(probe.root, probe.name);
        } else {
          // openspec/ 已刪但目錄還在：分頁轉錯誤態，不開初始化對話框。
          set({ tabErrors: { ...get().tabErrors, [key]: appT("store.tabInvalid") } });
        }
      } catch (e) {
        set({ tabErrors: { ...get().tabErrors, [key]: String(e) } });
      }
    },

    async retryRemoteWorkspace(key) {
      await reconnectRemoteTab(key, false);
    },

    showRemoteWorkspaceRecovery(key) {
      const tab = get().tabs.find((candidate) => locatorKey(candidate.locator) === key);
      if (tab?.locator.kind !== "remote" || !get().remoteRecovery[key]) return;
      set({
        activeKey: key,
        boardView: "board",
        ...workspaceActivationState(key),
      });
      persistTabs(get().tabs, key);
    },

    closeTab(key) {
      bumpRemoteOpenGeneration(key);
      const tabs = removeTab(get().tabs, key);
      const sessions = { ...get().sessions };
      delete sessions[key];
      const tabErrors = { ...get().tabErrors };
      delete tabErrors[key];
      const remoteRecovery = { ...get().remoteRecovery };
      delete remoteRecovery[key];
      const wasActive = get().activeKey === key;
      const activeKey = wasActive ? null : get().activeKey;
      workspaceSnapshots.delete(key);
      latestRefreshGeneration.delete(key);
      set({
        tabs,
        sessions,
        tabErrors,
        remoteRecovery,
        activeKey,
        ...(wasActive
          ? { ...emptyWorkspaceSnapshot(), ...resetWorkspaceTransientState() }
          : {}),
      });
      persistTabs(tabs, activeKey);
      // 關掉 active 分頁：切到最近的剩餘分頁（走完整開啟語意）；零分頁則空狀態。
      if (wasActive) {
        const next = tabs.at(-1);
        if (next) void get().activateTab(locatorKey(next.locator));
      }
    },

    async confirmInit(tools) {
      const dir = get().pendingInit;
      set({ pendingInit: null });
      if (!dir || !workspace) return;
      try {
        const probe = await workspace.initProject(dir, tools);
        if (probe.status === "project") await enterProject(probe.root, probe.name);
      } catch (e) {
        // 初始化失敗：顯示帶所選目錄的單行錯誤、不切換 root（spec）。
        showFailureToast(dir, "store.initFailed", e);
      }
    },

    cancelInit() {
      set({ pendingInit: null });
    },

    async confirmAdopt(tools) {
      const root = get().pendingAdopt;
      set({ pendingAdopt: null });
      if (!root || !workspace) return;
      try {
        const probe = await workspace.adoptProject(root, tools);
        if (probe.status === "project") await enterProject(probe.root, probe.name);
      } catch (e) {
        // 啟用失敗：顯示帶專案根的單行錯誤、不切換 root（spec）。
        showFailureToast(root, "store.adoptFailed", e);
      }
    },

    cancelAdopt() {
      set({ pendingAdopt: null });
    },

    async cycleTab() {
      const { tabs, activeKey } = get();
      if (tabs.length < 2) return;
      const idx = tabs.findIndex((t) => locatorKey(t.locator) === activeKey);
      const next = tabs[(idx + 1) % tabs.length];
      await get().activateTab(locatorKey(next.locator));
    },

    async gotoTab(n) {
      const target = get().tabs[n - 1];
      if (!target || locatorKey(target.locator) === get().activeKey) return;
      await get().activateTab(locatorKey(target.locator));
    },

    instructionPrompt: null,
    instructionUpdateError: null,
    instructionUpdating: false,

    async refreshInstructionPrompt() {
      const root = activeLocalRoot();
      // remote 分頁無本地受管指令檔可查（決策 4）；無 adapter 時 UI 不啟用。
      if (!workspace || !root) {
        set({ instructionPrompt: null, instructionUpdateError: null });
        return;
      }
      let probe;
      try {
        probe = await workspace.probeInstructions(root);
      } catch {
        // 探測不可用等同無法判定：靜默不提示，開專案不受影響（既有降級語意）。
        set({ instructionPrompt: null });
        return;
      }
      // 分頁在探測期間被切走：結果屬於前一個 root，不得落地。
      if (activeLocalRoot() !== root) return;
      set({
        instructionPrompt: instructionPrompt(probe, root, readInstructionSkips()),
        instructionUpdateError: null,
      });
    },

    async applyInstructionUpdate() {
      const root = activeLocalRoot();
      if (!workspace || !root || get().instructionUpdating) return;
      set({ instructionUpdating: true, instructionUpdateError: null });
      try {
        await workspace.updateInstructions(root);
      } catch (error) {
        // 失敗留在原位可重試——update() 冪等，重試即收斂（決策 5）。
        set({ instructionUpdateError: String(error), instructionUpdating: false });
        return;
      }
      set({ instructionUpdating: false });
      await get().refreshInstructionPrompt();
    },

    dismissInstructionPrompt() {
      const root = activeLocalRoot();
      const prompt = get().instructionPrompt;
      if (!root || !prompt) return;
      writeInstructionSkip(root, prompt.version);
      set({ instructionPrompt: null, instructionUpdateError: null });
    },

    async restoreTabs() {
      if (!workspace) return;
      const persisted = readPersistedTabs();
      const tabs: ProjectTab[] = persisted.tabs.map((t) => ({ ...t }));
      set({ tabs });
      // 首啟活躍專案由持久化 activeKey 決定（決策 4/6）。
      const activeTab = persisted.activeKey
        ? tabs.find((t) => locatorKey(t.locator) === persisted.activeKey)
        : undefined;
      if (activeTab) {
        // 切回上次活躍專案（完整開啟語意；失效即轉錯誤態、維持空狀態）。
        await get().activateTab(locatorKey(activeTab.locator));
      } else if (tabs.length === 0) {
        // 首啟無持久化分頁：啟動目錄（cwd 探索）若是專案即自動記入。
        try {
          const dir = await workspace.startupDir();
          const probe = await workspace.openProject(dir);
          if (probe.status === "project") await enterProject(probe.root, probe.name);
        } catch {
          // 非專案目錄啟動：維持零分頁空狀態。
        }
      }
      // 背景 local 分頁啟動時各探測一次路徑有效性；失效轉錯誤態（design D11）。
      const active = get().activeKey;
      await Promise.all(
        get()
          .tabs.filter((t) => locatorKey(t.locator) !== active)
          .map(async (t) => {
            const key = locatorKey(t.locator);
            if (t.locator.kind !== "local") return;
            try {
              await workspace.projectStats(t.locator.root);
            } catch (e) {
              set({ tabErrors: { ...get().tabErrors, [key]: String(e) } });
            }
          }),
      );
    },
    };
  });
}

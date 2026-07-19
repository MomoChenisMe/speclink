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

import { appT } from "./i18n/runtime";
import type { ConnectionsAdapter, ConnectionView } from "./adapter/connections";
import type { WorkspaceAdapter } from "./adapter/workspace";
import { locatorKey, type WorkspaceSession } from "./session";
import {
  pendingWrapUpCount,
  persistTabs,
  readPersistedTabs,
  removeTab,
  upsertTab,
  type ProjectTab,
} from "./tabs";
import { detectMacOS, type TrayStyle } from "./tray";

const FAILURE_TOAST_ID = "desktop-operation-failure";
type FailureMessageKey =
  | "store.deleteFailed"
  | "store.archiveFailed"
  | "store.discussionArchiveFailed"
  | "store.reorderFailed"
  | "store.openProjectFailed"
  | "store.initFailed";

function showFailureToast(subject: string, messageKey: FailureMessageKey, error: unknown): void {
  toast.error(`${subject} · ${appT(messageKey)} ✗ ${String(error)}`, { id: FAILURE_TOAST_ID });
}

/** 主頁面：變更看板（預設）、規格頁、已封存獨立頁或設定頁。 */
export type BoardView = "board" | "specs" | "archived" | "settings";

/** 逐連線登入互動狀態（desktop-connections）：patInput＝device flow 明確
 * 不支援（404/405），就地收 PAT（規格 PAT fallback）；notice＝非錯誤提示
 * （如 PAT 登出後的帳號頁撤銷提醒）。 */
export type ConnectionPhase =
  | { kind: "idle" }
  | { kind: "busy" }
  | { kind: "patInput"; error: string | null }
  | { kind: "notice"; message: string }
  | { kind: "error"; message: string };

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
  /** 待確認的討論歸檔（slug）。 */
  pendingArchiveDiscussion: string | null;
  /** 詳情抽屜內呈現的 validate／analyze 結構化結果（keyed by change）；null=無。 */
  drawerVerb: VerbDrawerResult | null;

  refresh: () => Promise<void>;
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
  requestDelete: (name: string) => void;
  confirmDelete: () => Promise<void>;
  cancelDelete: () => void;
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
  /** 目前 active 分頁的 locator key（null＝零分頁空狀態）。 */
  activeKey: string | null;
  /** 待確認的初始化目錄（uninitialized 判定觸發；null＝無對話框）。 */
  pendingInit: string | null;
  /** 失效分頁錯誤（locator key → 單行訊息）。 */
  tabErrors: Record<string, string>;
  /** 開啟專案（dialog 或還原路徑）：三態分流。 */
  openProjectAt: (path: string) => Promise<void>;
  /** 開啟 remote workspace（remote-data-source 決策 6）：handshake 成功才建
   * session 與分頁並切至看板；失敗上拋、由開啟表單就地呈現。 */
  openRemoteWorkspace: (connectionId: string, target: string) => Promise<void>;
  /** 開資料夾選擇器再走 openProjectAt；取消即無事。 */
  openProjectViaDialog: () => Promise<void>;
  /** 點分頁（locator key）：與開啟專案相同語意；失敗轉該分頁錯誤態、不切換。 */
  activateTab: (key: string) => Promise<void>;
  closeTab: (key: string) => void;
  confirmInit: (tools: string[]) => Promise<void>;
  cancelInit: () => void;
  /** 啟動：還原持久化分頁、依持久化 activeKey 切回最後活躍專案、背景分頁各查一次徽章快照。 */
  restoreTabs: () => Promise<void>;
  /** Ctrl+Tab：循環切至下一分頁（走開啟專案語意）。 */
  cycleTab: () => Promise<void>;
  /** Ctrl+1..9：直達第 N 個分頁（1-based；超界不動作）。 */
  gotoTab: (n: number) => Promise<void>;

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
  refreshConnections: () => Promise<void>;
  /** 新增（同 origin 即更新顯示名）並隨即進入登入流程（決策 7）；
   * 無效輸入上拋、由表單就地呈現。 */
  addConnection: (baseUrl: string, name: string) => Promise<void>;
  /** device 預設登入；明確不支援時轉 patInput、連線錯誤浮為可讀狀態。 */
  loginConnection: (origin: string) => Promise<void>;
  /** PAT 單次過境提交；無效 PAT 留在輸入面並就地浮錯。 */
  submitPat: (origin: string, pat: string) => Promise<void>;
  logoutConnection: (origin: string) => Promise<void>;
  /** 移除連線（Rust 側先登出再刪條目——決策 6）。 */
  removeConnection: (id: string) => Promise<void>;
}

/** createAppStore 的注入面（workspace-session 決策 6）：session 工廠取代全域
 * dataSource；workspace 為探測面（開專案／init／統計／選資料夾／監看重掛）。 */
export interface AppStoreDeps {
  /** local session 工廠（root、顯示名）；測試注入假 session。 */
  createSession: (root: string, name: string) => WorkspaceSession;
  /** remote session 工廠（remote-data-source 決策 6/7）：以 connectionId 與
   * workspace 識別（project 或 project/repo）走 handshake，成功回 session、
   * 失敗上拋——未注入時 remote 開啟入口不啟用。 */
  openRemote?: (connectionId: string, target: string) => Promise<WorkspaceSession>;
  /** workspace 探測面；未注入時對應 UI 不啟用。 */
  workspace?: WorkspaceAdapter;
  /** server 連線面（desktop-connections）；未注入時伺服器頁籤不啟用。 */
  connections?: ConnectionsAdapter;
}

/**
 * 建立 app 狀態 store（Zustand）。狀態集中此處、留在 apps/desktop；共用元件
 * （packages/ui）不依賴 store，仍經 props 取資料——守住資料源解耦。資料載入
 * 一律經活躍 session 的 dataSource（單活躍載入語意不變）。
 */
export function createAppStore(deps: AppStoreDeps): UseBoundStore<StoreApi<AppState>> {
  const { createSession, openRemote, workspace, connections: connectionsAdapter } = deps;
  return create<AppState>((set, get) => {
    // 全文查詢的去抖與 latest-wins 狀態（design D6）——閉包層、不進 store state。
    let searchSeq = 0;
    let searchTimer: ReturnType<typeof setTimeout> | null = null;

    /** 活躍 session；零分頁空狀態回 null。 */
    function activeSession(): WorkspaceSession | null {
      const { activeKey, sessions } = get();
      return (activeKey && sessions[activeKey]) || null;
    }

    /** 活躍 session 的 dataSource；零分頁空狀態回 null（資料操作一律早退）。 */
    function activeDataSource(): SpeclinkDataSource | null {
      return activeSession()?.dataSource ?? null;
    }

    /** 逐連線互動狀態的單點更新（desktop-connections）。 */
    function setConnectionPhase(origin: string, phase: ConnectionPhase) {
      set({ connectionPhases: { ...get().connectionPhases, [origin]: phase } });
    }

    /** 命中專案的共同尾聲：upsert session 與分頁（去重）、設 activeKey、清對話框、
     * persist、顯式重掛監看、整批 refresh。probe 為純探測，其回報值即後端真相。
     * 舊 active 分頁的徽章停留在最後一次 refresh 的派生值——即切走時的快照（design D11）。 */
    async function enterProject(root: string, name: string) {
      const locator = { kind: "local", root } as const;
      const key = locatorKey(locator);
      const tabs = upsertTab(get().tabs, { locator, name });
      const sessions = { ...get().sessions };
      if (!sessions[key]) sessions[key] = createSession(root, name);
      // 淘汰出分頁列的 session 一併回收（上限丟最舊）。
      const live = new Set(tabs.map((t) => locatorKey(t.locator)));
      for (const k of Object.keys(sessions)) if (!live.has(k)) delete sessions[k];
      const tabErrors = { ...get().tabErrors };
      delete tabErrors[key];
      set({ tabs, sessions, tabErrors, activeKey: key, pendingInit: null });
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
      const tabErrors = { ...get().tabErrors };
      delete tabErrors[key];
      set({ tabs, sessions, tabErrors, activeKey: key, pendingInit: null });
      persistTabs(tabs, key);
      await get().refresh();
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
    pendingArchiveDiscussion: null,
    drawerVerb: null,

    async refresh() {
      const session = activeSession();
      if (!session) return;
      const dataSource = session.dataSource;
      // capability 驅動（remote-data-source 決策 2）：server 未提供的讀取跳過、
      // 以空集呈現（archived 頁另有提示卡），不讓整批 refresh 失敗。
      const [changes, specs, archived, discussions] = await Promise.all([
        dataSource.listChanges(),
        dataSource.listSpecs(),
        session.capabilities.listArchived ? dataSource.listArchived() : Promise.resolve([]),
        dataSource.listDiscussions(),
      ]);
      set({ changes, specs, archived, discussions, loaded: true });
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
      // active 分頁徽章＝待收尾數（已就緒變更＋已結論未轉出討論），隨看板刷新
      // 派生（spec-archive-drawer design D6）；背景分頁不動、保留最後已知值
      //（design D11 背景快照制）。
      const { activeKey, tabs } = get();
      if (activeKey && tabs.some((t) => locatorKey(t.locator) === activeKey)) {
        set({
          tabs: tabs.map((t) =>
            locatorKey(t.locator) === activeKey
              ? { ...t, badge: pendingWrapUpCount(changes, discussions.active) }
              : t,
          ),
        });
      }
      // 清單就緒後遞增刷新世代——開著的內容檢視據此重載至磁碟現況。
      set((st) => ({ refreshGen: st.refreshGen + 1 }));
    },

    setBoardView(boardView) {
      set({ boardView });
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
        try {
          await dataSource.runVerb(verb, change);
          set({ detailChange: null });
        } catch (e) {
          // 失敗時呈現 core 的錯誤訊息，不靜默吞掉。
          showFailureToast(change, "store.archiveFailed", e);
        }
        await get().refresh();
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
      try {
        await dataSource.reorderCard(kind, id, prevId, nextId);
      } catch (e) {
        // 寫回失敗不留假象（spec）：單行錯誤 toast，refresh 回磁碟現況。
        showFailureToast(id, "store.reorderFailed", e);
      }
      await get().refresh();
    },

    // --- workspace／專案分頁列 ---
    tabs: [],
    sessions: {},
    activeKey: null,
    pendingInit: null,
    tabErrors: {},

    // --- 系統匣樣式（平台決定、不持久化） ---
    trayStyle: detectMacOS() ? "panel" : "native-menu",
    trayPanelError: null,
    panelFallback(message) {
      set({ trayStyle: "native-menu", trayPanelError: message });
    },

    // --- server 連線（desktop-connections；決策 5/6/7） ---
    connections: [],
    connectionPhases: {},
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
      if (!connectionsAdapter) return;
      const entry = await connectionsAdapter.add(baseUrl, name);
      // 規格「新增後清單即時反映」：條目先上清單、再進入登入流程。
      await get().refreshConnections();
      await get().loginConnection(entry.origin);
    },
    async loginConnection(origin) {
      if (!connectionsAdapter) return;
      setConnectionPhase(origin, { kind: "busy" });
      try {
        const result = await connectionsAdapter.deviceLogin(origin);
        if (result.status === "loggedIn") {
          setConnectionPhase(origin, { kind: "idle" });
          await get().refreshConnections();
        } else if (result.status === "unsupported") {
          setConnectionPhase(origin, { kind: "patInput", error: null });
        } else if (result.status === "denied") {
          setConnectionPhase(origin, { kind: "error", message: appT("servers.denied") });
        } else {
          setConnectionPhase(origin, { kind: "error", message: appT("servers.expired") });
        }
      } catch (e) {
        setConnectionPhase(origin, { kind: "error", message: String(e) });
      }
    },
    async submitPat(origin, pat) {
      if (!connectionsAdapter) return;
      setConnectionPhase(origin, { kind: "busy" });
      try {
        await connectionsAdapter.patLogin(origin, pat);
        setConnectionPhase(origin, { kind: "idle" });
        await get().refreshConnections();
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
      if (origin) setConnectionPhase(origin, { kind: "busy" });
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
        if (probe.status === "project") await enterProject(probe.root, probe.name);
        else set({ pendingInit: probe.dir });
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

    async openRemoteWorkspace(connectionId, target) {
      if (!openRemote) return;
      // handshake fail-closed（決策 6）：失敗原樣上拋、不建分頁不建 session。
      const session = await openRemote(connectionId, target);
      await adoptRemoteSession(session);
      // 開啟 workspace 的意圖是看板——自伺服器頁切回看板呈現 server 資料。
      set({ boardView: "board" });
    },

    async activateTab(key) {
      if (!workspace) return;
      const tab = get().tabs.find((t) => locatorKey(t.locator) === key);
      if (!tab) return;
      if (tab.locator.kind === "remote") {
        const existing = get().sessions[key];
        try {
          if (existing) {
            const tabErrors = { ...get().tabErrors };
            delete tabErrors[key];
            set({ activeKey: key, tabErrors });
            persistTabs(get().tabs, key);
            await get().refresh();
          } else if (openRemote) {
            // 重啟恢復（規格「重啟後 remote 分頁恢復需重驗」）：重走 handshake。
            const { connectionId, projectId, repoId } = tab.locator;
            await adoptRemoteSession(await openRemote(connectionId, `${projectId}/${repoId}`));
          }
        } catch (e) {
          // 失敗呈現狀態、分頁不消失（needs-reauth／server 錯誤原樣呈現）。
          set({ tabErrors: { ...get().tabErrors, [key]: String(e) } });
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

    closeTab(key) {
      const tabs = removeTab(get().tabs, key);
      const sessions = { ...get().sessions };
      delete sessions[key];
      const tabErrors = { ...get().tabErrors };
      delete tabErrors[key];
      const wasActive = get().activeKey === key;
      const activeKey = wasActive ? null : get().activeKey;
      set({ tabs, sessions, tabErrors, activeKey });
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

    async restoreTabs() {
      if (!workspace) return;
      const persisted = readPersistedTabs();
      const tabs: ProjectTab[] = persisted.tabs.map((t) => ({ ...t, badge: null }));
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
      // 背景分頁徽章：啟動時各查一次快照；失效路徑轉錯誤態（design D11）。
      const active = get().activeKey;
      await Promise.all(
        get()
          .tabs.filter((t) => locatorKey(t.locator) !== active)
          .map(async (t) => {
            const key = locatorKey(t.locator);
            if (t.locator.kind !== "local") return;
            try {
              const stats = await workspace.projectStats(t.locator.root);
              set({
                tabs: get().tabs.map((x) =>
                  locatorKey(x.locator) === key ? { ...x, badge: stats.pendingWrapUp } : x,
                ),
              });
            } catch (e) {
              set({ tabErrors: { ...get().tabErrors, [key]: String(e) } });
            }
          }),
      );
    },
    };
  });
}

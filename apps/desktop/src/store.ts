import { create, type StoreApi, type UseBoundStore } from "zustand";
import type {
  SpeclinkDataSource,
  CardKind,
  ChangeItem,
  SpecItem,
  ArchivedItem,
  DiscussionItem,
  DiscussionLists,
  ListView,
  Verb,
} from "@speclink/ui";

import { appT } from "./i18n/runtime";
import type { WorkspaceAdapter } from "./adapter/workspace";
import {
  inProgressCount,
  persistTabs,
  readPersistedTabs,
  removeTab,
  upsertTab,
  type ProjectTab,
} from "./tabs";

/** 主頁面：變更看板（預設）、已封存獨立頁或設定頁。 */
export type BoardView = "board" | "archived" | "settings";

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
  expandedName: string | null;

  /** 詳情抽屜當前的 change（null=關閉）。 */
  detailChange: ChangeItem | null;
  /** 討論抽屜當前的討論（null=關閉）。 */
  detailDiscussion: DiscussionItem | null;

  pendingArchive: string | null;
  pendingDelete: string | null;
  /** 待確認的轉為變更（討論 slug）。 */
  pendingPromote: string | null;
  /** 待確認的討論歸檔（slug）。 */
  pendingArchiveDiscussion: string | null;
  /** 最近一次轉為變更失敗的單行錯誤（討論抽屜呈現；null=無）。 */
  promoteError: string | null;
  verbResult: string | null;

  refresh: () => Promise<void>;
  setBoardView: (v: BoardView) => void;
  setView: (v: ListView) => void;
  setQuery: (q: string) => void;
  setBoardQuery: (q: string) => void;
  toggleExpand: (name: string) => void;
  openDetail: (name: string) => void;
  closeDetail: () => void;
  openDiscussion: (slug: string) => void;
  closeDiscussion: () => void;
  requestArchive: (name: string) => void;
  confirmArchive: () => Promise<void>;
  cancelArchive: () => void;
  requestDelete: (name: string) => void;
  confirmDelete: () => Promise<void>;
  cancelDelete: () => void;
  requestPromote: (slug: string) => void;
  /** 確認轉為變更；name 為對話框輸入的變更名（省略時由 slug 衍生）。 */
  confirmPromote: (name?: string) => Promise<void>;
  cancelPromote: () => void;
  requestArchiveDiscussion: (slug: string) => void;
  confirmArchiveDiscussion: () => Promise<void>;
  cancelArchiveDiscussion: () => void;
  runVerb: (verb: Verb, change: string) => Promise<void>;
  /** 看板拖排寫回：把卡片排到兩鄰居之間（null＝欄頂／欄底）；失敗浮上 verbResult。 */
  reorderCard: (kind: CardKind, id: string, prevId: string | null, nextId: string | null) => Promise<void>;

  // --- workspace／專案分頁列（注入 workspace adapter 時生效；design D3/D10/D11） ---
  /** 分頁清單（開啟順序；持久化於 app 本機）。 */
  tabs: ProjectTab[];
  /** 目前 active 分頁的 root（null＝零分頁空狀態）。 */
  activeRoot: string | null;
  /** 待確認的初始化目錄（uninitialized 判定觸發；null＝無對話框）。 */
  pendingInit: string | null;
  /** 失效分頁錯誤（root → 單行訊息）。 */
  tabErrors: Record<string, string>;
  /** 開啟專案（dialog 或還原路徑）：三態分流。 */
  openProjectAt: (path: string) => Promise<void>;
  /** 開資料夾選擇器再走 openProjectAt；取消即無事。 */
  openProjectViaDialog: () => Promise<void>;
  /** 點分頁：與開啟專案相同語意；失敗轉該分頁錯誤態、不切換。 */
  activateTab: (root: string) => Promise<void>;
  closeTab: (root: string) => void;
  confirmInit: (tools: string[]) => Promise<void>;
  cancelInit: () => void;
  /** 啟動：還原持久化分頁、切回最後活躍專案、背景分頁各查一次徽章快照。 */
  restoreTabs: () => Promise<void>;
  /** Ctrl+Tab：循環切至下一分頁（走開啟專案語意）。 */
  cycleTab: () => Promise<void>;
  /** Ctrl+1..9：直達第 N 個分頁（1-based；超界不動作）。 */
  gotoTab: (n: number) => Promise<void>;
}

/** 把動詞回傳的 payload 轉成簡潔的人眼訊息（取代生 JSON）。 */
function formatVerbResult(verb: Verb, r: unknown): string {
  const o = (r ?? {}) as Record<string, unknown>;
  if (verb === "validate") {
    return o.valid ? "validate ✓ valid" : `validate ✗ ${(o.errors as string[] | undefined)?.[0] ?? "invalid"}`;
  }
  if (verb === "analyze") {
    const n = Array.isArray(o.findings) ? o.findings.length : 0;
    return `analyze ✓ ${n} finding${n === 1 ? "" : "s"}`;
  }
  if (verb === "archive") {
    return `archive ✓ ${(o.datedName as string) ?? "archived"}`;
  }
  return `${verb} ✓`;
}

/**
 * 以注入的 dataSource 建立 app 狀態 store（Zustand）。狀態集中此處、留在 apps/desktop；
 * 共用元件（packages/ui）不依賴 store，仍經 props 取資料——守住資料源解耦。
 */
export function createAppStore(
  dataSource: SpeclinkDataSource,
  workspace?: WorkspaceAdapter,
): UseBoundStore<StoreApi<AppState>> {
  return create<AppState>((set, get) => {
    /** 命中專案的共同尾聲：記分頁（去重）、設 active、清對話框、persist、整批 refresh。
     * 舊 active 分頁的徽章停留在最後一次 refresh 的派生值——即切走時的快照（design D11）。 */
    async function enterProject(root: string, name: string) {
      // 切換後經 current_project 同步 active 分頁標示——後端 root 是唯一真相；
      // 查詢不可用時以委派回報值為準（同一次切換的返回值）。
      let cur = { root, name };
      if (workspace) {
        try {
          cur = await workspace.currentProject();
        } catch {
          // 委派值即後端切換後的回報，逕用。
        }
      }
      const tabs = upsertTab(get().tabs, cur);
      const tabErrors = { ...get().tabErrors };
      delete tabErrors[cur.root];
      set({ tabs, tabErrors, activeRoot: cur.root, pendingInit: null });
      persistTabs(tabs, cur.root);
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
    expandedName: null,
    detailChange: null,
    detailDiscussion: null,
    pendingArchive: null,
    pendingDelete: null,
    pendingPromote: null,
    pendingArchiveDiscussion: null,
    promoteError: null,
    verbResult: null,

    async refresh() {
      const [changes, specs, archived, discussions] = await Promise.all([
        dataSource.listChanges(),
        dataSource.listSpecs(),
        dataSource.listArchived(),
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
      // active 分頁徽章＝當前變更清單的進行中數（派生管顯示）；背景分頁不動、
      // 保留最後已知值（design D11 背景快照制）。
      const { activeRoot, tabs } = get();
      if (activeRoot && tabs.some((t) => t.root === activeRoot)) {
        set({
          tabs: tabs.map((t) =>
            t.root === activeRoot ? { ...t, badge: inProgressCount(changes) } : t,
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
    },

    toggleExpand(name) {
      set({ expandedName: get().expandedName === name ? null : name });
    },

    openDetail(name) {
      const c = get().changes.find((x) => x.name === name);
      if (c) set({ detailChange: c });
    },

    closeDetail() {
      set({ detailChange: null });
    },

    openDiscussion(slug) {
      const lists = get().discussions;
      const d =
        lists.active.find((x) => x.slug === slug) ??
        lists.archived.find((x) => x.slug === slug);
      if (d) set({ detailDiscussion: d, promoteError: null });
    },

    closeDiscussion() {
      set({ detailDiscussion: null, promoteError: null });
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
      try {
        await dataSource.deleteChange(name);
        set({ verbResult: `${name} · ${appT("store.deleted")}`, detailChange: null });
      } catch (e) {
        set({ verbResult: `${name} · ${appT("store.deleteFailed")} ✗ ${String(e)}` });
      }
      await get().refresh();
    },

    cancelDelete() {
      set({ pendingDelete: null });
    },

    requestPromote(slug) {
      set({ pendingPromote: slug });
    },

    async confirmPromote(name) {
      const slug = get().pendingPromote;
      set({ pendingPromote: null });
      if (!slug) return;
      try {
        const r = await dataSource.promoteDiscussion(slug, name?.trim() || undefined);
        set({ promoteError: null, verbResult: `${slug} · ${appT("store.promoted")} ${r.change}` });
      } catch (e) {
        // 轉為變更失敗：單行錯誤（討論抽屜與頂欄皆呈現），看板不變。
        set({ promoteError: String(e), verbResult: `${slug} · ${appT("store.promoteFailed")} ${String(e)}` });
      }
      await get().refresh();
    },

    cancelPromote() {
      set({ pendingPromote: null });
    },

    requestArchiveDiscussion(slug) {
      set({ pendingArchiveDiscussion: slug });
    },

    async confirmArchiveDiscussion() {
      const slug = get().pendingArchiveDiscussion;
      set({ pendingArchiveDiscussion: null });
      if (!slug) return;
      try {
        await dataSource.archiveDiscussion(slug);
        set({ verbResult: `${slug} · ${appT("store.discussionArchived")}`, detailDiscussion: null });
      } catch (e) {
        set({ verbResult: `${slug} · ${appT("store.discussionArchiveFailed")} ✗ ${String(e)}` });
      }
      await get().refresh();
    },

    cancelArchiveDiscussion() {
      set({ pendingArchiveDiscussion: null });
    },

    async runVerb(verb, change) {
      try {
        const r = await dataSource.runVerb(verb, change);
        set({ verbResult: `${change} · ${formatVerbResult(verb, r)}` });
      } catch (e) {
        // 失敗時呈現 core 的錯誤訊息，不靜默吞掉。
        set({ verbResult: `${change} · ${verb} ✗ ${String(e)}` });
      }
      await get().refresh();
    },

    async reorderCard(kind, id, prevId, nextId) {
      try {
        await dataSource.reorderCard(kind, id, prevId, nextId);
      } catch (e) {
        // 寫回失敗不留假象（spec）：單行錯誤浮上，refresh 回磁碟現況。
        set({ verbResult: `${id} · ${appT("store.reorderFailed")} ✗ ${String(e)}` });
      }
      await get().refresh();
    },

    // --- workspace／專案分頁列 ---
    tabs: [],
    activeRoot: null,
    pendingInit: null,
    tabErrors: {},

    async openProjectAt(path) {
      if (!workspace) return;
      try {
        const probe = await workspace.openProject(path);
        if (probe.status === "project") await enterProject(probe.root, probe.name);
        else set({ pendingInit: probe.dir });
      } catch (e) {
        // 開啟失敗（spec：顯示單行錯誤訊息並維持原專案）。
        set({ verbResult: String(e) });
      }
    },

    async openProjectViaDialog() {
      if (!workspace) return;
      const picked = await workspace.pickFolder();
      if (picked) await get().openProjectAt(picked);
    },

    async activateTab(root) {
      if (!workspace) return;
      try {
        const probe = await workspace.openProject(root);
        if (probe.status === "project") {
          await enterProject(probe.root, probe.name);
        } else {
          // openspec/ 已刪但目錄還在：分頁轉錯誤態，不開初始化對話框。
          set({ tabErrors: { ...get().tabErrors, [root]: appT("store.tabInvalid") } });
        }
      } catch (e) {
        set({ tabErrors: { ...get().tabErrors, [root]: String(e) } });
      }
    },

    closeTab(root) {
      const tabs = removeTab(get().tabs, root);
      const tabErrors = { ...get().tabErrors };
      delete tabErrors[root];
      const wasActive = get().activeRoot === root;
      const activeRoot = wasActive ? null : get().activeRoot;
      set({ tabs, tabErrors, activeRoot });
      persistTabs(tabs, activeRoot);
      // 關掉 active 分頁：切到最近的剩餘分頁（走完整開啟語意）；零分頁則空狀態。
      if (wasActive) {
        const next = tabs.at(-1);
        if (next) void get().activateTab(next.root);
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
        // 初始化失敗：單行錯誤、不切換 root（spec）。
        set({ verbResult: String(e) });
      }
    },

    cancelInit() {
      set({ pendingInit: null });
    },

    async cycleTab() {
      const { tabs, activeRoot } = get();
      if (tabs.length < 2) return;
      const idx = tabs.findIndex((t) => t.root === activeRoot);
      const next = tabs[(idx + 1) % tabs.length];
      await get().activateTab(next.root);
    },

    async gotoTab(n) {
      const target = get().tabs[n - 1];
      if (!target || target.root === get().activeRoot) return;
      await get().activateTab(target.root);
    },

    async restoreTabs() {
      if (!workspace) return;
      const persisted = readPersistedTabs();
      const tabs: ProjectTab[] = persisted.tabs.map((t) => ({ ...t, badge: null }));
      set({ tabs });
      if (persisted.activeRoot && tabs.some((t) => t.root === persisted.activeRoot)) {
        // 切回上次活躍專案（完整開啟語意；失效即轉錯誤態、維持空狀態）。
        await get().activateTab(persisted.activeRoot);
      } else if (tabs.length === 0) {
        // 首啟無持久化分頁：後端 root（cwd 探索）若是專案即自動記入。
        try {
          const cur = await workspace.currentProject();
          const probe = await workspace.openProject(cur.root);
          if (probe.status === "project") await enterProject(probe.root, probe.name);
        } catch {
          // 非專案目錄啟動：維持零分頁空狀態。
        }
      }
      // 背景分頁徽章：啟動時各查一次快照；失效路徑轉錯誤態（design D11）。
      const active = get().activeRoot;
      await Promise.all(
        get()
          .tabs.filter((t) => t.root !== active)
          .map(async (t) => {
            try {
              const stats = await workspace.projectStats(t.root);
              set({
                tabs: get().tabs.map((x) =>
                  x.root === t.root ? { ...x, badge: stats.inProgressChanges } : x,
                ),
              });
            } catch (e) {
              set({ tabErrors: { ...get().tabErrors, [t.root]: String(e) } });
            }
          }),
      );
    },
    };
  });
}

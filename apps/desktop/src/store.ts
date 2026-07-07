import { create, type StoreApi, type UseBoundStore } from "zustand";
import type {
  SpeclinkDataSource,
  ChangeItem,
  SpecItem,
  ArchivedItem,
  DiscussionItem,
  DiscussionLists,
  ListView,
  Verb,
} from "@speclink/ui";

/** 主頁面：變更看板（預設）或已封存獨立頁。 */
export type BoardView = "board" | "archived";

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
): UseBoundStore<StoreApi<AppState>> {
  return create<AppState>((set, get) => ({
    changes: [],
    specs: [],
    archived: [],
    discussions: { active: [], archived: [] },
    loaded: false,
    refreshGen: 0,
    boardView: "board",
    view: "active",
    query: "",
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
        set({ verbResult: `${name} · 已刪除`, detailChange: null });
      } catch (e) {
        set({ verbResult: `${name} · 刪除失敗 ✗ ${String(e)}` });
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
        set({ promoteError: null, verbResult: `${slug} · 轉為變更 ✓ ${r.change}` });
      } catch (e) {
        // 轉為變更失敗：單行錯誤（討論抽屜與頂欄皆呈現），看板不變。
        set({ promoteError: String(e), verbResult: `${slug} · 轉為變更 ✗ ${String(e)}` });
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
        set({ verbResult: `${slug} · 討論已封存`, detailDiscussion: null });
      } catch (e) {
        set({ verbResult: `${slug} · 討論封存失敗 ✗ ${String(e)}` });
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
  }));
}

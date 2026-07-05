import { create, type StoreApi, type UseBoundStore } from "zustand";
import type {
  SpeclinkDataSource,
  ChangeItem,
  SpecItem,
  ArchivedItem,
  ListView,
  Verb,
} from "@speclink/ui";

/** 主頁面：變更看板（預設）或已封存獨立頁。 */
export type BoardView = "board" | "archived";

export interface AppState {
  changes: ChangeItem[];
  specs: SpecItem[];
  archived: ArchivedItem[];
  loaded: boolean;

  boardView: BoardView;
  view: ListView;
  query: string;
  expandedName: string | null;

  /** 詳情抽屜當前的 change（null=關閉）。 */
  detailChange: ChangeItem | null;

  pendingArchive: string | null;
  pendingDelete: string | null;
  verbResult: string | null;

  refresh: () => Promise<void>;
  setBoardView: (v: BoardView) => void;
  setView: (v: ListView) => void;
  setQuery: (q: string) => void;
  toggleExpand: (name: string) => void;
  openDetail: (name: string) => void;
  closeDetail: () => void;
  requestArchive: (name: string) => void;
  confirmArchive: () => Promise<void>;
  cancelArchive: () => void;
  requestDelete: (name: string) => void;
  confirmDelete: () => Promise<void>;
  cancelDelete: () => void;
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
    loaded: false,
    boardView: "board",
    view: "active",
    query: "",
    expandedName: null,
    detailChange: null,
    pendingArchive: null,
    pendingDelete: null,
    verbResult: null,

    async refresh() {
      const [changes, specs, archived] = await Promise.all([
        dataSource.listChanges(),
        dataSource.listSpecs(),
        dataSource.listArchived(),
      ]);
      set({ changes, specs, archived, loaded: true });
      // 詳情開著時同步其資料（如任務數更新）
      const cur = get().detailChange;
      if (cur) {
        set({ detailChange: changes.find((c) => c.name === cur.name) ?? null });
      }
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

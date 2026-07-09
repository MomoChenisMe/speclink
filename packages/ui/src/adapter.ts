// 資料源 adapter 介面——以領域語彙（change/spec/document/verb）定義，與後端解耦。
//
// 桌面 app 注入以 Tauri invoke 為後端的實作；未來 web 端可注入 HTTP 後端。
// 元件本身不引用任何 Tauri 專屬全域，一律經此介面取資料。

/** 一個 active change 的清單項（CLI 同形欄位＋桌面疊加的生命週期標記）。 */
export interface ChangeItem {
  name: string;
  status: string;
  totalTasks: number;
  completedTasks: number;
  summary?: string;
  /** 開工站標記（in-progress add 蓋章）；null/缺席＝未開工。 */
  startedAt?: string | null;
  startedBy?: string | null;
  startedWith?: string | null;
  /** 來源討論 slug 清單（由討論轉出／併入的 change，第一個為出身討論）；空/缺席＝非討論而來。 */
  fromDiscussions?: string[];
}

/** 一個 canonical spec 的清單項（CLI 同形欄位＋桌面疊加的呈現層輔助欄位）。 */
export interface SpecItem {
  id: string;
  /** spec.md 最後修改日期（檔案系統 mtime 衍生，YYYY-MM-DD）；mtime 不可得時缺席。 */
  modifiedAt?: string | null;
}

/** change 的 metadata（.openspec.yaml，camelCase）。 */
export interface ChangeMetaInfo {
  schema?: string | null;
  created?: string | null;
  createdBy?: string | null;
  createdWith?: string | null;
  fromDiscussions?: string[];
  startedAt?: string | null;
  startedBy?: string | null;
  startedWith?: string | null;
}

/** 一個歸檔 change 的清單項。任務計數缺席（無 tasks.md）時不顯示徽章。 */
export interface ArchivedItem {
  datedName: string;
  date: string;
  name: string;
  tasksTotal?: number;
  tasksDone?: number;
}

/** 一筆討論的清單項（camelCase；status: open | concluded | promoted）。 */
export interface DiscussionItem {
  slug: string;
  topic: string;
  status: string;
  rounds: number;
  created: string;
  /** 轉出（扇出）的 change 名累積清單；未轉出為空陣列。 */
  promotedTo: string[];
}

/** 討論清單兩節：看板討論欄（active）與已封存頁討論節（archived）。 */
export interface DiscussionLists {
  active: DiscussionItem[];
  archived: DiscussionItem[];
}

/** 可對選定 change 執行的動詞。park/unpark 已從 speclink 移除，不在此列。 */
export type Verb = "validate" | "analyze" | "archive";

/** 看板卡片種類（拖排寫回的目標）：變更卡或討論卡。 */
export type CardKind = "change" | "discussion";

/** 一個 artifact 的狀態（對應 core status 的 artifacts 項）。 */
export interface ArtifactStatus {
  id: string;
  outputPath: string;
  status: string;
  missingDeps?: string[];
}

/** change 的 artifact DAG 狀態（對應 `speclink status --json`）。 */
export interface StatusReport {
  changeName: string;
  schemaName: string;
  isComplete: boolean;
  applyRequires: string[];
  artifacts: ArtifactStatus[];
}

/** 元件透過此介面取得資料與觸發動詞——不知道背後是 Tauri 還是 HTTP。 */
export interface SpeclinkDataSource {
  listChanges(): Promise<ChangeItem[]>;
  listSpecs(): Promise<SpecItem[]>;
  listArchived(): Promise<ArchivedItem[]>;
  /** 取得一個 change 的 artifact DAG 狀態。 */
  status(change: string): Promise<StatusReport>;
  /** 讀取一個 change 的 artifact（artifact 為 output path，如 `proposal.md`）。 */
  getDocument(change: string, artifact: string): Promise<string | null>;
  /** 讀取一個 capability 的正典 spec.md。 */
  getSpecDocument(capability: string): Promise<string | null>;
  /** 列出一個 change 的 delta capability 名。 */
  changeCapabilities(change: string): Promise<string[]>;
  /** 取得 change 的 metadata（createdBy/createdWith/created）。無此 change 回 null。 */
  changeMeta(change: string): Promise<ChangeMetaInfo | null>;
  /** 刪除一個 active change（破壞性；UI 需先確認）。 */
  deleteChange(change: string): Promise<void>;
  /** 勾選/取消 tasks.md 的第 ordinal（1-based）個任務。 */
  setTaskDone(change: string, ordinal: number, done: boolean): Promise<void>;
  /** 批次設定全部任務完成狀態（true＝全部已完成、false＝重置任務），單次寫回。 */
  setAllTasks(change: string, done: boolean): Promise<void>;
  /**
   * 把第 from 個任務移到以第 to 個任務為錨的位置（皆 1-based）。
   * before 省略時依方向推斷（向上插錨前、向下插錨後）；true＝明確插錨前
   * （跨群組標題即成為該群組組首）、false＝明確插錨後。
   */
  moveTask(change: string, from: number, to: number, before?: boolean): Promise<void>;
  /** 對選定 change 執行動詞，回傳 core 的結果 payload；失敗時 reject 附訊息。 */
  runVerb(verb: Verb, change: string): Promise<unknown>;
  /** 讀取一個已封存 change 的 artifact 原文（dated name 定址）。缺件回 null。 */
  getArchivedDocument(datedName: string, artifact: string): Promise<string | null>;
  /** 列出一個已封存 change 的 delta capability 名。 */
  archivedCapabilities(datedName: string): Promise<string[]>;
  /** 討論清單（active＋archived）。非 speclink 專案回兩個空清單。 */
  listDiscussions(): Promise<DiscussionLists>;
  /** 讀取討論記錄全文（slug 定址；live 優先、封存後備）。無則 null。 */
  getDiscussionDocument(slug: string): Promise<string | null>;
  /** 把討論轉為新 change（可選 change 名，省略時由 slug 衍生）。失敗 reject 單行訊息。 */
  promoteDiscussion(slug: string, name?: string): Promise<{ change: string }>;
  /** 歸檔一筆 live 討論（UI 需先確認）；失敗 reject 附訊息。 */
  archiveDiscussion(slug: string): Promise<void>;
  /**
   * 看板欄內拖排寫回（design D5）：把卡片排到 prevId 與 nextId 兩鄰居之間
   * （null＝欄頂／欄底）。id 為變更名或討論 slug；失敗 reject 附訊息。
   */
  reorderCard(kind: CardKind, id: string, prevId: string | null, nextId: string | null): Promise<void>;
}

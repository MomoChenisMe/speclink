// 資料源 adapter 介面——以領域語彙（change/spec/document/verb）定義，與後端解耦。
//
// 桌面 app 注入以 Tauri invoke 為後端的實作；未來 web 端可注入 HTTP 後端。
// 元件本身不引用任何 Tauri 專屬全域，一律經此介面取資料。

/** 一個 active change 的清單項（對應 core `list --json` 的 camelCase 形狀）。 */
export interface ChangeItem {
  name: string;
  status: string;
  totalTasks: number;
  completedTasks: number;
  summary?: string;
}

/** 一個 canonical spec 的清單項。 */
export interface SpecItem {
  id: string;
}

/** change 的 metadata（.openspec.yaml，camelCase）。 */
export interface ChangeMetaInfo {
  schema?: string | null;
  created?: string | null;
  createdBy?: string | null;
  createdWith?: string | null;
  fromDiscussion?: string | null;
}

/** 一個歸檔 change 的清單項。 */
export interface ArchivedItem {
  datedName: string;
  date: string;
  name: string;
}

/** 可對選定 change 執行的動詞。park/unpark 已從 speclink 移除，不在此列。 */
export type Verb = "validate" | "analyze" | "archive";

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
  /** 把第 from 個任務移到第 to 個位置（皆 1-based）。 */
  moveTask(change: string, from: number, to: number): Promise<void>;
  /** 對選定 change 執行動詞，回傳 core 的結果 payload；失敗時 reject 附訊息。 */
  runVerb(verb: Verb, change: string): Promise<unknown>;
}

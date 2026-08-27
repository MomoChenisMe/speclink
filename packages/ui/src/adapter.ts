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
  /** 寫碼任務的總數／完成數／剩餘數（`[M]` 手動任務不計；spec client-protocol
   * 「變更清單的寫碼進度欄位」）——「待手動」章的資料源。remote 變更摘要不帶此
   * 三欄，缺席時章缺席。 */
  codeTotal?: number;
  codeComplete?: number;
  codeRemaining?: number;
  summary?: string;
  /** 開工站標記（in-progress add 蓋章）；null/缺席＝未開工。 */
  startedAt?: string | null;
  startedBy?: string | null;
  startedWith?: string | null;
  /** 認領人（"Name <email>"）——RemoteOnly 的認領標記；null/缺席＝未認領。 */
  claimedBy?: string | null;
  /** 建立者（"Name <email>"）——卡片首字母圓標頭像的資料源；缺席時省略。 */
  createdBy?: string | null;
  /** 建立日期（.openspec.yaml 的 created，YYYY-MM-DD）——建立時間篩選的資料源；缺席時省略。 */
  created?: string | null;
  /** 來源討論 slug 清單（由討論轉出／併入的 change，第一個為出身討論）；空/缺席＝非討論而來。 */
  fromDiscussions?: string[];
  /** 待重新反映的討論 slug 清單：本 change 曾 seal 這些討論、其後被重新結論，內容過期待 re-ingest；空/缺席＝無旗標。 */
  restaleFrom?: string[];
  /** proposal Why 首句（呈現層輔助欄位）——變更卡描述列的資料源；null/缺席＝描述列缺席。 */
  whyExcerpt?: string | null;
  /** `.openspec.yaml` 存在但解析失敗的原因（fail-closed 診斷）——卡片顯示最小
   * invalid 標記、變更操作由引擎錯誤拒絕；缺席＝metadata 有效或不存在。 */
  metaError?: string | null;
  /** 審查狀態（spec client-protocol「變更清單的審查狀態欄位」）：工單存在＝
   * inReview；章存在依雙錨凍結度分 reviewed／reviewedStale；缺席＝none。 */
  reviewStatus?: "none" | "inReview" | "reviewed" | "reviewedStale";
  /** 蓋章時間與審查者——章存在（reviewed／reviewedStale）時才附。 */
  reviewedAt?: string | null;
  reviewedBy?: string | null;
  /** 驗證狀態（spec client-protocol「變更清單的驗證狀態欄位」）：工單存在＝
   * inVerify；章存在依雙錨凍結度分 verified／verifiedStale；缺席＝none。
   * 與 reviewStatus 各自獨立——兩站互不遮蔽。 */
  verifyStatus?: "none" | "inVerify" | "verified" | "verifiedStale";
  /** 蓋章時間與驗證者——章存在（verified／verifiedStale）時才附。 */
  verifiedAt?: string | null;
  verifiedBy?: string | null;
  /** 這個 change 正在其中實作的 linked worktree（僅本機主 checkout、政策開啟時
   * 才有）；缺席＝在主資料夾裡做。 */
  worktree?: { branch: string; path: string } | null;
}

/** 一個 canonical spec 的清單項（CLI 同形欄位＋桌面疊加的呈現層輔助欄位）。
 * 規格卡收合資訊由 Rust 端清單 payload 帶出（spec-archive-drawer design D4）；
 * 不可讀／缺席檔案容錯為 0／null，前端對缺席欄位同樣以 0／null 容錯。 */
export interface SpecItem {
  id: string;
  /** spec.md 最後修改日期（檔案系統 mtime 衍生，YYYY-MM-DD）；mtime 不可得時缺席。 */
  modifiedAt?: string | null;
  /** 正典 spec 的 `### Requirement:` 標題數。 */
  requirementCount?: number;
  /** Purpose 區段首個非空行原文；null＝區段缺席或無內容。 */
  purposeExcerpt?: string | null;
  /** Purpose 為封存流程產生的佔位文字（前端改顯「Purpose 待補」警示）。 */
  purposeTbd?: boolean;
  /** 全文 @trace 標記的 source 去重數（溯源變更數）。 */
  traceCount?: number;
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

/** 一個歸檔 change 的清單項。任務計數缺席（無 tasks.md）時不顯示徽章。
 * 封存卡收合資訊由快取清單 payload 帶出（spec-archive-drawer design D4/D5）。 */
export interface ArchivedItem {
  datedName: string;
  date: string;
  name: string;
  tasksTotal?: number;
  tasksDone?: number;
  /** specs/ 下 capability 目錄數（觸及規格數）。 */
  specCount?: number;
  /** 建立者（.openspec.yaml 的 created_by）；null/缺席＝不顯示頭像圓點。 */
  createdBy?: string | null;
  /** 來源討論 slug 清單；空/缺席＝來源討論標記缺席。 */
  fromDiscussions?: string[];
  /** 封存時的審查結局（spec client-protocol「已封存清單的審查結局欄位」）：
   * 含章＝reviewed；含化石工單而無章＝reviewedNotPassed；缺席＝none。 */
  reviewStatus?: "none" | "reviewed" | "reviewedNotPassed";
  /** 封存時的驗證結局（spec client-protocol「已封存清單的驗證結局欄位」）：
   * 含章＝verified；含化石工單而無章＝verifiedNotPassed；缺席＝none。
   * 與審查結局並存——同一項可以「審查通過」卻「曾驗證未通過」。 */
  verifyStatus?: "none" | "verified" | "verifiedNotPassed";
  /** 封存 proposal.md 的 Why 區段首個非空行（spec client-protocol「已封存清單的
   * 呈現輔助欄位」）；缺席＝卡片描述列不顯示、退回單行。 */
  whyExcerpt?: string;
  /** 建立日期 YYYY-MM-DD（封存目錄 metadata 的 created）；缺席＝抽屜出身列的
   * 建立日期欄位不顯示。 */
  created?: string;
}

/** 一筆討論的清單項（camelCase；status: open | concluded | promoted）。 */
export interface DiscussionItem {
  slug: string;
  topic: string;
  status: string;
  rounds: number;
  created: string;
  /** 建立者（"Name <email>"）——discuss new 由 git 身分蓋章；缺席時省略。 */
  createdBy?: string | null;
  /** 討論型別（目前唯一值 "improve"）——一般討論缺席；標示隨此欄恆定。 */
  kind?: string | null;
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

/** analyze 報告的一條發現項（對應 core AnalyzeReport.findings，snake_case 直出）。 */
export interface AnalyzeFinding {
  id: string;
  dimension: string;
  severity: string;
  location: string;
  summary: string;
  recommendation: string;
}

/** analyze 報告的一個維度狀態（Coverage/Consistency/Ambiguity/Gaps 之一）。 */
export interface AnalyzeDimension {
  dimension: string;
  status: string;
  finding_count: number;
}

/** `speclink analyze --json` 的報告形狀（桌面 analyze 動詞回傳同形）。 */
export interface AnalyzeReport {
  change_id: string;
  dimensions: AnalyzeDimension[];
  findings: AnalyzeFinding[];
  artifacts_analyzed: string[];
  artifacts_missing: string[];
}

/**
 * 「分析」於詳情抽屜內呈現的結構化結果（validate＋analyze 一鍵合併；archive 仍走視窗頂列）。
 * change 供抽屜比對——僅當前開啟的 change 相符時才呈現。
 */
export interface VerbDrawerResult {
  change: string;
  /** 結構驗證結果（speclink validate 同形）。 */
  validate?: { valid: boolean; errors: string[] };
  /** analyze 報告。 */
  analyze?: AnalyzeReport;
  /** 執行失敗的單行錯誤（任一動詞）。 */
  error?: string;
}

/** 清單檢視模式（作用中／已封存）——桌面 store 的檢視狀態沿用此型別。 */
export type ListView = "active" | "archived";

/** 看板卡片種類（拖排寫回的目標）：變更卡或討論卡。 */
export type CardKind = "change" | "discussion";

/** workspace 全文查詢的一筆命中（design D6）：卡片識別＋命中 artifact＋snippet。 */
export interface SearchHit {
  kind: CardKind;
  id: string;
  /** 命中的 artifact 檔名（如 design.md、specs/foo/spec.md、討論記錄檔名）。 */
  artifact: string;
  /** 命中前後文裁切（含命中原文；截斷端補 …）。 */
  snippet: string;
}

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

/** 退回提案中被守門擋下:引擎回傳的工作痕跡證據(守門對話框的資料源)。 */
export interface RevertBlockedEvidence {
  /** tasks.md 的已勾任務數。 */
  checkedTasks: number;
  /** touched 記錄 v1 與 v2 兩清單的檔案聯集(去重)。 */
  touchedFiles: string[];
}

/** revertChangeToProposed 的守門拒絕:證據隨錯誤走,App 據此開守門對話框。 */
export class RevertBlockedError extends Error {
  checkedTasks: number;
  touchedFiles: string[];
  constructor(evidence: RevertBlockedEvidence) {
    super("revert blocked: work traces exist");
    this.name = "RevertBlockedError";
    this.checkedTasks = evidence.checkedTasks;
    this.touchedFiles = evidence.touchedFiles;
  }
}

/**
 * 兩個 desktop adapter 共用的錯誤轉譯:bridge 以 JSON 字串回守門證據
 *(kind: "revertBlocked")——解析成功回 RevertBlockedError,否則原樣轉單行 Error。
 */
export function toRevertError(raw: unknown): Error {
  const text = raw instanceof Error ? raw.message : String(raw);
  try {
    const parsed = JSON.parse(text) as {
      kind?: string;
      checkedTasks?: number;
      touchedFiles?: string[];
    };
    if (parsed.kind === "revertBlocked") {
      return new RevertBlockedError({
        checkedTasks: parsed.checkedTasks ?? 0,
        touchedFiles: parsed.touchedFiles ?? [],
      });
    }
  } catch {
    // 非 JSON——一般錯誤訊息。
  }
  return raw instanceof Error ? raw : new Error(text);
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
  /**
   * workspace 全文查詢（design D6）：以不分大小寫子字串比對 active 變更的
   * artifacts 與 active 討論記錄全文，每卡回傳首個命中。空 query 回空陣列。
   */
  searchWorkspace(query: string): Promise<SearchHit[]>;
  /** 列出一個 change 的 delta capability 名。 */
  changeCapabilities(change: string): Promise<string[]>;
  /** 取得 change 的 metadata（createdBy/createdWith/created）。無此 change 回 null。 */
  changeMeta(change: string): Promise<ChangeMetaInfo | null>;
  /** 認領一個 change 防止撞工（RemoteOnly——本地後端不提供此方法，UI 因此
   * 不長出認領面）。已被他人持有時 reject 附持有人與建議動作的單行訊息。 */
  claim?(change: string): Promise<void>;
  /** 刪除一個 active change（破壞性；UI 需先確認）。 */
  deleteChange(change: string): Promise<void>;
  /** 把誤開工的變更退回提案中(移除 in-progress 標記;僅零工作痕跡可行;
   * UI 需先確認)。守門擋下時 reject RevertBlockedError(證據隨錯誤);
   * 其餘失敗 reject 單行訊息。 */
  revertChangeToProposed(change: string): Promise<void>;
  /** 勾選/取消任務：task 為 tsk_ stable ID 或 ordinal 字串（無 ID 相容路徑）。 */
  setTaskDone(change: string, task: string, done: boolean): Promise<void>;
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
  /** 放棄審查（刪工單、不蓋章）——封存入口三選項的資料面；未實作的後端
   * （如 remote）不觸發三選項（其清單項本就不帶 inReview）。 */
  discardReview?(change: string): Promise<void>;
  /** 放棄驗證（刪工單、不蓋章）——驗證站的同一面。 */
  discardVerify?(change: string): Promise<void>;
  /** 帶著未結工單封存（`--carry-review`／`--carry-verify`）：該站封存側永久顯示
   * 「曾審查／曾驗證未通過」。兩個旗標各自獨立，雙工單並存時可同時帶。 */
  archiveCarry?(change: string, carryReview: boolean, carryVerify: boolean): Promise<unknown>;
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

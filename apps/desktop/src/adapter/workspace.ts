// workspace 管理操作的桌面 adapter（design D6；workspace-session 決策 3/4）：
// 直接 invoke 桌面專屬 command，不經 packages/ui 的 SpeclinkDataSource——該介面
// 是 change／spec 瀏覽管理抽象，專案與設定語意屬宿主。探測面（開啟／初始化／
// 統計）收路徑參數；設定面經 createWorkspaceSettings 將 root 綁入閉包，每支
// command 顯式帶 root。
import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

import type { InvokeFn, WorkspaceSettingsProvider } from "../session";

/** open_project／init_project／adopt_project 的判定 payload（錯誤走 rejected Promise）。 */
export type ProjectProbe =
  | { status: "project"; root: string; name: string }
  | {
      status: "remoteBinding";
      url: string;
      repo?: string | null;
      hasLocalOpenspec: boolean;
    }
  | { status: "uninitialized"; dir: string }
  | { status: "unadopted"; root: string };

/** schema 的來源層級（引擎的 resolution 層命名：內建／專案層／user 層）。 */
export type SchemaSource = "package" | "project" | "user";

/** 產出流程清單一項（read_schemas 的 payload；desktop-schema-panel D1/D2）：
 * error 有值＝該 schema 解析失敗，內容欄位一律空。 */
export interface SchemaEntry {
  name: string;
  source: SchemaSource;
  /** artifact 圖（引擎顯示序，與產出規則分節固定鍵同源）。 */
  artifactIds: string[];
  artifacts: SchemaArtifactDetail[];
  /** schema 目錄絕對路徑（開啟所在資料夾的把手）；內建為 null。 */
  path: string | null;
  error: string | null;
}

/** 一個 artifact 的唯讀詳情全文。 */
export interface SchemaArtifactDetail {
  id: string;
  description: string;
  instruction: string | null;
  template: string | null;
}

/** read_settings 的快照 payload（欄位值＋各檔可選的 parseError）。 */
export interface SettingsSnapshot {
  app: { tools: string[]; customTools: string[]; parseError: string | null };
  workflow: {
    locale: string | null;
    specLocale: string | null;
    tdd: boolean;
    audit: boolean;
    /** worktree 政策現值；remote 快照恆為 false（該面不支援此軸）。 */
    worktree: boolean;
    /** 專案說明現值（config.yaml 的 context；null＝未設定）。 */
    context: string | null;
    /** 產出規則現值（依 artifact id 分組；條目順序即檔案順序）。 */
    rules: Record<string, string[]>;
    /** 活躍 schema 的 artifact id（引擎顯示序）——產出規則分節的固定鍵。 */
    schemaArtifacts: string[];
    /** 活躍 schema 名稱（config 的 schema 鍵；缺席預設 spec-driven、壞檔空字串）。 */
    schemaName: string;
    /** false＝remote 快照遇非內建 schema 名稱（遠端自訂尚不支援）；其餘恆 true。 */
    schemaKnown: boolean;
    parseError: string | null;
    /** remote policy 的 scope revision；local 快照不帶此欄。 */
    revision?: number;
  };
}

/** write_workflow_config 的完整目標狀態（null/false＝設回預設即移除鍵）。 */
export interface WorkflowFields {
  locale: string | null;
  specLocale: string | null;
  tdd: boolean;
  audit: boolean;
  /** 並行 apply 的 worktree 流程；remote 無此軸（寫入時忽略）。 */
  worktree: boolean;
}

/** probe_instructions 的回報（引擎 InstructionProbe 的 camelCase 序列化）：
 * 五態判定、目前產物層版號、逐工具狀態與將被新建或改寫的受管檔清單。 */
export interface InstructionProbeResult {
  status: "missing" | "stale" | "newer" | "current" | "unknown";
  currentVersion: string;
  tools: Array<{
    tool: string;
    workspaceVersion: string | null;
    stale: boolean;
    /** 此工具的檔案領先引擎（app 本體是舊版）；與 stale 互斥。 */
    newer: boolean;
    missing: boolean;
  }>;
  differingFiles: string[];
}

/** 探測面：開啟／初始化／統計皆收路徑參數、無全域語意（設定面見
 * createWorkspaceSettings，經 session 綁 root）。 */
export interface WorkspaceAdapter {
  openProject(path: string): Promise<ProjectProbe>;
  initProject(path: string, tools: string[]): Promise<ProjectProbe>;
  /** 對未啟用專案根補齊工作區檔（既有規格內容零觸碰），成功回報 project。 */
  adoptProject(path: string, tools: string[]): Promise<ProjectProbe>;
  /** 啟動語境的預設目錄（首啟無持久化分頁時據此顯式開啟）；純讀。 */
  startupDir(): Promise<string>;
  projectStats(path: string): Promise<{ pendingWrapUp: number }>;
  /** 指令檔過期探測（唯讀）；探測失敗以 status=unknown 表達，不 reject。 */
  probeInstructions(path: string): Promise<InstructionProbeResult>;
  /** 指令檔整套再生（委派引擎 update()）；失敗為單行錯誤訊息。 */
  updateInstructions(path: string): Promise<void>;
  /** 監看重掛（workspace-session 決策 5）：顯式跟隨活躍 session。 */
  watchWorkspace(root: string): Promise<void>;
  /** 原生資料夾選擇器；取消回 null。 */
  pickFolder(): Promise<string | null>;
}

export function createWorkspaceAdapter(): WorkspaceAdapter {
  return {
    openProject: (path) => tauriInvoke("open_project", { path }),
    initProject: (path, tools) => tauriInvoke("init_project", { path, tools }),
    adoptProject: (path, tools) => tauriInvoke("adopt_project", { path, tools }),
    startupDir: () => tauriInvoke("startup_dir"),
    projectStats: (path) => tauriInvoke("project_stats", { path }),
    probeInstructions: (path) => tauriInvoke("probe_instructions", { path }),
    updateInstructions: (path) => tauriInvoke("update_instructions", { path }),
    watchWorkspace: (root) => tauriInvoke("watch_workspace", { root }),
    pickFolder: async () => {
      const picked = await open({ directory: true, multiple: false });
      return typeof picked === "string" ? picked : null;
    },
  };
}

/** 設定面工廠（workspace-session 決策 3）：root 綁入閉包，每支 command 顯式
 * 帶 root 直通 desktop-core 的帶路徑函式。invoke 可注入以利測試。 */
export function createWorkspaceSettings(
  root: string,
  invoke: InvokeFn = tauriInvoke as InvokeFn,
): WorkspaceSettingsProvider {
  return {
    kind: "local",
    policyWrite: true,
    readSettings: () => invoke("read_settings", { root }),
    writeAppTools: (tools) => invoke("write_app_tools", { root, tools }),
    writeWorkflowConfig: (fields) =>
      invoke("write_workflow_config", {
        root,
        locale: fields.locale,
        specLocale: fields.specLocale,
        tdd: fields.tdd,
        audit: fields.audit,
        worktree: fields.worktree,
      }),
    writeWorkflowContext: (context) =>
      invoke("write_workflow_content", { root, context, rules: null }),
    writeWorkflowRules: (rules) => invoke("write_workflow_content", { root, context: null, rules }),
    readSchemas: () => invoke("read_schemas", { root }),
    writeWorkflowSchema: (name) => invoke("write_workflow_schema", { root, name }),
    forkSchema: (source) => invoke("fork_schema", { root, source }),
    createSchema: (name) => invoke("init_schema", { root, name }),
    // reveal 以快照給的絕對路徑為準（user 層路徑前端拼不出來；design D6）。
    revealSchema: (path) => invoke("reveal_in_folder", { path }),
    deleteSchema: (name) => invoke("delete_schema", { root, name }),
  };
}

/** remote 設定面：locator 綁入閉包；read 保存 server revision，三種 targeted
 * write 都必須帶該 expectedRevision，成功後才前進本地 token。 */
export function createRemoteSettings(
  connectionId: string,
  project: string,
  repo: string,
  policyWrite: boolean,
  invoke: InvokeFn = tauriInvoke as InvokeFn,
): WorkspaceSettingsProvider {
  const scope = { connectionId, project, repo };
  let revision: number | null = null;

  const expectedRevision = () => {
    if (!policyWrite) throw new Error("你的角色為檢視者，只能查看此 Workflow 政策。");
    if (revision === null) throw new Error("尚未讀取 policy revision，無法儲存");
    return revision;
  };
  const adoptRevision = async (request: Promise<number>) => {
    const next = await request;
    revision = next;
    return next;
  };

  return {
    kind: "remote",
    policyWrite,
    readSettings: async () => {
      const snapshot = await invoke<SettingsSnapshot>("remote_read_settings", scope);
      revision = snapshot.workflow.revision ?? null;
      return snapshot;
    },
    // fork／建立／reveal／刪除在 remote 一律拒絕：能力邊界的顯性表達——UI 依
    // kind 不渲染入口，拒絕是第二道防線，非 Refused Bequest（介面成員對 remote
    // 語意上就是「尚不支援」，與 writeAppTools 同款）。
    writeAppTools: () => Promise.reject(new Error("remote Workflow 無 .speclink.yaml 工具設定")),
    writeWorkflowConfig: (fields) =>
      adoptRevision(
        invoke<number>("remote_write_workflow_config", {
          ...scope,
          locale: fields.locale,
          specLocale: fields.specLocale,
          tdd: fields.tdd,
          audit: fields.audit,
          expectedRevision: expectedRevision(),
        }),
      ),
    writeWorkflowContext: (context) =>
      adoptRevision(
        invoke<number>("remote_write_workflow_content", {
          ...scope,
          context,
          rules: null,
          expectedRevision: expectedRevision(),
        }),
      ),
    writeWorkflowRules: (rules) =>
      adoptRevision(
        invoke<number>("remote_write_workflow_content", {
          ...scope,
          context: null,
          rules,
          expectedRevision: expectedRevision(),
        }),
      ),
    // remote 產出流程清單：desktop core 以內嵌內建組裝（不打 server、不帶 root）。
    readSchemas: () => invoke("read_schemas", {}),
    writeWorkflowSchema: (name) =>
      adoptRevision(
        invoke<number>("remote_write_workflow_schema", {
          ...scope,
          name,
          expectedRevision: expectedRevision(),
        }),
      ),
    forkSchema: () => Promise.reject(new Error("遠端工作區尚不支援 fork 產出流程")),
    createSchema: () => Promise.reject(new Error("遠端工作區尚不支援建立產出流程")),
    revealSchema: () => Promise.reject(new Error("遠端工作區沒有本機檔案可顯示")),
    deleteSchema: () => Promise.reject(new Error("遠端工作區尚不支援刪除產出流程")),
  };
}

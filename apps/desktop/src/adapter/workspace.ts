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
 * 四態判定、目前產物層版號、逐工具狀態與將被新建或改寫的受管檔清單。 */
export interface InstructionProbeResult {
  status: "missing" | "stale" | "current" | "unknown";
  currentVersion: string;
  tools: Array<{
    tool: string;
    workspaceVersion: string | null;
    stale: boolean;
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
  };
}

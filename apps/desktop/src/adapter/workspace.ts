// workspace 管理操作的桌面 adapter（design D6；workspace-session 決策 3/4）：
// 直接 invoke 桌面專屬 command，不經 packages/ui 的 SpeclinkDataSource——該介面
// 是 change／spec 瀏覽管理抽象，專案與設定語意屬宿主。探測面（開啟／初始化／
// 統計）收路徑參數；設定面經 createWorkspaceSettings 將 root 綁入閉包，每支
// command 顯式帶 root。
import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

import type { InvokeFn, WorkspaceSettingsProvider } from "../session";

/** open_project／init_project 的三態判定 payload。 */
export type ProjectProbe =
  | { status: "project"; root: string; name: string }
  | { status: "uninitialized"; dir: string };

/** read_settings 的快照 payload（欄位值＋各檔可選的 parseError）。 */
export interface SettingsSnapshot {
  app: { tools: string[]; customTools: string[]; parseError: string | null };
  workflow: {
    locale: string | null;
    specLocale: string | null;
    tdd: boolean;
    audit: boolean;
    /** 專案說明現值（config.yaml 的 context；null＝未設定）。 */
    context: string | null;
    /** 產出規則現值（依 artifact id 分組；條目順序即檔案順序）。 */
    rules: Record<string, string[]>;
    /** 活躍 schema 的 artifact id（引擎顯示序）——產出規則分節的固定鍵。 */
    schemaArtifacts: string[];
    parseError: string | null;
  };
}

/** write_workflow_config 的完整目標狀態（null/false＝設回預設即移除鍵）。 */
export interface WorkflowFields {
  locale: string | null;
  specLocale: string | null;
  tdd: boolean;
  audit: boolean;
}

/** 探測面：開啟／初始化／統計皆收路徑參數、無全域語意（設定面見
 * createWorkspaceSettings，經 session 綁 root）。 */
export interface WorkspaceAdapter {
  openProject(path: string): Promise<ProjectProbe>;
  initProject(path: string, tools: string[]): Promise<ProjectProbe>;
  /** 啟動語境的預設目錄（首啟無持久化分頁時據此顯式開啟）；純讀。 */
  startupDir(): Promise<string>;
  projectStats(path: string): Promise<{ pendingWrapUp: number }>;
  /** 監看重掛（workspace-session 決策 5）：顯式跟隨活躍 session。 */
  watchWorkspace(root: string): Promise<void>;
  /** 原生資料夾選擇器；取消回 null。 */
  pickFolder(): Promise<string | null>;
}

export function createWorkspaceAdapter(): WorkspaceAdapter {
  return {
    openProject: (path) => tauriInvoke("open_project", { path }),
    initProject: (path, tools) => tauriInvoke("init_project", { path, tools }),
    startupDir: () => tauriInvoke("startup_dir"),
    projectStats: (path) => tauriInvoke("project_stats", { path }),
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
    readSettings: () => invoke("read_settings", { root }),
    writeAppTools: (tools) => invoke("write_app_tools", { root, tools }),
    writeWorkflowConfig: (fields) =>
      invoke("write_workflow_config", {
        root,
        locale: fields.locale,
        specLocale: fields.specLocale,
        tdd: fields.tdd,
        audit: fields.audit,
      }),
    writeWorkflowContext: (context) =>
      invoke("write_workflow_content", { root, context, rules: null }),
    writeWorkflowRules: (rules) => invoke("write_workflow_content", { root, context: null, rules }),
  };
}

// workspace 管理操作的桌面 adapter（design D6）：直接 invoke 七支桌面專屬
// command，不經 packages/ui 的 SpeclinkDataSource——該介面是 change／spec
// 瀏覽管理抽象，專案與設定語意屬宿主。
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

/** open_project／init_project 的三態判定 payload。 */
export type ProjectProbe =
  | { status: "project"; root: string; name: string }
  | { status: "uninitialized"; dir: string };

export interface ProjectInfo {
  root: string;
  name: string;
}

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

export interface WorkspaceAdapter {
  openProject(path: string): Promise<ProjectProbe>;
  initProject(path: string, tools: string[]): Promise<ProjectProbe>;
  currentProject(): Promise<ProjectInfo>;
  projectStats(path: string): Promise<{ inProgressChanges: number }>;
  /** 原生資料夾選擇器；取消回 null。 */
  pickFolder(): Promise<string | null>;
  readSettings(): Promise<SettingsSnapshot>;
  writeAppTools(tools: string[]): Promise<void>;
  writeWorkflowConfig(fields: WorkflowFields): Promise<void>;
  /** 寫入專案說明（config.yaml 的 context）；空字串＝清空即移除鍵。rules 不動。 */
  writeWorkflowContext(context: string): Promise<void>;
  /** 整份代換產出規則（節序即寫入序；空節移除鍵、全空移除 rules 鍵）。context 不動。 */
  writeWorkflowRules(rules: Array<[string, string[]]>): Promise<void>;
}

export function createWorkspaceAdapter(): WorkspaceAdapter {
  return {
    openProject: (path) => invoke("open_project", { path }),
    initProject: (path, tools) => invoke("init_project", { path, tools }),
    currentProject: () => invoke("current_project"),
    projectStats: (path) => invoke("project_stats", { path }),
    pickFolder: async () => {
      const picked = await open({ directory: true, multiple: false });
      return typeof picked === "string" ? picked : null;
    },
    readSettings: () => invoke("read_settings"),
    writeAppTools: (tools) => invoke("write_app_tools", { tools }),
    writeWorkflowConfig: (fields) =>
      invoke("write_workflow_config", {
        locale: fields.locale,
        specLocale: fields.specLocale,
        tdd: fields.tdd,
        audit: fields.audit,
      }),
    writeWorkflowContext: (context) =>
      invoke("write_workflow_content", { context, rules: null }),
    writeWorkflowRules: (rules) =>
      invoke("write_workflow_content", { context: null, rules }),
  };
}

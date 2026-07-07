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
  };
}

// WorkspaceSession 模型（workspace-session design 決策 1/3；架構 §10.4）：
// 分頁身分＝WorkspaceLocator、locatorKey 為去重／持久化 activeKey／tray 識別
// 的唯一身分函式。remote 變體本刀僅型別宣告、無任何建構路徑（後續刀），
// 先釘死完整型別使持久化 schema 與 key 規則跨後續刀穩定。
import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { listen as tauriListen } from "@tauri-apps/api/event";
import type { SpeclinkDataSource } from "@speclink/ui";

import { createTauriDataSource } from "./adapter/tauriDataSource";
import {
  createWorkspaceSettings,
  type SettingsSnapshot,
  type WorkflowFields,
} from "./adapter/workspace";

export type WorkspaceLocator =
  | { kind: "local"; root: string }
  | {
      kind: "remote";
      connectionId: string;
      projectId: string;
      repoId: string;
      checkoutRoot?: string;
    };

/** 分頁去重、持久化 activeKey 與 tray 選單識別的唯一身分（design 決策 1）：
 * local 為 local:{root}、remote 為 remote:{connectionId}/{projectId}/{repoId}
 * （checkoutRoot 屬工作副本位置、不參與身分）。 */
export function locatorKey(locator: WorkspaceLocator): string {
  return locator.kind === "local"
    ? `local:${locator.root}`
    : `remote:${locator.connectionId}/${locator.projectId}/${locator.repoId}`;
}

/** 顯示資訊（§10.4 descriptor）：承接現有分頁顯示欄位（name、badge）。 */
export interface WorkspaceDescriptor {
  name: string;
  /** 待收尾數；null＝尚未取得。 */
  badge: number | null;
}

/** 設定面（design 決策 3）：現 WorkspaceAdapter 設定面的 root 綁定版——
 * 方法不收 root，root 已綁入 session 閉包。 */
export interface WorkspaceSettingsProvider {
  readSettings(): Promise<SettingsSnapshot>;
  writeAppTools(tools: string[]): Promise<void>;
  writeWorkflowConfig(fields: WorkflowFields): Promise<void>;
  writeWorkflowContext(context: string): Promise<void>;
  writeWorkflowRules(rules: Array<[string, string[]]>): Promise<void>;
}

/** 事件面（design 決策 5）：workspace-changed 以自身 locator 過濾後才觸發。 */
export interface WorkspaceEvents {
  /** 訂閱過濾後的變更通知；回傳解除訂閱函式。 */
  subscribe(onChange: () => void): () => void;
}

/** WorkspaceSession（§10.4 全欄位）：id 以 locatorKey 衍生——同 locator 同
 * session（獨立 id 留給多視窗需求出現時）。 */
export interface WorkspaceSession {
  id: string;
  locator: WorkspaceLocator;
  descriptor: WorkspaceDescriptor;
  dataSource: SpeclinkDataSource;
  settings: WorkspaceSettingsProvider;
  events: WorkspaceEvents;
}

export type InvokeFn = <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
export type ListenFn = (
  event: string,
  handler: (e: { payload: unknown }) => void,
) => Promise<() => void>;

export interface LocalSessionDeps {
  /** 測試注入假 invoke；預設 Tauri invoke。 */
  invoke?: InvokeFn;
  /** 測試注入假 listen；預設 Tauri listen。 */
  listen?: ListenFn;
  /** 顯示名（探測回報）；未給時以 root 末段推導。 */
  name?: string;
}

/** root 末段作為預設顯示名（Windows 反斜線與 POSIX 斜線皆可）。 */
function basenameOf(root: string): string {
  const trimmed = root.replace(/[\\/]+$/, "");
  return trimmed.split(/[\\/]/).pop() || root;
}

/** local session 工廠（design 決策 3）：root 綁入 dataSource／settings 閉包，
 * events 訂閱 workspace-changed 並以自身 root 過濾 payload（決策 5）——
 * 非活躍 session 的訂閱者天然收不到（watcher 只掛在活躍 root）。 */
export function createLocalSession(root: string, deps: LocalSessionDeps = {}): WorkspaceSession {
  const invoke = deps.invoke ?? (tauriInvoke as InvokeFn);
  const listen = deps.listen ?? (tauriListen as unknown as ListenFn);
  const locator: WorkspaceLocator = { kind: "local", root };
  return {
    id: locatorKey(locator),
    locator,
    descriptor: { name: deps.name ?? basenameOf(root), badge: null },
    dataSource: createTauriDataSource(root, invoke),
    settings: createWorkspaceSettings(root, invoke),
    events: {
      subscribe(onChange) {
        const unlisten = listen("workspace-changed", (e) => {
          if (e.payload === root) onChange();
        });
        return () => {
          void unlisten.then((f) => f());
        };
      },
    },
  };
}

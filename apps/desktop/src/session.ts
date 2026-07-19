// WorkspaceSession 模型（workspace-session design 決策 1/3；架構 §10.4）：
// 分頁身分＝WorkspaceLocator、locatorKey 為去重／持久化 activeKey／tray 識別
// 的唯一身分函式。remote 變體本刀僅型別宣告、無任何建構路徑（後續刀），
// 先釘死完整型別使持久化 schema 與 key 規則跨後續刀穩定。
import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { listen as tauriListen } from "@tauri-apps/api/event";
import type { SpeclinkDataSource } from "@speclink/ui";

import { createRemoteDataSource } from "./adapter/remoteDataSource";
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

/** 逐操作的 capability 描述（remote-data-source 決策 2）：UI 據此停用
 * affordance——欄位鏡射 Rust RemoteCapabilities 的 camelCase 序列化。
 * 本地 session 全真、同一 UI 路徑零分岐維護。 */
export interface WorkspaceCapabilities {
  listChanges: boolean;
  listSpecs: boolean;
  listArchived: boolean;
  status: boolean;
  getDocument: boolean;
  getSpecDocument: boolean;
  searchWorkspace: boolean;
  changeCapabilities: boolean;
  changeMeta: boolean;
  deleteChange: boolean;
  setTaskDone: boolean;
  setAllTasks: boolean;
  moveTask: boolean;
  validate: boolean;
  analyze: boolean;
  archive: boolean;
  getArchivedDocument: boolean;
  archivedCapabilities: boolean;
  listDiscussions: boolean;
  getDiscussionDocument: boolean;
  promoteDiscussion: boolean;
  archiveDiscussion: boolean;
  reorderCard: boolean;
  /** server 是否宣告事件能力（SSE／polling）；缺席時退化為手動重整。 */
  liveUpdates: boolean;
}

/** 本地 session 的 capability 描述：全真（決策 2）。 */
export const LOCAL_CAPABILITIES: WorkspaceCapabilities = {
  listChanges: true,
  listSpecs: true,
  listArchived: true,
  status: true,
  getDocument: true,
  getSpecDocument: true,
  searchWorkspace: true,
  changeCapabilities: true,
  changeMeta: true,
  deleteChange: true,
  setTaskDone: true,
  setAllTasks: true,
  moveTask: true,
  validate: true,
  analyze: true,
  archive: true,
  getArchivedDocument: true,
  archivedCapabilities: true,
  listDiscussions: true,
  getDiscussionDocument: true,
  promoteDiscussion: true,
  archiveDiscussion: true,
  reorderCard: true,
  liveUpdates: true,
};

/** WorkspaceSession（§10.4 全欄位）：id 以 locatorKey 衍生——同 locator 同
 * session（獨立 id 留給多視窗需求出現時）。 */
export interface WorkspaceSession {
  id: string;
  locator: WorkspaceLocator;
  descriptor: WorkspaceDescriptor;
  dataSource: SpeclinkDataSource;
  settings: WorkspaceSettingsProvider;
  events: WorkspaceEvents;
  capabilities: WorkspaceCapabilities;
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
    capabilities: LOCAL_CAPABILITIES,
  };
}

/** remote_open 的回傳 payload（Rust RemoteOpenInfo 的 camelCase 序列化）：
 * handshake 裁定後的 project/repo 識別、顯示名與 capability 描述。 */
export interface RemoteOpenInfo {
  projectKey: string;
  projectName: string;
  repoKey: string;
  repoName: string;
  capabilities: WorkspaceCapabilities;
}

export interface RemoteSessionDeps {
  invoke?: InvokeFn;
  listen?: ListenFn;
}

/** remote 設定面：server 無設定端點——每個方法一致回拒絕（決策 1 (c)）。 */
function remoteSettingsStub(): WorkspaceSettingsProvider {
  const refuse = () =>
    Promise.reject(new Error("此 server 尚未提供「workspace 設定」——功能已停用"));
  return {
    readSettings: refuse,
    writeAppTools: refuse,
    writeWorkflowConfig: refuse,
    writeWorkflowContext: refuse,
    writeWorkflowRules: refuse,
  };
}

/** remote session 工廠（決策 6/7）：以 remote_open（handshake）的結果建
 * session——handshake 成功是建立前置，這裡不再打 server。dataSource 為
 * remote_* 的薄 invoke 包裝；事件面訂閱時掛 remote_watch（Rust 端同
 * connection 同 scope 共用單流）、以 locator key 過濾 payload，解除時
 * remote_unwatch。 */
export function createRemoteSession(
  connectionId: string,
  info: RemoteOpenInfo,
  deps: RemoteSessionDeps = {},
): WorkspaceSession {
  const invoke = deps.invoke ?? (tauriInvoke as InvokeFn);
  const listen = deps.listen ?? (tauriListen as unknown as ListenFn);
  const locator: WorkspaceLocator = {
    kind: "remote",
    connectionId,
    projectId: info.projectKey,
    repoId: info.repoKey,
  };
  const key = locatorKey(locator);
  const watchArgs = { connectionId, project: info.projectKey, repo: info.repoKey };
  return {
    id: key,
    locator,
    descriptor: { name: `${info.projectName}/${info.repoName}`, badge: null },
    dataSource: createRemoteDataSource(connectionId, info.projectKey, info.repoKey, invoke),
    settings: remoteSettingsStub(),
    events: {
      subscribe(onChange) {
        void invoke("remote_watch", { ...watchArgs });
        const unlisten = listen("remote-workspace-changed", (e) => {
          if (e.payload === key) onChange();
        });
        return () => {
          void unlisten.then((f) => f());
          void invoke("remote_unwatch", { ...watchArgs });
        };
      },
    },
    capabilities: info.capabilities,
  };
}

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
  createRemoteSettings,
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
  kind: "local" | "remote";
  /** remote 取自 handshake；local 固定為 true。server 仍是最終防線。 */
  policyWrite: boolean;
  readSettings(): Promise<SettingsSnapshot>;
  writeAppTools(tools: string[]): Promise<void>;
  /** remote 成功回新 revision；local 維持 void。 */
  writeWorkflowConfig(fields: WorkflowFields): Promise<number | void>;
  writeWorkflowContext(context: string): Promise<number | void>;
  writeWorkflowRules(rules: Array<[string, string[]]>): Promise<number | void>;
}

/** 事件面（design 決策 5）：workspace-changed 以自身 locator 過濾後才觸發。 */
export interface WorkspaceEvents {
  /** 訂閱過濾後的變更通知；回傳解除訂閱函式。 */
  subscribe(onChange: () => void, onConnectionState?: (event: RemoteConnectionStateEvent) => void): () => void;
}

export type RemoteConnectionState = "online" | "offline" | "needs-reauth";

export interface RemoteConnectionStateEvent {
  connectionId: string;
  state: RemoteConnectionState;
  message: string | null;
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
  /** remote membership 是否可寫 policy；local 固定為 true。 */
  policyWrite: boolean;
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
  policyWrite: true,
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
  /** handshake 原始 capability；offline mask 解除時由此精確還原。 */
  baseCapabilities?: WorkspaceCapabilities;
  /** local 為缺席；remote 初始 online，之後只接受 Rust 事件更新。 */
  connectionState?: RemoteConnectionStateEvent;
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

export type RemoteRecoveryKind =
  | "unreachable"
  | "needs-reauth"
  | "access-denied"
  | "not-found"
  | "unknown";

export interface RemoteOpenFailure {
  kind: RemoteRecoveryKind;
  message: string;
  reason: string | null;
  status: number | null;
}

export type RemoteWorkspaceRecoveryState =
  | { status: "restoring"; failure: null }
  | { status: "error"; failure: RemoteOpenFailure };

export type RemoteWorkspaceStatus =
  | "ready"
  | "restoring"
  | "offline"
  | "needs-reauth"
  | "error";

/** 主視窗 tab、復原頁與 Tray 共用的單一狀態裁決。無 session 且無明確
 * recovery state 是不完整狀態，安全降階為 error，絕不偽裝成 ready。 */
export function remoteWorkspaceStatus(
  session: WorkspaceSession | undefined,
  recovery: RemoteWorkspaceRecoveryState | undefined,
): RemoteWorkspaceStatus {
  if (recovery) return recovery.status;
  if (!session) return "error";
  if (session.locator.kind !== "remote") return "ready";
  const connectionState = session.connectionState?.state ?? "online";
  return connectionState === "online" ? "ready" : connectionState;
}

function structuredRemoteOpenFailure(
  error: unknown,
): Omit<RemoteOpenFailure, "kind"> | null {
  if (!error || typeof error !== "object") return null;
  const candidate = error as Record<string, unknown>;
  if (typeof candidate.message !== "string") return null;
  const reason = candidate.reason;
  if (reason !== null && typeof reason !== "string") return null;
  const status = candidate.status;
  if (
    status !== null &&
    (typeof status !== "number" || !Number.isInteger(status) || status < 100 || status > 599)
  ) {
    return null;
  }
  return {
    message: candidate.message,
    reason,
    status,
  };
}

/** 將 Tauri object rejection 與舊版 string/Error rejection 收斂成封閉分類。
 * `message` 僅保留作 technical detail，分類永不比對其文字。 */
export function normalizeRemoteOpenFailure(error: unknown): RemoteOpenFailure {
  if (error instanceof RemoteOpenError) return error.failure;
  const failure = structuredRemoteOpenFailure(error);
  if (!failure) {
    return {
      kind: "unknown",
      message: error instanceof Error ? error.message : String(error),
      reason: null,
      status: null,
    };
  }

  let kind: RemoteRecoveryKind = "unknown";
  if (failure.status === 401 || failure.reason === "needs_reauth") {
    kind = "needs-reauth";
  } else if (failure.status === 403) {
    kind = "access-denied";
  } else if (failure.status === 404 || failure.reason === "not_found") {
    kind = "not-found";
  } else if (
    failure.reason === "unavailable" ||
    failure.reason === "offline" ||
    (failure.reason === null && failure.status === null) ||
    (failure.status !== null && failure.status >= 500)
  ) {
    kind = "unreachable";
  }
  return { kind, ...failure };
}

export class RemoteOpenError extends Error {
  readonly failure: RemoteOpenFailure;

  constructor(failure: RemoteOpenFailure) {
    super(failure.message);
    this.name = "RemoteOpenError";
    this.failure = failure;
  }
}

export interface RemoteSessionDeps {
  invoke?: InvokeFn;
  listen?: ListenFn;
}

const REMOTE_WRITE_CAPABILITIES: ReadonlyArray<keyof WorkspaceCapabilities> = [
  "deleteChange",
  "setTaskDone",
  "setAllTasks",
  "moveTask",
  "archive",
  "promoteDiscussion",
  "archiveDiscussion",
  "reorderCard",
  "policyWrite",
];

function maskRemoteWrites(capabilities: WorkspaceCapabilities): WorkspaceCapabilities {
  const masked = { ...capabilities };
  for (const key of REMOTE_WRITE_CAPABILITIES) masked[key] = false;
  return masked;
}

export function applyRemoteConnectionState(
  session: WorkspaceSession,
  event: RemoteConnectionStateEvent,
): WorkspaceSession {
  if (session.locator.kind !== "remote" || session.locator.connectionId !== event.connectionId) {
    return session;
  }
  const baseCapabilities = session.baseCapabilities ?? session.capabilities;
  const writable = event.state === "online";
  return {
    ...session,
    baseCapabilities,
    connectionState: event,
    capabilities: writable ? baseCapabilities : maskRemoteWrites(baseCapabilities),
    settings: {
      ...session.settings,
      policyWrite: writable ? baseCapabilities.policyWrite : false,
    },
  };
}

function isRemoteConnectionStateEvent(payload: unknown): payload is RemoteConnectionStateEvent {
  if (!payload || typeof payload !== "object") return false;
  const event = payload as Partial<RemoteConnectionStateEvent>;
  return (
    typeof event.connectionId === "string" &&
    (event.state === "online" || event.state === "offline" || event.state === "needs-reauth") &&
    (event.message === null || typeof event.message === "string")
  );
}

/** remote session 工廠（決策 6/7）：以 remote_open（handshake）的結果建
 * session——handshake 成功是建立前置，這裡不再打 server。dataSource 為
 * remote_* 的薄 invoke 包裝；事件面訂閱時掛 remote_watch（Rust 端同
 * connection 同 scope 共用單流）、以 locator key 過濾 payload，解除時
 * remote_unwatch。 */
export function createRemoteSession(
  connectionId: string,
  info: RemoteOpenInfo,
  checkoutRoot?: string,
  deps: RemoteSessionDeps = {},
): WorkspaceSession {
  const invoke = deps.invoke ?? (tauriInvoke as InvokeFn);
  const listen = deps.listen ?? (tauriListen as unknown as ListenFn);
  const locator: WorkspaceLocator = {
    kind: "remote",
    connectionId,
    projectId: info.projectKey,
    repoId: info.repoKey,
    ...(checkoutRoot ? { checkoutRoot } : {}),
  };
  const key = locatorKey(locator);
  const watchArgs = { connectionId, project: info.projectKey, repo: info.repoKey };
  return {
    id: key,
    locator,
    descriptor: { name: `${info.projectName}/${info.repoName}`, badge: null },
    dataSource: createRemoteDataSource(connectionId, info.projectKey, info.repoKey, invoke),
    settings: createRemoteSettings(
      connectionId,
      info.projectKey,
      info.repoKey,
      info.capabilities.policyWrite,
      invoke,
    ),
    events: {
      subscribe(onChange, onConnectionState) {
        void invoke("remote_watch", { ...watchArgs });
        const unlistenWorkspace = listen("remote-workspace-changed", (e) => {
          if (e.payload === key) onChange();
        });
        const unlistenState = listen("remote-connection-state", (e) => {
          if (
            isRemoteConnectionStateEvent(e.payload) &&
            e.payload.connectionId === connectionId
          ) {
            onConnectionState?.(e.payload);
          }
        });
        return () => {
          void unlistenWorkspace.then((f) => f());
          void unlistenState.then((f) => f());
          void invoke("remote_unwatch", { ...watchArgs });
        };
      },
    },
    capabilities: info.capabilities,
    baseCapabilities: info.capabilities,
    connectionState: { connectionId, state: "online", message: null },
  };
}

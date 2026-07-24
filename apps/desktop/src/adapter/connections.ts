// server 連線的桌面 adapter（desktop-connections；design 決策 2/5/7）：
// 清單/新增/登入/登出/移除的 invoke 包裝。型別面無任何 secret 欄位——
// credential 唯一落點是 Rust 側的 OS Keychain，TS 只見連線狀態與身分顯示名；
// PAT 僅作 pat_login 的參數單次過境，不回讀、不入狀態。
import { invoke as tauriInvoke } from "@tauri-apps/api/core";

import type { InvokeFn } from "../session";

/** 連線條目的 TS 檢視：registry 欄位＋由 Keychain 推導的登入狀態。 */
export interface ConnectionView {
  id: string;
  origin: string;
  name: string;
  /** 最後登入身分的顯示名；未登入過即缺席。 */
  lastActorDisplay?: string | null;
  loggedIn: boolean;
}

/** device_login 的可讀結果（規格「device login 預設與 PAT fallback」）：
 * unsupported＝明確不支援（404/405），觸發 PAT fallback；連線錯誤走 reject。 */
export type DeviceLoginResult =
  | { status: "loggedIn"; display: string }
  | { status: "unsupported" }
  | { status: "denied" }
  | { status: "expired" };

/** 登出結果：撤銷為盡力語意；patNotice＝請至伺服器帳號頁撤銷 PAT。 */
export interface LogoutResult {
  revokedOnServer: boolean;
  patNotice: boolean;
}

export interface ScopeRefView {
  id: string;
  key: string;
  name: string;
}

export interface ProjectScopeView extends ScopeRefView {
  repos: ScopeRefView[];
}

export interface ScopesView {
  projects: ProjectScopeView[];
}

/** inspect_checkout 的零寫入結果：確認過的 checkout 根路徑與要預選的既有 built-in
 * 工具選集（僅 claude／codex）。不含任何 credential 或 Server 資料。 */
export interface CheckoutInspection {
  root: string;
  tools: string[];
}

export interface ConnectionsAdapter {
  list(): Promise<ConnectionView[]>;
  add(baseUrl: string, name: string): Promise<ConnectionView>;
  /** 移除連線＝Rust 側先走登出語意再刪條目（決策 6）。 */
  remove(id: string): Promise<void>;
  deviceLogin(origin: string): Promise<DeviceLoginResult>;
  /** PAT 單次過境：呼叫後 TS 不保留任何拷貝。 */
  patLogin(origin: string, pat: string): Promise<{ status: "loggedIn"; display: string }>;
  logout(origin: string): Promise<LogoutResult>;
  /** 登入者 membership 過濾後的 Project／Repo 清單。 */
  scopes(connectionId: string): Promise<ScopesView>;
  /** 先檢查階段（零寫入）：驗證 checkout marker 一致性並回報要預選的既有工具選集。 */
  inspectCheckout(
    path: string,
    origin: string,
    project: string,
    repo: string,
  ): Promise<CheckoutInspection>;
  /** 提交階段：重做驗證、無 marker 時寫入 marker，再同步所選 built-in 工具的受管產物。 */
  bindCheckout(
    path: string,
    origin: string,
    project: string,
    repo: string,
    tools: string[],
  ): Promise<string>;
}

export function createConnectionsAdapter(
  invoke: InvokeFn = tauriInvoke as InvokeFn,
): ConnectionsAdapter {
  return {
    list: () => invoke("connection_list"),
    add: (baseUrl, name) => invoke("connection_add", { baseUrl, name }),
    remove: (id) => invoke("connection_remove", { id }),
    deviceLogin: (origin) => invoke("device_login", { origin }),
    patLogin: (origin, pat) => invoke("pat_login", { origin, pat }),
    logout: (origin) => invoke("connection_logout", { origin }),
    scopes: (connectionId) => invoke("remote_scopes", { connectionId }),
    inspectCheckout: (path, origin, project, repo) =>
      invoke("inspect_checkout", { path, origin, project, repo }),
    bindCheckout: (path, origin, project, repo, tools) =>
      invoke("bind_checkout", { path, origin, project, repo, tools }),
  };
}

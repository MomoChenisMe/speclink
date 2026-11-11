// server 連線的桌面 adapter（desktop-connections；client-browser-login D7、D9）：
// 清單/新增/登入/登出/移除的 invoke 包裝。型別面無任何 secret 欄位——
// credential 唯一落點是 Rust 側的 OS Keychain，TS 只見連線狀態與身分顯示名；
// PAT 僅作 pat_login 的參數單次過境，不回讀、不入狀態。登入的等待迴圈在 Rust：
// 這裡只轉送它送來的階段事件，並把終態交回呼叫端。
import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { listen as tauriListen } from "@tauri-apps/api/event";

import type { InvokeFn, ListenFn } from "../session";

/** Rust 送來登入階段事件的名稱。 */
const LOGIN_PHASE_EVENT = "connection-login-phase";

/** 連線條目的 TS 檢視：registry 欄位＋由 Keychain 推導的登入狀態。 */
export interface ConnectionView {
  id: string;
  origin: string;
  name: string;
  /** 最後登入身分的顯示名；未登入過即缺席。 */
  lastActorDisplay?: string | null;
  loggedIn: boolean;
}

/** 伺服器頁「預設」卡的有效預設連線：存在且已登入才算，否則等同不預選（null）。 */
export function defaultConnectionOf(
  connections: readonly ConnectionView[],
  defaultConnectionId: string | null | undefined,
): ConnectionView | null {
  return connections.find((entry) => entry.id === defaultConnectionId && entry.loggedIn) ?? null;
}

/** 登入等待中的階段（Rust 經事件送來）：`kind` 與欄位和 store 的
 * `ConnectionPhase` 一致。expiresAt＝截止時刻（epoch 毫秒），倒數依它計算。
 * 授權網址只帶 state 與 PKCE challenge，不含授權碼、verifier 或任何 token。 */
export type LoginPhase =
  | { kind: "awaitingBrowser"; url: string; expiresAt: number }
  | { kind: "awaitingApproval"; userCode: string; verificationUri: string; expiresAt: number };

/** 一次登入的終態（規格「device login 預設與 PAT fallback」）：patRequired＝server
 * 只收存取金鑰或沒有 device flow，就地改收 PAT；連線錯誤走 reject。 */
export type LoginResult =
  | { status: "loggedIn"; display: string }
  | { status: "patRequired" }
  | { status: "denied" }
  | { status: "expired" }
  | { status: "cancelled" };

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
 * 工具選集（claude／codex／copilot／pi／cursor）。不含任何 credential 或 Server 資料。 */
export interface CheckoutInspection {
  root: string;
  tools: string[];
}

export interface ConnectionsAdapter {
  list(): Promise<ConnectionView[]>;
  add(baseUrl: string, name: string): Promise<ConnectionView>;
  /** 移除連線＝Rust 側先走登出語意再刪條目（決策 6）。 */
  remove(id: string): Promise<void>;
  /** 登入到終態才 resolve：先試靜默換新，否則依 server 宣告的方式登入；途中的
   * 等待階段（只屬於這個 origin）經 onPhase 送來。 */
  loginStart(origin: string, onPhase: (phase: LoginPhase) => void): Promise<LoginResult>;
  /** 取消等待中的登入：Rust 結束等待、loginStart 以 cancelled 收場。 */
  loginCancel(origin: string): Promise<void>;
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
  listen: ListenFn = tauriListen as unknown as ListenFn,
): ConnectionsAdapter {
  return {
    list: () => invoke("connection_list"),
    add: (baseUrl, name) => invoke("connection_add", { baseUrl, name }),
    remove: (id) => invoke("connection_remove", { id }),
    async loginStart(origin, onPhase) {
      // 先掛好監聽再起登入：Rust 一開始就可能送出等待階段。
      const unlisten = await listen(LOGIN_PHASE_EVENT, (e) => {
        const { origin: from, ...phase } = e.payload as LoginPhase & { origin: string };
        if (from === origin) onPhase(phase as LoginPhase);
      });
      try {
        return await invoke<LoginResult>("connection_login_start", { origin });
      } finally {
        unlisten();
      }
    },
    loginCancel: (origin) => invoke("connection_login_cancel", { origin }),
    patLogin: (origin, pat) => invoke("pat_login", { origin, pat }),
    logout: (origin) => invoke("connection_logout", { origin }),
    scopes: (connectionId) => invoke("remote_scopes", { connectionId }),
    inspectCheckout: (path, origin, project, repo) =>
      invoke("inspect_checkout", { path, origin, project, repo }),
    bindCheckout: (path, origin, project, repo, tools) =>
      invoke("bind_checkout", { path, origin, project, repo, tools }),
  };
}

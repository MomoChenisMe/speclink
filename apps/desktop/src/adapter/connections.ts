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

export interface ConnectionsAdapter {
  list(): Promise<ConnectionView[]>;
  add(baseUrl: string, name: string): Promise<ConnectionView>;
  /** 移除連線＝Rust 側先走登出語意再刪條目（決策 6）。 */
  remove(id: string): Promise<void>;
  deviceLogin(origin: string): Promise<DeviceLoginResult>;
  /** PAT 單次過境：呼叫後 TS 不保留任何拷貝。 */
  patLogin(origin: string, pat: string): Promise<{ status: "loggedIn"; display: string }>;
  logout(origin: string): Promise<LogoutResult>;
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
  };
}

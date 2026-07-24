import { createContext, useContext, type ReactNode } from "react";
import type { SessionData, WebClient } from "../api/client";

// client 是 HTTP adapter（非 server 資料），放 context 供全 app 使用（D1 允許：
// 「跨路由且非伺服器真相」）。session 是 App 開機讀取一次的角色真相，供導覽與
// guard 使用；login／logout 後 reload。

const ClientContext = createContext<WebClient | null>(null);

export function ClientProvider({ client, children }: { client: WebClient; children: ReactNode }) {
  return <ClientContext.Provider value={client}>{children}</ClientContext.Provider>;
}

export function useClient(): WebClient {
  const client = useContext(ClientContext);
  if (!client) throw new Error("useClient must be used within ClientProvider");
  return client;
}

// refresh 重新讀取 session 並在完成後 resolve——login／logout 後須等它完成再導向，
// 否則 guard 讀到舊 session 會把使用者彈回登入。
export type SessionValue = { session: SessionData; refresh: () => Promise<void> };

const SessionContext = createContext<SessionValue | null>(null);

export function SessionProvider({ value, children }: { value: SessionValue; children: ReactNode }) {
  return <SessionContext.Provider value={value}>{children}</SessionContext.Provider>;
}

export function useSession(): SessionValue {
  const value = useContext(SessionContext);
  if (!value) throw new Error("useSession must be used within SessionProvider");
  return value;
}

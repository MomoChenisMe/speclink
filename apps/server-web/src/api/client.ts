// 唯一 raw HTTP 呼叫入口（D2 Implementation Contract）：route／page 只呼叫其
// typed operations，絕不散落 fetch。成功回 `{data}`、失敗拋 WebApiError；同源
// cookie 由瀏覽器隨附。

export type SessionUser = {
  id: string;
  email: string;
  display: string;
  admin: boolean;
};

export type SessionData = {
  authenticated: boolean;
  user: SessionUser | null;
  home: string;
};

export type LoginBody = {
  email: string;
  password: string;
  userCode?: string;
  returnTo?: string;
};

export type AdminOverview = {
  activeUsers: number;
  suspendedUsers: number;
  projects: number;
  repos: number;
  activeCredentials: number;
  storeHealthy: boolean;
  storeHealthError?: string;
  identitySchemaVersion: number;
};

/** The setup store-status panel (四要素之二). */
export type StoreStatus = {
  driver: string;
  contractVersion: number;
  level: string;
  capabilities: string[];
  healthy: boolean;
  healthError?: string;
  identitySchemaVersion: number;
};

/** The current setup step and store status. */
export type SetupState = {
  step: "admin" | "registry";
  store: StoreStatus;
};

export type SetupAdminBody = { email: string; display: string; password: string };
export type SetupRegistryBody = {
  projectKey: string;
  projectName?: string;
  repoKey: string;
  repoName?: string;
};

/** The connection info shown once setup completes. */
export type SetupComplete = {
  destination: string;
  connection: { publicUrl: string; projectKey: string; repoKey: string };
};

/** The non-secret invitation summary for the set-password form. */
export type InvitationSummary = { email: string; display: string; admin: boolean };

/** A browser-API failure carrying the `{error}` envelope's fields. */
export class WebApiError extends Error {
  constructor(
    public status: number,
    public code: string,
    message: string,
    public fieldErrors?: Record<string, string>,
  ) {
    super(message);
    this.name = "WebApiError";
  }
}

export interface WebClient {
  getSession(): Promise<SessionData>;
  login(body: LoginBody): Promise<{ destination: string }>;
  logout(): Promise<{ destination: string }>;
  getAdminOverview(): Promise<AdminOverview>;
  getSetupState(token: string): Promise<SetupState>;
  submitSetupAdmin(token: string, body: SetupAdminBody): Promise<SetupState>;
  submitSetupRegistry(token: string, body: SetupRegistryBody): Promise<SetupComplete>;
  getInvitation(token: string): Promise<InvitationSummary>;
  acceptInvitation(token: string, body: { password: string }): Promise<{ destination: string }>;
}

const BASE = "/api/speclink/v1/web";

async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
  const resp = await fetch(`${BASE}${path}`, {
    method,
    credentials: "same-origin",
    headers: body !== undefined ? { "Content-Type": "application/json" } : {},
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });
  const payload = (await resp.json().catch(() => ({}))) as {
    data?: T;
    error?: { code?: string; message?: string; fieldErrors?: Record<string, string> };
  };
  if (!resp.ok) {
    const err = payload.error ?? {};
    throw new WebApiError(resp.status, err.code ?? "error", err.message ?? "發生錯誤", err.fieldErrors);
  }
  return payload.data as T;
}

/** The production client — every method is a typed call over the same-origin API. */
export function createHttpClient(): WebClient {
  const q = (token: string) => `?token=${encodeURIComponent(token)}`;
  return {
    getSession: () => request("GET", "/session"),
    login: (body) => request("POST", "/login", body),
    logout: () => request("POST", "/logout", {}),
    getAdminOverview: () => request("GET", "/admin/overview"),
    getSetupState: (token) => request("GET", `/setup${q(token)}`),
    submitSetupAdmin: (token, body) => request("POST", `/setup/admin${q(token)}`, body),
    submitSetupRegistry: (token, body) => request("POST", `/setup/registry${q(token)}`, body),
    getInvitation: (token) => request("GET", `/invite/${encodeURIComponent(token)}`),
    acceptInvitation: (token, body) => request("POST", `/invite/${encodeURIComponent(token)}`, body),
  };
}

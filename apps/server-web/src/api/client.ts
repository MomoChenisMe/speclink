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

export type AdminConnection = { publicUrl: string; projectKey: string; repoKey: string };

export type AdminOverview = {
  activeUsers: number;
  suspendedUsers: number;
  projects: number;
  repos: number;
  activeCredentials: number;
  storeHealthy: boolean;
  storeHealthError?: string;
  identitySchemaVersion: number;
  connection?: AdminConnection;
};

export type AdminMembership = { projectKey: string; role: string };
export type AdminUser = {
  id: string;
  email: string;
  display: string;
  admin: boolean;
  active: boolean;
  memberships: AdminMembership[];
  canSuspend: boolean;
  canRemoveAdmin: boolean;
};
export type AdminUsers = { users: AdminUser[] };

export type AdminRepo = { key: string; name: string };
export type AdminProject = { key: string; name: string; repos: AdminRepo[] };
export type AdminRegistry = { projects: AdminProject[] };

export type AdminPat = {
  id: string;
  userId: string;
  prefix: string;
  name: string;
  createdAt: string;
  expiresAt: string | null;
  lastUsedAt: string | null;
  revokedAt: string | null;
};
export type AdminCredFamily = {
  id: string;
  userId: string;
  source: string;
  createdAt: string;
  lastRefreshAt: string;
  revokedAt: string | null;
};
export type AdminCredentials = { pats: AdminPat[]; deviceFamilies: AdminCredFamily[] };

export type AdminScope = { project: string; repo: string; exportPath: string };
export type AdminData = { scopes: AdminScope[]; storeHealthy: boolean; storeHealthError?: string };

export type AdminBacklog = { project: string; repo: string; backlog: number | null };
export type AdminSystem = {
  engineVersion: string;
  apiVersion: string;
  identitySchemaVersion: number | null;
  storeDriver: string;
  storeContractVersion: number;
  storeLevel: string;
  storeCapabilities: string[];
  storeHealthy: boolean;
  storeHealthError: string | null;
  outboxBacklogs: AdminBacklog[];
};

export type AdminAuditEntry = {
  id: string;
  actorId: string;
  action: string;
  subject: string;
  source: string;
  createdAt: string;
};
export type AdminAudit = { entries: AdminAuditEntry[] };

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

/** A PAT's non-secret metadata (prefix, never the plaintext or hash). */
export type PatMeta = {
  id: string;
  prefix: string;
  name: string;
  createdAt: string;
  expiresAt: string | null;
  lastUsedAt: string | null;
  revokedAt: string | null;
};

/** A Web session's metadata (the id is a metadata id, not the cookie secret). */
export type SessionMeta = {
  id: string;
  createdAt: string;
  expiresAt: string;
  revokedAt: string | null;
};

/** A device credential family's metadata (never the refresh credential). */
export type DeviceFamilyMeta = {
  id: string;
  source: string;
  createdAt: string;
  lastRefreshAt: string;
  revokedAt: string | null;
};

/** The account self-service summary: own user plus credential metadata. */
export type AccountSummary = {
  user: SessionUser;
  pats: PatMeta[];
  sessions: SessionMeta[];
  deviceFamilies: DeviceFamilyMeta[];
};

/** A freshly-created PAT: metadata plus the one-time plaintext. */
export type PatCreated = { pat: PatMeta; plaintext: string };

/** The device activation outcome the SPA reflects. */
export type ActivateResult = { status: "pending" | "approved" | "denied" };

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
  getAccount(): Promise<AccountSummary>;
  createPat(body: { name: string; expires?: string }): Promise<PatCreated>;
  revokePat(id: string): Promise<void>;
  revokeDevice(id: string): Promise<void>;
  checkActivation(userCode: string): Promise<ActivateResult>;
  decideActivation(userCode: string, action: "approve" | "deny"): Promise<ActivateResult>;
  getAdminUsers(): Promise<AdminUsers>;
  getAdminRegistry(): Promise<AdminRegistry>;
  getAdminCredentials(): Promise<AdminCredentials>;
  getAdminData(): Promise<AdminData>;
  getAdminSystem(): Promise<AdminSystem>;
  getAdminAudit(): Promise<AdminAudit>;
  adminInvite(body: {
    email: string;
    display: string;
    memberships: string[];
    admin: boolean;
  }): Promise<{ token: string }>;
  adminSuspend(id: string): Promise<void>;
  adminReactivate(id: string): Promise<void>;
  adminSetMembership(
    id: string,
    body: { projectKey: string; role: string; member: boolean },
  ): Promise<void>;
  adminSetAdminFlag(id: string, admin: boolean): Promise<void>;
  adminCreateProject(body: { key: string; name?: string }): Promise<void>;
  adminRenameProject(key: string, name: string): Promise<void>;
  adminCreateRepo(body: { projectKey: string; key: string; name?: string }): Promise<void>;
  adminRenameRepo(body: { projectKey: string; key: string; name: string }): Promise<void>;
  adminRevokeToken(id: string): Promise<void>;
  adminRevokeFamily(id: string): Promise<void>;
  adminMigrate(): Promise<void>;
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
    getAccount: () => request("GET", "/account"),
    createPat: (body) => request("POST", "/account/tokens", body),
    revokePat: (id) => request("POST", `/account/tokens/${encodeURIComponent(id)}/revoke`, {}),
    revokeDevice: (id) => request("POST", `/account/devices/${encodeURIComponent(id)}/revoke`, {}),
    checkActivation: (userCode) => request("POST", "/activate", { userCode }),
    decideActivation: (userCode, action) => request("POST", "/activate", { userCode, action }),
    getAdminUsers: () => request("GET", "/admin/users"),
    getAdminRegistry: () => request("GET", "/admin/registry"),
    getAdminCredentials: () => request("GET", "/admin/credentials"),
    getAdminData: () => request("GET", "/admin/data"),
    getAdminSystem: () => request("GET", "/admin/system"),
    getAdminAudit: () => request("GET", "/admin/audit"),
    adminInvite: (body) => request("POST", "/admin/users/invite", body),
    adminSuspend: (id) => request("POST", `/admin/users/${encodeURIComponent(id)}/suspend`, {}),
    adminReactivate: (id) => request("POST", `/admin/users/${encodeURIComponent(id)}/reactivate`, {}),
    adminSetMembership: (id, body) =>
      request("POST", `/admin/users/${encodeURIComponent(id)}/membership`, body),
    adminSetAdminFlag: (id, admin) =>
      request("POST", `/admin/users/${encodeURIComponent(id)}/admin-flag`, { admin }),
    adminCreateProject: (body) => request("POST", "/admin/registry/projects", body),
    adminRenameProject: (key, name) =>
      request("POST", `/admin/registry/projects/${encodeURIComponent(key)}/rename`, { name }),
    adminCreateRepo: (body) => request("POST", "/admin/registry/repos", body),
    adminRenameRepo: (body) => request("POST", "/admin/registry/repos/rename", body),
    adminRevokeToken: (id) =>
      request("POST", `/admin/credentials/tokens/${encodeURIComponent(id)}/revoke`, {}),
    adminRevokeFamily: (id) =>
      request("POST", `/admin/credentials/families/${encodeURIComponent(id)}/revoke`, {}),
    adminMigrate: () => request("POST", "/admin/data/migrate", {}),
  };
}

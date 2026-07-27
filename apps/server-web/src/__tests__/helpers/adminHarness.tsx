import { vi } from "vitest";
import { render } from "@testing-library/react";
import { App } from "../../App";

// 管理面各頁測試的共用裝置：admin session、typed client 的 fake 與 matchMedia 設定。
// 每個管理目的地一個測試檔（users／registry／credentials／audit／system／overview），
// 共用同一份 fake 以免六份 client 各自漂移。

export const ADMIN_SESSION = {
  authenticated: true,
  user: { id: "a1", email: "admin@example.com", display: "Admin", admin: true },
  home: "/admin",
};

export const USERS = {
  users: [
    {
      id: "a1",
      email: "admin@example.com",
      display: "Admin",
      admin: true,
      active: true,
      createdAt: "2026-01-01T00:00:00Z",
      memberships: [{ projectKey: "demo", role: "editor" }],
      canSuspend: false,
      canRemoveAdmin: false,
    },
    {
      id: "m1",
      email: "member@example.com",
      display: "Member",
      admin: false,
      active: true,
      createdAt: "2026-01-02T00:00:00Z",
      memberships: [{ projectKey: "demo", role: "editor" }],
      canSuspend: true,
      canRemoveAdmin: false,
    },
  ],
  pending: [
    {
      id: "i1",
      email: "invited@example.com",
      display: "Invited",
      admin: false,
      memberships: ["demo"],
      createdAt: "2026-01-03T00:00:00Z",
      expiresAt: "2026-01-10T00:00:00Z",
    },
  ],
};

export const REGISTRY = {
  projects: [{ key: "demo", name: "Demo", repos: [{ key: "backend", name: "Backend" }] }],
};

export const CREDENTIALS = {
  pats: [
    {
      id: "p1",
      userId: "m1",
      prefix: "slk_abc",
      name: "cli",
      createdAt: "2026-01-01T00:00:00Z",
      expiresAt: null,
      lastUsedAt: null,
      revokedAt: null,
    },
    {
      id: "p2",
      userId: "a1",
      prefix: "slk_xyz",
      name: "部署",
      createdAt: "2026-01-02T00:00:00Z",
      expiresAt: "2026-08-01T00:00:00Z",
      lastUsedAt: null,
      revokedAt: "2026-01-05T00:00:00Z",
    },
  ],
  deviceFamilies: [
    {
      id: "d1",
      userId: "m1",
      source: "desktop",
      createdAt: "2026-01-01T00:00:00Z",
      lastRefreshAt: "2026-01-03T00:00:00Z",
      revokedAt: null,
    },
  ],
};

export const EMPTY_CREDENTIALS = { pats: [], deviceFamilies: [] };

export const SYSTEM = {
  engineVersion: "0.1.0",
  apiVersion: "1",
  identitySchemaVersion: 1,
  storeDriver: "memory",
  storeContractVersion: 1,
  storeLevel: "full",
  storeCapabilities: ["read", "write"],
  storeHealthy: true,
  storeHealthError: null,
  outboxBacklogs: [{ project: "demo", repo: "backend", backlog: 0 }],
  scopes: [{ project: "demo", repo: "backend", exportPath: "/admin/data/export/demo/backend" }],
  migrateAvailable: true,
};

export const AUDIT = {
  entries: [
    {
      id: "e1",
      actorId: "a1",
      action: "user-suspended",
      subject: "m1",
      source: "web",
      createdAt: "2026-01-01T00:00:00Z",
    },
  ],
  totalPages: 1,
};

export const OVERVIEW = {
  activeUsers: 2,
  suspendedUsers: 0,
  projects: 1,
  repos: 1,
  activeCredentials: 1,
  pendingInvitations: 0,
  storeHealthy: true,
  identitySchemaVersion: 1,
  todos: [],
  recentAudit: [],
};

/** A typed-client fake covering every operation the admin pages call. */
export function makeAdminClient(overrides: Record<string, unknown> = {}) {
  return {
    getSession: vi.fn(async () => ADMIN_SESSION),
    login: vi.fn(),
    logout: vi.fn(async () => ({ destination: "/login" })),
    getAccount: vi.fn(),
    getAdminOverview: vi.fn(async () => OVERVIEW),
    getAdminUsers: vi.fn(async () => USERS),
    getAdminRegistry: vi.fn(async () => REGISTRY),
    getAdminCredentials: vi.fn(async () => CREDENTIALS),
    getAdminSystem: vi.fn(async () => SYSTEM),
    getAdminAudit: vi.fn(async () => AUDIT),
    adminInvite: vi.fn(async () => ({ token: "invite-token-123" })),
    adminRevokeInvitation: vi.fn(async () => {}),
    adminSuspend: vi.fn(async () => {}),
    adminReactivate: vi.fn(async () => {}),
    adminSetMembership: vi.fn(async () => {}),
    adminSetAdminFlag: vi.fn(async () => {}),
    adminCreateProject: vi.fn(async () => {}),
    adminRenameProject: vi.fn(async () => {}),
    adminCreateRepo: vi.fn(async () => {}),
    adminRenameRepo: vi.fn(async () => {}),
    adminRevokeToken: vi.fn(async () => {}),
    adminRevokeFamily: vi.fn(async () => {}),
    adminMigrate: vi.fn(async () => {}),
    ...overrides,
  };
}

export function renderAt(route: string, client: ReturnType<typeof makeAdminClient>) {
  // 預設是「用過主控台的管理員」：首次導覽已看過，不會蓋在受測畫面上。
  // 導覽本身的行為由 tour.test.tsx 明確清掉這個鍵來測。
  try {
    localStorage.setItem("speclink.tourSeen", "1");
  } catch {
    // localStorage 不可用時導覽本來就不會持久化，測試照常。
  }
  return render(<App client={client as never} initialEntries={[route]} />);
}

/** jsdom 無 matchMedia：寬螢幕（固定側欄）為預設，窄螢幕讓 max-width query 命中。 */
export function setViewport(narrow: boolean) {
  window.matchMedia = ((query: string) => ({
    matches: narrow && /max-width/.test(query),
    media: query,
    onchange: null,
    addEventListener: () => {},
    removeEventListener: () => {},
    addListener: () => {},
    removeListener: () => {},
    dispatchEvent: () => false,
  })) as unknown as typeof window.matchMedia;
}

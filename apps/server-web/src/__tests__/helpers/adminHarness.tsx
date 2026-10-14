import { vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { MemoryRouter, useLocation, useNavigate } from "react-router-dom";
import { I18nProvider } from "@speclink/ui";
import { App } from "../../App";
import { ClientProvider } from "../../app/context";
import { LocaleProvider } from "../../i18n/LocaleContext";
import { APP_MESSAGES } from "../../i18n/messages";
import { AppRoutes } from "../../routes/AppRoutes";

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

/** n 位一般成員（User 1…User n，user1@example.com…），測換頁用。 */
export function makeUsers(n: number) {
  return Array.from({ length: n }, (_, i) => ({
    id: `u${i + 1}`,
    email: `user${i + 1}@example.com`,
    display: `User ${i + 1}`,
    admin: false,
    active: true,
    createdAt: "2026-01-02T00:00:00Z",
    memberships: [],
    canSuspend: true,
    canRemoveAdmin: false,
  }));
}

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
  total: 1,
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

/** 預設是「用過主控台的管理員」：首次導覽已看過，不會蓋在受測畫面上。
 *  導覽本身的行為由 tour.test.tsx 明確清掉這個鍵來測。 */
function markTourSeen() {
  try {
    localStorage.setItem("speclink.tourSeen", "1");
  } catch {
    // localStorage 不可用時導覽本來就不會持久化，測試照常。
  }
}

export function renderAt(route: string, client: ReturnType<typeof makeAdminClient>) {
  markTourSeen();
  return render(<App client={client as never} initialEntries={[route]} />);
}

/** 目前網址與一顆「瀏覽器上一頁」：詳情頁的網址與瀏覽紀錄要從路由外讀。 */
function LocationProbe() {
  const location = useLocation();
  const navigate = useNavigate();
  return (
    <>
      <output data-testid="location">{location.pathname + location.search}</output>
      <button type="button" onClick={() => navigate(-1)}>
        瀏覽器上一頁（測試）
      </button>
    </>
  );
}

/** 同 renderAt（完整路由與殼），另在路由旁放 LocationProbe；entries 的最後一筆是目前網址。 */
export function renderWithLocation(entries: string[], client: ReturnType<typeof makeAdminClient>) {
  markTourSeen();
  return render(
    <MemoryRouter initialEntries={entries} initialIndex={entries.length - 1}>
      <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
        <LocaleProvider value={{ pref: null, setPref: () => {} }}>
          <ClientProvider client={client as never}>
            <AppRoutes />
            <LocationProbe />
          </ClientProvider>
        </LocaleProvider>
      </I18nProvider>
    </MemoryRouter>,
  );
}

export const currentLocation = () => screen.getByTestId("location").textContent;
export const browserBack = () => screen.getByRole("button", { name: "瀏覽器上一頁（測試）" });

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

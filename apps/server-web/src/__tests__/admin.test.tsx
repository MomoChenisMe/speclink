import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { App } from "../App";

// 管理面六頁 view model 與 mutation（server-admin spec, D4／D6）。每頁只讀對應
// view-model API 並呈現所需欄位（不取祕密），破壞性操作先以 AlertDialog 確認，registry
// key 不可改（只更名）。管理殼與 lazy chunk 由 App 提供，深連結可直接呈現。

const ADMIN = {
  authenticated: true,
  user: { id: "a1", email: "admin@example.com", display: "Admin", admin: true },
  home: "/admin",
};

const USERS = {
  users: [
    {
      id: "a1",
      email: "admin@example.com",
      display: "Admin",
      admin: true,
      active: true,
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
      memberships: [{ projectKey: "demo", role: "editor" }],
      canSuspend: true,
      canRemoveAdmin: false,
    },
  ],
};

const REGISTRY = {
  projects: [{ key: "demo", name: "Demo", repos: [{ key: "backend", name: "Backend" }] }],
};

const CREDENTIALS = {
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
  ],
  deviceFamilies: [],
};

const DATA = {
  scopes: [{ project: "demo", repo: "backend", exportPath: "/admin/data/export/demo/backend" }],
  storeHealthy: true,
};

const SYSTEM = {
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
};

const AUDIT = {
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
};

function makeClient(overrides: Record<string, unknown> = {}) {
  return {
    getSession: vi.fn(async () => ADMIN),
    login: vi.fn(),
    logout: vi.fn(async () => ({ destination: "/login" })),
    getAdminOverview: vi.fn(async () => ({
      activeUsers: 2,
      suspendedUsers: 0,
      projects: 1,
      repos: 1,
      activeCredentials: 1,
      storeHealthy: true,
      identitySchemaVersion: 1,
    })),
    getAccount: vi.fn(),
    getAdminUsers: vi.fn(async () => USERS),
    getAdminRegistry: vi.fn(async () => REGISTRY),
    getAdminCredentials: vi.fn(async () => CREDENTIALS),
    getAdminData: vi.fn(async () => DATA),
    getAdminSystem: vi.fn(async () => SYSTEM),
    getAdminAudit: vi.fn(async () => AUDIT),
    adminInvite: vi.fn(async () => ({ token: "invite-token-123" })),
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

function renderAt(route: string, client: ReturnType<typeof makeClient>) {
  return render(<App client={client as never} initialEntries={[route]} />);
}

beforeEach(() => {
  window.matchMedia = ((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addEventListener: () => {},
    removeEventListener: () => {},
    addListener: () => {},
    removeListener: () => {},
    dispatchEvent: () => false,
  })) as unknown as typeof window.matchMedia;
});

describe("管理使用者頁", () => {
  it("列出使用者，最後一位 admin 的停權不可用", async () => {
    renderAt("/admin/users", makeClient());
    expect(await screen.findByText(/member@example.com/)).toBeTruthy();
    // 一般成員可停權；最後一位 active admin 不可停權。
    expect(screen.getByRole("button", { name: /停權.*Member|停權 member@example.com/ })).toBeTruthy();
    const adminSuspend = screen.queryByRole("button", { name: /停權.*Admin|停權 admin@example.com/ });
    expect(adminSuspend === null || (adminSuspend as HTMLButtonElement).disabled).toBe(true);
  });

  it("停權需 AlertDialog 確認後呼叫 adminSuspend", async () => {
    const client = makeClient();
    renderAt("/admin/users", client);
    const user = userEvent.setup();
    await user.click(await screen.findByRole("button", { name: /停權.*Member|停權 member@example.com/ }));
    const dialog = await screen.findByRole("alertdialog");
    expect(client.adminSuspend).not.toHaveBeenCalled();
    await user.click(within(dialog).getByRole("button", { name: /停權|確認/ }));
    await waitFor(() => expect(client.adminSuspend).toHaveBeenCalledWith("m1"));
  });

  it("邀請表單提交呼叫 adminInvite 並顯示一次性 token", async () => {
    const client = makeClient();
    renderAt("/admin/users", client);
    const user = userEvent.setup();
    await user.type(await screen.findByLabelText(/邀請.*email|email|電子郵件/i), "new@example.com");
    await user.type(screen.getByLabelText(/顯示名稱|display/i), "New");
    await user.click(screen.getByRole("button", { name: /邀請|建立邀請|送出邀請/ }));
    await waitFor(() => expect(client.adminInvite).toHaveBeenCalledOnce());
    expect(client.adminInvite).toHaveBeenCalledWith(
      expect.objectContaining({ email: "new@example.com", display: "New" }),
    );
    expect(await screen.findByText(/invite-token-123/)).toBeTruthy();
  });
});

describe("管理專案與儲存庫頁", () => {
  it("列出 project／repo 並可建立 project", async () => {
    const client = makeClient();
    renderAt("/admin/registry", client);
    expect(await screen.findByText(/demo/)).toBeTruthy();
    const user = userEvent.setup();
    await user.type(await screen.findByLabelText(/project key/i), "web");
    await user.click(screen.getByRole("button", { name: /建立 project|新增 project|建立專案/ }));
    await waitFor(() => expect(client.adminCreateProject).toHaveBeenCalledOnce());
    expect(client.adminCreateProject).toHaveBeenCalledWith(expect.objectContaining({ key: "web" }));
  });
});

describe("管理憑證頁", () => {
  it("列出 PAT metadata 並強制撤銷需確認", async () => {
    const client = makeClient();
    renderAt("/admin/credentials", client);
    expect(await screen.findByText(/slk_abc/)).toBeTruthy();
    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /撤銷/ }));
    const dialog = await screen.findByRole("alertdialog");
    expect(client.adminRevokeToken).not.toHaveBeenCalled();
    await user.click(within(dialog).getByRole("button", { name: /撤銷|確認/ }));
    await waitFor(() => expect(client.adminRevokeToken).toHaveBeenCalledWith("p1"));
  });
});

describe("管理資料操作頁", () => {
  it("列出 scope 匯出連結，遷移需確認後呼叫 adminMigrate", async () => {
    const client = makeClient();
    renderAt("/admin/data", client);
    const link = await screen.findByRole("link", { name: /匯出|下載|export/i });
    expect(link.getAttribute("href")).toContain("/admin/data/export/demo/backend");
    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /遷移|migrate/i }));
    const dialog = await screen.findByRole("alertdialog");
    await user.click(within(dialog).getByRole("button", { name: /遷移|確認/ }));
    await waitFor(() => expect(client.adminMigrate).toHaveBeenCalledOnce());
  });
});

describe("管理系統狀態頁", () => {
  it("呈現引擎／store 資訊", async () => {
    renderAt("/admin/system", makeClient());
    expect(await screen.findByText(/memory/)).toBeTruthy();
    expect(screen.getByText(/0\.1\.0/)).toBeTruthy();
  });
});

describe("管理稽核紀錄頁", () => {
  it("列出稽核事件", async () => {
    renderAt("/admin/audit", makeClient());
    expect(await screen.findByText(/user-suspended/)).toBeTruthy();
    expect(screen.getByText(/web/)).toBeTruthy();
  });
});

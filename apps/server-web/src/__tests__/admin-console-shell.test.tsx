import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { App } from "../App";

// 主控台殼（server-web-console「全部 browser route 由單一 SPA 提供可發現導覽」
// 「共用設計系統維持高密度可存取體驗」）：管理員與一般成員共用同一個依角色裁切的殼。
// 側欄只在 session 帶 admin 旗標時渲染，恰六個目的地（不含資料操作與帳號）；帳號入口
// 移至 header 的電子郵件連結，與登出並列。管理員於 /account 側欄整條保留且無高亮。

const ADMIN = {
  authenticated: true,
  user: { id: "a1", email: "admin@example.com", display: "Admin", admin: true },
  home: "/admin",
};

const MEMBER = {
  authenticated: true,
  user: { id: "u1", email: "member@example.com", display: "Member", admin: false },
  home: "/account",
};

const DESTINATIONS = ["總覽", "使用者", "專案與儲存庫", "憑證", "系統", "稽核紀錄"];

const ACCOUNT = {
  user: { id: "u1", email: "member@example.com", display: "Member", admin: false },
  memberships: [],
  pats: [],
  sessions: [],
  deviceFamilies: [],
};

function makeClient(overrides: Record<string, unknown> = {}) {
  return {
    getSession: vi.fn(async () => ADMIN),
    login: vi.fn(),
    logout: vi.fn(async () => ({ destination: "/login" })),
    getAccount: vi.fn(async () => ACCOUNT),
    getAdminOverview: vi.fn(async () => ({
      activeUsers: 1,
      suspendedUsers: 0,
      projects: 1,
      repos: 1,
      activeCredentials: 1,
      pendingInvitations: 0,
      storeHealthy: true,
      identitySchemaVersion: 1,
      todos: [],
      recentAudit: [],
    })),
    ...overrides,
  };
}

function renderAt(route: string, client: ReturnType<typeof makeClient>) {
  return render(<App client={client as never} initialEntries={[route]} />);
}

function wide() {
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
}

beforeEach(wide);

describe("依角色裁切的主控台殼", () => {
  it("側欄恰有六個目的地，不含資料操作與帳號", async () => {
    renderAt("/admin", makeClient());
    const nav = await screen.findByRole("navigation", { name: "管理導覽" });
    const links = within(nav).getAllByRole("link");
    expect(links.map((l) => l.textContent?.trim())).toEqual(DESTINATIONS);
    expect(within(nav).queryByText("資料操作")).toBeNull();
    expect(within(nav).queryByText("帳號")).toBeNull();
  });

  it("header 以電子郵件連結進入帳號，並與登出並列", async () => {
    renderAt("/admin", makeClient());
    const banner = await screen.findByRole("banner");
    const account = within(banner).getByRole("link", { name: "admin@example.com" });
    expect(account.getAttribute("href")).toBe("/account");
    expect(within(banner).getByRole("button", { name: /登出/ })).toBeTruthy();
  });

  it("管理員於 /account 仍見完整側欄且無項目高亮", async () => {
    renderAt("/account", makeClient({ getSession: vi.fn(async () => ADMIN) }));
    const nav = await screen.findByRole("navigation", { name: "管理導覽" });
    for (const dest of DESTINATIONS) {
      expect(within(nav).getByText(dest), `sidebar has ${dest}`).toBeTruthy();
    }
    // 帳號已非側欄目的地，故側欄無任何項目高亮；高亮改由 header 的電子郵件連結承擔。
    expect(within(nav).queryByRole("link", { current: "page" })).toBeNull();
    const banner = screen.getByRole("banner");
    expect(
      within(banner).getByRole("link", { name: "admin@example.com" }).getAttribute("aria-current"),
    ).toBe("page");
  });

  it("一般成員於 /account 不渲染側欄", async () => {
    renderAt("/account", makeClient({ getSession: vi.fn(async () => MEMBER) }));
    await screen.findByRole("main");
    expect(screen.queryByRole("navigation", { name: "管理導覽" })).toBeNull();
    for (const dest of DESTINATIONS) {
      expect(screen.queryByRole("link", { name: dest }), `no admin dest ${dest}`).toBeNull();
    }
    // 帳號入口仍在 header：電子郵件連結與登出並列。
    const banner = screen.getByRole("banner");
    expect(within(banner).getByRole("link", { name: "member@example.com" })).toBeTruthy();
    expect(within(banner).getByRole("button", { name: /登出/ })).toBeTruthy();
  });

  it("768px 時 Sheet 內同為六個目的地", async () => {
    window.matchMedia = ((query: string) => ({
      matches: /max-width/.test(query),
      media: query,
      onchange: null,
      addEventListener: () => {},
      removeEventListener: () => {},
      addListener: () => {},
      removeListener: () => {},
      dispatchEvent: () => false,
    })) as unknown as typeof window.matchMedia;
    renderAt("/admin", makeClient());
    const user = userEvent.setup();
    await user.click(await screen.findByRole("button", { name: /開啟導覽|選單/ }));
    const dialog = await screen.findByRole("dialog");
    const links = within(dialog).getAllByRole("link");
    expect(links.map((l) => l.textContent?.trim())).toEqual(DESTINATIONS);
  });
});

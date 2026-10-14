import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { axe } from "jest-axe";
import { App } from "../App";
import { setViewport } from "./helpers/adminHarness";

// 可存取性基線（server-web-console D6）：殼層無 axe 違規。jsdom 無法計算色彩對比，
// color-contrast 停用，對比／zoom／reduced-motion 由手動 viewport 驗收涵蓋（7.3）。
const AXE = { rules: { "color-contrast": { enabled: false } } };

const ADMIN = {
  authenticated: true,
  user: { id: "a1", email: "admin@example.com", display: "Admin", admin: true },
  home: "/admin",
};

function client(overrides: Record<string, unknown> = {}) {
  return {
    getSession: vi.fn(async () => ({ authenticated: false, user: null, home: "/login" })),
    login: vi.fn(async () => ({ destination: "/admin" })),
    logout: vi.fn(async () => ({ destination: "/login" })),
    getAdminOverview: vi.fn(async () => ({
      activeUsers: 1,
      suspendedUsers: 0,
      projects: 1,
      repos: 1,
      activeCredentials: 0,
      pendingInvitations: 0,
      storeHealthy: true,
      identitySchemaVersion: 1,
      todos: [],
      recentAudit: [],
    })),
    ...overrides,
  } as never;
}

describe("可存取性（axe）", () => {
  it("登入頁無 axe 違規", async () => {
    const { container } = render(<App client={client()} initialEntries={["/login"]} />);
    await screen.findByRole("button", { name: /登入/ });
    const results = await axe(container, AXE);
    expect(results.violations).toEqual([]);
  });

  it("管理殼無 axe 違規", async () => {
    const { container } = render(
      <App client={client({ getSession: vi.fn(async () => ADMIN) })} initialEntries={["/admin"]} />,
    );
    // 殼有兩個 nav landmark（側欄導覽與麵包屑），以名稱區分；axe 也檢查兩者名稱不重複。
    await screen.findByRole("navigation", { name: "管理導覽" });
    await screen.findByRole("heading", { name: /總覽/ });
    const results = await axe(container, AXE);
    expect(results.violations).toEqual([]);
  });

  // 窄螢幕沒有固定側欄、頂列多一顆「開啟導覽」鈕：檢查內容都落在 landmark 內（axe 的 region
  // 規則，這裡明確打開）。region 只在掃描範圍含 body 時適用，對 container 跑會判為不適用，所以
  // 這一題掃整個 body；不傳 documentElement——jest-axe 會把 body 外的元素複製進 body 再掃，
  // 活的畫面會被換成靜態複本。
  it("窄螢幕管理殼無 axe 違規，且內容都在 landmark 內", async () => {
    setViewport(true);
    try {
      render(
        <App client={client({ getSession: vi.fn(async () => ADMIN) })} initialEntries={["/admin"]} />,
      );
      await screen.findByRole("button", { name: "開啟導覽" });
      await screen.findByRole("heading", { name: /總覽/ });
      const results = await axe(document.body, {
        rules: { ...AXE.rules, region: { enabled: true } },
      });
      expect(results.violations).toEqual([]);
    } finally {
      setViewport(false);
    }
  });
});

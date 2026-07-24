import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { axe } from "jest-axe";
import { App } from "../App";

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
      storeHealthy: true,
      identitySchemaVersion: 1,
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
    await screen.findByRole("navigation");
    await screen.findByRole("heading", { name: /總覽/ });
    const results = await axe(container, AXE);
    expect(results.violations).toEqual([]);
  });
});

import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, within } from "@testing-library/react";
import { makeAdminClient, renderAt, setViewport } from "./helpers/adminHarness";

// 管理總覽（server-web-console「總覽提供可行動入口與待辦」）：四張可點入對應目的地的
// 指標卡，加上待辦、系統健康摘要與最近活動三個區塊。沒有待處理事項時整塊不渲染——
// 空標題與空清單只是視覺噪音。識別資料結構版本屬系統健康摘要，不是獨立指標卡。

const HEALTHY = {
  activeUsers: 2,
  suspendedUsers: 1,
  projects: 3,
  repos: 4,
  activeCredentials: 5,
  pendingInvitations: 1,
  storeHealthy: true,
  identitySchemaVersion: 6,
  todos: [],
  recentAudit: [
    {
      id: "e1",
      actorId: "a1",
      action: "user-invited",
      subject: "new@example.com",
      source: "web",
      createdAt: "2026-01-03T10:00:00Z",
    },
  ],
};

beforeEach(() => setViewport(false));

describe("總覽指標卡", () => {
  it("四張指標卡皆可點入對應目的地", async () => {
    renderAt("/admin", makeAdminClient({ getAdminOverview: vi.fn(async () => HEALTHY) }));
    // 標題在資料閘門之外就渲染，不能當等待錨點——等一個依賴資料的區塊。
    await screen.findByRole("region", { name: "系統健康" });
    // 側欄也有「使用者」等連結，指標卡的查詢限定在主內容區。
    const main = screen.getByRole("main");
    const expected: [RegExp, string][] = [
      [/使用者/, "/admin/users"],
      [/專案/, "/admin/registry"],
      [/憑證/, "/admin/credentials"],
      [/待啟用/, "/admin/users"],
    ];
    for (const [name, href] of expected) {
      const card = within(main).getByRole("link", { name });
      expect(card.getAttribute("href"), `${name} 指標卡連往 ${href}`).toBe(href);
    }
  });

  it("識別資料結構版本呈現於系統健康摘要，而非獨立指標卡", async () => {
    renderAt("/admin", makeAdminClient({ getAdminOverview: vi.fn(async () => HEALTHY) }));
    const health = await screen.findByRole("region", { name: "系統健康" });
    expect(health.textContent).toContain("6");
    expect(health.textContent).toContain("資料結構");
    expect(screen.queryByRole("link", { name: /資料結構版本/ })).toBeNull();
  });

  it("系統健康摘要與最近活動各自提供前往系統與稽核的入口", async () => {
    renderAt("/admin", makeAdminClient({ getAdminOverview: vi.fn(async () => HEALTHY) }));
    const health = await screen.findByRole("region", { name: "系統健康" });
    expect(within(health).getByRole("link", { name: /系統/ }).getAttribute("href")).toBe(
      "/admin/system",
    );
    const activity = screen.getByRole("region", { name: "最近活動" });
    expect(within(activity).getByRole("link", { name: /稽核/ }).getAttribute("href")).toBe(
      "/admin/audit",
    );
    expect(activity.textContent).toContain("new@example.com");
  });
});

describe("總覽待辦區塊", () => {
  it("無待處理事項時整塊不渲染", async () => {
    renderAt("/admin", makeAdminClient({ getAdminOverview: vi.fn(async () => HEALTHY) }));
    await screen.findByRole("region", { name: "系統健康" });
    expect(screen.queryByRole("region", { name: "需要處理" })).toBeNull();
  });

  it("無有效憑證時呈現該事項並提供建立存取金鑰入口", async () => {
    renderAt(
      "/admin",
      makeAdminClient({
        getAdminOverview: vi.fn(async () => ({
          ...HEALTHY,
          activeCredentials: 0,
          todos: [{ kind: "no-active-credentials", destination: "/account", count: 0 }],
        })),
      }),
    );
    const todos = await screen.findByRole("region", { name: "需要處理" });
    expect(todos.textContent).toContain("憑證");
    expect(within(todos).getByRole("link", { name: /建立存取金鑰/ }).getAttribute("href")).toBe(
      "/account",
    );
  });

  it("有待啟用邀請時呈現該事項並連往使用者目的地", async () => {
    renderAt(
      "/admin",
      makeAdminClient({
        getAdminOverview: vi.fn(async () => ({
          ...HEALTHY,
          todos: [{ kind: "pending-invitations", destination: "/admin/users", count: 2 }],
        })),
      }),
    );
    const todos = await screen.findByRole("region", { name: "需要處理" });
    expect(todos.textContent).toContain("2");
    expect(within(todos).getByRole("link", { name: /查看邀請/ }).getAttribute("href")).toBe(
      "/admin/users",
    );
  });
});

// server-setup「完成 setup 即可邀請與連線」要求導向 /admin?welcome=1 顯示連線資訊；
// server-admin「Store 不健康時 identity 管理仍可用」要求 overview 明確顯示 storeHealthy:false。
describe("總覽的歡迎區塊與儲存後端健康", () => {
  const CONNECTION = {
    publicUrl: "https://speclink.example",
    projectKey: "demo",
    repoKey: "backend",
  };

  it("welcome=1 呈現初始設定完成的連線資訊三欄位且可複製", async () => {
    renderAt(
      "/admin?welcome=1",
      makeAdminClient({
        getAdminOverview: vi.fn(async () => ({ ...HEALTHY, connection: CONNECTION })),
      }),
    );
    const welcome = await screen.findByRole("region", { name: /開始使用/ });
    expect(within(welcome).getByText(CONNECTION.publicUrl)).toBeTruthy();
    expect(within(welcome).getByText(CONNECTION.projectKey)).toBeTruthy();
    expect(within(welcome).getByText(CONNECTION.repoKey)).toBeTruthy();
    expect(within(welcome).getAllByRole("button", { name: /複製/ }).length).toBe(3);
  });

  it("無 welcome 參數時不呈現歡迎區塊", async () => {
    renderAt(
      "/admin",
      makeAdminClient({
        getAdminOverview: vi.fn(async () => ({ ...HEALTHY, connection: CONNECTION })),
      }),
    );
    await screen.findByRole("region", { name: "系統健康" });
    expect(screen.queryByRole("region", { name: /開始使用/ })).toBeNull();
  });

  it("storeHealthy:false 時以 role=alert 呈現降級狀態與可公開錯誤", async () => {
    renderAt(
      "/admin",
      makeAdminClient({
        getAdminOverview: vi.fn(async () => ({
          ...HEALTHY,
          storeHealthy: false,
          storeHealthError: "store unreachable",
        })),
      }),
    );
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("store unreachable");
  });
});

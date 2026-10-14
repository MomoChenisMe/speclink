import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { makeAdminClient, renderAt, setViewport } from "./helpers/adminHarness";

// 管理總覽（server-web-console「總覽提供可行動入口與待辦」；server-web-list-pages design D10）：
// 768px 置中欄；頂端四格數字磚（與 desktop 閱讀卡同款、每格可點進對應頁），其下「需要處理」
// 「系統健康」「最近活動」三張設定型卡。沒有待處理事項時整張卡不渲染——空標題與空清單只是
// 視覺噪音。識別資料結構版本屬系統健康摘要，不是獨立數字磚。

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

/** 主內容區內的數字磚（側欄也有「使用者」等連結，查詢限定在主內容區）。 */
const tiles = () => Array.from(screen.getByRole("main").querySelectorAll<HTMLElement>("[data-doc-tile]"));

describe("總覽數字磚與版面", () => {
  it("四格數字磚皆為連結，目的地依序為使用者、專案、憑證與邀請中分頁，標籤帶次要計數", async () => {
    renderAt("/admin", makeAdminClient({ getAdminOverview: vi.fn(async () => HEALTHY) }));
    // 標題在資料閘門之外就渲染，不能當等待錨點——等一個依賴資料的區塊。
    await screen.findByRole("region", { name: "系統健康" });
    expect(tiles().map((tile) => tile.tagName)).toEqual(["A", "A", "A", "A"]);
    expect(tiles().map((tile) => tile.getAttribute("href"))).toEqual([
      "/admin/users",
      "/admin/registry",
      "/admin/credentials",
      "/admin/users?tab=pending",
    ]);
    expect(tiles().map((tile) => tile.textContent)).toEqual([
      "2使用者 · 停權 1",
      "3專案 · 儲存庫 4",
      "5憑證 · 有效",
      "1待啟用 · 邀請未使用",
    ]);
  });

  it("點待啟用數字磚直接看到使用者頁的邀請中分頁", async () => {
    renderAt("/admin", makeAdminClient({ getAdminOverview: vi.fn(async () => HEALTHY) }));
    await screen.findByRole("region", { name: "系統健康" });
    const user = userEvent.setup();
    await user.click(tiles()[3]);
    const pending = await screen.findByRole("tab", { name: /^邀請中/ });
    expect(pending.getAttribute("aria-selected")).toBe("true");
  });

  it("內容欄 768px 置中", async () => {
    renderAt("/admin", makeAdminClient({ getAdminOverview: vi.fn(async () => HEALTHY) }));
    await screen.findByRole("region", { name: "系統健康" });
    const column = screen
      .getByRole("heading", { level: 1, name: "總覽" })
      .closest("[data-page-header]")?.parentElement as HTMLElement;
    expect(column.className).toContain("max-w-[768px]");
    expect(column.className).toContain("mx-auto");
  });

  it("識別資料結構版本呈現於系統健康摘要，而非獨立數字磚", async () => {
    renderAt("/admin", makeAdminClient({ getAdminOverview: vi.fn(async () => HEALTHY) }));
    const health = await screen.findByRole("region", { name: "系統健康" });
    const schema = within(health).getByRole("group", { name: "資料結構版本" });
    expect(schema.textContent).toContain("v6");
    expect(tiles()).toHaveLength(4);
    expect(screen.queryByRole("link", { name: /資料結構版本/ })).toBeNull();
  });

  it("系統健康與最近活動的標頭各自提供前往系統與稽核的入口", async () => {
    renderAt("/admin", makeAdminClient({ getAdminOverview: vi.fn(async () => HEALTHY) }));
    const health = await screen.findByRole("region", { name: "系統健康" });
    expect(within(health).getByRole("link", { name: /系統/ }).getAttribute("href")).toBe(
      "/admin/system",
    );
    const activity = screen.getByRole("region", { name: "最近活動" });
    expect(within(activity).getByRole("link", { name: /稽核/ }).getAttribute("href")).toBe(
      "/admin/audit",
    );
  });

  it("最近活動一列含動作名稱、對象、日期與操作者", async () => {
    renderAt("/admin", makeAdminClient({ getAdminOverview: vi.fn(async () => HEALTHY) }));
    const activity = await screen.findByRole("region", { name: "最近活動" });
    const row = within(activity).getByRole("group", { name: "邀請使用者 new@example.com" });
    expect(row.textContent).toContain("2026-01-03 · a1");
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

  it("有待啟用邀請時呈現該事項並連往使用者頁的邀請中分頁", async () => {
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
      "/admin/users?tab=pending",
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
    // spec「儲存健康徽章」：徽章與同頁的降級警示語意一致——異常為紅系，
    // 不能與正常一樣壓成灰（同頁上下文一紅一灰講同一件事，讀起來自相矛盾）。
    expect(screen.getByText("異常").className).toContain("destructive");
  });

  it("儲存健康徽章：正常為綠系，與異常的紅系一眼可辨", async () => {
    renderAt("/admin", makeAdminClient({ getAdminOverview: vi.fn(async () => HEALTHY) }));
    const badge = await screen.findByText("正常");
    expect(badge.className).toContain("status-success");
  });
});

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { App } from "../App";

// 帳號自助頁（server-identity「帳號 browser API 保持憑證祕密邊界」, D4／D6）。顯示
// 使用者、存取金鑰、登入工作階段與裝置；建立金鑰的明文只顯示一次；撤銷等破壞性操作先以
// AlertDialog 確認才送出。

const MEMBER = {
  authenticated: true,
  user: { id: "u1", email: "member@example.com", display: "Member", admin: false },
  home: "/account",
};

const ACCOUNT = {
  user: { id: "u1", email: "member@example.com", display: "Member", admin: false },
  memberships: [
    { projectKey: "alpha", projectName: "Alpha 專案", role: "editor" },
    { projectKey: "beta", projectName: "Beta 專案", role: "reader" },
  ],
  pats: [
    {
      id: "p1",
      prefix: "slk_abc",
      name: "cli",
      createdAt: "2026-01-01T00:00:00Z",
      expiresAt: null,
      lastUsedAt: null,
      revokedAt: null,
    },
  ],
  sessions: [
    { id: "s1", createdAt: "2026-01-01T00:00:00Z", expiresAt: "2026-02-01T00:00:00Z", revokedAt: null },
  ],
  deviceFamilies: [
    {
      id: "d1",
      source: "device 授權",
      createdAt: "2026-01-01T00:00:00Z",
      lastRefreshAt: "2026-01-02T00:00:00Z",
      revokedAt: null,
    },
  ],
};

function makeClient(overrides: Record<string, unknown> = {}) {
  return {
    getSession: vi.fn(async () => MEMBER),
    login: vi.fn(),
    logout: vi.fn(async () => ({ destination: "/login" })),
    getAdminOverview: vi.fn(),
    getAccount: vi.fn(async () => ACCOUNT),
    createPat: vi.fn(async () => ({
      pat: {
        id: "p2",
        prefix: "slk_new",
        name: "laptop",
        createdAt: "2026-01-03T00:00:00Z",
        expiresAt: null,
        lastUsedAt: null,
        revokedAt: null,
      },
      plaintext: "slk_new_SECRETPLAINTEXT",
    })),
    revokePat: vi.fn(async () => {}),
    revokeDevice: vi.fn(async () => {}),
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

describe("帳號自助頁", () => {
  it("顯示使用者、存取金鑰、登入工作階段與裝置", async () => {
    renderAt("/account", makeClient());
    // 等帳號資料本身載入——header 的電子郵件連結在資料回來前就已渲染，不能當等待錨點。
    expect(await screen.findByRole("cell", { name: "cli" })).toBeTruthy();
    expect(screen.getAllByText(/member@example.com/).length).toBeGreaterThan(0);
    // PAT、session、device 各區塊的識別資料（精確 cell 查詢，避開撤銷鈕 aria-label）。
    expect(screen.getByRole("cell", { name: "slk_abc" })).toBeTruthy();
    expect(screen.getByRole("cell", { name: "device 授權" })).toBeTruthy();
  });

  // 建立動作與管理列表頁一致：頁面唯一 primary action 開抽屜，欄位不常駐頁面。
  it("建立表單不常駐頁面，按下建立存取金鑰後才出現於抽屜", async () => {
    renderAt("/account", makeClient());
    await screen.findByRole("cell", { name: "cli" });
    expect(screen.queryByLabelText(/到期日/)).toBeNull();
    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /建立存取金鑰/ }));
    const drawer = await screen.findByRole("dialog", { name: /建立存取金鑰/ });
    expect(within(drawer).getByLabelText(/名稱/)).toBeTruthy();
    expect(within(drawer).getByLabelText(/到期日/)).toBeTruthy();
  });

  it("建立存取金鑰後抽屜關閉，明文只顯示一次且可複製", async () => {
    const client = makeClient();
    renderAt("/account", client);
    const user = userEvent.setup();
    await screen.findByRole("cell", { name: "cli" });
    await user.click(screen.getByRole("button", { name: /建立存取金鑰/ }));
    const drawer = await screen.findByRole("dialog", { name: /建立存取金鑰/ });
    await user.type(within(drawer).getByLabelText(/名稱/), "laptop");
    await user.click(within(drawer).getByRole("button", { name: /^建立$|建立存取金鑰/ }));
    await waitFor(() => expect(client.createPat).toHaveBeenCalledOnce());
    expect(client.createPat).toHaveBeenCalledWith(expect.objectContaining({ name: "laptop" }));
    // 送出成功後抽屜關閉，明文出現在頁面的一次性回饋中並附複製鈕。
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    const plaintext = await screen.findByText(/slk_new_SECRETPLAINTEXT/);
    expect(screen.getByRole("button", { name: /複製/ })).toBeTruthy();
    // spec「揭示橫幅為成功語意」：一次性明碼是「建好了」的回饋，走綠系而非主色。
    const notice = plaintext.closest("[role='status']") as HTMLElement;
    expect(notice.className).toContain("emerald");
    expect(notice.className).not.toContain("border-primary");
  });

  it("工作階段狀態徽章：有效為綠系、已撤銷為中性", async () => {
    // spec「憑證狀態徽章」：帳號頁與後台憑證頁講同一件事，色彩層級也要一致。
    renderAt("/account", makeClient());
    const valid = await screen.findAllByText("有效");
    expect(valid[0].className).toContain("emerald");
  });

  // 我的專案區塊（server-web-console「帳號頁呈現我的專案」）：資料來自 account
  // summary、唯讀、admin 與一般成員同一形狀，無隸屬時仍顯示區塊與空狀態。
  it("列出自己的專案顯示名與角色，且無任何編輯操作", async () => {
    renderAt("/account", makeClient());
    const section = await screen.findByRole("region", { name: /我的專案/ });
    expect(within(section).getByText("Alpha 專案")).toBeTruthy();
    expect(within(section).getByText("editor")).toBeTruthy();
    expect(within(section).getByText("Beta 專案")).toBeTruthy();
    expect(within(section).getByText("reader")).toBeTruthy();
    expect(within(section).queryAllByRole("button")).toHaveLength(0);
    expect(within(section).queryAllByRole("link")).toHaveLength(0);
  });

  it("專案顯示名缺席時以專案代號呈現", async () => {
    const client = makeClient({
      getAccount: vi.fn(async () => ({
        ...ACCOUNT,
        memberships: [{ projectKey: "alpha", projectName: "", role: "editor" }],
      })),
    });
    renderAt("/account", client);
    const section = await screen.findByRole("region", { name: /我的專案/ });
    expect(within(section).getByText("alpha")).toBeTruthy();
  });

  // admin 也走同一區塊：個人視角只列自己的隸屬，全部專案是 /admin 的治理視角。
  it("admin 看到的是自己的隸屬而非全部專案", async () => {
    const admin = { id: "u9", email: "admin@example.com", display: "Admin", admin: true };
    const client = makeClient({
      getSession: vi.fn(async () => ({ authenticated: true, user: admin, home: "/admin" })),
      getAccount: vi.fn(async () => ({
        ...ACCOUNT,
        user: admin,
        memberships: [{ projectKey: "alpha", projectName: "Alpha 專案", role: "editor" }],
      })),
    });
    renderAt("/account", client);
    const section = await screen.findByRole("region", { name: /我的專案/ });
    expect(within(section).getByText("Alpha 專案")).toBeTruthy();
    expect(within(section).queryByText("Beta 專案")).toBeNull();
    expect(within(section).getAllByRole("row")).toHaveLength(2); // 表頭＋唯一一列
  });

  it("無任何隸屬時區塊仍可見並呈現空狀態", async () => {
    const client = makeClient({
      getAccount: vi.fn(async () => ({ ...ACCOUNT, memberships: [] })),
    });
    renderAt("/account", client);
    const section = await screen.findByRole("region", { name: /我的專案/ });
    expect(within(section).getByText(/管理員/)).toBeTruthy();
  });

  it("撤銷存取金鑰需先經 AlertDialog 確認才送出", async () => {
    const client = makeClient();
    renderAt("/account", client);
    const user = userEvent.setup();
    await screen.findByText(/slk_abc/);
    // 觸發撤銷：先出現確認對話框，尚未呼叫撤銷。
    await user.click(screen.getByRole("button", { name: /撤銷存取金鑰/ }));
    const dialog = await screen.findByRole("alertdialog");
    expect(client.revokePat).not.toHaveBeenCalled();
    // 確認後才送出，帶正確 PAT id。
    await user.click(within(dialog).getByRole("button", { name: /撤銷|確認/ }));
    await waitFor(() => expect(client.revokePat).toHaveBeenCalledWith("p1"));
  });
});

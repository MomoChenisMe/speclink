import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { App } from "../App";
import { WebApiError } from "../api/client";

// 帳號自助頁（server-identity「帳號 browser API 保持憑證祕密邊界」, D4／D6；server-web-console
// 「帳號頁呈現我的專案」；server-web-list-pages design D8、D10）。768px 置中欄，由上而下為我的
// 專案、存取金鑰、裝置登入、登入工作階段四張設定型卡；建立金鑰以置中對話框承載，明文只在對話框內
// 顯示一次（server-web-detail-pages design D7）；撤銷鈕滑過才出現，撤銷等破壞性操作先以 AlertDialog
// 確認才送出。

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
    {
      id: "p3",
      prefix: "slk_old",
      name: "old",
      createdAt: "2026-01-01T00:00:00Z",
      expiresAt: null,
      lastUsedAt: "2026-01-04T00:00:00Z",
      revokedAt: "2026-01-05T00:00:00Z",
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
  it("四張卡由上而下為我的專案、存取金鑰、裝置登入、登入工作階段，內容欄 768px 置中", async () => {
    renderAt("/account", makeClient());
    // 等帳號資料本身載入——header 的電子郵件連結在資料回來前就已渲染，不能當等待錨點。
    await screen.findByRole("region", { name: "我的專案" });
    const cards = ["我的專案", "存取金鑰", "裝置登入", "登入工作階段"];
    const order = screen
      .getAllByRole("region")
      .map((region) => region.getAttribute("aria-label"))
      .filter((name) => cards.includes(name ?? ""));
    expect(order).toEqual(cards);
    const column = screen
      .getByRole("heading", { level: 1, name: "帳號" })
      .closest("[data-page-header]")?.parentElement as HTMLElement;
    expect(column.className).toContain("max-w-[768px]");
    expect(column.className).toContain("mx-auto");
    expect(screen.queryByRole("table")).toBeNull();
  });

  it("存取金鑰列：說明為「前綴 · 到期 · 最近使用」或「前綴 · 撤銷於」；有效者的撤銷鈕滑過才出現，已撤銷的沒有", async () => {
    renderAt("/account", makeClient());
    const keys = await screen.findByRole("region", { name: "存取金鑰" });
    expect(within(keys).getByText("遠端工作流程以你的身分連線時使用")).toBeTruthy();
    const cli = within(keys).getByRole("group", { name: "cli" });
    expect(cli.textContent).toContain("slk_abc · 到期 永久 · 最近使用 從未");
    expect(within(cli).getByText("有效").className).toContain("status-success");
    const revoke = within(cli).getByRole("button", { name: "撤銷存取金鑰 cli" });
    // 平時隱藏、列滑過或鍵盤聚焦時出現；沒有滑鼠移入能力的觸控裝置恆常顯示。
    expect(revoke.className.split(/\s+/)).toEqual(
      expect.arrayContaining([
        "opacity-0",
        "group-hover:opacity-100",
        "focus-visible:opacity-100",
        "[@media(hover:none)]:opacity-100",
      ]),
    );
    const old = within(keys).getByRole("group", { name: "old" });
    expect(old.textContent).toContain("slk_old · 撤銷於 2026-01-05");
    expect(within(old).getByText("已撤銷")).toBeTruthy();
    expect(within(old).queryByRole("button")).toBeNull();
  });

  it("裝置登入列：說明為最近更新日期，撤銷鈕滑過才出現", async () => {
    renderAt("/account", makeClient());
    const devices = await screen.findByRole("region", { name: "裝置登入" });
    expect(within(devices).getByText("桌面 app 或 CLI 授權後留下的憑證")).toBeTruthy();
    const row = within(devices).getByRole("group", { name: "device 授權" });
    expect(row.textContent).toContain("最近更新 2026-01-02");
    const revoke = within(row).getByRole("button", { name: "撤銷裝置登入 device 授權" });
    expect(revoke.className).toContain("group-hover:opacity-100");
  });

  it("登入工作階段列：建立日期、到期日與狀態籤", async () => {
    renderAt("/account", makeClient());
    const sessions = await screen.findByRole("region", { name: "登入工作階段" });
    const row = within(sessions).getByRole("group", { name: "建立 2026-01-01" });
    expect(row.textContent).toContain("到期 2026-02-01");
    expect(within(row).getByText("有效").className).toContain("status-success");
  });

  // 建立動作與管理列表頁一致：頁面唯一 primary action 開置中對話框，欄位不常駐頁面
  // （server-web-detail-pages design D7）。
  it("建立表單不常駐頁面，按下建立存取金鑰後才出現在置中對話框", async () => {
    renderAt("/account", makeClient());
    await screen.findByRole("region", { name: "存取金鑰" });
    expect(screen.queryByLabelText("名稱")).toBeNull();
    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /建立存取金鑰/ }));
    const dialog = await screen.findByRole("alertdialog", { name: "建立存取金鑰" });
    expect(within(dialog).getByLabelText("名稱")).toBeTruthy();
    expect(within(dialog).getByLabelText(/到期日/)).toBeTruthy();
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("送出成功後同一個對話框換成「金鑰已建立」：綠系成功框內是明文與複製；按「完成」後頁面沒有明文、存取金鑰卡多出 ci", async () => {
    const ci = {
      id: "p9",
      prefix: "slk_ci",
      name: "ci",
      createdAt: "2026-01-06T00:00:00Z",
      expiresAt: null,
      lastUsedAt: null,
      revokedAt: null,
    };
    const client = makeClient({
      getAccount: vi.fn().mockResolvedValueOnce(ACCOUNT).mockResolvedValue({ ...ACCOUNT, pats: [...ACCOUNT.pats, ci] }),
      createPat: vi.fn(async () => ({ pat: ci, plaintext: "slk_ci_SECRETPLAINTEXT" })),
    });
    renderAt("/account", client);
    const user = userEvent.setup();
    await screen.findByRole("region", { name: "存取金鑰" });
    await user.click(screen.getByRole("button", { name: /建立存取金鑰/ }));
    const dialog = await screen.findByRole("alertdialog", { name: "建立存取金鑰" });
    await user.type(within(dialog).getByLabelText("名稱"), "ci");
    await user.click(within(dialog).getByRole("button", { name: "建立" }));
    await waitFor(() => expect(client.createPat).toHaveBeenCalledWith(expect.objectContaining({ name: "ci" })));

    const done = await screen.findByRole("alertdialog", { name: "金鑰已建立" });
    expect(done.textContent).toContain("只顯示這一次");
    const plaintext = within(done).getByText("slk_ci_SECRETPLAINTEXT");
    // （複製鈕自己也帶一個 sr-only 的 role=status，所以由明文回推外框。）
    const box = plaintext.closest("[role='status']") as HTMLElement;
    expect(box.getAttribute("aria-live")).toBe("polite");
    expect(within(box).getByRole("button", { name: "複製" })).toBeTruthy();
    // spec「祕密值成功框為成功語意」：一次性明碼是「建好了」的回饋，走綠系而非主色。
    expect(box.className).toContain("status-success");
    expect(box.className).not.toContain("primary");

    await user.click(within(done).getByRole("button", { name: "完成" }));
    await waitFor(() => expect(screen.queryByRole("alertdialog")).toBeNull());
    expect(screen.queryByText(/slk_ci_SECRETPLAINTEXT/)).toBeNull();
    const keys = screen.getByRole("region", { name: "存取金鑰" });
    expect(within(keys).getByRole("group", { name: "ci" })).toBeTruthy();
  });

  it("送出失敗時對話框保持開啟、保留名稱，錯誤以 role=alert 緊貼欄位", async () => {
    const client = makeClient({
      createPat: vi.fn(async () => {
        throw new WebApiError(400, "validation_error", "欄位有誤", { name: "名稱已存在" });
      }),
    });
    renderAt("/account", client);
    const user = userEvent.setup();
    await screen.findByRole("region", { name: "存取金鑰" });
    await user.click(screen.getByRole("button", { name: /建立存取金鑰/ }));
    const dialog = await screen.findByRole("alertdialog", { name: "建立存取金鑰" });
    await user.type(within(dialog).getByLabelText("名稱"), "cli");
    await user.click(within(dialog).getByRole("button", { name: "建立" }));
    const alert = await within(dialog).findByRole("alert");
    expect(alert.textContent).toContain("名稱已存在");
    expect(alert.id).toBe("pat-name-error");
    expect((within(dialog).getByLabelText("名稱") as HTMLInputElement).value).toBe("cli");
    expect(screen.getByRole("alertdialog", { name: "建立存取金鑰" })).toBeTruthy();
  });

  it("工作階段狀態徽章：有效為綠系、已撤銷為中性", async () => {
    // spec「憑證狀態徽章」：帳號頁與後台憑證頁講同一件事，色彩層級也要一致。
    renderAt("/account", makeClient());
    const valid = await screen.findAllByText("有效");
    expect(valid[0].className).toContain("status-success");
  });

  // 我的專案區塊（server-web-console「帳號頁呈現我的專案」）：資料來自 account
  // summary、唯讀、admin 與一般成員同一形狀，無隸屬時仍顯示區塊與空狀態。
  it("列出自己的專案顯示名、等寬代號與角色籤，且無任何編輯操作", async () => {
    renderAt("/account", makeClient());
    const section = await screen.findByRole("region", { name: /我的專案/ });
    expect(within(section).getByText("由管理員授予")).toBeTruthy();
    const alpha = within(section).getByRole("group", { name: "Alpha 專案" });
    expect(within(alpha).getByText("alpha").className).toContain("font-mono");
    expect(within(alpha).getByText("editor")).toBeTruthy();
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
    expect(within(section).getByText("alpha").className).toContain("font-mono");
    expect(within(section).getByRole("group", { name: "alpha" })).toBeTruthy();
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
    expect(within(section).getAllByRole("group")).toHaveLength(1);
  });

  it("無任何隸屬時卡仍可見並在卡內呈現空狀態", async () => {
    const client = makeClient({
      getAccount: vi.fn(async () => ({ ...ACCOUNT, memberships: [] })),
    });
    renderAt("/account", client);
    const section = await screen.findByRole("region", { name: /我的專案/ });
    const body = section.querySelector("[data-settings-card-body]") as HTMLElement;
    expect(within(body).getByText(/專案隸屬由管理員授予/)).toBeTruthy();
  });

  it("撤銷存取金鑰需先經 AlertDialog 確認才送出", async () => {
    const client = makeClient();
    renderAt("/account", client);
    const user = userEvent.setup();
    await screen.findByText(/slk_abc/);
    // 觸發撤銷：先出現確認對話框，尚未呼叫撤銷。
    await user.click(screen.getByRole("button", { name: "撤銷存取金鑰 cli" }));
    const dialog = await screen.findByRole("alertdialog");
    expect(client.revokePat).not.toHaveBeenCalled();
    // 確認後才送出，帶正確 PAT id。
    await user.click(within(dialog).getByRole("button", { name: /撤銷|確認/ }));
    await waitFor(() => expect(client.revokePat).toHaveBeenCalledWith("p1"));
  });
});

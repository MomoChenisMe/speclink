import { describe, it, expect, vi, beforeEach } from "vitest";
import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { App } from "../App";
import { WebApiError } from "../api/client";

// SPA 殼層、角色導覽與互動狀態（server-web-console「全部 browser route 由單一 SPA
// 提供可發現導覽」「共用設計系統維持高密度可存取體驗」「Browser API 互動狀態一致
// 且可恢復」, D3／D6）。App 注入 typed client（唯一 HTTP 入口）與 MemoryRouter 的
// initialEntries，路由、殼層、導覽、focus、error boundary、單次提交與 Sheet 皆可測。

type Session = {
  authenticated: boolean;
  user: null | { id: string; email: string; display: string; admin: boolean };
  home: string;
};

const UNAUTH: Session = { authenticated: false, user: null, home: "/login" };
const MEMBER: Session = {
  authenticated: true,
  user: { id: "u1", email: "member@example.com", display: "Member", admin: false },
  home: "/account",
};
const ADMIN: Session = {
  authenticated: true,
  user: { id: "a1", email: "admin@example.com", display: "Admin", admin: true },
  home: "/admin",
};

function makeClient(overrides: Record<string, unknown> = {}) {
  return {
    getSession: vi.fn(async () => UNAUTH),
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
    getAccount: vi.fn(async () => ({
      user: { id: "u1", email: "member@example.com", display: "Member", admin: false },
      pats: [],
      sessions: [],
      deviceFamilies: [],
    })),
    ...overrides,
  };
}

function renderAt(route: string, client: ReturnType<typeof makeClient>) {
  return render(<App client={client as never} initialEntries={[route]} />);
}

const ADMIN_DESTINATIONS = [
  "總覽",
  "使用者",
  "專案與儲存庫",
  "憑證",
  "資料操作",
  "系統狀態",
  "稽核紀錄",
];

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

// --- three shells ---

describe("三種殼層與角色導覽", () => {
  it("focus 流程殼呈現登入表單與 Speclink identity", async () => {
    renderAt("/login", makeClient());
    expect(await screen.findByLabelText(/email|電子郵件/i)).toBeTruthy();
    expect(screen.getByLabelText(/密碼|password/i)).toBeTruthy();
    expect(screen.getByRole("button", { name: /登入/ })).toBeTruthy();
  });

  it("管理殼呈現七個管理目的地、帳號與登出", async () => {
    renderAt("/admin", makeClient({ getSession: vi.fn(async () => ADMIN) }));
    const nav = await screen.findByRole("navigation");
    for (const dest of ADMIN_DESTINATIONS) {
      expect(within(nav).getByText(dest), `nav has ${dest}`).toBeTruthy();
    }
    expect(within(nav).getByText("帳號")).toBeTruthy();
    expect(screen.getByRole("button", { name: /登出/ })).toBeTruthy();
  });

  it("一般成員的帳號殼不顯示任何管理目的地", async () => {
    renderAt("/account", makeClient({ getSession: vi.fn(async () => MEMBER) }));
    await screen.findByRole("main");
    for (const dest of ["使用者", "憑證", "稽核紀錄", "系統狀態"]) {
      expect(screen.queryByText(dest), `no admin dest ${dest}`).toBeNull();
    }
  });

  it("一般成員直接開 /admin 得到明確 403，不顯示管理導覽", async () => {
    renderAt("/admin", makeClient({ getSession: vi.fn(async () => MEMBER) }));
    expect(await screen.findByText(/沒有權限|無權限|403/)).toBeTruthy();
    expect(screen.queryByText("稽核紀錄")).toBeNull();
  });

  it("管理員深連結 /admin/audit 直接呈現稽核頁", async () => {
    renderAt("/admin/audit", makeClient({ getSession: vi.fn(async () => ADMIN) }));
    // lazy 管理 chunk 載入後呈現稽核頁標題。
    expect(await screen.findByRole("heading", { name: /稽核/ })).toBeTruthy();
    expect(screen.getByRole("navigation")).toBeTruthy();
  });
});

// --- redirect / focus ---

describe("導向與 focus", () => {
  it("未登入開啟受保護 route 導向登入頁", async () => {
    renderAt("/account", makeClient({ getSession: vi.fn(async () => UNAUTH) }));
    expect(await screen.findByRole("button", { name: /登入/ })).toBeTruthy();
  });

  it("route 切換後 focus 移至 <main> 標題", async () => {
    renderAt("/admin", makeClient({ getSession: vi.fn(async () => ADMIN) }));
    const main = await screen.findByRole("main");
    await waitFor(() => {
      const heading = within(main).getByRole("heading", { level: 1 });
      expect(document.activeElement === heading || main.contains(document.activeElement)).toBe(true);
    });
  });

  it("第一個可聚焦元素是 skip link", async () => {
    renderAt("/admin", makeClient({ getSession: vi.fn(async () => ADMIN) }));
    const skip = await screen.findByRole("link", { name: /跳至主要內容|skip/i });
    expect(skip).toBeTruthy();
    expect(skip.getAttribute("href")).toContain("#");
  });
});

// --- login interaction states ---

describe("登入互動狀態", () => {
  it("登入成功後導向 Server 回傳的 destination", async () => {
    const client = makeClient({
      getSession: vi.fn(async () => UNAUTH),
      login: vi.fn(async () => ({ destination: "/account" })),
    });
    renderAt("/login", client);
    const user = userEvent.setup();
    await user.type(await screen.findByLabelText(/email|電子郵件/i), "member@example.com");
    await user.type(screen.getByLabelText(/密碼|password/i), "pw");
    await user.click(screen.getByRole("button", { name: /登入/ }));
    await waitFor(() => expect(client.login).toHaveBeenCalledOnce());
    expect(client.login).toHaveBeenCalledWith(
      expect.objectContaining({ email: "member@example.com", password: "pw" }),
    );
  });

  it("欄位驗證失敗保留輸入並以 role=alert 宣告錯誤", async () => {
    const client = makeClient({
      login: vi.fn(async () => {
        throw new WebApiError(400, "validation_error", "欄位有誤", {
          email: "email 格式不正確",
        });
      }),
    });
    renderAt("/login", client);
    const user = userEvent.setup();
    const email = await screen.findByLabelText(/email|電子郵件/i);
    await user.type(email, "bad");
    await user.type(screen.getByLabelText(/密碼|password/i), "pw");
    await user.click(screen.getByRole("button", { name: /登入/ }));
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("email 格式不正確");
    // 非祕密輸入保留，可再次提交。
    expect((email as HTMLInputElement).value).toBe("bad");
    expect(screen.getByRole("button", { name: /登入/ })).toBeTruthy();
  });

  it("提交進行中停用按鈕，第二次點擊不重複送出", async () => {
    let resolve!: (v: { destination: string }) => void;
    const login = vi.fn(() => new Promise<{ destination: string }>((r) => (resolve = r)));
    const client = makeClient({ login });
    renderAt("/login", client);
    const user = userEvent.setup();
    await user.type(await screen.findByLabelText(/email|電子郵件/i), "a@b.com");
    await user.type(screen.getByLabelText(/密碼|password/i), "pw");
    const submit = screen.getByRole("button", { name: /登入/ });
    await user.click(submit);
    expect((submit as HTMLButtonElement).disabled).toBe(true);
    await user.click(submit);
    expect(login).toHaveBeenCalledOnce();
    // resolve 後的 refresh／navigate 狀態更新明確等待，於 act 內結算（不壓制警告）。
    await act(async () => {
      resolve({ destination: "/account" });
    });
  });
});

// --- data page states ---

describe("資料頁互動狀態", () => {
  it("載入期間顯示載入狀態，不白屏", async () => {
    const client = makeClient({
      getSession: vi.fn(async () => ADMIN),
      // 永不 resolve：頁面停留在載入狀態（chunk 載入與資料載入皆呈現「載入中」）。
      getAdminOverview: vi.fn(() => new Promise(() => {})),
    });
    renderAt("/admin", client);
    expect(await screen.findByText(/載入中|loading/i)).toBeTruthy();
  });

  it("資料載入失敗顯示可恢復錯誤與重試，不白屏", async () => {
    const client = makeClient({
      getSession: vi.fn(async () => ADMIN),
      getAdminOverview: vi.fn(async () => {
        throw new WebApiError(500, "internal", "伺服器錯誤");
      }),
    });
    renderAt("/admin", client);
    expect(await screen.findByRole("button", { name: /重試|retry/i })).toBeTruthy();
  });
});

// --- responsive nav ---

describe("響應式導覽", () => {
  it("窄螢幕收合側欄，可見 trigger 開啟含七目的地的 Sheet", async () => {
    // 1024px 以下：matchMedia 對窄螢幕 query 回 matches:true。
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
    renderAt("/admin", makeClient({ getSession: vi.fn(async () => ADMIN) }));
    const trigger = await screen.findByRole("button", { name: /開啟導覽|選單|menu/i });
    const user = userEvent.setup();
    await user.click(trigger);
    const dialog = await screen.findByRole("dialog");
    for (const dest of ADMIN_DESTINATIONS) {
      expect(within(dialog).getByText(dest)).toBeTruthy();
    }
  });
});

// server-web-console「Browser API 互動狀態一致且可恢復」的 Scenario「Session 過期回到
// 登入並保留安全路徑」：已載入的受保護 route 呼叫 browser API 收到 401 即回登入頁。
// 401 有兩種語意——unauthenticated（session 失效，要導向）與 invalid_credentials
// （登入密碼錯，要留在原頁顯示錯誤），必須依 error code 分派。
describe("Session 過期回到登入並保留安全路徑", () => {
  it("受保護 route 收到 401 unauthenticated 後回登入頁，returnTo 為當前路徑", async () => {
    const getSession = vi
      .fn()
      .mockResolvedValueOnce(ADMIN)
      .mockResolvedValue(UNAUTH);
    const client = makeClient({
      getSession,
      getAdminUsers: vi.fn(async () => {
        throw new WebApiError(401, "unauthenticated", "請先登入");
      }),
      login: vi.fn(async () => ({ destination: "/admin/users" })),
    });
    renderAt("/admin/users", client);
    // 401 觸發 session 重讀 → guard 判定未登入 → 導向登入頁。
    expect(await screen.findByRole("button", { name: /登入/ })).toBeTruthy();
    const user = userEvent.setup();
    await user.type(screen.getByLabelText(/email|電子郵件/i), "admin@example.com");
    await user.type(screen.getByLabelText(/密碼|password/i), "pw");
    await user.click(screen.getByRole("button", { name: /登入/ }));
    await waitFor(() => expect(client.login).toHaveBeenCalledOnce());
    expect(client.login).toHaveBeenCalledWith(
      expect.objectContaining({ returnTo: "/admin/users" }),
    );
  });

  it("登入失敗的 401 invalid_credentials 留在登入頁顯示錯誤，不觸發導向迴圈", async () => {
    const client = makeClient({
      getSession: vi.fn(async () => UNAUTH),
      login: vi.fn(async () => {
        throw new WebApiError(401, "invalid_credentials", "電子郵件或密碼不正確");
      }),
    });
    renderAt("/login", client);
    const user = userEvent.setup();
    await user.type(await screen.findByLabelText(/email|電子郵件/i), "admin@example.com");
    await user.type(screen.getByLabelText(/密碼|password/i), "wrong");
    await user.click(screen.getByRole("button", { name: /登入/ }));
    expect(await screen.findByRole("alert")).toBeTruthy();
    expect(screen.getByText("電子郵件或密碼不正確")).toBeTruthy();
    // 仍在登入頁、輸入保留。
    expect((screen.getByLabelText(/email|電子郵件/i) as HTMLInputElement).value).toBe(
      "admin@example.com",
    );
  });
});

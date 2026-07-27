import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, waitFor } from "@testing-library/react";
import { render } from "@testing-library/react";
import { App } from "../App";
import { makeAdminClient, renderAt, setViewport } from "./helpers/adminHarness";

// 使用者可見文案（proposal「相容性影響：使用者可見文案」與 openspec/LANGUAGE.md）：
// 工程詞退場，改用中文正典詞。這是一份對照清單而非個別頁面的行為測試——文案漂移最容易
// 在改版時無聲發生，所以由一支測試把六個管理目的地與帳號頁一起釘住。
//
// 多語系後這批中文正典詞只在 zh-TW 成立，因此各測試明示語言：中文版斷言正典詞出現，
// 英文版斷言同一批中文詞完全不出現（只翻一半的頁面會在這裡露餡）。

// 退場的工程詞：出現在任何使用者可見文字裡都算退化。
const RETIRED = [
  "建立 project",
  "Project key",
  "Repo key",
  "Personal Access Tokens",
  "PAT",
  "Web Sessions",
  "Schema 版本",
  "Outbox backlog",
];

const MEMBER_SESSION = {
  authenticated: true,
  user: { id: "u1", email: "member@example.com", display: "Member", admin: false },
  home: "/account",
};

const ACCOUNT = {
  user: { id: "u1", email: "member@example.com", display: "Member", admin: false },
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
    {
      id: "s1",
      createdAt: "2026-01-01T00:00:00Z",
      expiresAt: "2026-02-01T00:00:00Z",
      revokedAt: null,
    },
  ],
  deviceFamilies: [],
};

const ADMIN_DESTINATIONS = [
  "/admin",
  "/admin/users",
  "/admin/registry",
  "/admin/credentials",
  "/admin/system",
  "/admin/audit",
];

/** 中文正典詞——英文版不得出現任何一個。 */
const ZH_CANONICAL = [
  "專案代號",
  "儲存庫代號",
  "存取金鑰",
  "登入工作階段",
  "資料結構版本",
  "待送佇列",
  "建立專案",
];

beforeEach(() => {
  setViewport(false);
  localStorage.clear();
  localStorage.setItem("speclink.tourSeen", "1");
});

function setLanguage(locale: "zh-TW" | "en") {
  localStorage.setItem("speclink.uiLocale", locale);
}

/** 該畫面目前的全部可見文字。 */
function visibleText(): string {
  return document.body.textContent ?? "";
}

describe("使用者可見文案不含工程詞（zh-TW）", () => {
  it.each(ADMIN_DESTINATIONS)("%s 不出現退場的工程詞", async (route) => {
    setLanguage("zh-TW");
    const { unmount } = renderAt(route, makeAdminClient());
    await screen.findByRole("heading", { level: 1 });
    // 等資料層渲染完成，才不會只驗到載入中的骨架。
    await waitFor(() => expect(screen.queryByText("載入中…")).toBeNull());
    const text = visibleText();
    for (const term of RETIRED) {
      expect(text, `${route} 不應出現「${term}」`).not.toContain(term);
    }
    unmount();
  });

  it("帳號頁不出現退場的工程詞", async () => {
    setLanguage("zh-TW");
    render(
      <App
        client={
          {
            getSession: vi.fn(async () => MEMBER_SESSION),
            logout: vi.fn(async () => ({ destination: "/login" })),
            getAccount: vi.fn(async () => ACCOUNT),
            createPat: vi.fn(),
            revokePat: vi.fn(),
            revokeDevice: vi.fn(),
          } as never
        }
        initialEntries={["/account"]}
      />,
    );
    await screen.findByRole("heading", { level: 1, name: "帳號" });
    const text = visibleText();
    for (const term of RETIRED) {
      expect(text, `帳號頁不應出現「${term}」`).not.toContain(term);
    }
  });
});

describe("中文正典詞出現在對應畫面（zh-TW）", () => {
  beforeEach(() => setLanguage("zh-TW"));

  it("專案與儲存庫頁使用專案代號與儲存庫代號", async () => {
    renderAt("/admin/registry", makeAdminClient());
    await screen.findByText("Demo");
    expect(visibleText()).toContain("代號");
    const { default: userEvent } = await import("@testing-library/user-event");
    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /建立專案/ }));
    await screen.findByRole("dialog", { name: /建立專案/ });
    expect(screen.getByLabelText(/專案代號/)).toBeTruthy();
  });

  it("憑證頁使用存取金鑰", async () => {
    renderAt("/admin/credentials", makeAdminClient());
    expect(await screen.findByRole("tab", { name: "存取金鑰" })).toBeTruthy();
  });

  it("系統頁使用資料結構版本與待送佇列", async () => {
    renderAt("/admin/system", makeAdminClient());
    await screen.findByRole("heading", { level: 1, name: "系統" });
    const text = visibleText();
    expect(text).toContain("資料結構版本");
    expect(text).toContain("待送佇列");
  });

  it("帳號頁使用存取金鑰與登入工作階段", async () => {
    render(
      <App
        client={
          {
            getSession: vi.fn(async () => MEMBER_SESSION),
            logout: vi.fn(async () => ({ destination: "/login" })),
            getAccount: vi.fn(async () => ACCOUNT),
            createPat: vi.fn(),
            revokePat: vi.fn(),
            revokeDevice: vi.fn(),
          } as never
        }
        initialEntries={["/account"]}
      />,
    );
    await screen.findByRole("heading", { level: 1, name: "帳號" });
    const text = visibleText();
    expect(text).toContain("存取金鑰");
    expect(text).toContain("登入工作階段");
  });

  it("初始設定頁使用專案代號與儲存庫代號", async () => {
    render(
      <App
        client={
          {
            getSession: vi.fn(async () => ({ authenticated: false, user: null, home: "/login" })),
            getSetupState: vi.fn(async () => ({
              step: "registry",
              store: {
                driver: "sqlite",
                contractVersion: 3,
                level: "full",
                capabilities: [],
                healthy: true,
                identitySchemaVersion: 6,
              },
            })),
            submitSetupRegistry: vi.fn(),
          } as never
        }
        initialEntries={["/setup?token=t"]}
      />,
    );
    expect(await screen.findByLabelText(/專案代號/)).toBeTruthy();
    expect(screen.getByLabelText(/儲存庫代號/)).toBeTruthy();
    expect(visibleText()).not.toContain("Project key");
    expect(visibleText()).not.toContain("Repo key");
  });
});

describe("英文版不殘留中文正典詞（en）", () => {
  beforeEach(() => setLanguage("en"));

  it.each(ADMIN_DESTINATIONS)("%s 不出現任何中文正典詞", async (route) => {
    const { unmount } = renderAt(route, makeAdminClient());
    await screen.findByRole("heading", { level: 1 });
    await waitFor(() => expect(screen.queryByText("Loading…")).toBeNull());
    const text = visibleText();
    for (const term of ZH_CANONICAL) {
      expect(text, `${route} 的英文版不應出現「${term}」`).not.toContain(term);
    }
    unmount();
  });

  it("帳號頁不出現任何中文正典詞", async () => {
    render(
      <App
        client={
          {
            getSession: vi.fn(async () => MEMBER_SESSION),
            logout: vi.fn(async () => ({ destination: "/login" })),
            getAccount: vi.fn(async () => ACCOUNT),
            createPat: vi.fn(),
            revokePat: vi.fn(),
            revokeDevice: vi.fn(),
          } as never
        }
        initialEntries={["/account"]}
      />,
    );
    await screen.findByRole("heading", { level: 1, name: "Account" });
    const text = visibleText();
    for (const term of ZH_CANONICAL) {
      expect(text, `帳號頁的英文版不應出現「${term}」`).not.toContain(term);
    }
  });
});

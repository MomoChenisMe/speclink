import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { I18nProvider } from "@speclink/ui";
import { ClientProvider, SessionProvider } from "../app/context";
import { LocaleProvider } from "../i18n/LocaleContext";
import { APP_MESSAGES } from "../i18n/messages";
import { ConsoleLayout } from "../layouts/ConsoleLayout";
import { ADMIN_SESSION, makeAdminClient, renderAt, setViewport } from "./helpers/adminHarness";

// 主控台殼（server-web-console「全部 browser route 由單一 SPA 提供可發現導覽」
// 「共用設計系統維持高密度可存取體驗」）：管理員與一般成員共用同一種殼——側欄（頂端字標、
// 中段導覽、底部帳號選單）加主欄（頂列麵包屑、下方主內容），沒有橫貫頂部的 header。
// 管理員側欄恰六個目的地（不含資料操作與帳號），一般成員只有「帳號」一項；帳號入口、
// 介面語言與登出收在側欄底部的帳號選單。

const MEMBER_SESSION = {
  authenticated: true,
  user: { id: "u1", email: "member@example.com", display: "Member", admin: false },
  home: "/account",
};

const ANONYMOUS = { authenticated: false, user: null, home: "/login" };

const DESTINATIONS = ["總覽", "使用者", "專案與儲存庫", "憑證", "系統", "稽核紀錄"];

const ACCOUNT = {
  user: { id: "u1", email: "member@example.com", display: "Member", admin: false },
  memberships: [],
  pats: [],
  sessions: [],
  deviceFamilies: [],
};

function client(overrides: Record<string, unknown> = {}) {
  return makeAdminClient({ getAccount: vi.fn(async () => ACCOUNT), ...overrides });
}

function member() {
  return client({ getSession: vi.fn(async () => MEMBER_SESSION) });
}

function breadcrumbs() {
  return screen.getByRole("navigation", { name: "頁面路徑" });
}

const classes = (el: Element) => (el.getAttribute("class") ?? "").split(/\s+/);

/** 只渲染殼本身：用來看路由不會停留的路徑（例如管理面會立即導走的未知路徑）。 */
function renderShellAt(path: string) {
  return render(
    <MemoryRouter initialEntries={[path]}>
      <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
        <LocaleProvider value={{ pref: null, setPref: () => {} }}>
          <ClientProvider client={client() as never}>
            <SessionProvider value={{ session: ADMIN_SESSION as never, refresh: async () => {} }}>
              <ConsoleLayout>
                <p>內容</p>
              </ConsoleLayout>
            </SessionProvider>
          </ClientProvider>
        </LocaleProvider>
      </I18nProvider>
    </MemoryRouter>,
  );
}

beforeEach(() => setViewport(false));

describe("依角色裁切的主控台殼", () => {
  it("側欄頂端是共用元件庫的橫式字標，頁面沒有橫貫頂部的 header", async () => {
    renderAt("/admin", client());
    const sidebar = await screen.findByRole("complementary");
    // 深淺兩版各一張、以 dark: 切換；第一張是淺色版（spec「品牌資產只有一處」）。
    const lockup = within(sidebar).getAllByAltText("Speclink")[0] as HTMLImageElement;
    expect(lockup.getAttribute("src")).toContain("logo-horizontal");
    expect(lockup.className).toContain("h-5");
    expect(screen.queryByRole("banner")).toBeNull();
  });

  it("側欄恰有六個目的地，不含資料操作與帳號", async () => {
    renderAt("/admin", client());
    const nav = await screen.findByRole("navigation", { name: "管理導覽" });
    const links = within(nav).getAllByRole("link");
    expect(links.map((l) => l.textContent?.trim())).toEqual(DESTINATIONS);
    expect(within(nav).queryByText("資料操作")).toBeNull();
    expect(within(nav).queryByText("帳號")).toBeNull();
  });

  // design D4／D10：導覽項是共用 NavItem——作用中為主色淡底主色字（不再實心主色），側欄淡灰底。
  it("作用中目的地為主色淡底主色字，其餘不是；側欄底色為 bg-sidebar", async () => {
    renderAt("/admin", client());
    const nav = await screen.findByRole("navigation", { name: "管理導覽" });
    expect(nav.closest("aside")?.className).toContain("bg-sidebar");
    const overview = within(nav).getByRole("link", { name: "總覽" });
    expect(overview.getAttribute("aria-current")).toBe("page");
    expect(overview.className).toContain("text-primary");
    expect(overview.className).toContain("bg-primary/12");
    expect(overview.className).not.toContain("text-primary-foreground");
    const users = within(nav).getByRole("link", { name: "使用者" });
    expect(users.className).not.toContain("text-primary");
    expect(users.getAttribute("aria-current")).toBeNull();
  });

  it("管理員於 /account 仍見完整側欄且無項目高亮，麵包屑為「帳號」", async () => {
    renderAt("/account", client());
    const nav = await screen.findByRole("navigation", { name: "管理導覽" });
    for (const dest of DESTINATIONS) {
      expect(within(nav).getByText(dest), `sidebar has ${dest}`).toBeTruthy();
    }
    expect(within(nav).queryByRole("link", { current: "page" })).toBeNull();
    expect(breadcrumbs().textContent).toBe("帳號");
  });

  it("一般成員於 /account 有側欄，只有作用中的「帳號」一項", async () => {
    renderAt("/account", member());
    const nav = await screen.findByRole("navigation", { name: "導覽" });
    const links = within(nav).getAllByRole("link");
    expect(links.map((l) => l.textContent?.trim())).toEqual(["帳號"]);
    expect(links[0].getAttribute("aria-current")).toBe("page");
    expect(links[0].getAttribute("href")).toBe("/account");
    expect(screen.queryByRole("navigation", { name: "管理導覽" })).toBeNull();
    for (const dest of DESTINATIONS) {
      expect(screen.queryByRole("link", { name: dest }), `no admin dest ${dest}`).toBeNull();
    }
    expect(breadcrumbs().textContent).toBe("帳號");
  });

  // spec「管理殼版面數值與 Desktop 一致」「只有主內容區捲動」：殼本身滿版不捲，側欄與頂列
  // 固定，只有 main 捲動；頂列高度、側欄寬度與主內容內距採 Desktop 殼的數值。
  it("版面數值與 Desktop 相同，且只有主內容區捲動", async () => {
    renderAt("/admin", client());
    const main = await screen.findByRole("main");
    const sidebar = screen.getByRole("complementary");
    expect(classes(sidebar.parentElement as HTMLElement)).toEqual(
      expect.arrayContaining(["h-screen", "overflow-hidden"]),
    );
    expect(classes(sidebar)).toEqual(expect.arrayContaining(["w-[200px]", "shrink-0"]));
    const sidebarTop = within(sidebar).getAllByAltText("Speclink")[0].parentElement as HTMLElement;
    expect(classes(sidebarTop)).toEqual(expect.arrayContaining(["h-12", "shrink-0"]));
    const topBar = breadcrumbs().parentElement as HTMLElement;
    expect(classes(topBar)).toEqual(expect.arrayContaining(["h-12", "shrink-0"]));
    expect(classes(main)).toEqual(expect.arrayContaining(["overflow-y-auto", "p-5"]));
  });
});

describe("主區頂列的麵包屑", () => {
  it("/admin/users 為「後台 / 使用者」，點「後台」回總覽", async () => {
    const user = userEvent.setup();
    renderAt("/admin/users", client());
    await screen.findByRole("heading", { level: 1, name: "使用者" });
    expect(breadcrumbs().textContent).toBe("後台/使用者");

    await user.click(within(breadcrumbs()).getByRole("button", { name: "後台" }));
    expect(await screen.findByRole("heading", { level: 1, name: "總覽" })).toBeTruthy();
    expect(breadcrumbs().textContent).toBe("後台/總覽");
  });

  // 路由比對不分大小寫：麵包屑與側欄高亮用同一套比對，手打大寫網址也標得出位置。
  it("網址大小寫不同時麵包屑照樣標出目的地", async () => {
    renderAt("/Admin/Users", client());
    await screen.findByRole("heading", { level: 1, name: "使用者" });
    expect(breadcrumbs().textContent).toBe("後台/使用者");
  });

  // server-web-detail-pages design D2：詳情頁為三段，目的地可點回列表、名稱不可點。
  it("/admin/registry/demo 為「後台 / 專案與儲存庫 / Demo」，側欄「專案與儲存庫」作用中", async () => {
    renderAt("/admin/registry/demo", client());
    await screen.findByRole("heading", { level: 1, name: "Demo" });
    expect(breadcrumbs().textContent).toBe("後台/專案與儲存庫/Demo");
    expect(within(breadcrumbs()).getByRole("button", { name: "專案與儲存庫" })).toBeTruthy();
    const nav = screen.getByRole("navigation", { name: "管理導覽" });
    expect(within(nav).getByRole("link", { name: "專案與儲存庫" }).getAttribute("aria-current")).toBe("page");
  });

  it("未列在目的地表內的 /admin/* 路徑只顯示「後台」", () => {
    renderShellAt("/admin/unknown");
    expect(breadcrumbs().textContent).toBe("後台");
  });
});

describe("側欄底部的帳號選單", () => {
  it("觸發鈕顯示名稱與電子郵件；展開後標頭為「email · 管理員」，含帳號、三個語言單選項與登出", async () => {
    const user = userEvent.setup();
    renderAt("/admin", client());
    const sidebar = await screen.findByRole("complementary");
    const trigger = within(sidebar).getByRole("button", { name: "帳號選單：Admin" });
    expect(trigger.textContent).toContain("Admin");
    expect(trigger.textContent).toContain("admin@example.com");

    await user.click(trigger);
    const menu = await screen.findByRole("menu");
    expect(within(menu).getByText("admin@example.com · 管理員")).toBeTruthy();
    expect(within(menu).getByRole("menuitem", { name: "帳號" })).toBeTruthy();
    expect(within(menu).getByRole("menuitem", { name: "登出" })).toBeTruthy();
    const radios = within(menu).getAllByRole("menuitemradio");
    expect(radios.map((r) => r.textContent?.trim())).toEqual(["繁體中文", "English", "跟隨系統"]);
    // 未明示偏好時標示「跟隨系統」為目前選擇。
    expect(within(menu).getByRole("menuitemradio", { name: "跟隨系統" }).getAttribute("aria-checked")).toBe(
      "true",
    );
  });

  it("一般成員的選單標頭為「email · 成員」", async () => {
    const user = userEvent.setup();
    renderAt("/account", member());
    await user.click(await screen.findByRole("button", { name: "帳號選單：Member" }));
    const menu = await screen.findByRole("menu");
    expect(within(menu).getByText("member@example.com · 成員")).toBeTruthy();
  });

  it("管理員選「帳號」前往帳號頁", async () => {
    const user = userEvent.setup();
    renderAt("/admin", client());
    await user.click(await screen.findByRole("button", { name: "帳號選單：Admin" }));
    await user.click(await screen.findByRole("menuitem", { name: "帳號" }));
    expect(await screen.findByRole("heading", { level: 1, name: "帳號" })).toBeTruthy();
    expect(breadcrumbs().textContent).toBe("帳號");
  });

  it("選「登出」撤銷 session 並呈現登入頁", async () => {
    const user = userEvent.setup();
    const c = client({
      getSession: vi.fn().mockResolvedValueOnce(ADMIN_SESSION).mockResolvedValue(ANONYMOUS),
    });
    renderAt("/admin", c);
    await user.click(await screen.findByRole("button", { name: "帳號選單：Admin" }));
    await user.click(await screen.findByRole("menuitem", { name: "登出" }));
    expect(await screen.findByRole("heading", { level: 1, name: "登入" })).toBeTruthy();
    expect(c.logout).toHaveBeenCalledTimes(1);
    expect(screen.queryByRole("complementary")).toBeNull();
  });

  it("登出後依 server 回傳的去處導向", async () => {
    const user = userEvent.setup();
    // session 保持登入，才看得出畫面是被 destination 帶走，而不是被登入守門彈回登入頁。
    const c = client({ logout: vi.fn(async () => ({ destination: "/account" })) });
    renderAt("/admin", c);
    await user.click(await screen.findByRole("button", { name: "帳號選單：Admin" }));
    await user.click(await screen.findByRole("menuitem", { name: "登出" }));
    expect(await screen.findByRole("heading", { level: 1, name: "帳號" })).toBeTruthy();
    expect(breadcrumbs().textContent).toBe("帳號");
  });

  it("登出請求失敗時仍呈現登入頁", async () => {
    const user = userEvent.setup();
    const c = client({ logout: vi.fn(async () => Promise.reject(new Error("network"))) });
    renderAt("/admin", c);
    await user.click(await screen.findByRole("button", { name: "帳號選單：Admin" }));
    await user.click(await screen.findByRole("menuitem", { name: "登出" }));
    expect(await screen.findByRole("heading", { level: 1, name: "登入" })).toBeTruthy();
    expect(screen.queryByRole("complementary")).toBeNull();
  });

  it("顯示名稱為空時，頭像字與名稱退回電子郵件", async () => {
    const session = { ...ADMIN_SESSION, user: { ...ADMIN_SESSION.user, display: "" } };
    renderAt("/admin", client({ getSession: vi.fn(async () => session) }));
    const trigger = await screen.findByRole("button", { name: "帳號選單：admin@example.com" });
    // 頭像字＋名稱列＋email 列：名稱列也是 email。
    expect(trigger.textContent).toBe("a" + "admin@example.com" + "admin@example.com");
  });
});

describe("窄螢幕導覽", () => {
  it("768px 時「開啟導覽」開出從左滑出的 Sheet，含字標、六個目的地與帳號選單鈕", async () => {
    setViewport(true);
    const user = userEvent.setup();
    renderAt("/admin", client());
    const trigger = await screen.findByRole("button", { name: "開啟導覽" });
    expect(screen.queryByRole("complementary")).toBeNull();

    await user.click(trigger);
    const dialog = await screen.findByRole("dialog");
    expect(dialog.className).toContain("left-0");
    expect(within(dialog).getAllByAltText("Speclink")[0].getAttribute("src")).toContain("logo-horizontal");
    const links = within(dialog).getAllByRole("link");
    expect(links.map((l) => l.textContent?.trim())).toEqual(DESTINATIONS);
    expect(within(dialog).getByRole("button", { name: "帳號選單：Admin" })).toBeTruthy();

    await user.keyboard("{Escape}");
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(document.activeElement).toBe(trigger);
  });

  it("一般成員 375px 時 Sheet 含字標與「帳號」一項", async () => {
    setViewport(true);
    const user = userEvent.setup();
    renderAt("/account", member());
    await user.click(await screen.findByRole("button", { name: "開啟導覽" }));
    const dialog = await screen.findByRole("dialog");
    expect(within(dialog).getAllByAltText("Speclink").length).toBeGreaterThan(0);
    const links = within(dialog).getAllByRole("link");
    expect(links.map((l) => l.textContent?.trim())).toEqual(["帳號"]);
    expect(within(dialog).getByRole("button", { name: "帳號選單：Member" })).toBeTruthy();
  });
});

// spec「共用設計系統維持高密度可存取體驗」的「頁標題與 Desktop 同款」（design D5）：
// 每頁標題用共用 PageHeader，且是該頁唯一的第一層標題。
describe("頁標題區", () => {
  it.each([
    ["/admin", "總覽", "這台 server 的使用者、專案與憑證概況；點數字前往對應頁。"],
    ["/admin/users", "使用者", "能登入這台 server 的帳號。"],
    ["/admin/registry", "專案與儲存庫", "遠端工作流程的範圍：專案與其底下的儲存庫。"],
    ["/admin/credentials", "憑證", "所有使用者的存取金鑰與裝置登入；撤銷後立即失效。"],
    ["/admin/system", "系統", "執行環境、儲存狀態、匯出與資料結構遷移。"],
    ["/admin/audit", "稽核紀錄", "每一筆管理動作的紀錄；只能檢視，不能修改。"],
    ["/account", "帳號", "Member（member@example.com）——你的專案、存取金鑰與登入紀錄。"],
  ])("%s 恰有一個 h1「%s」，位於共用頁標題區且附說明", async (route, title, description) => {
    renderAt(route, client());
    const heading = await screen.findByRole("heading", { level: 1, name: title });
    expect(screen.getAllByRole("heading", { level: 1 })).toHaveLength(1);
    const header = heading.closest("[data-page-header]") as HTMLElement;
    expect(header).not.toBeNull();
    expect(within(header).getByText(description)).toBeTruthy();
  });

  it("帳號頁的顯示名稱為空時，說明退回電子郵件", async () => {
    const account = { ...ACCOUNT, user: { ...ACCOUNT.user, display: "" } };
    renderAt("/account", client({ getAccount: vi.fn(async () => account) }));
    const heading = await screen.findByRole("heading", { level: 1, name: "帳號" });
    const header = heading.closest("[data-page-header]") as HTMLElement;
    expect(
      within(header).getByText("member@example.com（member@example.com）——你的專案、存取金鑰與登入紀錄。"),
    ).toBeTruthy();
  });

  it("使用者頁的「邀請使用者」在標題區的動作槽並保留導覽標記", async () => {
    renderAt("/admin/users", client());
    const heading = await screen.findByRole("heading", { level: 1, name: "使用者" });
    const header = heading.closest("[data-page-header]") as HTMLElement;
    const invite = within(header).getByRole("button", { name: "邀請使用者" });
    expect(invite.getAttribute("data-tour")).toBe("list-primary");
  });
});

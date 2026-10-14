import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Route, Routes, useLocation, useNavigate } from "react-router-dom";
import { I18nProvider } from "@speclink/ui";
import { WebApiError } from "../api/client";
import { ClientProvider, SessionProvider } from "../app/context";
import { APP_MESSAGES } from "../i18n/messages";
import { UsersPage } from "../pages/admin/UsersPage";
import {
  ADMIN_SESSION,
  makeAdminClient,
  makeUsers,
  renderAt,
  setViewport,
  USERS,
} from "./helpers/adminHarness";

// 管理使用者頁（server-web-console「管理列表以抽屜承載建立與編輯」「管理列表提供搜尋、篩選、
// 分頁與具引導的空狀態」「使用者頁呈現尚未接受的邀請」；server-web-list-pages design D2–D4）：
// 頁標題區右側是搜尋與狀態篩選，其下一張列表卡，卡頂「使用者｜邀請中」兩個分頁，卡底共用換頁列；
// 使用者列整列可點開細節抽屜、列內不含輸入控制項；建立與編輯一律進抽屜；破壞性動作維持確認框。

beforeEach(() => setViewport(false));

/** 目前網址與一顆「返回」鈕：驗證分頁狀態寫在網址、且切換分頁不新增瀏覽紀錄。 */
function LocationProbe() {
  const location = useLocation();
  const navigate = useNavigate();
  return (
    <>
      <output data-testid="location">{location.pathname + location.search}</output>
      <button type="button" onClick={() => navigate(-1)}>
        返回（測試）
      </button>
    </>
  );
}

/** 只渲染使用者頁本身（不經殼），網址由 LocationProbe 讀出。 */
function renderUsersPage(entries: string[], client = makeAdminClient()) {
  return render(
    <MemoryRouter initialEntries={entries} initialIndex={entries.length - 1}>
      <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
        <ClientProvider client={client as never}>
          <SessionProvider value={{ session: ADMIN_SESSION as never, refresh: async () => {} }}>
            <Routes>
              <Route path="/admin" element={<p>總覽頁</p>} />
              <Route path="/admin/users" element={<UsersPage />} />
            </Routes>
            <LocationProbe />
          </SessionProvider>
        </ClientProvider>
      </I18nProvider>
    </MemoryRouter>,
  );
}

const location = () => screen.getByTestId("location").textContent;
const tab = (name: RegExp) => screen.getByRole("tab", { name });
const tabCount = (name: RegExp) => tab(name).querySelector("[data-tab-count]")?.textContent;
const userRowNames = () =>
  screen.getAllByRole("button", { name: /^User \d+$/ }).map((b) => b.textContent?.match(/User \d+/)?.[0]);

describe("使用者分頁的列", () => {
  it("使用者列為可聚焦的按鈕、名稱即無障礙名稱；列內有頭像字、email、管理員籤、狀態籤與專案數，不含輸入控制項", async () => {
    renderAt("/admin/users", makeAdminClient());
    const admin = await screen.findByRole("button", { name: "Admin" });
    expect(admin.getAttribute("tabindex")).toBe("0");
    expect(admin.querySelector("[data-avatar]")?.textContent).toBe("A");
    expect(admin.textContent).toContain("admin@example.com");
    expect(within(admin).getByText("管理員")).toBeTruthy();
    expect(within(admin).getByText("有效").className).toContain("status-success");
    expect(within(admin).getByText("1 個專案")).toBeTruthy();

    const member = screen.getByRole("button", { name: "Member" });
    expect(within(member).queryByText("管理員")).toBeNull();
    // 列內不得有下拉、勾選框、文字輸入或按鈕。
    for (const role of ["combobox", "checkbox", "textbox", "button"]) {
      expect(within(member).queryAllByRole(role), `列內沒有 ${role}`).toHaveLength(0);
    }
  });

  it("已停權為琥珀系狀態籤、沒有隸屬時為「尚無專案」；顯示名稱為空時頭像字與名稱退回 email", async () => {
    const users = {
      ...USERS,
      users: [
        USERS.users[0],
        { ...USERS.users[1], active: false, memberships: [] },
        { ...USERS.users[1], id: "n1", display: "", email: "zoe@example.com" },
      ],
    };
    renderAt("/admin/users", makeAdminClient({ getAdminUsers: vi.fn(async () => users) }));
    const member = await screen.findByRole("button", { name: "Member" });
    expect(within(member).getByText("已停權").className).toContain("status-warning");
    expect(within(member).getByText("尚無專案")).toBeTruthy();
    const nameless = screen.getByRole("button", { name: "zoe@example.com" });
    expect(nameless.querySelector("[data-avatar]")?.textContent).toBe("z");
  });

  it("點整列開啟含概要／成員資格／憑證／稽核的細節抽屜", async () => {
    renderAt("/admin/users", makeAdminClient());
    const user = userEvent.setup();
    await user.click(await screen.findByRole("button", { name: "Member" }));
    const drawer = await screen.findByRole("dialog", { name: /Member/ });
    for (const name of ["概要", "成員資格", "憑證", "稽核"]) {
      expect(within(drawer).getByRole("tab", { name }), `抽屜有 ${name} 分頁`).toBeTruthy();
    }
  });

  it("以鍵盤移到列上按 Enter 同樣開啟細節抽屜", async () => {
    renderAt("/admin/users", makeAdminClient());
    const user = userEvent.setup();
    (await screen.findByRole("button", { name: "Member" })).focus();
    await user.keyboard("{Enter}");
    expect(await screen.findByRole("dialog", { name: /Member/ })).toBeTruthy();
  });

  it("沒有列尾的「檢視詳細資料」鈕，也沒有表格", async () => {
    renderAt("/admin/users", makeAdminClient());
    await screen.findByRole("button", { name: "Member" });
    expect(screen.queryByRole("button", { name: /檢視/ })).toBeNull();
    expect(screen.queryByRole("table")).toBeNull();
  });
});

describe("卡頂分頁與網址", () => {
  it("卡頂兩個分頁「使用者」「邀請中」標示未經搜尋與篩選的總數", async () => {
    renderAt("/admin/users", makeAdminClient());
    await screen.findByRole("button", { name: "Member" });
    expect(tabCount(/^使用者/)).toBe("2");
    expect(tabCount(/^邀請中/)).toBe("1");
    const user = userEvent.setup();
    await user.type(screen.getByRole("textbox", { name: "搜尋" }), "member");
    await waitFor(() => expect(screen.queryByRole("button", { name: "Admin" })).toBeNull());
    expect(tabCount(/^使用者/)).toBe("2");
  });

  it("開 /admin/users?tab=pending 直接顯示邀請中；切回使用者後網址沒有 tab，返回不會回到邀請中", async () => {
    renderUsersPage(["/admin", "/admin/users?tab=pending"]);
    expect(await screen.findByText("invited@example.com")).toBeTruthy();
    expect(tab(/^邀請中/).getAttribute("aria-selected")).toBe("true");
    const user = userEvent.setup();
    await user.click(tab(/^使用者/));
    expect(await screen.findByRole("button", { name: "Member" })).toBeTruthy();
    expect(location()).toBe("/admin/users");
    await user.click(screen.getByRole("button", { name: "返回（測試）" }));
    expect(location()).toBe("/admin");
  });

  it("從使用者分頁切到邀請中，網址帶 ?tab=pending；其他 tab 值視為使用者分頁", async () => {
    renderUsersPage(["/admin/users?tab=bogus"]);
    expect(await screen.findByRole("button", { name: "Member" })).toBeTruthy();
    expect(tab(/^使用者/).getAttribute("aria-selected")).toBe("true");
    const user = userEvent.setup();
    await user.click(tab(/^邀請中/));
    expect(location()).toBe("/admin/users?tab=pending");
  });

  it("狀態篩選只在使用者分頁出現", async () => {
    renderAt("/admin/users", makeAdminClient());
    expect(await screen.findByRole("combobox", { name: /狀態/ })).toBeTruthy();
    const user = userEvent.setup();
    await user.click(tab(/^邀請中/));
    await screen.findByText("invited@example.com");
    expect(screen.queryByRole("combobox", { name: /狀態/ })).toBeNull();
    expect(screen.getByRole("textbox", { name: "搜尋" })).toBeTruthy();
  });
});

describe("邀請中分頁", () => {
  it("說明文字之下每筆一列：email、「顯示名稱 · 角色 · 加入 …」、到期日與撤回鈕；列本身不可點開", async () => {
    renderAt("/admin/users?tab=pending", makeAdminClient());
    const email = await screen.findByText("invited@example.com");
    const row = email.closest("[data-invite]") as HTMLElement;
    expect(row.getAttribute("role")).toBeNull();
    expect(row.textContent).toContain("Invited · 成員 · 加入 demo");
    expect(row.textContent).toContain("到期 2026-01-10");
    expect(within(row).getByRole("button", { name: "撤回 invited@example.com 的邀請" })).toBeTruthy();
    expect(
      screen.getByText("已寄出但尚未接受的邀請。受邀者接受後才會成為「使用者」分頁裡的一列。"),
    ).toBeTruthy();
    // 正式使用者不混入邀請中分頁。
    expect(screen.queryByRole("button", { name: "Member" })).toBeNull();
  });

  it("邀請沒有要加入的專案時，說明省略「加入」那一段", async () => {
    const users = { ...USERS, pending: [{ ...USERS.pending[0], admin: true, memberships: [] }] };
    renderAt("/admin/users?tab=pending", makeAdminClient({ getAdminUsers: vi.fn(async () => users) }));
    const row = (await screen.findByText("invited@example.com")).closest("[data-invite]") as HTMLElement;
    expect(row.querySelector("[data-desc]")?.textContent).toBe("Invited · 管理員");
  });

  it("沒有有效邀請時邀請中計數 0，切過去是空狀態、沒有換頁列", async () => {
    renderAt(
      "/admin/users",
      makeAdminClient({ getAdminUsers: vi.fn(async () => ({ users: USERS.users, pending: [] })) }),
    );
    await screen.findByRole("button", { name: "Member" });
    expect(tabCount(/^邀請中/)).toBe("0");
    const user = userEvent.setup();
    await user.click(tab(/^邀請中/));
    expect(await screen.findByText("目前沒有待接受的邀請")).toBeTruthy();
    expect(screen.queryByText(/^第 .+ 筆，共 .+ 筆$/)).toBeNull();
  });

  // 邀請寄錯人或寄錯權限時要收得回來——連結一旦流出去，唯一的止血就是讓它失效。
  // 用「撤回」而非「取消」：確認框裡的取消鈕是「放棄這個操作」，兩者並排會分不清。
  it("撤回邀請經 AlertDialog 確認後才送出，且對話框指名受邀者", async () => {
    const client = makeAdminClient();
    renderAt("/admin/users?tab=pending", client);
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await user.click(await screen.findByRole("button", { name: "撤回 invited@example.com 的邀請" }));
    const confirm = await screen.findByRole("alertdialog");
    expect(confirm.textContent).toContain("invited@example.com");
    expect(client.adminRevokeInvitation).not.toHaveBeenCalled();
    await user.click(within(confirm).getByRole("button", { name: "撤回邀請" }));
    await waitFor(() => expect(client.adminRevokeInvitation).toHaveBeenCalledWith("i1"));
  });
});

describe("搜尋、篩選與換頁", () => {
  // 狀態篩選是 Radix Select：點開 trigger 再點選項（Radix 開啟時關閉 body 的
  // pointer-events，jsdom 沒有真實命中測試，故關掉 userEvent 的檢查）。
  it("關鍵字搜尋與狀態篩選收斂列表，無結果時呈現無符合狀態並保留控制項、沒有換頁列", async () => {
    renderAt("/admin/users", makeAdminClient());
    await screen.findByRole("button", { name: "Member" });
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await user.type(screen.getByRole("textbox", { name: "搜尋" }), "member");
    await waitFor(() => expect(screen.queryByRole("button", { name: "Admin" })).toBeNull());
    expect(screen.getByText("第 1–1 筆，共 1 筆")).toBeTruthy();
    await user.click(screen.getByRole("combobox", { name: /狀態/ }));
    await user.click(await screen.findByRole("option", { name: "已停權" }));
    expect(await screen.findByText(/沒有符合/)).toBeTruthy();
    expect(screen.getByRole("textbox", { name: "搜尋" })).toBeTruthy();
    expect(screen.getByRole("combobox", { name: /狀態/ })).toBeTruthy();
    expect(screen.queryByText(/^第 .+ 筆，共 .+ 筆$/)).toBeNull();
  });

  it("25 位使用者時換頁列為「第 1–20 筆，共 25 筆」，按第 2 頁後只列第 21–25 位", async () => {
    const client = makeAdminClient({ getAdminUsers: vi.fn(async () => ({ users: makeUsers(25), pending: [] })) });
    renderAt("/admin/users", client);
    expect(await screen.findByText("第 1–20 筆，共 25 筆")).toBeTruthy();
    expect(userRowNames()).toHaveLength(20);
    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "第 2 頁" }));
    expect(await screen.findByText("第 21–25 筆，共 25 筆")).toBeTruthy();
    expect(userRowNames()).toEqual(["User 21", "User 22", "User 23", "User 24", "User 25"]);
    // 換頁在前端進行，不重新呼叫 browser API。
    expect(client.getAdminUsers).toHaveBeenCalledTimes(1);
  });

  it("停在第 2 頁時搜尋使清單回到第 1 頁；只符合 3 位時沒有頁碼鈕", async () => {
    renderAt(
      "/admin/users",
      makeAdminClient({ getAdminUsers: vi.fn(async () => ({ users: makeUsers(21), pending: [] })) }),
    );
    await screen.findByText("第 1–20 筆，共 21 筆");
    const user = userEvent.setup();
    const search = screen.getByRole("textbox", { name: "搜尋" });
    await user.click(screen.getByRole("button", { name: "第 2 頁" }));
    await screen.findByText("第 21–21 筆，共 21 筆");
    // 「@」仍符合全部 21 位：頁碼若沒重設會停在第 2 頁。
    await user.type(search, "@");
    expect(await screen.findByText("第 1–20 筆，共 21 筆")).toBeTruthy();
    await user.click(screen.getByRole("button", { name: "第 2 頁" }));
    await screen.findByText("第 21–21 筆，共 21 筆");
    await user.clear(search);
    await user.type(search, "user2");
    expect(await screen.findByText("第 1–3 筆，共 3 筆")).toBeTruthy();
    expect(userRowNames()).toEqual(["User 2", "User 20", "User 21"]);
    expect(screen.queryByRole("button", { name: "第 1 頁" })).toBeNull();
  });
});

describe("抽屜照舊", () => {
  it("邀請欄位不常駐列表頁，按下邀請後才出現於抽屜", async () => {
    renderAt("/admin/users", makeAdminClient());
    await screen.findByRole("button", { name: "Member" });
    expect(screen.queryByLabelText(/電子郵件/)).toBeNull();
    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /邀請使用者/ }));
    const drawer = await screen.findByRole("dialog", { name: /邀請使用者/ });
    expect(within(drawer).getByLabelText(/電子郵件/)).toBeTruthy();
    expect(within(drawer).getByLabelText(/顯示名稱/)).toBeTruthy();
  });

  it("成員資格於抽屜內新增與移除", async () => {
    const client = makeAdminClient();
    renderAt("/admin/users", client);
    const user = userEvent.setup();
    await user.click(await screen.findByRole("button", { name: "Member" }));
    const drawer = await screen.findByRole("dialog", { name: /Member/ });
    await user.click(within(drawer).getByRole("tab", { name: "成員資格" }));
    await user.click(await within(drawer).findByRole("button", { name: /移除.*demo/ }));
    await waitFor(() =>
      expect(client.adminSetMembership).toHaveBeenCalledWith("m1", {
        projectKey: "demo",
        role: "editor",
        member: false,
      }),
    );
  });

  it("抽屜提交收到含 fieldErrors 的 400 時保持開啟並保留輸入", async () => {
    const client = makeAdminClient({
      adminInvite: vi.fn(async () => {
        throw new WebApiError(400, "validation_error", "欄位有誤", {
          email: "email 格式不正確",
        });
      }),
    });
    renderAt("/admin/users", client);
    await screen.findByRole("button", { name: "Member" });
    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /邀請使用者/ }));
    const drawer = await screen.findByRole("dialog", { name: /邀請使用者/ });
    await user.type(within(drawer).getByLabelText(/電子郵件/), "bad");
    await user.type(within(drawer).getByLabelText(/顯示名稱/), "New");
    await user.click(within(drawer).getByRole("button", { name: /送出邀請/ }));
    const alert = await within(drawer).findByRole("alert");
    expect(alert.textContent).toContain("email 格式不正確");
    // 抽屜保持開啟且非祕密輸入保留。
    expect(screen.getByRole("dialog", { name: /邀請使用者/ })).toBeTruthy();
    expect((within(drawer).getByLabelText(/電子郵件/) as HTMLInputElement).value).toBe("bad");
    expect((within(drawer).getByLabelText(/顯示名稱/) as HTMLInputElement).value).toBe("New");
  });

  it("停權於抽屜內觸發並經 AlertDialog 確認後才送出", async () => {
    const client = makeAdminClient();
    renderAt("/admin/users", client);
    const user = userEvent.setup();
    await user.click(await screen.findByRole("button", { name: "Member" }));
    const drawer = await screen.findByRole("dialog", { name: /Member/ });
    await user.click(within(drawer).getByRole("button", { name: /停權/ }));
    const confirm = await screen.findByRole("alertdialog");
    expect(client.adminSuspend).not.toHaveBeenCalled();
    await user.click(within(confirm).getByRole("button", { name: /停權|確認/ }));
    await waitFor(() => expect(client.adminSuspend).toHaveBeenCalledWith("m1"));
  });

  it("最後一位管理員的停權不可用", async () => {
    renderAt("/admin/users", makeAdminClient());
    const user = userEvent.setup();
    await user.click(await screen.findByRole("button", { name: "Admin" }));
    const drawer = await screen.findByRole("dialog", { name: /Admin/ });
    const suspend = within(drawer).getByRole("button", { name: /停權/ }) as HTMLButtonElement;
    expect(suspend.disabled).toBe(true);
  });

  // 加入專案以下拉挑選再逐一移除，而不是每個專案一列 checkbox——專案數量會成長，
  // 一整欄的勾選框在十個專案之後就不可用了。
  it("邀請抽屜的加入專案是下拉挑選，選中的專案列出且可移除", async () => {
    const client = makeAdminClient();
    renderAt("/admin/users", client);
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await screen.findByRole("button", { name: "Member" });
    await user.click(screen.getByRole("button", { name: /邀請使用者/ }));
    const drawer = await screen.findByRole("dialog", { name: /邀請使用者/ });

    // 不再是每個專案一個 checkbox。
    expect(within(drawer).queryByRole("checkbox", { name: "demo" })).toBeNull();
    await user.click(within(drawer).getByRole("combobox", { name: /加入專案/ }));
    // 選項標籤是「名稱（代號）」，代號與名稱相同時只印一次。
    await user.click(await screen.findByRole("option", { name: /demo/ }));

    // 選中的專案以可移除的項目呈現，且不再出現在下拉的可選項裡。
    expect(within(drawer).getByRole("button", { name: /移除 demo/ })).toBeTruthy();
    await user.type(within(drawer).getByLabelText(/電子郵件/), "new@example.com");
    await user.click(within(drawer).getByRole("button", { name: /送出邀請/ }));
    await waitFor(() =>
      expect(client.adminInvite).toHaveBeenCalledWith(
        expect.objectContaining({ memberships: ["demo"] }),
      ),
    );
  });

  it("剛送出的邀請立即出現在邀請中分頁，分頁計數加一", async () => {
    const invited = {
      ...USERS,
      pending: [...USERS.pending, { ...USERS.pending[0], id: "i2", email: "new@example.com", display: "New" }],
    };
    const client = makeAdminClient({
      getAdminUsers: vi.fn().mockResolvedValueOnce(USERS).mockResolvedValue(invited),
    });
    renderAt("/admin/users", client);
    await screen.findByRole("button", { name: "Member" });
    expect(tabCount(/^邀請中/)).toBe("1");
    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /邀請使用者/ }));
    const drawer = await screen.findByRole("dialog", { name: /邀請使用者/ });
    await user.type(within(drawer).getByLabelText(/電子郵件/), "new@example.com");
    await user.click(within(drawer).getByRole("button", { name: /送出邀請/ }));
    await waitFor(() => expect(tabCount(/^邀請中/)).toBe("2"));
    await user.click(tab(/^邀請中/));
    expect((await screen.findByText("new@example.com")).closest("[data-invite]")).toBeTruthy();
  });

  it("邀請成功後抽屜關閉，頁面以 aria-live 呈現可複製的邀請連結而非裸 token", async () => {
    const client = makeAdminClient();
    renderAt("/admin/users", client);
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await screen.findByRole("button", { name: "Member" });
    await user.click(screen.getByRole("button", { name: /邀請使用者/ }));
    const drawer = await screen.findByRole("dialog", { name: /邀請使用者/ });
    await user.type(within(drawer).getByLabelText(/電子郵件/), "new@example.com");
    await user.type(within(drawer).getByLabelText(/顯示名稱/), "New");
    await user.click(within(drawer).getByRole("combobox", { name: /加入專案/ }));
    await user.click(await screen.findByRole("option", { name: /demo/ }));
    await user.click(within(drawer).getByRole("button", { name: /送出邀請/ }));
    await waitFor(() =>
      expect(client.adminInvite).toHaveBeenCalledWith(
        expect.objectContaining({
          email: "new@example.com",
          display: "New",
          memberships: ["demo"],
        }),
      ),
    );
    await waitFor(() => expect(screen.queryByRole("dialog", { name: /邀請使用者/ })).toBeNull());

    // 給受邀者的是可直接開啟的連結，不是要他自己拼網址的 token。
    // （複製鈕自己也帶一個 sr-only 的 role=status，所以由連結文字回推外框。）
    const link = await screen.findByText(/\/invite\/invite-token-123/);
    const notice = link.closest("[role='status']") as HTMLElement;
    expect(notice, "邀請連結以 aria-live 區塊回饋").toBeTruthy();
    expect(within(notice).getByRole("button", { name: /複製/ })).toBeTruthy();
    // spec「揭示橫幅為成功語意」：主色是連結／互動的顏色，用在這裡讀不出「成功了」。
    expect(notice.className).toContain("status-success");
    expect(notice.className).not.toContain("border-primary");
  });
});

describe("窄螢幕", () => {
  it("使用者列說明為「email · 管理員」、meta 只留狀態籤；換頁列為窄版", async () => {
    setViewport(true);
    renderAt("/admin/users", makeAdminClient());
    const admin = await screen.findByRole("button", { name: "Admin" });
    expect(admin.querySelector("[data-desc]")?.textContent).toBe("admin@example.com · 管理員");
    expect(within(admin).queryByText("1 個專案")).toBeNull();
    expect(within(admin).getByText("有效")).toBeTruthy();
    const member = screen.getByRole("button", { name: "Member" });
    expect(member.querySelector("[data-desc]")?.textContent).toBe("member@example.com");
    expect(screen.getByText("第 1–2 筆，共 2 筆")).toBeTruthy();
    expect(screen.queryByRole("combobox", { name: /每頁/ })).toBeNull();
  });

  it("窄螢幕點列同樣開啟細節抽屜", async () => {
    setViewport(true);
    renderAt("/admin/users", makeAdminClient());
    const user = userEvent.setup();
    await user.click(await screen.findByRole("button", { name: "Member" }));
    expect(await screen.findByRole("dialog", { name: /Member/ })).toBeTruthy();
  });
});

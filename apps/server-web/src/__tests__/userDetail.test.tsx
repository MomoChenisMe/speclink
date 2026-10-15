import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { REVEAL_ON_GROUP_HOVER } from "@speclink/ui";
import {
  AUDIT,
  REGISTRY,
  USERS,
  browserBack,
  currentLocation,
  makeAdminClient,
  makeUsers,
  renderAt,
  renderWithLocation,
  setViewport,
} from "./helpers/adminHarness";

// 使用者詳情頁（server-web-console「管理列表以詳情頁與對話框承載檢視與建立」「全部 browser route
// 由單一 SPA 提供可發現導覽」；server-web-detail-pages design D1、D2、D5）：點使用者列換成有網址的
// 詳情頁——頁標題接狀態籤、卡片標頭分頁（概要／成員資格／憑證／稽核）；頂列麵包屑三段、右端
// ‹ n / m › 依列表目前呈現的順序切換；回列表時搜尋、篩選與頁碼保持原樣。

beforeEach(() => setViewport(false));

const crumbs = () => screen.getByRole("navigation", { name: "頁面路徑" });
const tab = (name: RegExp) => screen.getByRole("tab", { name });
const tabCount = (name: RegExp) => tab(name).querySelector("[data-tab-count]")?.textContent;
const hasRevealClasses = (el: Element) =>
  REVEAL_ON_GROUP_HOVER.split(" ").every((cls) => el.classList.contains(cls));

async function openMember(client = makeAdminClient()) {
  renderWithLocation(["/admin/users"], client);
  const user = userEvent.setup({ pointerEventsCheck: 0 });
  await user.click(await screen.findByRole("button", { name: "Member" }));
  await screen.findByRole("heading", { level: 1, name: "Member" });
  return user;
}

describe("進入詳情頁", () => {
  it("點使用者列換成有網址的詳情頁：頁標題接綠系狀態籤、四個分頁，沒有側邊滑出的面板", async () => {
    await openMember();
    expect(currentLocation()).toBe("/admin/users/m1");
    const heading = screen.getByRole("heading", { level: 1, name: "Member" });
    expect(screen.getAllByRole("heading", { level: 1 })).toHaveLength(1);
    const header = heading.closest("[data-page-header]") as HTMLElement;
    expect(within(header).getByText("有效").className).toContain("status-success");
    expect(tab(/^概要/)).toBeTruthy();
    expect(tabCount(/^成員資格/)).toBe("1");
    // m1 有一把存取金鑰與一個裝置登入（憑證在詳情頁另外讀取）。
    await waitFor(() => expect(tabCount(/^憑證/)).toBe("2"));
    expect(tab(/^稽核/)).toBeTruthy();
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("以鍵盤移到列上按 Enter 同樣進入詳情頁", async () => {
    renderWithLocation(["/admin/users"], makeAdminClient());
    const user = userEvent.setup();
    (await screen.findByRole("button", { name: "Member" })).focus();
    await user.keyboard("{Enter}");
    expect(await screen.findByRole("heading", { level: 1, name: "Member" })).toBeTruthy();
    expect(currentLocation()).toBe("/admin/users/m1");
  });

  it("直接開 /admin/users/m1 呈現同一個詳情頁", async () => {
    renderWithLocation(["/admin/users/m1"], makeAdminClient());
    expect(await screen.findByRole("heading", { level: 1, name: "Member" })).toBeTruthy();
    expect(tabCount(/^成員資格/)).toBe("1");
    expect(currentLocation()).toBe("/admin/users/m1");
  });

  it("對不到資料時說明找不到並提供回到列表，頂列沒有 ‹ n / m ›", async () => {
    renderWithLocation(["/admin/users/no-such-id"], makeAdminClient());
    expect(await screen.findByText("找不到這位使用者")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "上一個" })).toBeNull();
    expect(screen.queryByRole("button", { name: "下一個" })).toBeNull();
    expect(crumbs().textContent).toBe("後台/使用者");
    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "回到使用者列表" }));
    expect(await screen.findByRole("heading", { level: 1, name: "使用者" })).toBeTruthy();
    expect(currentLocation()).toBe("/admin/users");
  });
});

describe("頂列：麵包屑與 ‹ n / m ›", () => {
  it("麵包屑為「後台 / 使用者 / Member」、側欄高亮使用者；點「使用者」回到列表", async () => {
    const user = await openMember();
    expect(crumbs().textContent).toBe("後台/使用者/Member");
    const nav = screen.getByRole("navigation", { name: "管理導覽" });
    expect(within(nav).getByRole("link", { name: "使用者" }).getAttribute("aria-current")).toBe("page");
    // 名稱不可點。
    expect(within(crumbs()).queryByRole("button", { name: "Member" })).toBeNull();
    await user.click(within(crumbs()).getByRole("button", { name: "使用者" }));
    expect(await screen.findByRole("heading", { level: 1, name: "使用者" })).toBeTruthy();
    expect(currentLocation()).toBe("/admin/users");
    expect(crumbs().textContent).toBe("後台/使用者");
  });

  it("回到列表保留狀態篩選與頁碼", async () => {
    const client = makeAdminClient({ getAdminUsers: vi.fn(async () => ({ users: makeUsers(45), pending: [] })) });
    renderWithLocation(["/admin/users"], client);
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await screen.findByText("第 1–20 筆，共 45 筆");
    await user.click(screen.getByRole("combobox", { name: /狀態/ }));
    await user.click(await screen.findByRole("option", { name: "有效" }));
    await user.click(screen.getByRole("button", { name: "第 3 頁" }));
    await screen.findByText("第 41–45 筆，共 45 筆");
    await user.click(screen.getByRole("button", { name: "User 41" }));
    await screen.findByRole("heading", { level: 1, name: "User 41" });
    await user.click(within(crumbs()).getByRole("button", { name: "使用者" }));
    expect(await screen.findByText("第 41–45 筆，共 45 筆")).toBeTruthy();
    expect(screen.getByRole("combobox", { name: /狀態/ }).textContent).toContain("有效");
  });

  it("‹ n / m › 依搜尋後的順序切換、到頭停用；切換不新增瀏覽紀錄，上一頁回到列表且搜尋保留", async () => {
    // 12 位裡有 3 位的 email 在 ops.example——關鍵字「ops.」只符合這 3 位（User 4、7、10）。
    const users = makeUsers(12).map((u, i) =>
      [3, 6, 9].includes(i) ? { ...u, email: u.email.replace("example.com", "ops.example") } : u,
    );
    const client = makeAdminClient({ getAdminUsers: vi.fn(async () => ({ users, pending: [] })) });
    renderWithLocation(["/admin/users"], client);
    const user = userEvent.setup();
    await screen.findByText("第 1–12 筆，共 12 筆");
    await user.type(screen.getByRole("textbox", { name: "搜尋" }), "ops.");
    await screen.findByText("第 1–3 筆，共 3 筆");
    await user.click(screen.getByRole("button", { name: "User 7" }));
    await screen.findByRole("heading", { level: 1, name: "User 7" });
    expect(screen.getByText("2 / 3")).toBeTruthy();

    await user.click(screen.getByRole("button", { name: "下一個" }));
    expect(await screen.findByRole("heading", { level: 1, name: "User 10" })).toBeTruthy();
    expect(screen.getByText("3 / 3")).toBeTruthy();
    expect((screen.getByRole("button", { name: "下一個" }) as HTMLButtonElement).disabled).toBe(true);
    expect(currentLocation()).toBe("/admin/users/u10");

    await user.click(browserBack());
    expect(await screen.findByRole("heading", { level: 1, name: "使用者" })).toBeTruthy();
    expect(currentLocation()).toBe("/admin/users");
    expect((screen.getByRole("textbox", { name: "搜尋" }) as HTMLInputElement).value).toBe("ops.");
  });

  it("以鍵盤連按 ‹ ›：焦點留在按鈕上，不跳回頁標題", async () => {
    const client = makeAdminClient({ getAdminUsers: vi.fn(async () => ({ users: makeUsers(3), pending: [] })) });
    renderWithLocation(["/admin/users/u1"], client);
    const user = userEvent.setup();
    await screen.findByRole("heading", { level: 1, name: "User 1" });
    const next = screen.getByRole("button", { name: "下一個" });
    next.focus();
    await user.keyboard("{Enter}");
    expect(await screen.findByRole("heading", { level: 1, name: "User 2" })).toBeTruthy();
    expect(document.activeElement).toBe(screen.getByRole("button", { name: "下一個" }));
    await user.keyboard("{Enter}");
    expect(await screen.findByRole("heading", { level: 1, name: "User 3" })).toBeTruthy();
  });

  it("從列表進入詳情頁時焦點移到頁標題", async () => {
    await openMember();
    expect(document.activeElement).toBe(screen.getByRole("heading", { level: 1, name: "Member" }));
  });
});

describe("頁級動作與概要", () => {
  it("唯一有效管理員的管理員開關為開且停用，停權停用", async () => {
    renderAt("/admin/users/a1", makeAdminClient());
    await screen.findByRole("heading", { level: 1, name: "Admin" });
    const flag = screen.getByRole("switch", { name: "管理員" });
    expect(flag.getAttribute("aria-checked")).toBe("true");
    expect((flag as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByRole("button", { name: "停權" }) as HTMLButtonElement).disabled).toBe(true);
  });

  it("概要列出電子郵件（可複製）、狀態與建立日期；切換管理員開關即送出", async () => {
    const client = makeAdminClient();
    const user = await openMember(client);
    expect(screen.getByText("member@example.com")).toBeTruthy();
    expect(screen.getByRole("button", { name: /複製/ })).toBeTruthy();
    expect(screen.getByText("2026-01-02")).toBeTruthy();
    await user.click(screen.getByRole("switch", { name: "管理員" }));
    await waitFor(() => expect(client.adminSetAdminFlag).toHaveBeenCalledWith("m1", true));
  });

  it("「停權」開出指名使用者的確認框，確認後才送出", async () => {
    const client = makeAdminClient();
    const user = await openMember(client);
    await user.click(screen.getByRole("button", { name: "停權" }));
    const confirm = await screen.findByRole("alertdialog");
    expect(confirm.textContent).toContain("Member");
    expect(client.adminSuspend).not.toHaveBeenCalled();
    await user.click(within(confirm).getByRole("button", { name: "停權" }));
    await waitFor(() => expect(client.adminSuspend).toHaveBeenCalledWith("m1"));
  });
});

describe("成員資格分頁", () => {
  it("角色下拉改為 reader 立即送出，重載後仍停在成員資格分頁、該列顯示 reader", async () => {
    const asReader = {
      ...USERS,
      users: [USERS.users[0], { ...USERS.users[1], memberships: [{ projectKey: "demo", role: "reader" }] }],
    };
    const client = makeAdminClient({
      getAdminUsers: vi.fn().mockResolvedValueOnce(USERS).mockResolvedValue(asReader),
    });
    const user = await openMember(client);
    await user.click(tab(/^成員資格/));
    await user.click(screen.getByRole("combobox", { name: "demo 的角色" }));
    await user.click(await screen.findByRole("option", { name: "reader" }));
    await waitFor(() =>
      expect(client.adminSetMembership).toHaveBeenCalledWith("m1", {
        projectKey: "demo",
        role: "reader",
        member: true,
      }),
    );
    await waitFor(() => expect(client.getAdminUsers).toHaveBeenCalledTimes(2));
    await waitFor(() => expect(tab(/^成員資格/).getAttribute("aria-selected")).toBe("true"));
    expect(screen.getByRole("combobox", { name: "demo 的角色" }).textContent).toContain("reader");
    expect(screen.queryByRole("button", { name: /儲存/ })).toBeNull();
  });

  it("移除鈕滑過才出現；按下先確認並指名使用者與專案，取消不送出、確認才送出；重載後該列消失、計數減一", async () => {
    const removed = { ...USERS, users: [USERS.users[0], { ...USERS.users[1], memberships: [] }] };
    const client = makeAdminClient({
      getAdminUsers: vi.fn().mockResolvedValueOnce(USERS).mockResolvedValue(removed),
    });
    const user = await openMember(client);
    await user.click(tab(/^成員資格/));
    const remove = screen.getByRole("button", { name: "移除 Member 在 demo 的成員資格" });
    expect(hasRevealClasses(remove)).toBe(true);

    await user.click(remove);
    let confirm = await screen.findByRole("alertdialog");
    expect(confirm.textContent).toContain("Member");
    expect(confirm.textContent).toContain("demo");
    await user.click(within(confirm).getByRole("button", { name: "取消" }));
    await waitFor(() => expect(screen.queryByRole("alertdialog")).toBeNull());
    expect(client.adminSetMembership).not.toHaveBeenCalled();

    await user.click(screen.getByRole("button", { name: "移除 Member 在 demo 的成員資格" }));
    confirm = await screen.findByRole("alertdialog");
    await user.click(within(confirm).getByRole("button", { name: "移除" }));
    await waitFor(() =>
      expect(client.adminSetMembership).toHaveBeenCalledWith("m1", {
        projectKey: "demo",
        role: "editor",
        member: false,
      }),
    );
    await waitFor(() => expect(screen.queryByRole("combobox", { name: "demo 的角色" })).toBeNull());
    expect(tabCount(/^成員資格/)).toBe("0");
    expect(screen.getByText("尚未加入任何專案。")).toBeTruthy();
  });

  it("「加入專案」列的專案下拉只列尚未加入的專案，選好後按加入送出", async () => {
    const registry = {
      projects: [...REGISTRY.projects, { key: "ops", name: "Ops", repos: [] }],
    };
    const client = makeAdminClient({ getAdminRegistry: vi.fn(async () => registry) });
    const user = await openMember(client);
    await user.click(tab(/^成員資格/));
    const add = screen.getByRole("button", { name: "加入" }) as HTMLButtonElement;
    expect(add.disabled).toBe(true);
    await user.click(screen.getByRole("combobox", { name: "要加入的專案" }));
    const options = (await screen.findAllByRole("option")).map((o) => o.textContent);
    expect(options).toEqual(["Ops（ops）"]);
    await user.click(screen.getByRole("option", { name: "Ops（ops）" }));
    await user.click(screen.getByRole("button", { name: "加入" }));
    await waitFor(() =>
      expect(client.adminSetMembership).toHaveBeenCalledWith("m1", {
        projectKey: "ops",
        role: "editor",
        member: true,
      }),
    );
  });
  it("加入列選好專案後按 ‹ › 換人：選擇清空，不會替對方改掉既有角色", async () => {
    const registry = { projects: [...REGISTRY.projects, { key: "ops", name: "Ops", repos: [] }] };
    const users = {
      ...USERS,
      users: [
        { ...USERS.users[0], memberships: [{ projectKey: "ops", role: "reader" }] },
        USERS.users[1],
      ],
    };
    const client = makeAdminClient({
      getAdminRegistry: vi.fn(async () => registry),
      getAdminUsers: vi.fn(async () => users),
    });
    const user = await openMember(client);
    await user.click(tab(/^成員資格/));
    await user.click(screen.getByRole("combobox", { name: "要加入的專案" }));
    await user.click(await screen.findByRole("option", { name: "Ops（ops）" }));
    expect((screen.getByRole("button", { name: "加入" }) as HTMLButtonElement).disabled).toBe(false);
    await user.click(screen.getByRole("button", { name: "上一個" }));
    await screen.findByRole("heading", { level: 1, name: "Admin" });
    expect((screen.getByRole("button", { name: "加入" }) as HTMLButtonElement).disabled).toBe(true);
  });

  it("專案清單還沒載入時，加入列不說「已加入全部專案」", async () => {
    const client = makeAdminClient({ getAdminRegistry: vi.fn(() => new Promise(() => {})) });
    const user = await openMember(client);
    await user.click(tab(/^成員資格/));
    const picker = screen.getByRole("combobox", { name: "要加入的專案" });
    expect(picker.textContent).not.toContain("已加入全部專案");
    expect(picker.textContent).toContain("專案清單還沒載入");
  });
});

describe("憑證與稽核分頁", () => {
  it("憑證分頁列出存取金鑰與裝置登入；有效項目的撤銷鈕滑過才出現，確認後才撤銷", async () => {
    const client = makeAdminClient();
    const user = await openMember(client);
    await user.click(tab(/^憑證/));
    const panel = screen.getByRole("tabpanel");
    expect(await within(panel).findByText("cli")).toBeTruthy();
    expect(within(panel).getByText("slk_abc · 存取金鑰 · 到期 永久 · 最近使用 從未")).toBeTruthy();
    expect(within(panel).getByText("desktop")).toBeTruthy();
    expect(within(panel).getByText("裝置登入 · 最近更新 2026-01-03")).toBeTruthy();

    const revoke = screen.getByRole("button", { name: "撤銷存取金鑰 cli" });
    expect(hasRevealClasses(revoke)).toBe(true);
    await user.click(revoke);
    const confirm = await screen.findByRole("alertdialog");
    expect(client.adminRevokeToken).not.toHaveBeenCalled();
    await user.click(within(confirm).getByRole("button", { name: "撤銷" }));
    await waitFor(() => expect(client.adminRevokeToken).toHaveBeenCalledWith("p1"));
  });

  it("稽核分頁只列該使用者的事件，動作為中文名稱並附操作者", async () => {
    const audit = {
      ...AUDIT,
      entries: [
        ...AUDIT.entries,
        { id: "e2", actorId: "a1", action: "project-created", subject: "demo", source: "web", createdAt: "2026-01-04T00:00:00Z" },
      ],
    };
    const user = await openMember(makeAdminClient({ getAdminAudit: vi.fn(async () => audit) }));
    await user.click(tab(/^稽核/));
    const panel = screen.getByRole("tabpanel");
    expect(await within(panel).findByText("停權使用者")).toBeTruthy();
    // 操作者以名稱呈現。
    expect(within(panel).getByText("Admin")).toBeTruthy();
    expect(within(panel).getByText("2026-01-01")).toBeTruthy();
    expect(within(panel).queryByText("建立專案")).toBeNull();
  });
});

// 停權中的使用者頁級動作為重新啟用（spec 使用者詳情頁的頁級動作）。
it("停權中的使用者頁級動作為「重新啟用」，按下即送出", async () => {
  const users = { ...USERS, users: [USERS.users[0], { ...USERS.users[1], active: false }] };
  const client = makeAdminClient({ getAdminUsers: vi.fn(async () => users) });
  renderAt("/admin/users/m1", client);
  const heading = await screen.findByRole("heading", { level: 1, name: "Member" });
  const header = heading.closest("[data-page-header]") as HTMLElement;
  expect(within(header).getByText("已停權").className).toContain("status-warning");
  const user = userEvent.setup();
  await user.click(screen.getByRole("button", { name: "重新啟用" }));
  await waitFor(() => expect(client.adminReactivate).toHaveBeenCalledWith("m1"));
});

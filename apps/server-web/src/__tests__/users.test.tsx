import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { WebApiError } from "../api/client";
import { makeAdminClient, renderAt, setViewport } from "./helpers/adminHarness";

// 管理使用者頁（server-web-console「管理列表以抽屜承載建立與編輯」）：列表為主體，
// 列內不含任何輸入控制項；建立與編輯一律進抽屜；破壞性動作維持 AlertDialog 確認。

beforeEach(() => setViewport(false));

describe("使用者列表", () => {
  it("列只呈現使用者、狀態、角色、成員資格與建立日期，不含輸入控制項", async () => {
    renderAt("/admin/users", makeAdminClient());
    const table = await screen.findByRole("table");
    const row = within(table).getByRole("row", { name: /member@example\.com/ });
    const text = row.textContent ?? "";
    for (const shown of ["Member", "member@example.com", "有效", "成員", "demo", "2026-01-02"]) {
      expect(text, `列呈現 ${shown}`).toContain(shown);
    }
    // 列內不得有下拉、勾選框、文字輸入或提交按鈕。
    expect(within(table).queryAllByRole("combobox")).toHaveLength(0);
    expect(within(table).queryAllByRole("checkbox")).toHaveLength(0);
    expect(within(table).queryAllByRole("textbox")).toHaveLength(0);
    expect(within(table).queryAllByRole("button", { name: /新增|送出|停權|復權/ })).toHaveLength(0);
  });

  it("邀請欄位不常駐列表頁，按下邀請後才出現於抽屜", async () => {
    renderAt("/admin/users", makeAdminClient());
    await screen.findByRole("table");
    expect(screen.queryByLabelText(/電子郵件/)).toBeNull();
    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /邀請使用者/ }));
    const drawer = await screen.findByRole("dialog", { name: /邀請使用者/ });
    expect(within(drawer).getByLabelText(/電子郵件/)).toBeTruthy();
    expect(within(drawer).getByLabelText(/顯示名稱/)).toBeTruthy();
  });

  it("點整列開啟含概要／成員資格／憑證／稽核的細節抽屜", async () => {
    renderAt("/admin/users", makeAdminClient());
    const table = await screen.findByRole("table");
    const user = userEvent.setup();
    await user.click(within(table).getByRole("row", { name: /member@example\.com/ }));
    const drawer = await screen.findByRole("dialog", { name: /Member/ });
    for (const tab of ["概要", "成員資格", "憑證", "稽核"]) {
      expect(within(drawer).getByRole("tab", { name: tab }), `抽屜有 ${tab} 分頁`).toBeTruthy();
    }
  });

  it("成員資格於抽屜內新增與移除", async () => {
    const client = makeAdminClient();
    renderAt("/admin/users", client);
    const table = await screen.findByRole("table");
    const user = userEvent.setup();
    await user.click(within(table).getByRole("row", { name: /member@example\.com/ }));
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
    await screen.findByRole("table");
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
    const table = await screen.findByRole("table");
    const user = userEvent.setup();
    await user.click(within(table).getByRole("row", { name: /member@example\.com/ }));
    const drawer = await screen.findByRole("dialog", { name: /Member/ });
    await user.click(within(drawer).getByRole("button", { name: /停權/ }));
    const confirm = await screen.findByRole("alertdialog");
    expect(client.adminSuspend).not.toHaveBeenCalled();
    await user.click(within(confirm).getByRole("button", { name: /停權|確認/ }));
    await waitFor(() => expect(client.adminSuspend).toHaveBeenCalledWith("m1"));
  });

  it("最後一位管理員的停權不可用", async () => {
    renderAt("/admin/users", makeAdminClient());
    const table = await screen.findByRole("table");
    const user = userEvent.setup();
    await user.click(within(table).getByRole("row", { name: /admin@example\.com/ }));
    const drawer = await screen.findByRole("dialog", { name: /Admin/ });
    const suspend = within(drawer).getByRole("button", { name: /停權/ }) as HTMLButtonElement;
    expect(suspend.disabled).toBe(true);
  });

  // server-web-console「管理列表提供搜尋、篩選、分頁與具引導的空狀態」：使用者頁
  // SHALL 提供關鍵字搜尋與狀態篩選。
  it("關鍵字搜尋與狀態篩選收斂列表，無結果時呈現空狀態並保留控制項", async () => {
    renderAt("/admin/users", makeAdminClient());
    const table = await screen.findByRole("table");
    expect(within(table).getAllByRole("row")).toHaveLength(3); // 表頭 + 兩位使用者
    // 狀態篩選是 Radix Select：點開 trigger 再點選項（Radix 開啟時關閉 body 的
    // pointer-events，jsdom 沒有真實命中測試，故關掉 userEvent 的檢查）。
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await user.type(screen.getByRole("searchbox", { name: /搜尋/ }), "member");
    await waitFor(() =>
      expect(within(screen.getByRole("table")).getAllByRole("row")).toHaveLength(2),
    );
    await user.click(screen.getByRole("combobox", { name: /狀態/ }));
    await user.click(await screen.findByRole("option", { name: "已停權" }));
    expect(await screen.findByText(/沒有符合/)).toBeTruthy();
    expect(screen.getByRole("searchbox", { name: /搜尋/ })).toBeTruthy();
    expect(screen.getByLabelText(/狀態/)).toBeTruthy();
  });

  // 加入專案以下拉挑選再逐一移除，而不是每個專案一列 checkbox——專案數量會成長，
  // 一整欄的勾選框在十個專案之後就不可用了。
  it("邀請抽屜的加入專案是下拉挑選，選中的專案列出且可移除", async () => {
    const client = makeAdminClient();
    renderAt("/admin/users", client);
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await screen.findByRole("table");
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

  // 受邀者在接受邀請前沒有 user row，卻是剛按下邀請的管理員最想確認的對象——
  // 總覽的「待啟用」指標也連到這一頁。
  it("待啟用的邀請列在使用者清單之外的邀請中區塊", async () => {
    renderAt("/admin/users", makeAdminClient());
    const pending = await screen.findByRole("region", { name: /邀請中/ });
    expect(within(pending).getByText("invited@example.com")).toBeTruthy();
    expect(within(pending).getByText(/demo/)).toBeTruthy();
    // 正式使用者的表格不混入待啟用者。
    const table = screen.getByRole("table");
    expect(within(table).queryByText("invited@example.com")).toBeNull();
  });

  it("沒有待啟用邀請時不渲染邀請中區塊", async () => {
    renderAt(
      "/admin/users",
      makeAdminClient({
        getAdminUsers: vi.fn(async () => ({ users: [], pending: [] })),
      }),
    );
    await screen.findByRole("heading", { level: 1, name: "使用者" });
    await waitFor(() => expect(screen.queryByText("載入中…")).toBeNull());
    expect(screen.queryByRole("region", { name: /邀請中/ })).toBeNull();
  });

  it("邀請成功後呈現可複製的邀請連結，而非裸 token", async () => {
    const client = makeAdminClient();
    renderAt("/admin/users", client);
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await screen.findByRole("table");
    await user.click(screen.getByRole("button", { name: /邀請使用者/ }));
    const drawer = await screen.findByRole("dialog", { name: /邀請使用者/ });
    await user.type(within(drawer).getByLabelText(/電子郵件/), "new@example.com");
    await user.click(within(drawer).getByRole("button", { name: /送出邀請/ }));

    // 給受邀者的是可直接開啟的連結，不是要他自己拼網址的 token。
    // （複製鈕自己也帶一個 sr-only 的 role=status，所以由連結文字回推外框。）
    const link = await screen.findByText(/\/invite\/invite-token-123/);
    const notice = link.closest("[role='status']") as HTMLElement;
    expect(notice, "邀請連結以 aria-live 區塊回饋").toBeTruthy();
    expect(within(notice).getByRole("button", { name: /複製/ })).toBeTruthy();
  });

  // 邀請寄錯人或寄錯權限時要收得回來——連結一旦流出去，唯一的止血就是讓它失效。
  // 用「撤回」而非「取消」：確認框裡的取消鈕是「放棄這個操作」，兩者並排會分不清。
  it("撤回邀請經 AlertDialog 確認後才送出，且對話框指名受邀者", async () => {
    const client = makeAdminClient();
    renderAt("/admin/users", client);
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    const pending = await screen.findByRole("region", { name: /邀請中/ });

    await user.click(within(pending).getByRole("button", { name: /撤回 invited@example\.com/ }));
    const confirm = await screen.findByRole("alertdialog");
    expect(confirm.textContent).toContain("invited@example.com");
    expect(client.adminRevokeInvitation).not.toHaveBeenCalled();

    await user.click(within(confirm).getByRole("button", { name: "撤回邀請" }));
    await waitFor(() => expect(client.adminRevokeInvitation).toHaveBeenCalledWith("i1"));
  });

  // server-web-console「共用設計系統維持高密度可存取體驗」：手機版資料轉為卡片，
  // 不以表格橫捲或裁切收場。
  it("窄螢幕以卡片列取代表格，欄位標籤仍在", async () => {
    setViewport(true);
    renderAt("/admin/users", makeAdminClient());
    expect(await screen.findByText("member@example.com")).toBeTruthy();
    expect(screen.queryByRole("table")).toBeNull();
    // 卡片仍帶得動整份欄位：狀態、角色、成員資格與建立日期。
    const card = screen.getByText("member@example.com").closest("li");
    expect(card).toBeTruthy();
    for (const shown of ["有效", "成員", "demo", "2026-01-02"]) {
      expect(card?.textContent, `卡片呈現 ${shown}`).toContain(shown);
    }
    expect(
      within(card as HTMLElement).getByRole("button", { name: /檢視 Member/ }),
    ).toBeTruthy();
  });

  it("窄螢幕點卡片同樣開啟細節抽屜", async () => {
    setViewport(true);
    renderAt("/admin/users", makeAdminClient());
    const user = userEvent.setup();
    await user.click(await screen.findByRole("button", { name: /檢視 Member/ }));
    expect(await screen.findByRole("dialog", { name: /Member/ })).toBeTruthy();
  });

  it("邀請成功後抽屜關閉並以 aria-live 回饋一次性 token", async () => {
    const client = makeAdminClient();
    renderAt("/admin/users", client);
    await screen.findByRole("table");
    const user = userEvent.setup({ pointerEventsCheck: 0 });
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
    expect(await screen.findByText(/invite-token-123/)).toBeTruthy();
  });
});

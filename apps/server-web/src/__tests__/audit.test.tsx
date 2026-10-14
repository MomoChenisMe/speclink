import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { makeAdminClient, renderAt, setViewport } from "./helpers/adminHarness";

// 管理稽核紀錄頁（server-web-console「管理列表提供搜尋、篩選、分頁與具引導的空狀態」
// 與 server-admin「管理 browser API 提供最小且完整的頁面 view model」；server-web-list-pages
// design D7）：頁標題區是關鍵字、動作、來源與時間區間篩選；寬螢幕的表格放進列表卡、卡底是與其他
// 列表同一條換頁列。SPA 只把篩選、頁碼與每頁筆數送給 browser API，並只呈現伺服器回傳的當頁
// 事件；「共 N 筆」取自回應的 total。

const PAGE_ONE = {
  entries: [
    {
      id: "e1",
      actorId: "a1",
      action: "user-invited",
      subject: "new@example.com",
      source: "web",
      createdAt: "2026-01-03T10:00:00Z",
    },
    {
      id: "e2",
      actorId: "a1",
      action: "project-created",
      subject: "demo",
      source: "cli",
      createdAt: "2026-01-02T10:00:00Z",
    },
  ],
  totalPages: 3,
  total: 45,
};

beforeEach(() => setViewport(false));

describe("稽核紀錄頁", () => {
  it("首次載入以第 1 頁、每頁 20 筆呼叫 browser API", async () => {
    const getAdminAudit = vi.fn(async () => PAGE_ONE);
    renderAt("/admin/audit", makeAdminClient({ getAdminAudit }));
    await screen.findByRole("table");
    expect(getAdminAudit).toHaveBeenLastCalledWith(expect.objectContaining({ page: 1, limit: 20 }));
  });

  it("換頁列取自回應：「第 1–20 筆，共 45 筆」與頁碼鈕 1–3", async () => {
    renderAt("/admin/audit", makeAdminClient({ getAdminAudit: vi.fn(async () => PAGE_ONE) }));
    expect(await screen.findByText("第 1–20 筆，共 45 筆")).toBeTruthy();
    for (const n of [1, 2, 3]) {
      expect(screen.getByRole("button", { name: `第 ${n} 頁` })).toBeTruthy();
    }
    expect(screen.getByRole("combobox", { name: "每頁 20 個" })).toBeTruthy();
  });

  it("按第 2 頁以 page=2 重新呼叫", async () => {
    const getAdminAudit = vi.fn(async () => PAGE_ONE);
    renderAt("/admin/audit", makeAdminClient({ getAdminAudit }));
    await screen.findByText("第 1–20 筆，共 45 筆");
    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "第 2 頁" }));
    await waitFor(() =>
      expect(getAdminAudit).toHaveBeenLastCalledWith(expect.objectContaining({ page: 2, limit: 20 })),
    );
    expect(await screen.findByText("第 21–40 筆，共 45 筆")).toBeTruthy();
  });

  it("換頁時列表卡與換頁列不卸載：載入期間照舊顯示上一頁，回應到了才替換", async () => {
    let release: (value: typeof PAGE_ONE) => void = () => {};
    const getAdminAudit = vi
      .fn()
      .mockImplementationOnce(async () => PAGE_ONE)
      .mockImplementationOnce(
        () =>
          new Promise<typeof PAGE_ONE>((resolve) => {
            release = resolve;
          }),
      );
    renderAt("/admin/audit", makeAdminClient({ getAdminAudit }));
    await screen.findByText("第 1–20 筆，共 45 筆");
    const user = userEvent.setup();
    const pageTwo = screen.getByRole("button", { name: "第 2 頁" });
    await user.click(pageTwo);
    await waitFor(() => expect(getAdminAudit).toHaveBeenCalledTimes(2));
    // 載入中：表格、換頁列與剛按的頁碼鈕都還在（焦點不會掉回頁首）。
    expect(screen.getByRole("table")).toBeTruthy();
    expect(pageTwo.isConnected).toBe(true);
    expect(screen.queryByText("載入中…")).toBeNull();
    release({ ...PAGE_ONE, entries: [PAGE_ONE.entries[1]] });
    await waitFor(() => expect(screen.queryByText("new@example.com")).toBeNull());
    expect(screen.getByText("第 21–40 筆，共 45 筆")).toBeTruthy();
  });

  // 換頁列在重抓期間仍可按：若點了舊頁碼列上較後面的頁、而新結果的頁數更少，伺服器會回空頁。
  // 回應到了發現頁碼越界，就拉回最後一頁重抓，不讓使用者卡在空白頁、換頁列又消失。
  it("回應的總頁數比目前頁碼少時，頁碼拉回最後一頁並重新呼叫", async () => {
    const lastPage = { entries: [PAGE_ONE.entries[0]], totalPages: 1, total: 3 };
    const getAdminAudit = vi
      .fn()
      .mockImplementationOnce(async () => PAGE_ONE)
      .mockImplementationOnce(async () => ({ entries: [], totalPages: 1, total: 3 }))
      .mockImplementation(async () => lastPage);
    renderAt("/admin/audit", makeAdminClient({ getAdminAudit }));
    await screen.findByText("第 1–20 筆，共 45 筆");
    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "第 3 頁" }));
    await waitFor(() =>
      expect(getAdminAudit).toHaveBeenLastCalledWith(expect.objectContaining({ page: 1 })),
    );
    expect(getAdminAudit).toHaveBeenCalledTimes(3);
    expect(await screen.findByText("第 1–3 筆，共 3 筆")).toBeTruthy();
    expect(screen.getByText("new@example.com")).toBeTruthy();
  });

  it("停在第 3 頁時改每頁 50 筆，以 page=1、limit=50 重新呼叫", async () => {
    const getAdminAudit = vi.fn(async () => PAGE_ONE);
    renderAt("/admin/audit", makeAdminClient({ getAdminAudit }));
    await screen.findByText("第 1–20 筆，共 45 筆");
    // 每頁筆數是 Radix Select：關掉 userEvent 的 pointer-events 檢查（jsdom 沒有命中測試）。
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await user.click(screen.getByRole("button", { name: "第 3 頁" }));
    await screen.findByText("第 41–45 筆，共 45 筆");
    await user.click(screen.getByRole("combobox", { name: "每頁 20 個" }));
    await user.click(await screen.findByRole("option", { name: "每頁 50 個" }));
    await waitFor(() =>
      expect(getAdminAudit).toHaveBeenLastCalledWith(expect.objectContaining({ page: 1, limit: 50 })),
    );
  });

  it("任一篩選變更回到第 1 頁，篩選值送進 browser API", async () => {
    const getAdminAudit = vi.fn(async () => PAGE_ONE);
    renderAt("/admin/audit", makeAdminClient({ getAdminAudit }));
    await screen.findByText("第 1–20 筆，共 45 筆");
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await user.click(screen.getByRole("button", { name: "第 2 頁" }));
    await waitFor(() => expect(getAdminAudit).toHaveBeenLastCalledWith(expect.objectContaining({ page: 2 })));
    await user.click(screen.getByRole("combobox", { name: "動作" }));
    await user.click(await screen.findByRole("option", { name: "邀請使用者" }));
    await waitFor(() =>
      expect(getAdminAudit).toHaveBeenLastCalledWith(
        expect.objectContaining({ action: "user-invited", page: 1 }),
      ),
    );
  });

  it("關鍵字與時間區間也送進 browser API", async () => {
    const getAdminAudit = vi.fn(async () => PAGE_ONE);
    renderAt("/admin/audit", makeAdminClient({ getAdminAudit }));
    await screen.findByRole("table");
    const user = userEvent.setup();
    await user.type(screen.getByRole("textbox", { name: "搜尋" }), "demo");
    await user.type(screen.getByLabelText("起始日期"), "2026-01-01");
    await user.type(screen.getByLabelText("結束日期"), "2026-01-31");
    await waitFor(() =>
      expect(getAdminAudit).toHaveBeenLastCalledWith(
        expect.objectContaining({ q: "demo", from: "2026-01-01", to: "2026-01-31", page: 1 }),
      ),
    );
  });

  it("寬螢幕表格五欄照舊、表頭固定且在列表卡內，來源為中性狀態籤", async () => {
    renderAt("/admin/audit", makeAdminClient({ getAdminAudit: vi.fn(async () => PAGE_ONE) }));
    const table = await screen.findByRole("table");
    const card = table.closest("[data-list-card]");
    expect(card).toBeTruthy();
    const headers = within(table).getAllByRole("columnheader").map((h) => h.textContent);
    expect(headers).toEqual(["時間", "動作", "對象", "操作者", "來源"]);
    const head = table.querySelector("thead") as HTMLElement;
    expect(head.className).toContain("sticky");
    expect(within(table).getAllByRole("row")).toHaveLength(3); // 表頭 + 兩筆
    expect(within(table).getByText("瀏覽器").className).toContain("bg-muted");
  });

  it("篩選後沒有事件時呈現空狀態、工具列仍在、沒有換頁列", async () => {
    const getAdminAudit = vi.fn(async () => ({ entries: [], totalPages: 0, total: 0 }));
    renderAt("/admin/audit", makeAdminClient({ getAdminAudit }));
    expect(await screen.findByText(/沒有符合/)).toBeTruthy();
    expect(screen.queryByRole("table")).toBeNull();
    // 篩選控制項仍在，使用者才能改回來。
    expect(screen.getByRole("textbox", { name: "搜尋" })).toBeTruthy();
    expect(screen.getByRole("combobox", { name: "動作" })).toBeTruthy();
    expect(screen.queryByText(/^第 .+ 筆，共 .+ 筆$/)).toBeNull();
  });

  it("沒有舊的「第 n / m 頁」與上一頁／下一頁文字鈕", async () => {
    renderAt("/admin/audit", makeAdminClient({ getAdminAudit: vi.fn(async () => PAGE_ONE) }));
    await screen.findByText("第 1–20 筆，共 45 筆");
    expect(screen.queryByText(/第 1 \/ 3 頁/)).toBeNull();
  });

  it("窄螢幕：每筆一列（日期、動作名稱、「對象 · 操作者」、來源籤），換頁列為窄版", async () => {
    setViewport(true);
    renderAt("/admin/audit", makeAdminClient({ getAdminAudit: vi.fn(async () => PAGE_ONE) }));
    const row = (await screen.findByText("邀請使用者")).closest("[data-audit]") as HTMLElement;
    expect(screen.queryByRole("table")).toBeNull();
    expect(row.getAttribute("role")).toBeNull();
    expect(row.textContent).toContain("2026-01-03");
    expect(row.querySelector("[data-desc]")?.textContent).toBe("new@example.com · a1");
    expect(within(row).getByText("瀏覽器")).toBeTruthy();
    expect(screen.getByText("第 1–20 筆，共 45 筆")).toBeTruthy();
    expect(screen.getByText("1 / 3")).toBeTruthy();
    expect(screen.queryByRole("combobox", { name: /每頁/ })).toBeNull();
  });
});

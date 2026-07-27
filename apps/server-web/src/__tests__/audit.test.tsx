import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { makeAdminClient, renderAt, setViewport } from "./helpers/adminHarness";

// 管理稽核紀錄頁（server-web-console「管理列表提供搜尋、篩選、分頁與具引導的空狀態」
// 與 server-admin「稽核篩選與分頁由伺服器套用」）：關鍵字、動作、來源與時間區間篩選
// 加分頁控制項；SPA 只把參數送給 browser API，並只呈現伺服器回傳的當頁事件與總頁數。

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
};

beforeEach(() => setViewport(false));

describe("稽核紀錄頁", () => {
  it("提供關鍵字、動作、來源與時間區間篩選及分頁控制項", async () => {
    renderAt("/admin/audit", makeAdminClient({ getAdminAudit: vi.fn(async () => PAGE_ONE) }));
    expect(await screen.findByRole("searchbox", { name: /搜尋/ })).toBeTruthy();
    expect(screen.getByLabelText(/動作/)).toBeTruthy();
    expect(screen.getByLabelText(/來源/)).toBeTruthy();
    expect(screen.getByLabelText(/起/)).toBeTruthy();
    expect(screen.getByLabelText(/迄/)).toBeTruthy();
    expect(screen.getByRole("button", { name: /上一頁/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: /下一頁/ })).toBeTruthy();
  });

  it("只呈現伺服器回傳的當頁事件與總頁數", async () => {
    const getAdminAudit = vi.fn(async () => PAGE_ONE);
    renderAt("/admin/audit", makeAdminClient({ getAdminAudit }));
    const table = await screen.findByRole("table");
    expect(within(table).getAllByRole("row")).toHaveLength(3); // 表頭 + 兩筆
    expect(screen.getByText(/第 1 \/ 3 頁/)).toBeTruthy();
  });

  it("套用動作篩選並換頁時以參數呼叫 browser API", async () => {
    const getAdminAudit = vi.fn(async () => PAGE_ONE);
    renderAt("/admin/audit", makeAdminClient({ getAdminAudit }));
    await screen.findByRole("table");
    // 動作篩選是 Radix Select：點開 trigger 再點選項（Radix 開啟時關閉 body 的
    // pointer-events，jsdom 沒有真實命中測試，故關掉 userEvent 的檢查）。
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await user.click(screen.getByRole("combobox", { name: /動作/ }));
    await user.click(await screen.findByRole("option", { name: "邀請使用者" }));
    await waitFor(() =>
      expect(getAdminAudit).toHaveBeenLastCalledWith(
        expect.objectContaining({ action: "user-invited", page: 1 }),
      ),
    );
    await user.click(screen.getByRole("button", { name: /下一頁/ }));
    await waitFor(() =>
      expect(getAdminAudit).toHaveBeenLastCalledWith(
        expect.objectContaining({ action: "user-invited", page: 2 }),
      ),
    );
  });

  it("第一頁時上一頁停用，最後一頁時下一頁停用", async () => {
    const getAdminAudit = vi.fn(async () => ({ ...PAGE_ONE, totalPages: 1 }));
    renderAt("/admin/audit", makeAdminClient({ getAdminAudit }));
    await screen.findByRole("table");
    expect((screen.getByRole("button", { name: /上一頁/ }) as HTMLButtonElement).disabled).toBe(
      true,
    );
    expect((screen.getByRole("button", { name: /下一頁/ }) as HTMLButtonElement).disabled).toBe(
      true,
    );
  });

  it("篩選無結果時呈現空狀態並保留篩選控制項", async () => {
    const getAdminAudit = vi.fn(async () => ({ entries: [], totalPages: 0 }));
    renderAt("/admin/audit", makeAdminClient({ getAdminAudit }));
    expect(await screen.findByText(/沒有符合/)).toBeTruthy();
    expect(screen.queryByRole("table")).toBeNull();
    // 篩選控制項仍在，使用者才能改回來。
    expect(screen.getByRole("searchbox", { name: /搜尋/ })).toBeTruthy();
    expect(screen.getByLabelText(/動作/)).toBeTruthy();
  });

  it("窄螢幕以卡片列取代表格，事件欄位仍完整", async () => {
    setViewport(true);
    renderAt("/admin/audit", makeAdminClient({ getAdminAudit: vi.fn(async () => PAGE_ONE) }));
    expect(await screen.findByText("new@example.com")).toBeTruthy();
    expect(screen.queryByRole("table")).toBeNull();
    const card = screen.getByText("new@example.com").closest("li");
    expect(card?.textContent).toContain("邀請使用者");
    expect(card?.textContent).toContain("瀏覽器");
  });

  it("關鍵字與時間區間也送進 browser API", async () => {
    const getAdminAudit = vi.fn(async () => PAGE_ONE);
    renderAt("/admin/audit", makeAdminClient({ getAdminAudit }));
    await screen.findByRole("table");
    const user = userEvent.setup();
    await user.type(screen.getByRole("searchbox", { name: /搜尋/ }), "demo");
    await user.type(screen.getByLabelText(/起/), "2026-01-01");
    await user.type(screen.getByLabelText(/迄/), "2026-01-31");
    await waitFor(() =>
      expect(getAdminAudit).toHaveBeenLastCalledWith(
        expect.objectContaining({ q: "demo", from: "2026-01-01", to: "2026-01-31" }),
      ),
    );
  });
});

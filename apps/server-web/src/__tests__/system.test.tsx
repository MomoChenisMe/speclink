import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, within } from "@testing-library/react";
import { WebApiError } from "../api/client";
import { makeAdminClient, renderAt, setViewport } from "./helpers/adminHarness";

// 系統頁（server-web-console「資料操作目的地已併入系統」「共用設計系統維持高密度可存取體驗」
// 的設定型頁條款；server-web-list-pages design D10）：768px 置中欄，執行環境、儲存狀態、匯出與
// 危險區四張設定型卡，各值為卡內的列；資料來自單一 view model。取得失敗時整頁呈現錯誤與重試
// 入口——不部分渲染陳舊資料，因為 Store 健康在整個介面只該有一處權威來源。

beforeEach(() => setViewport(false));

describe("系統頁", () => {
  it("側欄不含資料操作目的地", async () => {
    renderAt("/admin/system", makeAdminClient());
    const nav = await screen.findByRole("navigation", { name: "管理導覽" });
    expect(within(nav).queryByText("資料操作")).toBeNull();
    expect(within(nav).getByText("系統")).toBeTruthy();
  });

  it("單頁呈現執行環境、儲存狀態、匯出與危險區四個區段", async () => {
    renderAt("/admin/system", makeAdminClient());
    for (const section of ["執行環境", "儲存狀態", "匯出", "危險區"]) {
      expect(
        await screen.findByRole("region", { name: section }),
        `系統頁有「${section}」區段`,
      ).toBeTruthy();
    }
  });

  it("執行環境與儲存狀態的欄位來自同一份 view model", async () => {
    const getAdminSystem = vi.fn(async () => ({
      engineVersion: "0.1.0",
      apiVersion: "1",
      identitySchemaVersion: 6,
      storeDriver: "sqlite",
      storeContractVersion: 3,
      storeLevel: "full",
      storeCapabilities: ["migration", "backup"],
      storeHealthy: true,
      storeHealthError: null,
      outboxBacklogs: [{ project: "demo", repo: "backend", backlog: 0 }],
      scopes: [{ project: "demo", repo: "backend", exportPath: "/admin/data/export/demo/backend" }],
      migrateAvailable: true,
    }));
    renderAt("/admin/system", makeAdminClient({ getAdminSystem }));
    const runtime = await screen.findByRole("region", { name: "執行環境" });
    expect(within(runtime).getByRole("group", { name: "引擎版本" }).textContent).toContain("0.1.0");
    expect(within(runtime).getByRole("group", { name: "儲存後端" }).textContent).toContain("sqlite");
    const storage = screen.getByRole("region", { name: "儲存狀態" });
    expect(within(storage).getByRole("group", { name: "儲存後端" }).textContent).toContain("正常");
    expect(within(storage).getByRole("group", { name: "demo/backend" }).textContent).toContain("0");
    const exports = screen.getByRole("region", { name: "匯出" });
    const scope = within(exports).getByRole("group", { name: "demo/backend" });
    expect(
      within(scope).getByRole("link", { name: /demo\/backend|匯出/ }).getAttribute("href"),
    ).toBe("/admin/data/export/demo/backend");
    // 單一 view model：整頁只打一支 API。
    expect(getAdminSystem).toHaveBeenCalledTimes(1);
  });

  it("view model 取得失敗時整頁呈現錯誤與重試入口，不部分渲染", async () => {
    renderAt(
      "/admin/system",
      makeAdminClient({
        getAdminSystem: vi.fn(async () => {
          throw new WebApiError(500, "internal", "伺服器錯誤");
        }),
      }),
    );
    expect(await screen.findByRole("button", { name: /重試/ })).toBeTruthy();
    for (const section of ["執行環境", "儲存狀態", "匯出", "危險區"]) {
      expect(
        screen.queryByRole("region", { name: section }),
        `失敗時不渲染「${section}」`,
      ).toBeNull();
    }
  });

  it("內容欄 768px 置中、沒有表格", async () => {
    renderAt("/admin/system", makeAdminClient());
    await screen.findByRole("region", { name: "執行環境" });
    const column = screen
      .getByRole("heading", { level: 1, name: "系統" })
      .closest("[data-page-header]")?.parentElement as HTMLElement;
    expect(column.className).toContain("max-w-[768px]");
    expect(column.className).toContain("mx-auto");
    expect(screen.queryByRole("table")).toBeNull();
  });

  it("遷移在危險區的「資料結構遷移」列，經 AlertDialog 確認後才送出", async () => {
    const client = makeAdminClient();
    renderAt("/admin/system", client);
    const danger = await screen.findByRole("region", { name: "危險區" });
    expect(within(danger).getByRole("group", { name: "資料結構遷移" })).toBeTruthy();
    const { default: userEvent } = await import("@testing-library/user-event");
    const user = userEvent.setup();
    await user.click(within(danger).getByRole("button", { name: /遷移/ }));
    const dialog = await screen.findByRole("alertdialog");
    expect(client.adminMigrate).not.toHaveBeenCalled();
    await user.click(within(dialog).getByRole("button", { name: /遷移|確認/ }));
    expect(client.adminMigrate).toHaveBeenCalledOnce();
  });

  it("儲存後端不可用時停用遷移並說明原因", async () => {
    renderAt(
      "/admin/system",
      makeAdminClient({
        getAdminSystem: vi.fn(async () => ({
          engineVersion: "0.1.0",
          apiVersion: "1",
          identitySchemaVersion: 6,
          storeDriver: "sqlite",
          storeContractVersion: 3,
          storeLevel: "full",
          storeCapabilities: [],
          storeHealthy: false,
          storeHealthError: "store unreachable",
          outboxBacklogs: [],
          scopes: [],
          migrateAvailable: false,
        })),
      }),
    );
    const danger = await screen.findByRole("region", { name: "危險區" });
    expect((within(danger).getByRole("button", { name: /遷移/ }) as HTMLButtonElement).disabled).toBe(
      true,
    );
    const storage = screen.getByRole("region", { name: "儲存狀態" });
    expect(within(storage).getByRole("group", { name: "儲存後端" }).textContent).toContain(
      "store unreachable",
    );
    // spec「儲存健康徽章」：徽章與同區的錯誤訊息同語意色層級。
    expect(within(storage).getByText("異常").className).toContain("destructive");
  });

  it("舊的 /admin/data 連結導向系統頁", async () => {
    renderAt("/admin/data", makeAdminClient());
    expect(await screen.findByRole("heading", { name: "系統" })).toBeTruthy();
  });
});

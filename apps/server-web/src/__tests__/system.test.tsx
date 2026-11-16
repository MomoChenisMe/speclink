import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, within } from "@testing-library/react";
import { WebApiError } from "../api/client";
import { makeAdminClient, renderAt, setViewport } from "./helpers/adminHarness";

// 系統頁（server-web-console「資料操作目的地已併入系統」「共用設計系統維持高密度可存取體驗」
// 的設定型頁條款；server-web-list-pages design D10）：768px 置中欄，執行環境、儲存狀態與匯出
// 三張設定型卡，各值為卡內的列；資料來自單一 view model。取得失敗時整頁呈現錯誤與重試
// 入口——不部分渲染陳舊資料，因為 Store 健康在整個介面只該有一處權威來源。
// 沒有危險區與資料結構遷移（server-on-port-examples design D7）。

beforeEach(() => setViewport(false));

describe("系統頁", () => {
  it("側欄不含資料操作目的地", async () => {
    renderAt("/admin/system", makeAdminClient());
    const nav = await screen.findByRole("navigation", { name: "管理導覽" });
    expect(within(nav).queryByText("資料操作")).toBeNull();
    expect(within(nav).getByText("系統")).toBeTruthy();
  });

  it("單頁呈現執行環境、儲存狀態與匯出三個區段，沒有危險區與遷移", async () => {
    renderAt("/admin/system", makeAdminClient());
    for (const section of ["執行環境", "儲存狀態", "匯出"]) {
      expect(
        await screen.findByRole("region", { name: section }),
        `系統頁有「${section}」區段`,
      ).toBeTruthy();
    }
    expect(screen.getAllByRole("region")).toHaveLength(3);
    expect(screen.queryByRole("region", { name: "危險區" })).toBeNull();
    expect(screen.queryByRole("button", { name: /遷移/ })).toBeNull();
    expect(screen.queryByRole("alertdialog")).toBeNull();
    expect(document.body.textContent).not.toContain("遷移");
  });

  it("執行環境與儲存狀態的欄位來自同一份 view model", async () => {
    const getAdminSystem = vi.fn(async () => ({
      engineVersion: "0.1.0",
      apiVersion: "1",
      identitySchemaVersion: 6,
      storeDriver: "sqlite",
      storeHealthy: true,
      storeHealthError: null,
      outboxBacklogs: [{ project: "demo", repo: "backend", backlog: 7 }],
      scopes: [{ project: "demo", repo: "backend", exportPath: "/admin/data/export/demo/backend" }],
    }));
    renderAt("/admin/system", makeAdminClient({ getAdminSystem }));
    const runtime = await screen.findByRole("region", { name: "執行環境" });
    expect(within(runtime).getByRole("group", { name: "引擎版本" }).textContent).toContain("0.1.0");
    // 執行環境只有三個版本；契約版本、等級與能力是舊儲存層的概念，已移除。
    expect(within(runtime).getAllByRole("group")).toHaveLength(3);
    for (const gone of ["契約版本", "等級", "能力"]) {
      expect(screen.queryByRole("group", { name: gone }), `沒有「${gone}」列`).toBeNull();
    }
    // 儲存狀態：種類、是否可用，與各範圍的事件筆數。
    const storage = screen.getByRole("region", { name: "儲存狀態" });
    const backend = within(storage).getByRole("group", { name: "儲存後端" });
    expect(backend.textContent).toContain("sqlite");
    expect(backend.textContent).toContain("正常");
    expect(within(storage).getByRole("group", { name: "demo/backend" }).textContent).toContain("7");
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
    for (const section of ["執行環境", "儲存狀態", "匯出"]) {
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

  it("儲存後端不可用時在儲存狀態說明原因", async () => {
    renderAt(
      "/admin/system",
      makeAdminClient({
        getAdminSystem: vi.fn(async () => ({
          engineVersion: "0.1.0",
          apiVersion: "1",
          identitySchemaVersion: 6,
          storeDriver: "sqlite",
          storeHealthy: false,
          storeHealthError: "store unreachable",
          outboxBacklogs: [{ project: "demo", repo: "backend", backlog: null }],
          scopes: [],
        })),
      }),
    );
    const storage = await screen.findByRole("region", { name: "儲存狀態" });
    expect(within(storage).getByRole("group", { name: "儲存後端" }).textContent).toContain(
      "store unreachable",
    );
    // spec「儲存健康徽章」：徽章與同區的錯誤訊息同語意色層級。
    expect(within(storage).getByText("異常").className).toContain("destructive");
    // 讀不到事件筆數的範圍顯示破折號，不是 0。
    expect(within(storage).getByRole("group", { name: "demo/backend" }).textContent).toContain("—");
  });

  it("舊的 /admin/data 連結導向系統頁", async () => {
    renderAt("/admin/data", makeAdminClient());
    expect(await screen.findByRole("heading", { name: "系統" })).toBeTruthy();
  });
});

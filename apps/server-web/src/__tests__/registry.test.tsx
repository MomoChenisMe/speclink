import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { WebApiError } from "../api/client";
import {
  currentLocation,
  makeAdminClient,
  REGISTRY,
  renderAt,
  renderWithLocation,
  setViewport,
} from "./helpers/adminHarness";

// 管理專案與儲存庫頁（server-web-console「管理列表以詳情頁與對話框承載檢視與建立」「管理列表提供
// 搜尋、篩選、分頁與具引導的空狀態」；server-web-list-pages design D2、D3、D5；server-web-detail-pages
// design D1、D7）：專案以列表卡內的共用列呈現，整列可點進專案詳情頁（詳情頁本身見
// projectDetail.test.tsx）；建立專案以置中對話框承載，不常駐列表頁。

beforeEach(() => setViewport(false));

async function openCreate(client = makeAdminClient()) {
  renderAt("/admin/registry", client);
  await screen.findByRole("button", { name: "Demo" });
  const user = userEvent.setup({ pointerEventsCheck: 0 });
  await user.click(screen.getByRole("button", { name: /建立專案/ }));
  return { user, dialog: await screen.findByRole("alertdialog", { name: "建立專案" }) };
}

describe("專案與儲存庫", () => {
  it("專案列為可聚焦的按鈕、名稱為無障礙名稱、說明為「代號 <key> · n 個儲存庫」，列內沒有輸入控制項", async () => {
    renderAt("/admin/registry", makeAdminClient());
    const row = await screen.findByRole("button", { name: "Demo" });
    expect(row.getAttribute("tabindex")).toBe("0");
    expect(row.querySelector("[data-desc]")?.textContent).toBe("代號 demo · 1 個儲存庫");
    expect(within(row).queryAllByRole("textbox")).toHaveLength(0);
    // 列尾沒有另一顆「檢視詳細資料」鈕：整列本身就是入口。
    expect(screen.queryByRole("button", { name: /檢視/ })).toBeNull();
  });

  it("點專案列或以鍵盤移到列上按 Enter 都進入 /admin/registry/<代號>", async () => {
    renderWithLocation(["/admin/registry"], makeAdminClient());
    const user = userEvent.setup();
    await user.click(await screen.findByRole("button", { name: "Demo" }));
    expect(await screen.findByRole("heading", { level: 1, name: "Demo" })).toBeTruthy();
    expect(currentLocation()).toBe("/admin/registry/demo");
    expect(screen.queryByRole("dialog")).toBeNull();

    await user.click(within(screen.getByRole("navigation", { name: "頁面路徑" })).getByRole("button", { name: "專案與儲存庫" }));
    (await screen.findByRole("button", { name: "Demo" })).focus();
    await user.keyboard("{Enter}");
    expect(await screen.findByRole("heading", { level: 1, name: "Demo" })).toBeTruthy();
    expect(currentLocation()).toBe("/admin/registry/demo");
  });

  it("沒有專案時空狀態說明用途、空狀態內沒有建立鈕，頁首「建立專案」仍在", async () => {
    renderAt("/admin/registry", makeAdminClient({ getAdminRegistry: vi.fn(async () => ({ projects: [] })) }));
    expect(await screen.findByText("尚無專案")).toBeTruthy();
    expect(screen.getAllByRole("button", { name: /建立專案/ })).toHaveLength(1);
    expect(screen.queryByText(/^第 .+ 筆，共 .+ 筆$/)).toBeNull();
  });

  // server-web-console「管理列表提供搜尋、篩選、分頁與具引導的空狀態」：專案與儲存庫頁
  // SHALL 提供關鍵字搜尋。
  it("關鍵字搜尋收斂專案清單，無結果時呈現無符合狀態並保留搜尋框", async () => {
    renderAt("/admin/registry", makeAdminClient());
    expect(await screen.findByRole("button", { name: "Demo" })).toBeTruthy();
    expect(screen.getByText("第 1–1 筆，共 1 筆")).toBeTruthy();
    const user = userEvent.setup();
    await user.type(screen.getByRole("textbox", { name: "搜尋" }), "找不到的東西");
    await waitFor(() => expect(screen.queryByRole("button", { name: "Demo" })).toBeNull());
    expect(screen.getByText(/沒有符合/)).toBeTruthy();
    expect(screen.getByRole("textbox", { name: "搜尋" })).toBeTruthy();
    expect(screen.queryByText(/^第 .+ 筆，共 .+ 筆$/)).toBeNull();
  });

  it("搜尋專案代號或儲存庫名稱可篩出該專案", async () => {
    const registry = {
      projects: [...REGISTRY.projects, { key: "ops", name: "Operations", repos: [{ key: "infra", name: "Infra" }] }],
    };
    renderAt("/admin/registry", makeAdminClient({ getAdminRegistry: vi.fn(async () => registry) }));
    await screen.findByRole("button", { name: "Demo" });
    const user = userEvent.setup();
    const search = screen.getByRole("textbox", { name: "搜尋" });
    await user.type(search, "backend");
    await waitFor(() => expect(screen.queryByRole("button", { name: "Operations" })).toBeNull());
    expect(screen.getByRole("button", { name: "Demo" })).toBeTruthy();
    await user.clear(search);
    await user.type(search, "ops");
    await waitFor(() => expect(screen.queryByRole("button", { name: "Demo" })).toBeNull());
    expect(screen.getByRole("button", { name: "Operations" })).toBeTruthy();
  });

});

describe("建立專案對話框", () => {
  // 「建立後不可變更」與初始設定同一種寫法：標籤只放名稱，限制放在欄位下方的灰字說明。
  it("「建立專案」開出置中對話框，代號欄以灰字說明標示建立後不可變更", async () => {
    const { dialog } = await openCreate();
    const projectKey = within(dialog).getByLabelText("專案代號");
    expect(document.getElementById(projectKey.getAttribute("aria-describedby") ?? "")?.textContent).toBe(
      "建立後不可變更。",
    );
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("輸入代號後點遮罩不關閉、代號仍在；按 Esc 關閉", async () => {
    const { user, dialog } = await openCreate();
    await user.type(within(dialog).getByLabelText("專案代號"), "ops");
    // 遮罩是對話框內容的前一個兄弟節點（同 packages/ui 的確認框測試）。
    await user.click(dialog.previousElementSibling as HTMLElement);
    expect(screen.getByRole("alertdialog", { name: "建立專案" })).toBeTruthy();
    expect((within(dialog).getByLabelText("專案代號") as HTMLInputElement).value).toBe("ops");
    await user.keyboard("{Escape}");
    await waitFor(() => expect(screen.queryByRole("alertdialog")).toBeNull());
  });

  it("送出成功後對話框關閉，列表多出新專案", async () => {
    const created = { projects: [...REGISTRY.projects, { key: "ops", name: "ops", repos: [] }] };
    const client = makeAdminClient({
      getAdminRegistry: vi.fn().mockResolvedValueOnce(REGISTRY).mockResolvedValue(created),
    });
    const { user, dialog } = await openCreate(client);
    await user.type(within(dialog).getByLabelText("專案代號"), "ops");
    await user.click(within(dialog).getByRole("button", { name: "建立專案" }));
    await waitFor(() => expect(client.adminCreateProject).toHaveBeenCalledWith({ key: "ops", name: undefined }));
    await waitFor(() => expect(screen.queryByRole("alertdialog")).toBeNull());
    expect(await screen.findByRole("button", { name: "ops" })).toBeTruthy();
  });

  it("欄位錯誤時對話框保持開啟、保留輸入，錯誤緊貼欄位", async () => {
    const client = makeAdminClient({
      adminCreateProject: vi.fn(async () => {
        throw new WebApiError(400, "validation_error", "欄位有誤", { key: "代號已存在" });
      }),
    });
    const { user, dialog } = await openCreate(client);
    await user.type(within(dialog).getByLabelText("專案代號"), "demo");
    await user.click(within(dialog).getByRole("button", { name: "建立專案" }));
    const alert = await within(dialog).findByRole("alert");
    expect(alert.textContent).toContain("代號已存在");
    expect(alert.id).toBe("new-project-key-error");
    expect((within(dialog).getByLabelText("專案代號") as HTMLInputElement).value).toBe("demo");
  });
});

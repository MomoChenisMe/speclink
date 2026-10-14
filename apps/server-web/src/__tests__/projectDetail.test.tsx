import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { WebApiError } from "../api/client";
import { REGISTRY, currentLocation, makeAdminClient, renderAt, renderWithLocation, setViewport } from "./helpers/adminHarness";

// 專案詳情頁（server-web-console「管理列表以詳情頁與對話框承載檢視與建立」「不可變識別欄位唯讀且
// 更名為顯式動作」；server-web-detail-pages design D1、D6、D7）：代號是身分、建立後不可變更，只以
// 唯讀文字呈現；名稱預設唯讀，按「更名」才在對話框裡出現預填目前名稱的輸入框。分頁為儲存庫與成員，
// 儲存庫每列可更名、末列新增儲存庫；成員唯讀。

beforeEach(() => setViewport(false));

const tab = (name: RegExp) => screen.getByRole("tab", { name });
const tabCount = (name: RegExp) => tab(name).querySelector("[data-tab-count]")?.textContent;
const crumbs = () => screen.getByRole("navigation", { name: "頁面路徑" });

async function openDemo(client = makeAdminClient()) {
  renderAt("/admin/registry/demo", client);
  const heading = await screen.findByRole("heading", { level: 1, name: "Demo" });
  const user = userEvent.setup({ pointerEventsCheck: 0 });
  return { user, header: heading.closest("[data-page-header]") as HTMLElement };
}

describe("專案詳情頁", () => {
  it("直接開 /admin/registry/demo：頁標題為名稱、說明標示代號建立後不可變更、儲存庫與成員兩個分頁，沒有可編輯代號的輸入框", async () => {
    const { header } = await openDemo();
    expect(header.textContent).toContain("代號 demo");
    expect(header.textContent).toContain("建立後不可變更");
    expect(tabCount(/^儲存庫/)).toBe("1");
    // 成員由使用者清單反查：a1 與 m1 都在 demo。
    await waitFor(() => expect(tabCount(/^成員/)).toBe("2"));
    expect(screen.queryAllByRole("textbox")).toHaveLength(0);
  });

  it("儲存庫分頁每列以唯讀文字標示代號建立後不可變更", async () => {
    await openDemo();
    const panel = screen.getByRole("tabpanel");
    expect(within(panel).getByText("Backend")).toBeTruthy();
    expect(within(panel).getByText("代號 backend · 建立後不可變更")).toBeTruthy();
  });

  it("頁標題區「更名」開出預填目前名稱的對話框；確認後頁標題與麵包屑換成新名稱", async () => {
    const renamed = { projects: [{ ...REGISTRY.projects[0], name: "Demo 2" }] };
    const client = makeAdminClient({
      getAdminRegistry: vi.fn().mockResolvedValueOnce(REGISTRY).mockResolvedValue(renamed),
    });
    const { user, header } = await openDemo(client);
    await user.click(within(header).getByRole("button", { name: "更名" }));
    const dialog = await screen.findByRole("alertdialog", { name: "更名專案" });
    expect(dialog.textContent).toContain("代號 demo 不會改變");
    const input = within(dialog).getByLabelText("專案名稱") as HTMLInputElement;
    expect(input.value).toBe("Demo");
    await user.clear(input);
    await user.type(input, "Demo 2");
    await user.click(within(dialog).getByRole("button", { name: "確認" }));
    await waitFor(() => expect(client.adminRenameProject).toHaveBeenCalledWith("demo", "Demo 2"));
    await waitFor(() => expect(screen.queryByRole("alertdialog")).toBeNull());
    expect(await screen.findByRole("heading", { level: 1, name: "Demo 2" })).toBeTruthy();
    expect(crumbs().textContent).toBe("後台/專案與儲存庫/Demo 2");
    expect(screen.getByRole("heading", { level: 1 }).closest("[data-page-header]")?.textContent).toContain("代號 demo");
  });

  it("更名失敗時對話框保持開啟、保留輸入，錯誤以 role=alert 宣告", async () => {
    const client = makeAdminClient({
      adminRenameProject: vi.fn(async () => {
        throw new WebApiError(400, "validation_error", "名稱太長");
      }),
    });
    const { user, header } = await openDemo(client);
    await user.click(within(header).getByRole("button", { name: "更名" }));
    const dialog = await screen.findByRole("alertdialog", { name: "更名專案" });
    const input = within(dialog).getByLabelText("專案名稱") as HTMLInputElement;
    await user.clear(input);
    await user.type(input, "Demo 2");
    await user.click(within(dialog).getByRole("button", { name: "確認" }));
    expect((await within(dialog).findByRole("alert")).textContent).toContain("名稱太長");
    expect(screen.getByRole("alertdialog", { name: "更名專案" })).toBeTruthy();
    expect(input.value).toBe("Demo 2");
  });

  it("儲存庫列的「更名 Backend」開出「更名儲存庫」對話框並以 adminRenameRepo 送出", async () => {
    const client = makeAdminClient();
    const { user } = await openDemo(client);
    await user.click(screen.getByRole("button", { name: "更名 Backend" }));
    const dialog = await screen.findByRole("alertdialog", { name: "更名儲存庫" });
    expect(dialog.textContent).toContain("代號 backend 不會改變");
    const input = within(dialog).getByLabelText("儲存庫名稱") as HTMLInputElement;
    expect(input.value).toBe("Backend");
    await user.clear(input);
    await user.type(input, "後端");
    await user.click(within(dialog).getByRole("button", { name: "確認" }));
    await waitFor(() =>
      expect(client.adminRenameRepo).toHaveBeenCalledWith({ projectKey: "demo", key: "backend", name: "後端" }),
    );
    await waitFor(() => expect(screen.queryByRole("alertdialog")).toBeNull());
  });

  it("末列「新增儲存庫」開對話框；代號欄標示建立後不可變更，送出後關閉且儲存庫分頁多一列", async () => {
    const added = {
      projects: [{ ...REGISTRY.projects[0], repos: [...REGISTRY.projects[0].repos, { key: "web", name: "web" }] }],
    };
    const client = makeAdminClient({
      getAdminRegistry: vi.fn().mockResolvedValueOnce(REGISTRY).mockResolvedValue(added),
    });
    const { user } = await openDemo(client);
    expect(screen.queryByLabelText(/儲存庫代號/)).toBeNull();
    await user.click(screen.getByRole("button", { name: "新增儲存庫" }));
    const dialog = await screen.findByRole("alertdialog", { name: "新增儲存庫" });
    const key = within(dialog).getByLabelText("儲存庫代號");
    expect(document.getElementById(key.getAttribute("aria-describedby") ?? "")?.textContent).toBe("建立後不可變更。");
    await user.type(key, "web");
    await user.click(within(dialog).getByRole("button", { name: "建立" }));
    await waitFor(() =>
      expect(client.adminCreateRepo).toHaveBeenCalledWith(expect.objectContaining({ projectKey: "demo", key: "web" })),
    );
    await waitFor(() => expect(screen.queryByRole("alertdialog")).toBeNull());
    await waitFor(() => expect(tabCount(/^儲存庫/)).toBe("2"));
    expect(within(screen.getByRole("tabpanel")).getByText("代號 web · 建立後不可變更")).toBeTruthy();
  });

  it("成員分頁唯讀列出顯示名稱、電子郵件與角色籤，沒有按鈕", async () => {
    const { user } = await openDemo();
    await waitFor(() => expect(tabCount(/^成員/)).toBe("2"));
    await user.click(tab(/^成員/));
    const panel = screen.getByRole("tabpanel");
    expect(within(panel).getByText("Member")).toBeTruthy();
    expect(within(panel).getByText("member@example.com")).toBeTruthy();
    expect(within(panel).getAllByText("editor")).toHaveLength(2);
    expect(within(panel).queryAllByRole("button")).toHaveLength(0);
  });

  it("對不到資料時說明找不到並提供回到專案列表", async () => {
    renderWithLocation(["/admin/registry/nope"], makeAdminClient());
    expect(await screen.findByText("找不到這個專案")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "上一個" })).toBeNull();
    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "回到專案列表" }));
    expect(await screen.findByRole("heading", { level: 1, name: "專案與儲存庫" })).toBeTruthy();
    expect(currentLocation()).toBe("/admin/registry");
  });
});

import { describe, it, expect, beforeEach } from "vitest";
import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { makeAdminClient, renderAt, setViewport } from "./helpers/adminHarness";

// 管理專案與儲存庫頁（server-web-console「不可變識別欄位唯讀且更名為顯式動作」）：
// 代號建立後不可變更，以唯讀文字呈現；名稱預設唯讀，按下更名才出現輸入框與確認／取消。
// 建立專案與新增儲存庫由抽屜承載，不常駐列表頁。

beforeEach(() => setViewport(false));

async function openProjectDrawer() {
  const user = userEvent.setup();
  await user.click(await screen.findByRole("button", { name: /檢視 Demo/ }));
  return { user, drawer: await screen.findByRole("dialog", { name: /Demo/ }) };
}

describe("專案與儲存庫", () => {
  it("列表頁不含任何輸入控制項，建立專案由抽屜承載", async () => {
    renderAt("/admin/registry", makeAdminClient());
    expect(await screen.findByText("Demo")).toBeTruthy();
    expect(screen.queryAllByRole("textbox")).toHaveLength(0);
    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /建立專案/ }));
    const drawer = await screen.findByRole("dialog", { name: /建立專案/ });
    expect(within(drawer).getByLabelText(/專案代號/)).toBeTruthy();
  });

  it("專案代號以唯讀文字呈現，畫面上沒有可編輯代號的輸入框", async () => {
    renderAt("/admin/registry", makeAdminClient());
    const { drawer } = await openProjectDrawer();
    expect(drawer.textContent).toContain("demo");
    expect(drawer.textContent).toContain("建立後不可變更");
    // 代號沒有任何輸入框；名稱在未按更名前也還不是輸入框。
    expect(within(drawer).queryByLabelText(/代號/)).toBeNull();
    expect(within(drawer).queryAllByRole("textbox")).toHaveLength(0);
  });

  it("專案名稱預設唯讀，按下更名後才出現輸入框與確認及取消", async () => {
    const client = makeAdminClient();
    renderAt("/admin/registry", client);
    const { user, drawer } = await openProjectDrawer();
    await user.click(within(drawer).getByRole("button", { name: "更名" }));
    const input = within(drawer).getByLabelText(/專案名稱/) as HTMLInputElement;
    expect(input.value).toBe("Demo");
    expect(within(drawer).getByRole("button", { name: "取消" })).toBeTruthy();
    await user.clear(input);
    await user.type(input, "示範專案");
    await user.click(within(drawer).getByRole("button", { name: "確認" }));
    await waitFor(() =>
      expect(client.adminRenameProject).toHaveBeenCalledWith("demo", "示範專案"),
    );
  });

  it("儲存庫代號唯讀，更名同樣是顯式動作", async () => {
    const client = makeAdminClient();
    renderAt("/admin/registry", client);
    const { user, drawer } = await openProjectDrawer();
    const repos = within(drawer).getByRole("tabpanel", { name: "儲存庫" });
    expect(repos.textContent).toContain("backend");
    expect(within(repos).queryAllByRole("textbox")).toHaveLength(0);
    await user.click(within(repos).getByRole("button", { name: /更名.*Backend/ }));
    const input = within(repos).getByLabelText(/儲存庫名稱/) as HTMLInputElement;
    await user.clear(input);
    await user.type(input, "後端");
    await user.click(within(repos).getByRole("button", { name: "確認" }));
    await waitFor(() =>
      expect(client.adminRenameRepo).toHaveBeenCalledWith({
        projectKey: "demo",
        key: "backend",
        name: "後端",
      }),
    );
  });

  // server-web-console「管理列表提供搜尋、篩選、分頁與具引導的空狀態」：專案與儲存庫頁
  // SHALL 提供關鍵字搜尋。
  it("關鍵字搜尋收斂專案清單，無結果時呈現空狀態並保留搜尋框", async () => {
    renderAt("/admin/registry", makeAdminClient());
    expect(await screen.findByText("Demo")).toBeTruthy();
    const user = userEvent.setup();
    await user.type(screen.getByRole("searchbox", { name: /搜尋/ }), "找不到的東西");
    await waitFor(() => expect(screen.queryByText("Demo")).toBeNull());
    expect(screen.getByText(/沒有符合/)).toBeTruthy();
    expect(screen.getByRole("searchbox", { name: /搜尋/ })).toBeTruthy();
  });

  it("新增儲存庫在抽屜內按下動作後才出現欄位", async () => {
    const client = makeAdminClient();
    renderAt("/admin/registry", client);
    const { user, drawer } = await openProjectDrawer();
    expect(within(drawer).queryByLabelText(/儲存庫代號/)).toBeNull();
    await user.click(within(drawer).getByRole("button", { name: /新增儲存庫/ }));
    await user.type(within(drawer).getByLabelText(/儲存庫代號/), "web");
    await user.click(within(drawer).getByRole("button", { name: "建立" }));
    await waitFor(() =>
      expect(client.adminCreateRepo).toHaveBeenCalledWith(
        expect.objectContaining({ projectKey: "demo", key: "web" }),
      ),
    );
  });
});

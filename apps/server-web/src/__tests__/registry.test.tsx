import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { makeAdminClient, REGISTRY, renderAt, setViewport } from "./helpers/adminHarness";

// 管理專案與儲存庫頁（server-web-console「不可變識別欄位唯讀且更名為顯式動作」「管理列表提供
// 搜尋、篩選、分頁與具引導的空狀態」；server-web-list-pages design D2、D3、D5）：專案以列表卡
// 內的共用列呈現，整列可點開專案抽屜；代號建立後不可變更，以唯讀文字呈現；名稱預設唯讀，按下
// 更名才出現輸入框與確認／取消。建立專案與新增儲存庫由抽屜承載，不常駐列表頁。

beforeEach(() => setViewport(false));

async function openProjectDrawer() {
  const user = userEvent.setup();
  await user.click(await screen.findByRole("button", { name: "Demo" }));
  return { user, drawer: await screen.findByRole("dialog", { name: /Demo/ }) };
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

  it("以鍵盤移到專案列按 Enter 開出專案抽屜", async () => {
    renderAt("/admin/registry", makeAdminClient());
    const user = userEvent.setup();
    (await screen.findByRole("button", { name: "Demo" })).focus();
    await user.keyboard("{Enter}");
    expect(await screen.findByRole("dialog", { name: /Demo/ })).toBeTruthy();
  });

  it("沒有專案時空狀態說明用途、空狀態內沒有建立鈕，頁首「建立專案」仍在", async () => {
    renderAt("/admin/registry", makeAdminClient({ getAdminRegistry: vi.fn(async () => ({ projects: [] })) }));
    expect(await screen.findByText("尚無專案")).toBeTruthy();
    expect(screen.getAllByRole("button", { name: /建立專案/ })).toHaveLength(1);
    expect(screen.queryByText(/^第 .+ 筆，共 .+ 筆$/)).toBeNull();
  });

  it("建立專案由抽屜承載", async () => {
    renderAt("/admin/registry", makeAdminClient());
    expect(await screen.findByRole("button", { name: "Demo" })).toBeTruthy();
    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /建立專案/ }));
    const drawer = await screen.findByRole("dialog", { name: /建立專案/ });
    expect(within(drawer).getByLabelText(/專案代號/)).toBeTruthy();
  });

  // 「建立後不可變更」與初始設定同一種寫法：標籤只放名稱，限制放在欄位下方的灰字說明。
  it("建立專案與新增儲存庫的代號欄以灰字說明標示建立後不可變更", async () => {
    renderAt("/admin/registry", makeAdminClient());
    const user = userEvent.setup();
    await user.click(await screen.findByRole("button", { name: /建立專案/ }));
    const create = await screen.findByRole("dialog", { name: /建立專案/ });
    const projectKey = within(create).getByLabelText("專案代號");
    expect(document.getElementById(projectKey.getAttribute("aria-describedby") ?? "")?.textContent).toBe(
      "建立後不可變更。",
    );
    await user.keyboard("{Escape}");

    const { drawer } = await openProjectDrawer();
    await user.click(within(drawer).getByRole("button", { name: /新增儲存庫/ }));
    const repoKey = within(drawer).getByLabelText("儲存庫代號");
    expect(document.getElementById(repoKey.getAttribute("aria-describedby") ?? "")?.textContent).toBe(
      "建立後不可變更。",
    );
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

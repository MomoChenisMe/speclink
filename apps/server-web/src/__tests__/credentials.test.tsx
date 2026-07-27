import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import {
  EMPTY_CREDENTIALS,
  makeAdminClient,
  renderAt,
  setViewport,
} from "./helpers/adminHarness";

// 管理憑證頁（server-web-console「管理列表提供搜尋、篩選、分頁與具引導的空狀態」）：
// 存取金鑰與裝置以分頁區分，工具列提供關鍵字搜尋與狀態篩選；空清單以說明用途的空狀態
// 與 primary action 取代單行「尚無資料」。撤銷仍是列尾的明確動作並經 AlertDialog 確認。

beforeEach(() => setViewport(false));

describe("憑證頁", () => {
  it("以存取金鑰與裝置兩個分頁呈現", async () => {
    renderAt("/admin/credentials", makeAdminClient());
    const keys = await screen.findByRole("tab", { name: /存取金鑰/ });
    expect(keys).toBeTruthy();
    expect(screen.getByRole("tab", { name: /裝置/ })).toBeTruthy();
    // 預設分頁列出存取金鑰的 metadata（絕不含祕密值）。
    expect(await screen.findByText("slk_abc")).toBeTruthy();
  });

  it("關鍵字搜尋只留下相符的存取金鑰", async () => {
    renderAt("/admin/credentials", makeAdminClient());
    expect(await screen.findByText("slk_abc")).toBeTruthy();
    const user = userEvent.setup();
    await user.type(screen.getByRole("searchbox", { name: /搜尋/ }), "部署");
    await waitFor(() => expect(screen.queryByText("slk_abc")).toBeNull());
    expect(screen.getByText("slk_xyz")).toBeTruthy();
  });

  it("狀態篩選只留下有效或已撤銷的項目", async () => {
    renderAt("/admin/credentials", makeAdminClient());
    expect(await screen.findByText("slk_xyz")).toBeTruthy();
    // 狀態篩選是 Radix Select：點開 trigger 再點選項（Radix 開啟時關閉 body 的
    // pointer-events，jsdom 沒有真實命中測試，故關掉 userEvent 的檢查）。
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await user.click(screen.getByRole("combobox", { name: /狀態/ }));
    await user.click(await screen.findByRole("option", { name: "有效" }));
    await waitFor(() => expect(screen.queryByText("slk_xyz")).toBeNull());
    expect(screen.getByText("slk_abc")).toBeTruthy();
  });

  // 空狀態的建立入口不能只在「一個憑證都沒有」時存在——建完第一把之後就再也找不到
  // 新增的地方了。primary action 常駐頁首。
  it("已有憑證時仍提供建立存取金鑰的入口", async () => {
    renderAt("/admin/credentials", makeAdminClient());
    await screen.findByText("slk_abc");
    const create = screen.getByRole("link", { name: /建立存取金鑰/ });
    expect(create.getAttribute("href")).toBe("/account");
  });

  it("篩選無結果時呈現空狀態並保留篩選控制項", async () => {
    renderAt("/admin/credentials", makeAdminClient());
    expect(await screen.findByText("slk_abc")).toBeTruthy();
    const user = userEvent.setup();
    const search = screen.getByRole("searchbox", { name: /搜尋/ });
    await user.type(search, "找不到的東西");
    await waitFor(() => expect(screen.queryByText("slk_abc")).toBeNull());
    expect(screen.getByText(/沒有符合/)).toBeTruthy();
    // 篩選控制項仍在，使用者才能改回來。
    expect(screen.getByRole("searchbox", { name: /搜尋/ })).toBeTruthy();
    expect(screen.getByLabelText(/狀態/)).toBeTruthy();
  });

  it("空憑證頁說明用途並提供建立存取金鑰的 primary action", async () => {
    renderAt(
      "/admin/credentials",
      makeAdminClient({ getAdminCredentials: vi.fn(async () => EMPTY_CREDENTIALS) }),
    );
    const empty = await screen.findByText(/遠端工作流程/);
    expect(empty).toBeTruthy();
    const action = screen.getByRole("link", { name: /建立存取金鑰/ });
    expect(action.getAttribute("href")).toBe("/account");
  });

  it("撤銷是列尾的明確動作，經 AlertDialog 確認後才送出", async () => {
    const client = makeAdminClient();
    renderAt("/admin/credentials", client);
    const table = await screen.findByRole("table");
    const row = within(table).getByRole("row", { name: /slk_abc/ });
    const user = userEvent.setup();
    await user.click(within(row).getByRole("button", { name: /撤銷/ }));
    const dialog = await screen.findByRole("alertdialog");
    expect(client.adminRevokeToken).not.toHaveBeenCalled();
    await user.click(within(dialog).getByRole("button", { name: /撤銷|確認/ }));
    await waitFor(() => expect(client.adminRevokeToken).toHaveBeenCalledWith("p1"));
  });

  it("窄螢幕以卡片列取代表格，撤銷動作仍在", async () => {
    setViewport(true);
    const client = makeAdminClient();
    renderAt("/admin/credentials", client);
    expect(await screen.findByText("slk_abc")).toBeTruthy();
    expect(screen.queryByRole("table")).toBeNull();
    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /撤銷存取金鑰 cli/ }));
    const dialog = await screen.findByRole("alertdialog");
    await user.click(within(dialog).getByRole("button", { name: /撤銷|確認/ }));
    await waitFor(() => expect(client.adminRevokeToken).toHaveBeenCalledWith("p1"));
  });

  it("裝置分頁列出裝置憑證並可撤銷", async () => {
    const client = makeAdminClient();
    renderAt("/admin/credentials", client);
    const user = userEvent.setup();
    await user.click(await screen.findByRole("tab", { name: /裝置/ }));
    expect(await screen.findByText("desktop")).toBeTruthy();
    await user.click(screen.getByRole("button", { name: /撤銷裝置/ }));
    const dialog = await screen.findByRole("alertdialog");
    await user.click(within(dialog).getByRole("button", { name: /撤銷|確認/ }));
    await waitFor(() => expect(client.adminRevokeFamily).toHaveBeenCalledWith("d1"));
  });
});

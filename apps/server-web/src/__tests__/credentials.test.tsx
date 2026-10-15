import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import {
  CREDENTIALS,
  EMPTY_CREDENTIALS,
  makeAdminClient,
  renderAt,
  setViewport,
} from "./helpers/adminHarness";

// 管理憑證頁（server-web-console「管理列表提供搜尋、篩選、分頁與具引導的空狀態」「共用設計系統
// 維持高密度可存取體驗」的撤銷鈕條款；server-web-list-pages design D2、D3、D6、D8）：列表卡頂端
// 以「存取金鑰｜裝置」兩個分頁區分（各帶總數），頁標題區提供關鍵字搜尋與狀態篩選；列不可點，
// 列尾是狀態籤與滑過才出現的撤銷鈕，撤銷經 AlertDialog 確認。空清單以說明用途的空狀態呈現，
// 建立入口常駐頁首。

beforeEach(() => setViewport(false));

const tabCount = (name: RegExp) =>
  screen.getByRole("tab", { name }).querySelector("[data-tab-count]")?.textContent;
const patRow = (prefix: string) => screen.getByText(prefix).closest("[data-pat]") as HTMLElement;

describe("憑證頁", () => {
  it("卡頂「存取金鑰」「裝置」兩個分頁帶總數，預設列出存取金鑰的 metadata", async () => {
    renderAt("/admin/credentials", makeAdminClient());
    expect(await screen.findByText("slk_abc")).toBeTruthy();
    expect(tabCount(/^存取金鑰/)).toBe("2");
    expect(tabCount(/^裝置/)).toBe("1");
  });

  it("存取金鑰列不可點：標題為名稱、說明為「前綴 · 使用者 · 建立 · 到期」", async () => {
    renderAt("/admin/credentials", makeAdminClient());
    await screen.findByText("slk_abc");
    const row = patRow("slk_abc");
    expect(row.getAttribute("role")).toBeNull();
    expect(within(row).getByText("cli")).toBeTruthy();
    expect(row.querySelector("[data-desc]")?.textContent).toBe(
      "slk_abc · 使用者 m1 · 建立 2026-01-01 · 到期 永久",
    );
  });

  it("有效列有滑過才出現的撤銷鈕（鍵盤聚焦時也出現），已撤銷列沒有", async () => {
    renderAt("/admin/credentials", makeAdminClient());
    await screen.findByText("slk_abc");
    const revoke = within(patRow("slk_abc")).getByRole("button", { name: "撤銷存取金鑰 cli" });
    const classes = revoke.className.split(/\s+/);
    // 平時隱藏、列滑過或鍵盤聚焦時出現；沒有滑鼠移入能力的裝置恆常顯示。
    expect(classes).toEqual(
      expect.arrayContaining([
        "opacity-0",
        "group-hover:opacity-100",
        "focus-visible:opacity-100",
        "[@media(hover:none)]:opacity-100",
      ]),
    );
    expect(classes).not.toContain("hidden");
    expect(within(patRow("slk_xyz")).queryByRole("button")).toBeNull();
  });

  it("憑證狀態籤：有效為綠系、已撤銷為中性，兩態可區辨", async () => {
    // spec「憑證狀態徽章」：撤銷是不可逆的事，得一眼看出哪把還活著。
    renderAt("/admin/credentials", makeAdminClient());
    await screen.findByText("slk_abc");
    const valid = within(patRow("slk_abc")).getByText("有效");
    const revoked = within(patRow("slk_xyz")).getByText("已撤銷");
    expect(valid.className).toContain("status-success");
    expect(revoked.className).not.toContain("status-success");
    expect(revoked.className).toContain("text-muted-foreground");
  });

  it("關鍵字搜尋只留下相符的存取金鑰", async () => {
    renderAt("/admin/credentials", makeAdminClient());
    expect(await screen.findByText("slk_abc")).toBeTruthy();
    const user = userEvent.setup();
    await user.type(screen.getByRole("textbox", { name: "搜尋" }), "部署");
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

  it("篩選無結果時呈現無符合狀態並保留篩選控制項、沒有換頁列", async () => {
    renderAt("/admin/credentials", makeAdminClient());
    expect(await screen.findByText("slk_abc")).toBeTruthy();
    const user = userEvent.setup();
    const search = screen.getByRole("textbox", { name: "搜尋" });
    await user.type(search, "找不到的東西");
    await waitFor(() => expect(screen.queryByText("slk_abc")).toBeNull());
    expect(screen.getByText(/沒有符合/)).toBeTruthy();
    // 篩選控制項仍在，使用者才能改回來。
    expect(screen.getByRole("textbox", { name: "搜尋" })).toBeTruthy();
    expect(screen.getByRole("combobox", { name: /狀態/ })).toBeTruthy();
    expect(screen.queryByText(/^第 .+ 筆，共 .+ 筆$/)).toBeNull();
  });

  it("空憑證頁說明用途並提供建立存取金鑰的 primary action，不出現分頁", async () => {
    renderAt(
      "/admin/credentials",
      makeAdminClient({ getAdminCredentials: vi.fn(async () => EMPTY_CREDENTIALS) }),
    );
    const empty = await screen.findByText(/遠端工作流程/);
    expect(empty).toBeTruthy();
    const action = screen.getByRole("link", { name: /建立存取金鑰/ });
    expect(action.getAttribute("href")).toBe("/account");
    expect(screen.queryByRole("tab")).toBeNull();
  });

  // design D2：該類資料完全沒有時是說明用途的空狀態，與「搜尋後沒有符合」可區辨。
  it("某一類原本就 0 筆時，該分頁呈現說明該類用途的空狀態，而不是無符合結果", async () => {
    renderAt(
      "/admin/credentials",
      makeAdminClient({
        getAdminCredentials: vi.fn(async () => ({ ...CREDENTIALS, pats: [] })),
      }),
    );
    expect(await screen.findByText("尚無存取金鑰")).toBeTruthy();
    expect(screen.queryByText(/沒有符合/)).toBeNull();
    const user = userEvent.setup();
    await user.click(screen.getByRole("tab", { name: /^裝置/ }));
    expect(await screen.findByText("desktop")).toBeTruthy();
  });

  it("沒有裝置登入時，裝置分頁呈現說明裝置憑證來源的空狀態", async () => {
    renderAt(
      "/admin/credentials",
      makeAdminClient({
        getAdminCredentials: vi.fn(async () => ({ ...CREDENTIALS, deviceFamilies: [] })),
      }),
    );
    await screen.findByText("slk_abc");
    const user = userEvent.setup();
    await user.click(screen.getByRole("tab", { name: /^裝置/ }));
    expect(await screen.findByText("尚無裝置登入")).toBeTruthy();
    expect(screen.queryByText(/沒有符合/)).toBeNull();
  });

  it("撤銷經 AlertDialog 確認後才送出", async () => {
    const client = makeAdminClient();
    renderAt("/admin/credentials", client);
    await screen.findByText("slk_abc");
    const user = userEvent.setup();
    await user.click(within(patRow("slk_abc")).getByRole("button", { name: "撤銷存取金鑰 cli" }));
    const dialog = await screen.findByRole("alertdialog");
    expect(client.adminRevokeToken).not.toHaveBeenCalled();
    await user.click(within(dialog).getByRole("button", { name: /撤銷|確認/ }));
    await waitFor(() => expect(client.adminRevokeToken).toHaveBeenCalledWith("p1"));
  });

  it("裝置分頁的列：標題為來源、說明為「使用者 · 建立 · 最近更新」，可撤銷", async () => {
    const client = makeAdminClient();
    renderAt("/admin/credentials", client);
    const user = userEvent.setup();
    await user.click(await screen.findByRole("tab", { name: /^裝置/ }));
    const row = (await screen.findByText("desktop")).closest("[data-device]") as HTMLElement;
    expect(row.getAttribute("role")).toBeNull();
    expect(row.querySelector("[data-desc]")?.textContent).toBe(
      "使用者 m1 · 建立 2026-01-01 · 最近更新 2026-01-03",
    );
    await user.click(within(row).getByRole("button", { name: /撤銷裝置/ }));
    const dialog = await screen.findByRole("alertdialog");
    await user.click(within(dialog).getByRole("button", { name: /撤銷|確認/ }));
    await waitFor(() => expect(client.adminRevokeFamily).toHaveBeenCalledWith("d1"));
  });

  it("窄螢幕：說明只留第一段，撤銷動作仍在", async () => {
    setViewport(true);
    const client = makeAdminClient();
    renderAt("/admin/credentials", client);
    expect(await screen.findByText("slk_abc")).toBeTruthy();
    expect(patRow("slk_abc").querySelector("[data-desc]")?.textContent).toBe("slk_abc");
    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "撤銷存取金鑰 cli" }));
    const dialog = await screen.findByRole("alertdialog");
    await user.click(within(dialog).getByRole("button", { name: /撤銷|確認/ }));
    await waitFor(() => expect(client.adminRevokeToken).toHaveBeenCalledWith("p1"));
    await user.click(screen.getByRole("tab", { name: /^裝置/ }));
    const device = (await screen.findByText("desktop")).closest("[data-device]") as HTMLElement;
    expect(device.querySelector("[data-desc]")?.textContent).toBe("使用者 m1");
  });
});

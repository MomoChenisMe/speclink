// 伺服器頁（desktop-settings-mode design D7；spec「伺服器管理最小面」）：頁標題右側「新增連線」
// 展開新增卡；連線卡一列一個 server（狀態籤、右側動作與「⋯」）；預設卡；登入方式卡。
import { describe, expect, it, vi } from "vitest";
import { fireEvent, render as rtlRender, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ComponentProps, ReactElement, ReactNode } from "react";
import { I18nProvider } from "@speclink/ui";

import { ServersPage } from "../views/settings/ServersPage";
import type { ConnectionView } from "../adapter/connections";
import { APP_MESSAGES } from "../i18n/messages";

vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({ writeText: vi.fn() }));

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);
const render = (ui: ReactElement) => rtlRender(ui, { wrapper: zhWrapper });

const A: ConnectionView = { id: "c1", origin: "https://a.example", name: "甲站", lastActorDisplay: "Momo", loggedIn: true };
const B: ConnectionView = { id: "c2", origin: "https://b.example", name: "乙站", lastActorDisplay: null, loggedIn: true };
const C: ConnectionView = { id: "c3", origin: "https://c.example", name: "丙站", lastActorDisplay: null, loggedIn: false };

type Props = ComponentProps<typeof ServersPage>;
function page(over: Partial<Props> = {}) {
  return (
    <ServersPage
      connections={[A, B]}
      phases={{}}
      onAdd={vi.fn().mockResolvedValue(undefined)}
      onLogin={vi.fn()}
      onSubmitPat={vi.fn()}
      onLogout={vi.fn()}
      onRemove={vi.fn()}
      onOpenWorkspace={vi.fn()}
      defaultConnectionId={null}
      onDefaultConnectionChange={vi.fn()}
      {...over}
    />
  );
}
const serverRow = (origin: string) => screen.getByTestId(`server-row-${origin}`);

describe("ServersPage", () => {
  it("頁標題「伺服器」；「新增連線」展開新增卡，送出成功後收起", async () => {
    const onAdd = vi.fn().mockResolvedValue(undefined);
    render(page({ onAdd }));
    expect(screen.getByRole("heading", { level: 2, name: "伺服器" })).toBeTruthy();
    expect(screen.queryByPlaceholderText("伺服器位址（http://…）")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "新增連線" }));
    const card = screen.getByRole("region", { name: "新增連線" });
    fireEvent.change(within(card).getByPlaceholderText("伺服器位址（http://…）"), { target: { value: "https://new.example" } });
    fireEvent.change(within(card).getByPlaceholderText("顯示名"), { target: { value: "新站" } });
    fireEvent.click(within(card).getByRole("button", { name: "新增" }));
    expect(onAdd).toHaveBeenCalledWith("https://new.example", "新站");
    await waitFor(() => expect(screen.queryByRole("region", { name: "新增連線" })).toBeNull());
  });

  it("新增卡的取消收起卡片", () => {
    render(page());
    fireEvent.click(screen.getByRole("button", { name: "新增連線" }));
    fireEvent.click(within(screen.getByRole("region", { name: "新增連線" })).getByRole("button", { name: "取消" }));
    expect(screen.queryByRole("region", { name: "新增連線" })).toBeNull();
  });

  it("連線卡：已登入列為「已登入」籤＋登出、登入過期列為警示籤＋重新登入；兩列皆有等寬網址與「⋯」", () => {
    render(page({ connections: [A, B], focusConnectionId: "c2" }));
    const rowA = serverRow("https://a.example");
    expect(within(rowA).getByText("已登入").className).toContain("bg-status-success/12");
    expect(within(rowA).getByText("https://a.example").className).toContain("font-mono");
    expect(within(rowA).getByText("Momo")).toBeTruthy();
    expect(within(rowA).getByRole("button", { name: "登出" })).toBeTruthy();
    expect(within(rowA).getByRole("button", { name: "更多動作" })).toBeTruthy();
    const rowB = serverRow("https://b.example");
    expect(within(rowB).getByText("登入過期").className).toContain("bg-status-warning/12");
    expect(within(rowB).getByRole("button", { name: "重新登入" })).toBeTruthy();
    expect(within(rowB).queryByRole("button", { name: "登出" })).toBeNull();
    expect(within(rowB).getByRole("button", { name: "更多動作" })).toBeTruthy();
  });

  it("連線失效（session 回報需重新登入）時不經導向也呈現登入過期＋重新登入，且不搶焦點", () => {
    render(page({ connections: [A, B], expiredConnectionIds: ["c2"] }));
    const rowB = serverRow("https://b.example");
    expect(within(rowB).getByText("登入過期")).toBeTruthy();
    const reauth = within(rowB).getByRole("button", { name: "重新登入" });
    expect(document.activeElement).not.toBe(reauth);
    expect(within(serverRow("https://a.example")).getByRole("button", { name: "登出" })).toBeTruthy();
  });

  it("未登入列為「未登入」籤＋登入", () => {
    render(page({ connections: [C] }));
    const row = serverRow("https://c.example");
    expect(within(row).getByText("未登入")).toBeTruthy();
    expect(within(row).getByRole("button", { name: "登入" })).toBeTruthy();
  });

  it("「⋯」選單含開啟專案與移除", async () => {
    const onOpenWorkspace = vi.fn();
    const onRemove = vi.fn();
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    render(page({ onOpenWorkspace, onRemove }));
    await user.click(within(serverRow("https://a.example")).getByRole("button", { name: "更多動作" }));
    expect(screen.getAllByRole("menuitem").map((i) => i.textContent)).toEqual(["開啟專案", "移除"]);
    await user.click(screen.getByRole("menuitem", { name: "開啟專案" }));
    expect(onOpenWorkspace).toHaveBeenCalledWith("c1");
    await user.click(within(serverRow("https://a.example")).getByRole("button", { name: "更多動作" }));
    await user.click(screen.getByRole("menuitem", { name: "移除" }));
    expect(onRemove).toHaveBeenCalledWith("c1");
  });

  it("無連線時顯示空狀態", () => {
    render(page({ connections: [] }));
    expect(screen.getByTestId("servers-empty")).toBeTruthy();
  });

  it("預設卡：下拉只列已登入連線與「不預選」，選取回呼", async () => {
    const onDefaultConnectionChange = vi.fn();
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    render(page({ connections: [A, C], defaultConnectionId: null, onDefaultConnectionChange }));
    const trigger = within(screen.getByRole("region", { name: "預設" })).getByRole("combobox", { name: "新增遠端專案時預設使用" });
    expect(trigger.textContent).toContain("不預選");
    await user.click(trigger);
    expect(screen.getAllByRole("option").map((o) => o.textContent)).toEqual(["不預選", "甲站"]);
    await user.click(screen.getByRole("option", { name: "甲站" }));
    expect(onDefaultConnectionChange).toHaveBeenCalledWith("c1");
  });

  it("預設指向已登出的連線時下拉顯示「不預選」", () => {
    render(page({ connections: [A, C], defaultConnectionId: "c3" }));
    const trigger = screen.getByRole("combobox", { name: "新增遠端專案時預設使用" });
    expect(trigger.textContent).toContain("不預選");
  });

  it("登入方式卡：裝置登入（預設）與存取金鑰兩列、無按鈕", () => {
    render(page());
    const card = screen.getByRole("region", { name: "登入方式" });
    expect(within(card).getAllByRole("group")).toHaveLength(2);
    expect(within(card).getByRole("group", { name: "裝置登入" }).textContent).toContain("預設");
    expect(within(card).getByRole("group", { name: "存取金鑰" })).toBeTruthy();
    expect(within(card).queryAllByRole("button")).toHaveLength(0);
  });
});

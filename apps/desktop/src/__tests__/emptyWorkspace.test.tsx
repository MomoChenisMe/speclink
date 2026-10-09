// 零專案空狀態（desktop-config「零分頁時顯示空狀態引導頁」、workspace-chooser「零專案空狀態
// 列出同一份最近開啟」，design D7）。
import { describe, expect, it, vi } from "vitest";
import { act, fireEvent, render as rtlRender, screen, waitFor } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider } from "@speclink/ui";

import { EmptyWorkspace, type EmptyWorkspaceProps } from "../components/EmptyWorkspace";
import { APP_MESSAGES } from "../i18n/messages";
import type { RecentEntry } from "../recents";

const zh = APP_MESSAGES["zh-TW"];
const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

const LOCAL: RecentEntry = { locator: { kind: "local", root: "/work/speclink" }, name: "speclink" };

async function renderEmpty(over: Partial<EmptyWorkspaceProps> = {}) {
  const props: EmptyWorkspaceProps = {
    recents: [],
    connections: [],
    onRefreshConnections: vi.fn().mockResolvedValue(true),
    workspace: { openProject: vi.fn().mockResolvedValue({ status: "project", root: "/work/speclink", name: "speclink" }) },
    connectionAdapter: { inspectCheckout: vi.fn() },
    onOpenLocal: vi.fn().mockResolvedValue(undefined),
    onOpenRemote: vi.fn().mockResolvedValue(undefined),
    onRemoveRecent: vi.fn(),
    onOpenChooser: vi.fn(),
    ...over,
  };
  render(<EmptyWorkspace {...props} />);
  await act(async () => {});
  return props;
}

describe("EmptyWorkspace", () => {
  it("字標、標題、含初始化提示的說明與底部 CLI 提示", async () => {
    await renderEmpty();
    const wordmark = screen.getByAltText("Speclink");
    expect(wordmark.className).toContain("h-10");
    const dark = wordmark.parentElement?.querySelector("source") as HTMLSourceElement;
    expect(dark.getAttribute("media")).toBe("(prefers-color-scheme: dark)");
    expect(screen.getByRole("heading", { level: 2, name: "開啟一個專案開始" })).toBeTruthy();
    expect(screen.getByText(zh["app.emptyDesc"]).textContent).toContain("openspec/");
    expect(screen.getByText(zh["app.emptyCli"])).toBeTruthy();
  });

  it("「新增專案」開對話框；「連線 Server」帶 initialStep: \"server\"", async () => {
    const { onOpenChooser } = await renderEmpty();
    fireEvent.click(screen.getByRole("button", { name: zh["app.newProject"] }));
    expect(onOpenChooser).toHaveBeenLastCalledWith();
    fireEvent.click(screen.getByRole("button", { name: zh["app.connectServer"] }));
    expect(onOpenChooser).toHaveBeenLastCalledWith({ initialStep: "server" });
    expect(onOpenChooser).toHaveBeenCalledTimes(2);
  });

  it("有最近開啟記錄時顯示卡片；點本機條目走與對話框相同的探測與開啟", async () => {
    const props = await renderEmpty({ recents: [LOCAL] });
    expect(document.querySelector('[data-recent-list="card"]')).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: /\/work\/speclink/ }));
    await waitFor(() => expect(props.onOpenLocal).toHaveBeenCalledWith("/work/speclink"));
    expect(props.workspace.openProject).toHaveBeenCalledWith("/work/speclink");
    fireEvent.click(screen.getByRole("button", { name: "自最近開啟移除 speclink" }));
    expect(props.onRemoveRecent).toHaveBeenCalledWith("local:/work/speclink");
  });

  it("remote 條目開啟失敗後可再點重試：重試開始即清掉錯誤態，成功就開啟（伺服器暫時離線的情境）", async () => {
    const REMOTE: RecentEntry = {
      locator: { kind: "remote", connectionId: "conn_1", projectId: "prj_1", repoId: "repo_1" },
      name: "Demo/api",
    };
    const onOpenRemote = vi
      .fn()
      .mockRejectedValueOnce(new Error("server unreachable"))
      .mockResolvedValueOnce(undefined);
    await renderEmpty({
      recents: [REMOTE],
      connections: [{ id: "conn_1", name: "團隊 Server", origin: "https://s", loggedIn: true } as never],
      onOpenRemote,
    });
    fireEvent.click(screen.getByRole("button", { name: /^Demo\/api/ }));
    await waitFor(() => expect(screen.getByText(/server unreachable/)).toBeTruthy());
    const row = screen.getByRole("button", { name: /^Demo\/api/ }) as HTMLButtonElement;
    expect(row.disabled).toBe(false);
    fireEvent.click(row);
    await waitFor(() => expect(onOpenRemote).toHaveBeenCalledTimes(2));
    await waitFor(() => expect(screen.queryByText(/server unreachable/)).toBeNull());
  });

  it("掛載時讀一次連線清單", async () => {
    const props = await renderEmpty();
    expect(props.onRefreshConnections).toHaveBeenCalledTimes(1);
  });

  it("無記錄時不渲染最近開啟區段", async () => {
    await renderEmpty();
    expect(document.querySelector("[data-recent-list]")).toBeNull();
    expect(screen.queryByText("最近開啟")).toBeNull();
  });
});

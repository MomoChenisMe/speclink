// 最近開啟清單元件（workspace-chooser「最近開啟清單」，design D7）：對話框第一步與
// 零專案空狀態共用同一份；兩個 variant 只差外框。
import { describe, expect, it, vi } from "vitest";
import { fireEvent, render as rtlRender, screen } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider } from "@speclink/ui";

import { RecentList, type RecentListProps } from "../components/RecentList";
import type { ConnectionView } from "../adapter/connections";
import { APP_MESSAGES } from "../i18n/messages";
import type { RecentEntry } from "../recents";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

const CONNECTION: ConnectionView = {
  id: "conn_1",
  origin: "https://spec.example.test",
  name: "團隊 Server",
  loggedIn: true,
};
const LOCAL: RecentEntry = { locator: { kind: "local", root: "/work/speclink" }, name: "speclink" };
const REMOTE: RecentEntry = {
  locator: { kind: "remote", connectionId: "conn_1", projectId: "prj_1", repoId: "repo_1" },
  name: "Speclink/Desktop",
};

function renderList(over: Partial<RecentListProps> = {}) {
  const props: RecentListProps = {
    variant: "dialog",
    entries: [LOCAL, REMOTE],
    connections: [CONNECTION],
    connectionsReady: true,
    errors: {},
    onOpen: vi.fn(),
    onRemove: vi.fn(),
    ...over,
  };
  render(<RecentList {...props} />);
  return props;
}

describe("RecentList", () => {
  it("dialog：小標「最近開啟」與計數，列放在細框卡內", () => {
    renderList();
    const section = document.querySelector('[data-recent-list="dialog"]') as HTMLElement;
    expect(section.textContent).toContain("最近開啟");
    expect(section.textContent).toContain("2");
    const box = section.querySelector("[data-recent-rows]") as HTMLElement;
    expect(box.className).toContain("rounded-xl");
    expect(box.className).toContain("border");
  });

  it("card：整段包在卡片底色內（空狀態用）", () => {
    renderList({ variant: "card" });
    const section = document.querySelector('[data-recent-list="card"]') as HTMLElement;
    expect(section.className).toContain("bg-card");
    expect(section.className).toContain("rounded-xl");
  });

  it("本機列顯示名稱與路徑；remote 列顯示顯示名、連線名稱與「遠端」籤", () => {
    renderList();
    const local = screen.getByRole("button", { name: /\/work\/speclink/ });
    expect(local.textContent).toContain("speclink");
    expect(local.textContent).not.toContain("遠端");
    const remote = screen.getByRole("button", { name: /團隊 Server/ });
    expect(remote.textContent).toContain("Speclink/Desktop");
    expect(remote.textContent).toContain("遠端");
  });

  it("點列以該條目呼叫 onOpen", () => {
    const { onOpen } = renderList();
    fireEvent.click(screen.getByRole("button", { name: /\/work\/speclink/ }));
    expect(onOpen).toHaveBeenCalledWith(LOCAL);
  });

  it("開啟失敗的錯誤態：顯示原因、仍可再點重試，× 仍以該條目呼叫 onRemove", () => {
    const { onOpen, onRemove } = renderList({ errors: { "local:/work/speclink": "目錄不存在" } });
    const row = screen.getByRole("button", { name: /目錄不存在/ }) as HTMLButtonElement;
    expect(row.disabled).toBe(false);
    fireEvent.click(row);
    expect(onOpen).toHaveBeenCalledWith(LOCAL);
    fireEvent.click(screen.getByRole("button", { name: "自最近開啟移除 speclink" }));
    expect(onRemove).toHaveBeenCalledWith(LOCAL);
  });

  it("連線清單就緒且連線已移除時 remote 列為錯誤態；未就緒時不判定", () => {
    const { unmount } = rtlRender(
      <RecentList
        variant="dialog"
        entries={[REMOTE]}
        connections={[]}
        connectionsReady
        errors={{}}
        onOpen={vi.fn()}
        onRemove={vi.fn()}
      />,
      { wrapper: zhWrapper },
    );
    expect(screen.getByText("連線已移除")).toBeTruthy();
    unmount();
    renderList({ entries: [REMOTE], connections: [], connectionsReady: false });
    expect(screen.queryByText("連線已移除")).toBeNull();
  });

  it("無條目時不渲染任何區段", () => {
    renderList({ entries: [] });
    expect(document.querySelector("[data-recent-list]")).toBeNull();
    expect(screen.queryByText("最近開啟")).toBeNull();
  });
});

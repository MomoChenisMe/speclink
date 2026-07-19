// capability 驅動停用（remote-data-source 規格「capability 驅動停用且不偽造
// 缺口」；design 決策 2）：remote 分頁停用搜尋/拖排/validate/analyze/刪除附
// 繁中說明、archived 頁呈現提示卡；本地分頁全功能不變（迴歸斷言）。
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";

import { App } from "../App";
import { createLocalSession, type WorkspaceSession } from "../session";
import type { SpeclinkDataSource } from "@speclink/ui";
import { fakeRemoteDs, fakeRemoteSession, REMOTE_KEY } from "./helpers/remoteFixtures";

// Tauri 事件層 mock（App 的 session 事件訂閱走它）。
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

// sonner Toaster 需要 jsdom 沒有的 matchMedia——以 marker 取代（比照 App.test）。
vi.mock("@speclink/ui", async (importOriginal) => {
  const mod = await importOriginal<typeof import("@speclink/ui")>();
  return { ...mod, Toaster: () => <div data-testid="app-toaster" /> };
});

beforeEach(() => {
  localStorage.removeItem("speclink.projectTabs");
  localStorage.setItem("speclink.uiLocale", "zh-TW");
});

function fakeWorkspace() {
  return {
    openProject: vi.fn().mockRejectedValue("not a project"),
    initProject: vi.fn(),
    startupDir: vi.fn().mockRejectedValue("no startup dir"),
    projectStats: vi.fn().mockResolvedValue({ pendingWrapUp: 0 }),
    watchWorkspace: vi.fn().mockResolvedValue(undefined),
    pickFolder: vi.fn().mockResolvedValue(null),
  };
}

/** 以持久化 remote 分頁啟動 App：restoreTabs → activateTab → openRemote 重驗。 */
function renderRemoteApp(ds: SpeclinkDataSource) {
  localStorage.setItem(
    "speclink.projectTabs",
    JSON.stringify({
      version: 2,
      tabs: [
        {
          locator: { kind: "remote", connectionId: "c1", projectId: "demo", repoId: "backend" },
          name: "Demo/backend",
        },
      ],
      activeKey: REMOTE_KEY,
    }),
  );
  const openRemote = vi.fn(async (): Promise<WorkspaceSession> => fakeRemoteSession(ds));
  render(
    <App
      createSession={() => {
        throw new Error("remote 流程不應觸發 local 工廠");
      }}
      openRemote={openRemote}
      workspace={fakeWorkspace() as never}
    />,
  );
  return { openRemote };
}

/** 本地對照組：持久化 local 分頁＋全真 capability 的 local session。 */
function renderLocalApp(ds: SpeclinkDataSource) {
  localStorage.setItem(
    "speclink.projectTabs",
    JSON.stringify({
      version: 2,
      tabs: [{ locator: { kind: "local", root: "A" }, name: "proj-a" }],
      activeKey: "local:A",
    }),
  );
  const ws = fakeWorkspace();
  ws.openProject = vi.fn().mockResolvedValue({ status: "project", root: "A", name: "proj-a" });
  const invoke = (async () => null) as never;
  render(
    <App
      createSession={(root, name) => {
        const session = createLocalSession(root, { name, invoke });
        return { ...session, dataSource: ds };
      }}
      workspace={ws as never}
    />,
  );
}

describe("remote 分頁的 capability 停用", () => {
  it("搜尋輸入停用附繁中說明、看板照常呈現 server 資料", async () => {
    renderRemoteApp(fakeRemoteDs());
    await waitFor(() => expect(screen.getByText("remote-change")).toBeTruthy());
    const input = screen.getByPlaceholderText("搜尋看板卡片…") as HTMLInputElement;
    expect(input.disabled).toBe(true);
    expect(input.title).toContain("全文搜尋");
  });

  it("archived 頁呈現尚未提供提示卡、不打 listArchived", async () => {
    const ds = fakeRemoteDs();
    renderRemoteApp(ds);
    await waitFor(() => expect(screen.getByText("remote-change")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: "已封存" }));
    await waitFor(() => expect(screen.getByTestId("archived-unavailable")).toBeTruthy());
    expect(screen.getByText("此 server 尚未提供封存瀏覽")).toBeTruthy();
    expect(ds.listArchived).not.toHaveBeenCalled();
  });

  it("詳情抽屜的分析與刪除停用附繁中說明、封存照常可用", async () => {
    renderRemoteApp(fakeRemoteDs());
    await waitFor(() => expect(screen.getByText("remote-change")).toBeTruthy());
    fireEvent.click(screen.getByText("remote-change"));
    const analyze = (await screen.findByRole("button", { name: /分析/ })) as HTMLButtonElement;
    expect(analyze.disabled).toBe(true);
    expect(analyze.title).toContain("validate/analyze");
    const del = screen.getByRole("button", { name: /刪除/ }) as HTMLButtonElement;
    expect(del.disabled).toBe(true);
    expect(del.title).toContain("刪除變更");
    const archive = screen.getByRole("button", { name: /封存/ }) as HTMLButtonElement;
    expect(archive.disabled).toBe(false);
  });
});

describe("本地分頁不受影響（迴歸）", () => {
  it("搜尋照常可用、分析與刪除照常可點", async () => {
    const ds = fakeRemoteDs({
      // 本地全功能：改用可解析的本地行為樣本。
      listChanges: vi.fn().mockResolvedValue([
        { name: "local-change", status: "in-progress", totalTasks: 2, completedTasks: 0 },
      ]),
      listArchived: vi.fn().mockResolvedValue([]),
      changeCapabilities: vi.fn().mockResolvedValue([]),
      changeMeta: vi.fn().mockResolvedValue(null),
    });
    renderLocalApp(ds);
    await waitFor(() => expect(screen.getByText("local-change")).toBeTruthy());
    const input = screen.getByPlaceholderText("搜尋看板卡片…") as HTMLInputElement;
    expect(input.disabled).toBe(false);

    fireEvent.click(screen.getByText("local-change"));
    const analyze = (await screen.findByRole("button", { name: /分析/ })) as HTMLButtonElement;
    expect(analyze.disabled).toBe(false);
    const del = screen.getByRole("button", { name: /刪除/ }) as HTMLButtonElement;
    expect(del.disabled).toBe(false);
    // 本地照常打 listArchived（capability 全真）。
    expect(ds.listArchived).toHaveBeenCalled();
  });
});

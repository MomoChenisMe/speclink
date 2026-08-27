// capability 驅動停用（remote-data-source 規格「capability 驅動停用且不偽造
// 缺口」；design 決策 2）：remote 分頁的封存／搜尋／spec 內文直達，仍停用
// 拖排/validate/analyze/刪除附繁中說明；本地分頁全功能不變（迴歸斷言）。
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent, within } from "@testing-library/react";

import { App } from "../App";
import {
  applyRemoteConnectionState,
  createLocalSession,
  type WorkspaceSession,
} from "../session";
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
function renderRemoteApp(
  ds: SpeclinkDataSource,
  capsOver: Partial<import("../session").WorkspaceCapabilities> = {},
) {
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
  const openRemote = vi.fn(async (): Promise<WorkspaceSession> => {
    const session = fakeRemoteSession(ds);
    return { ...session, capabilities: { ...session.capabilities, ...capsOver } };
  });
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
  it("reader 看板不渲染拖排把手並顯示繁中角色說明", async () => {
    renderRemoteApp(fakeRemoteDs());
    await waitFor(() => expect(screen.getByText("remote-change")).toBeTruthy());
    expect(document.querySelector('[aria-roledescription="sortable"]')).toBeNull();
    expect(screen.getByText(/唯讀狀態.*無法拖曳調整卡片順序/)).toBeTruthy();
  });

  it("搜尋輸入啟用並直達 server、看板照常呈現資料", async () => {
    const ds = fakeRemoteDs();
    renderRemoteApp(ds);
    await waitFor(() => expect(screen.getByText("remote-change")).toBeTruthy());
    const input = screen.getByPlaceholderText("搜尋看板卡片…") as HTMLInputElement;
    expect(input.disabled).toBe(false);
    fireEvent.change(input, { target: { value: "needle" } });
    await waitFor(() => expect(ds.searchWorkspace).toHaveBeenCalledWith("needle"));
  });

  it("archived 頁呈現 server 清單、不再顯示缺口提示卡", async () => {
    const ds = fakeRemoteDs();
    renderRemoteApp(ds);
    await waitFor(() => expect(screen.getByText("remote-change")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: "已封存" }));
    await waitFor(() => expect(screen.getByText("remote-old")).toBeTruthy());
    expect(screen.queryByTestId("archived-unavailable")).toBeNull();
    expect(ds.listArchived).toHaveBeenCalled();
  });

  it("規格卡直達 server 正典內文、不再以提示文字代替", async () => {
    const ds = fakeRemoteDs();
    renderRemoteApp(ds);
    await waitFor(() => expect(screen.getByText("remote-change")).toBeTruthy());
    const aside = document.querySelector("aside") as HTMLElement;
    fireEvent.click(within(aside).getByRole("button", { name: "規格" }));
    await waitFor(() => expect(screen.getByText("auth")).toBeTruthy());
    fireEvent.click(screen.getByText("auth"));
    await waitFor(() => expect(screen.getByText("Remote canonical truth.")).toBeTruthy());
    expect(ds.getSpecDocument).toHaveBeenCalledWith("auth");
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

  it("詳情抽屜的詮釋資料與 capability 清單直達、不再呈現停用說明", async () => {
    // remote-read-parity「詮釋資料與 capability 清單直達且誠實降級」：兩讀取
    // 走 dataSource 方法（status payload 映射），建立者列與本地同形呈現。
    const ds = fakeRemoteDs();
    renderRemoteApp(ds);
    await waitFor(() => expect(screen.getByText("remote-change")).toBeTruthy());
    fireEvent.click(screen.getByText("remote-change"));
    await waitFor(() => expect(ds.changeMeta).toHaveBeenCalledWith("remote-change"));
    expect(ds.changeCapabilities).toHaveBeenCalledWith("remote-change");
    await waitFor(() => expect(screen.getByText("Remote Creator")).toBeTruthy());
    expect(screen.queryByText(/尚未提供「capability 清單」/)).toBeNull();
    expect(screen.queryByText(/尚未提供「change 詮釋資料」/)).toBeNull();
  });

  it("舊 server 缺歸屬欄位時對應列缺席且無錯誤", async () => {
    // 誠實降級：舊 server 的 status payload 無新欄位——changeMeta 組出的欄位
    // 為 null／缺席，UI 對應列不顯示、抽屜照常呈現，不偽造任何值。
    const ds = fakeRemoteDs({
      changeMeta: vi.fn().mockResolvedValue({
        schema: "spec-driven",
        created: null,
        createdBy: null,
        createdWith: null,
        fromDiscussions: [],
        startedAt: null,
        startedBy: null,
      }),
      changeCapabilities: vi.fn().mockResolvedValue([]),
    });
    renderRemoteApp(ds);
    await waitFor(() => expect(screen.getByText("remote-change")).toBeTruthy());
    fireEvent.click(screen.getByText("remote-change"));
    await waitFor(() => expect(ds.changeMeta).toHaveBeenCalledWith("remote-change"));
    // 抽屜正常開啟（封存鈕在）且建立者列缺席。
    await screen.findByRole("button", { name: "封存" });
    expect(screen.queryByText("Remote Creator")).toBeNull();
  });

  it("remote 封存沿用確認路徑、描述指出 Project/Repo scope，取消不寫入而確認會寫 server", async () => {
    const ds = fakeRemoteDs();
    renderRemoteApp(ds);
    await screen.findByText("remote-change");
    fireEvent.click(screen.getByText("remote-change"));
    fireEvent.click(await screen.findByRole("button", { name: "封存" }));

    let dialog = await screen.findByRole("alertdialog");
    expect(within(dialog).getByText(/server 上的 scope：Demo\/backend/)).toBeTruthy();
    fireEvent.click(within(dialog).getByRole("button", { name: "取消" }));
    expect(ds.runVerb).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("button", { name: "封存" }));
    dialog = await screen.findByRole("alertdialog");
    fireEvent.click(within(dialog).getByRole("button", { name: "封存" }));
    await waitFor(() => expect(ds.runVerb).toHaveBeenCalledWith("archive", "remote-change"));
  });

  it("offline mask 同時維持 deleteChange 停用並關閉 archive", () => {
    const session = fakeRemoteSession(fakeRemoteDs());
    const offline = applyRemoteConnectionState(
      {
        ...session,
        baseCapabilities: session.capabilities,
        connectionState: { connectionId: "c1", state: "online", message: null },
      },
      { connectionId: "c1", state: "offline", message: "offline" },
    );
    expect(offline.capabilities.deleteChange).toBe(false);
    expect(offline.capabilities.archive).toBe(false);
  });
});

describe("認領面依 capability 與模式呈現（remote-claim-ownership）", () => {
  it("editor 的 remote 抽屜提供可點的認領操作", async () => {
    const claim = vi.fn().mockResolvedValue(undefined);
    renderRemoteApp(fakeRemoteDs({ claim } as never));
    await waitFor(() => expect(screen.getByText("remote-change")).toBeTruthy());
    fireEvent.click(screen.getByText("remote-change"));
    const button = (await screen.findByRole("button", { name: /認領/ })) as HTMLButtonElement;
    expect(button.disabled).toBe(false);
    fireEvent.click(button);
    await waitFor(() => expect(claim).toHaveBeenCalledWith("remote-change"));
  });

  it("reader 的認領操作停用並附繁中角色說明", async () => {
    const claim = vi.fn();
    renderRemoteApp(
      fakeRemoteDs({ claim } as never),
      { claim: false },
    );
    await waitFor(() => expect(screen.getByText("remote-change")).toBeTruthy());
    fireEvent.click(screen.getByText("remote-change"));
    const button = (await screen.findByRole("button", { name: /認領/ })) as HTMLButtonElement;
    expect(button.disabled).toBe(true);
    expect(button.title).toMatch(/唯讀/);
    fireEvent.click(button);
    expect(claim).not.toHaveBeenCalled();
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
      // 本地後端沒有 claim（RemoteOnly）——鏡射真實 tauriDataSource 的形狀。
      claim: undefined,
    } as never);
    renderLocalApp(ds);
    await waitFor(() => expect(screen.getByText("local-change")).toBeTruthy());
    const input = screen.getByPlaceholderText("搜尋看板卡片…") as HTMLInputElement;
    expect(input.disabled).toBe(false);

    fireEvent.click(screen.getByText("local-change"));
    const analyze = (await screen.findByRole("button", { name: /分析/ })) as HTMLButtonElement;
    expect(analyze.disabled).toBe(false);
    const del = screen.getByRole("button", { name: /刪除/ }) as HTMLButtonElement;
    expect(del.disabled).toBe(false);
    // claim 是 RemoteOnly 動詞——本地分頁連停用的入口都不長出來。
    expect(screen.queryByRole("button", { name: /認領/ })).toBeNull();
    // 本地照常打 listArchived（capability 全真）。
    expect(ds.listArchived).toHaveBeenCalled();
  });
});

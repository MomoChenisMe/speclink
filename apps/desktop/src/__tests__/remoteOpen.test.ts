// remote workspace 開啟與恢復（remote-data-source design 決策 6；規格
// 「handshake 成功後才建立 remote session」）：openRemoteWorkspace 成功開分頁
// 設 active、失敗上拋不留分頁；重啟恢復（activateTab 無 session）重走
// handshake、失敗轉分頁錯誤態不消失；remote refresh 依 capability 跳過
// 不支援讀取（listArchived）。
import { describe, it, expect, vi } from "vitest";

import { createAppStore } from "../store";
import type { WorkspaceAdapter } from "../adapter/workspace";
import { locatorKey, type WorkspaceSession } from "../session";
import { fakeRemoteDs, fakeRemoteSession, REMOTE_KEY as KEY } from "./helpers/remoteFixtures";

function fakeWorkspace(): WorkspaceAdapter {
  return {
    openProject: vi.fn(),
    initProject: vi.fn(),
    pickFolder: vi.fn(),
    projectStats: vi.fn(),
    startupDir: vi.fn(),
    watchWorkspace: vi.fn(),
  } as unknown as WorkspaceAdapter;
}

function storeWith(openRemote: (connectionId: string, target: string) => Promise<WorkspaceSession>) {
  return createAppStore({
    createSession: () => {
      throw new Error("local 工廠不應被 remote 流程觸發");
    },
    workspace: fakeWorkspace(),
    openRemote,
  });
}

describe("openRemoteWorkspace（決策 6：handshake fail-closed）", () => {
  it("success opens a tab, activates it, and refreshes through the remote dataSource", async () => {
    const ds = fakeRemoteDs();
    const session = fakeRemoteSession(ds);
    const openRemote = vi.fn().mockResolvedValue(session);
    const store = storeWith(openRemote);

    await store.getState().openRemoteWorkspace("c1", "demo/backend");

    expect(openRemote).toHaveBeenCalledWith("c1", "demo/backend");
    const st = store.getState();
    expect(st.activeKey).toBe(KEY);
    expect(st.tabs.map((t) => locatorKey(t.locator))).toEqual([KEY]);
    expect(st.sessions[KEY]).toBe(session);
    expect(st.boardView).toBe("board");
    expect(st.loaded).toBe(true);
    expect(ds.listChanges).toHaveBeenCalled();
  });

  it("failure rethrows for the form and leaves no tab or session behind", async () => {
    const openRemote = vi.fn().mockRejectedValue(new Error("access denied — no access"));
    const store = storeWith(openRemote);

    await expect(store.getState().openRemoteWorkspace("c1", "demo")).rejects.toThrow(
      "access denied",
    );
    const st = store.getState();
    expect(st.tabs).toHaveLength(0);
    expect(st.activeKey).toBeNull();
    expect(Object.keys(st.sessions)).toHaveLength(0);
  });
});

describe("remote refresh（規格「capability 驅動停用且不偽造缺口」）", () => {
  it("skips listArchived when the capability is off instead of failing the batch", async () => {
    const ds = fakeRemoteDs();
    const openRemote = vi.fn().mockResolvedValue(fakeRemoteSession(ds));
    const store = storeWith(openRemote);
    await store.getState().openRemoteWorkspace("c1", "demo/backend");

    expect(ds.listArchived).not.toHaveBeenCalled();
    expect(store.getState().archived).toEqual([]);
    expect(store.getState().loaded).toBe(true);
  });
});

describe("重啟恢復（規格「重啟後 remote 分頁恢復需重驗」）", () => {
  it("activating a restored remote tab re-runs the handshake and adopts the session", async () => {
    const ds = fakeRemoteDs();
    const session = fakeRemoteSession(ds);
    const openRemote = vi.fn().mockResolvedValue(session);
    const store = storeWith(openRemote);
    store.setState({
      tabs: [
        {
          locator: { kind: "remote", connectionId: "c1", projectId: "demo", repoId: "backend" },
          name: "Demo/backend",
          badge: null,
        },
      ],
    });

    await store.getState().activateTab(KEY);

    expect(openRemote).toHaveBeenCalledWith("c1", "demo/backend");
    expect(store.getState().activeKey).toBe(KEY);
    expect(store.getState().sessions[KEY]).toBe(session);
  });

  it("a failed re-handshake shows an error on the tab without removing it", async () => {
    const openRemote = vi.fn().mockRejectedValue(new Error("登入已失效——請重新登入"));
    const store = storeWith(openRemote);
    store.setState({
      tabs: [
        {
          locator: { kind: "remote", connectionId: "c1", projectId: "demo", repoId: "backend" },
          name: "Demo/backend",
          badge: null,
        },
      ],
    });

    await store.getState().activateTab(KEY);

    const st = store.getState();
    expect(st.tabErrors[KEY]).toContain("重新登入");
    expect(st.tabs).toHaveLength(1);
    expect(st.activeKey).toBeNull();
  });

  it("an existing remote session activates without a second handshake", async () => {
    const ds = fakeRemoteDs();
    const session = fakeRemoteSession(ds);
    const openRemote = vi.fn().mockResolvedValue(session);
    const store = storeWith(openRemote);
    await store.getState().openRemoteWorkspace("c1", "demo/backend");
    openRemote.mockClear();
    vi.mocked(ds.listChanges as ReturnType<typeof vi.fn>).mockClear();

    await store.getState().activateTab(KEY);

    expect(openRemote).not.toHaveBeenCalled();
    expect(store.getState().activeKey).toBe(KEY);
    expect(ds.listChanges).toHaveBeenCalled();
  });
});

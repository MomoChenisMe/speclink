// remote workspace 開啟與恢復（remote-data-source design 決策 6；規格
// 「handshake 成功後才建立 remote session」）：openRemoteWorkspace 成功開分頁
// 設 active、失敗上拋不留分頁；重啟恢復（activateTab 無 session）重走
// handshake、失敗轉分頁錯誤態不消失；remote refresh 依 capability 直接讀取
// server truth（listArchived）。
import { describe, it, expect, vi } from "vitest";

import { createAppStore } from "../store";
import type { ConnectionsAdapter, ConnectionView } from "../adapter/connections";
import type { WorkspaceAdapter } from "../adapter/workspace";
import type { MigrationAdapter } from "../adapter/migration";
import {
  LOCAL_CAPABILITIES,
  locatorKey,
  normalizeRemoteOpenFailure,
  type WorkspaceSession,
} from "../session";
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

function fakeConnections(
  entries: ConnectionView[],
  over: Partial<ConnectionsAdapter> = {},
): ConnectionsAdapter {
  return {
    list: vi.fn().mockResolvedValue(entries),
    add: vi.fn(),
    remove: vi.fn(),
    deviceLogin: vi.fn(),
    patLogin: vi.fn(),
    logout: vi.fn(),
    scopes: vi.fn(),
    // 既有 marker 的預設：帶一個 built-in 選集，走自動 reconciliation 路徑。
    inspectCheckout: vi
      .fn()
      .mockImplementation(async (path: string) => ({ root: path, tools: ["claude"] })),
    bindCheckout: vi.fn().mockImplementation(async (path: string) => path),
    ...over,
  } as unknown as ConnectionsAdapter;
}

function localSession(root: string, name: string): WorkspaceSession {
  return {
    id: `local:${root}`,
    locator: { kind: "local", root },
    descriptor: { name, badge: null },
    dataSource: fakeRemoteDs(),
    settings: {
      kind: "local",
      policyWrite: true,
      readSettings: vi.fn(),
      writeAppTools: vi.fn(),
      writeWorkflowConfig: vi.fn(),
      writeWorkflowContext: vi.fn(),
      writeWorkflowRules: vi.fn(),
    },
    events: { subscribe: () => () => {} },
    capabilities: LOCAL_CAPABILITIES,
  };
}

function storeWith(
  openRemote: (
    connectionId: string,
    target: string,
    checkoutRoot?: string,
  ) => Promise<WorkspaceSession>,
  options: {
    workspace?: WorkspaceAdapter;
    connections?: ConnectionsAdapter;
    migration?: MigrationAdapter;
  } = {},
) {
  return createAppStore({
    createSession: () => {
      throw new Error("local 工廠不應被 remote 流程觸發");
    },
    workspace: options.workspace ?? fakeWorkspace(),
    connections: options.connections,
    migration: options.migration,
    openRemote,
  });
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
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

describe("remote_open structured failure normalization", () => {
  it.each([
    [{ message: "auth", reason: "permission_denied", status: 401 }, "needs-reauth"],
    [{ message: "denied", reason: "permission_denied", status: 403 }, "access-denied"],
    [{ message: "missing", reason: "not_found", status: 404 }, "not-found"],
    [{ message: "offline", reason: null, status: null }, "unreachable"],
  ] as const)("maps %o to %s without inspecting message text", (input, expected) => {
    expect(normalizeRemoteOpenFailure(input)).toEqual({
      kind: expected,
      message: input.message,
      reason: input.reason,
      status: input.status,
    });
  });

  it("fails safely for legacy strings and unknown objects", () => {
    expect(normalizeRemoteOpenFailure("legacy rejection")).toEqual({
      kind: "unknown",
      message: "legacy rejection",
      reason: null,
      status: null,
    });
    expect(normalizeRemoteOpenFailure({ future: true })).toEqual({
      kind: "unknown",
      message: "[object Object]",
      reason: null,
      status: null,
    });
  });
});

describe("local migration 分頁原地轉換", () => {
  it("chooser 入口先建立 local 分頁，再開 MigrationDialog", async () => {
    const workspace = fakeWorkspace();
    vi.mocked(workspace.openProject).mockResolvedValue({
      status: "project",
      root: "/work/local",
      name: "Local",
    });
    const store = createAppStore({
      createSession: localSession,
      workspace,
    });

    await store.getState().requestMigration("/work/local");

    expect(store.getState().migrationRoot).toBe("/work/local");
    expect(store.getState().activeKey).toBe("local:/work/local");
    expect(store.getState().tabs.map((tab) => locatorKey(tab.locator))).toEqual([
      "local:/work/local",
    ]);
  });

  it("handshake 成功後以 checkout remote 分頁取代原 local 分頁", async () => {
    const workspace = fakeWorkspace();
    vi.mocked(workspace.openProject).mockResolvedValue({
      status: "project",
      root: "/work/local",
      name: "Local",
    });
    const session = fakeRemoteSession(fakeRemoteDs());
    session.locator = { ...session.locator, checkoutRoot: "/work/local" };
    const openRemote = vi.fn().mockResolvedValue(session);
    const store = createAppStore({
      createSession: localSession,
      workspace,
      openRemote,
    });
    await store.getState().openProjectAt("/work/local");

    await store
      .getState()
      .replaceLocalWorkspaceWithRemote("/work/local", "c1", "demo/backend");

    expect(openRemote).toHaveBeenCalledWith("c1", "demo/backend", "/work/local");
    const state = store.getState();
    expect(state.tabs.map((tab) => locatorKey(tab.locator))).toEqual([KEY]);
    expect(state.sessions["local:/work/local"]).toBeUndefined();
    expect(state.sessions[KEY]).toBe(session);
    expect(state.activeKey).toBe(KEY);
  });

  it("handshake 失敗時保留原 local 分頁與 session", async () => {
    const workspace = fakeWorkspace();
    vi.mocked(workspace.openProject).mockResolvedValue({
      status: "project",
      root: "/work/local",
      name: "Local",
    });
    const openRemote = vi.fn().mockRejectedValue(new Error("server temporarily unavailable"));
    const store = createAppStore({
      createSession: localSession,
      workspace,
      openRemote,
    });
    await store.getState().openProjectAt("/work/local");

    await expect(
      store
        .getState()
        .replaceLocalWorkspaceWithRemote("/work/local", "c1", "demo/backend"),
    ).rejects.toThrow("temporarily unavailable");

    const state = store.getState();
    expect(state.tabs.map((tab) => locatorKey(tab.locator))).toEqual(["local:/work/local"]);
    expect(state.sessions["local:/work/local"]).toBeTruthy();
    expect(state.activeKey).toBe("local:/work/local");
  });
});

describe("remote marker probe 分流", () => {
  const markerUrl = "https://spec.example.test/api/speclink/v1/projects/demo";
  const loggedIn: ConnectionView = {
    id: "c1",
    origin: "https://spec.example.test",
    name: "Team",
    loggedIn: true,
  };

  it("僅 remote marker、已登入且有工具選集：先 reconciliation 後 handshake", async () => {
    const workspace = fakeWorkspace();
    vi.mocked(workspace.openProject).mockResolvedValue({
      status: "remoteBinding",
      url: markerUrl,
      repo: "backend",
      hasLocalOpenspec: false,
    });
    const order: string[] = [];
    const session = fakeRemoteSession(fakeRemoteDs());
    session.locator = { ...session.locator, checkoutRoot: "/work/backend" };
    const inspectCheckout = vi.fn().mockImplementation(async (path: string) => {
      order.push("inspect");
      return { root: path, tools: ["codex"] };
    });
    const bindCheckout = vi.fn().mockImplementation(async (path: string) => {
      order.push("bind");
      return path;
    });
    const openRemote = vi.fn().mockImplementation(async () => {
      order.push("handshake");
      return session;
    });
    const store = storeWith(openRemote, {
      workspace,
      connections: fakeConnections([loggedIn], { inspectCheckout, bindCheckout }),
    });

    await store.getState().openProjectAt("/work/backend");

    // 先補齊既有選集的受管產物，成功後才 handshake。
    expect(bindCheckout).toHaveBeenCalledWith(
      "/work/backend",
      "https://spec.example.test",
      "demo",
      "backend",
      ["codex"],
    );
    expect(order).toEqual(["inspect", "bind", "handshake"]);
    expect(openRemote).toHaveBeenCalledWith("c1", "demo/backend", "/work/backend");
    expect(store.getState().activeKey).toBe(KEY);
    expect(store.getState().workspaceChooser).toBeNull();
  });

  it("僅 remote marker、已登入但缺工具選集：導向 chooser checkout 並預填 scope／path，不 handshake", async () => {
    const workspace = fakeWorkspace();
    vi.mocked(workspace.openProject).mockResolvedValue({
      status: "remoteBinding",
      url: markerUrl,
      repo: "backend",
      hasLocalOpenspec: false,
    });
    const inspectCheckout = vi
      .fn()
      .mockResolvedValue({ root: "/work/backend", tools: [] });
    const bindCheckout = vi.fn();
    const openRemote = vi.fn();
    const store = storeWith(openRemote, {
      workspace,
      connections: fakeConnections([loggedIn], { inspectCheckout, bindCheckout }),
    });

    await store.getState().openProjectAt("/work/backend");

    expect(openRemote).not.toHaveBeenCalled();
    expect(bindCheckout).not.toHaveBeenCalled();
    expect(store.getState().activeKey).toBeNull();
    expect(store.getState().workspaceChooser).toEqual({
      initialConnectionId: "c1",
      initialScope: { projectKey: "demo", repoKey: "backend" },
      initialCheckoutPath: "/work/backend",
    });
  });

  it("僅 remote marker、已登入且有選集：reconciliation 失敗不建 tab／session且不 handshake", async () => {
    const workspace = fakeWorkspace();
    vi.mocked(workspace.openProject).mockResolvedValue({
      status: "remoteBinding",
      url: markerUrl,
      repo: "backend",
      hasLocalOpenspec: false,
    });
    const bindCheckout = vi.fn().mockRejectedValue(new Error("同步技能時檔案系統錯誤"));
    const openRemote = vi.fn();
    const store = storeWith(openRemote, {
      workspace,
      connections: fakeConnections([loggedIn], {
        inspectCheckout: vi.fn().mockResolvedValue({ root: "/work/backend", tools: ["codex"] }),
        bindCheckout,
      }),
    });

    await store.getState().openProjectAt("/work/backend");

    expect(openRemote).not.toHaveBeenCalled();
    expect(store.getState().activeKey).toBeNull();
    expect(store.getState().tabs).toHaveLength(0);
  });

  it("無已登入 connection：不 handshake，改開 chooser 並預填 marker origin", async () => {
    const workspace = fakeWorkspace();
    vi.mocked(workspace.openProject).mockResolvedValue({
      status: "remoteBinding",
      url: markerUrl,
      repo: "backend",
      hasLocalOpenspec: false,
    });
    const openRemote = vi.fn();
    const store = storeWith(openRemote, {
      workspace,
      connections: fakeConnections([{ ...loggedIn, loggedIn: false }]),
    });

    await store.getState().openProjectAt("/work/backend");

    expect(openRemote).not.toHaveBeenCalled();
    expect(store.getState().workspaceChooser).toEqual({
      initialServerUrl: "https://spec.example.test",
    });
  });

  it("marker 與 openspec 並存：停在強制選擇，繼續本機可用且不 handshake", async () => {
    const workspace = fakeWorkspace();
    vi.mocked(workspace.openProject).mockResolvedValue({
      status: "remoteBinding",
      url: markerUrl,
      repo: "backend",
      hasLocalOpenspec: true,
    });
    const openRemote = vi.fn();
    const store = createAppStore({
      createSession: localSession,
      workspace,
      connections: fakeConnections([loggedIn]),
      openRemote,
    });

    await store.getState().openProjectAt("/work/coexists");

    expect(store.getState().pendingRemoteConflict).toEqual({
      path: "/work/coexists",
      url: markerUrl,
      repo: "backend",
    });
    expect(store.getState().activeKey).toBeNull();
    expect(openRemote).not.toHaveBeenCalled();

    await store.getState().continueLocalFromConflict();
    expect(store.getState().activeKey).toBe("local:/work/coexists");
    expect(store.getState().pendingRemoteConflict).toBeNull();
    expect(openRemote).not.toHaveBeenCalled();
  });

  it("以 server 為準：先 handshake、再備份本機且不 import，最後開 checkout remote 分頁", async () => {
    const workspace = fakeWorkspace();
    vi.mocked(workspace.openProject).mockResolvedValue({
      status: "remoteBinding",
      url: markerUrl,
      repo: "backend",
      hasLocalOpenspec: true,
    });
    const order: string[] = [];
    const session = fakeRemoteSession(fakeRemoteDs());
    session.locator = { ...session.locator, checkoutRoot: "/work/coexists" };
    const bindCheckout = vi.fn().mockImplementation(async (path: string) => {
      order.push("reconcile");
      return path;
    });
    const openRemote = vi.fn().mockImplementation(async () => {
      order.push("handshake");
      return session;
    });
    const migration: MigrationAdapter = {
      migrate: vi.fn(),
      adoptRemote: vi.fn().mockImplementation(async (root: string) => {
        order.push("backup");
        return {
          backupPath: `${root}/openspec.migrated-2026-07-21`,
          checkoutRoot: root,
        };
      }),
    };
    const store = createAppStore({
      createSession: localSession,
      workspace,
      connections: fakeConnections([loggedIn], {
        inspectCheckout: vi.fn().mockResolvedValue({ root: "/work/coexists", tools: ["codex"] }),
        bindCheckout,
      }),
      openRemote,
      migration,
    });
    await store.getState().openProjectAt("/work/coexists");

    await store.getState().useServerFromConflict();

    // 先同步（reconcile）受管產物，再備份本機，最後開啟——建 tab 前工具已收斂。
    expect(order).toEqual(["reconcile", "handshake", "backup"]);
    expect(migration.migrate).not.toHaveBeenCalled();
    expect(migration.adoptRemote).toHaveBeenCalledWith("/work/coexists");
    expect(store.getState().pendingRemoteConflict).toBeNull();
    expect(store.getState().activeKey).toBe(KEY);
    expect(store.getState().tabs[0]?.locator).toEqual(
      expect.objectContaining({ kind: "remote", checkoutRoot: "/work/coexists" }),
    );
  });

  it("遷移本機內容：關閉並存衝突並把同一路徑交給 MigrationDialog", async () => {
    const workspace = fakeWorkspace();
    vi.mocked(workspace.openProject).mockResolvedValue({
      status: "remoteBinding",
      url: markerUrl,
      repo: "backend",
      hasLocalOpenspec: true,
    });
    const openRemote = vi.fn();
    const store = createAppStore({
      createSession: localSession,
      workspace,
      connections: fakeConnections([loggedIn]),
      openRemote,
    });
    await store.getState().openProjectAt("/work/coexists");

    await store.getState().migrateLocalFromConflict();

    expect(store.getState().pendingRemoteConflict).toBeNull();
    expect(store.getState().migrationRoot).toBe("/work/coexists");
    expect(store.getState().activeKey).toBe("local:/work/coexists");
    expect(openRemote).not.toHaveBeenCalled();
  });
});

describe("remote refresh（規格「capability 驅動停用且不偽造缺口」）", () => {
  it("loads archived server truth when the read capability is on", async () => {
    const ds = fakeRemoteDs();
    const openRemote = vi.fn().mockResolvedValue(fakeRemoteSession(ds));
    const store = storeWith(openRemote);
    await store.getState().openRemoteWorkspace("c1", "demo/backend");

    expect(ds.listArchived).toHaveBeenCalledTimes(1);
    expect(store.getState().archived).toEqual([
      expect.objectContaining({
        datedName: "2026-07-04-remote-old",
        name: "remote-old",
        specCount: 1,
      }),
    ]);
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

  it("重啟恢復會把持久化 checkoutRoot 帶回 handshake session", async () => {
    const openRemote = vi.fn().mockResolvedValue(fakeRemoteSession(fakeRemoteDs()));
    const store = storeWith(openRemote);
    store.setState({
      tabs: [
        {
          locator: {
            kind: "remote",
            connectionId: "c1",
            projectId: "demo",
            repoId: "backend",
            checkoutRoot: "/work/backend",
          },
          name: "Demo/backend",
          badge: null,
        },
      ],
    });

    await store.getState().activateTab(KEY);

    expect(openRemote).toHaveBeenCalledWith("c1", "demo/backend", "/work/backend");
  });

  it("a failed re-handshake keeps the remote tab selected as an error destination", async () => {
    const openRemote = vi.fn().mockRejectedValue({
      message: "登入已失效——請重新登入",
      reason: "permission_denied",
      status: 401,
    });
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
    expect(st.activeKey).toBe(KEY);
    expect(st.sessions[KEY]).toBeUndefined();
    expect(st.remoteRecovery[KEY]).toEqual({
      status: "error",
      failure: expect.objectContaining({ kind: "needs-reauth", status: 401 }),
    });
  });

  it("selects the no-session tab synchronously as restoring before handshake settles", async () => {
    const pending = deferred<WorkspaceSession>();
    const openRemote = vi.fn(() => pending.promise);
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

    const activation = store.getState().activateTab(KEY);
    expect(store.getState().activeKey).toBe(KEY);
    expect(store.getState().sessions[KEY]).toBeUndefined();
    expect(store.getState().remoteRecovery[KEY]).toEqual({
      status: "restoring",
      failure: null,
    });

    pending.resolve(fakeRemoteSession(fakeRemoteDs()));
    await activation;
    expect(store.getState().remoteRecovery[KEY]).toBeUndefined();
  });

  it("retry succeeds in place without adding a duplicate tab", async () => {
    const openRemote = vi
      .fn()
      .mockRejectedValueOnce({ message: "offline", reason: null, status: null })
      .mockResolvedValueOnce(fakeRemoteSession(fakeRemoteDs()));
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

    await store.getState().retryRemoteWorkspace(KEY);

    expect(store.getState().tabs).toHaveLength(1);
    expect(store.getState().sessions[KEY]).toBeTruthy();
    expect(store.getState().remoteRecovery[KEY]).toBeUndefined();
    expect(openRemote).toHaveBeenCalledTimes(2);
  });

  it("an older handshake result updates its own tab without stealing activeKey", async () => {
    const pending = deferred<WorkspaceSession>();
    const openRemote = vi.fn(() => pending.promise);
    const store = storeWith(openRemote);
    const other = fakeRemoteSession(fakeRemoteDs());
    other.id = "remote:c1/demo/frontend";
    other.locator = {
      kind: "remote",
      connectionId: "c1",
      projectId: "demo",
      repoId: "frontend",
    };
    store.setState({
      tabs: [
        {
          locator: { kind: "remote", connectionId: "c1", projectId: "demo", repoId: "backend" },
          name: "Demo/backend",
          badge: null,
        },
        { locator: other.locator, name: "Demo/frontend", badge: null },
      ],
      sessions: { [other.id]: other },
    });

    const activation = store.getState().activateTab(KEY);
    await store.getState().activateTab(other.id);
    pending.resolve(fakeRemoteSession(fakeRemoteDs()));
    await activation;

    expect(store.getState().activeKey).toBe(other.id);
    expect(store.getState().sessions[KEY]).toBeTruthy();
  });

  it("only accepts the latest retry generation for the same tab", async () => {
    const first = deferred<WorkspaceSession>();
    const second = deferred<WorkspaceSession>();
    const openRemote = vi
      .fn()
      .mockImplementationOnce(() => first.promise)
      .mockImplementationOnce(() => second.promise);
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

    const older = store.getState().activateTab(KEY);
    const latest = store.getState().retryRemoteWorkspace(KEY);
    const session = fakeRemoteSession(fakeRemoteDs());
    second.resolve(session);
    await latest;
    first.reject({ message: "late failure", reason: null, status: null });
    await older;

    expect(store.getState().sessions[KEY]).toBe(session);
    expect(store.getState().remoteRecovery[KEY]).toBeUndefined();
  });

  it("closing a restoring tab clears state and invalidates its in-flight handshake", async () => {
    const pending = deferred<WorkspaceSession>();
    const openRemote = vi.fn(() => pending.promise);
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

    const activation = store.getState().activateTab(KEY);
    store.getState().closeTab(KEY);
    pending.reject({ message: "late failure", reason: null, status: null });
    await activation;

    expect(store.getState().tabs).toHaveLength(0);
    expect(store.getState().sessions[KEY]).toBeUndefined();
    expect(store.getState().remoteRecovery[KEY]).toBeUndefined();
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

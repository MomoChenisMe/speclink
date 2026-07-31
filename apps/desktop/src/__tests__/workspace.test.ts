// 開啟專案 action 三態與分頁列狀態（design D3/D10/D11；spec 需求
// 「專案分頁列存於 app 本機」）。workspace 探測面與 session 工廠以 mock 注入
// （workspace-session 決策 6）：資料載入一律經活躍 session 的 dataSource。
import { describe, it, expect, vi, beforeEach } from "vitest";
import type { SpeclinkDataSource } from "@speclink/ui";

import { createAppStore } from "../store";
import { locatorKey, LOCAL_CAPABILITIES, type WorkspaceSession } from "../session";
import { persistTabs, readPersistedTabs, type ProjectTab } from "../tabs";
import type { WorkspaceAdapter } from "../adapter/workspace";

const { toastError } = vi.hoisted(() => ({ toastError: vi.fn() }));
vi.mock("sonner", () => ({ toast: { error: toastError } }));

function fakeDataSource(): SpeclinkDataSource {
  return {
    listChanges: vi.fn().mockResolvedValue([
      { name: "started", status: "in-progress", totalTasks: 10, completedTasks: 0, startedAt: "2026-07-06" },
      { name: "proposed", status: "in-progress", totalTasks: 28, completedTasks: 0 },
    ]),
    listSpecs: vi.fn().mockResolvedValue([]),
    listArchived: vi.fn().mockResolvedValue([]),
    listDiscussions: vi.fn().mockResolvedValue({ active: [], archived: [] }),
    getDocument: vi.fn().mockResolvedValue(null),
    getArchivedDocument: vi.fn().mockResolvedValue(null),
    getDiscussionDocument: vi.fn().mockResolvedValue(null),
    changeCapabilities: vi.fn().mockResolvedValue([]),
    archivedCapabilities: vi.fn().mockResolvedValue([]),
    changeMeta: vi.fn().mockResolvedValue(null),
    status: vi.fn().mockResolvedValue(null),
    runVerb: vi.fn().mockResolvedValue({}),
    deleteChange: vi.fn().mockResolvedValue(undefined),
    setTaskDone: vi.fn().mockResolvedValue(undefined),
    moveTask: vi.fn().mockResolvedValue(undefined),
    promoteDiscussion: vi.fn().mockResolvedValue({ change: "x" }),
    archiveDiscussion: vi.fn().mockResolvedValue(undefined),
  } as unknown as SpeclinkDataSource;
}

function fakeWorkspace(over: Partial<WorkspaceAdapter> = {}): WorkspaceAdapter {
  return {
    openProject: vi.fn().mockResolvedValue({ status: "project", root: "C:\\proj\\alpha", name: "alpha" }),
    initProject: vi.fn().mockResolvedValue({ status: "project", root: "C:\\proj\\fresh", name: "fresh" }),
    adoptProject: vi.fn().mockResolvedValue({ status: "project", root: "C:\\migrated", name: "migrated" }),
    // 預設非專案語境：首啟回退路徑走 catch、維持零分頁。
    startupDir: vi.fn().mockRejectedValue("startup dir unavailable in this fake"),
    projectStats: vi.fn().mockResolvedValue({ pendingWrapUp: 2 }),
    watchWorkspace: vi.fn().mockResolvedValue(undefined),
    pickFolder: vi.fn().mockResolvedValue(null),
    ...over,
  } as WorkspaceAdapter;
}

/** 假 session：dataSource 共用測試注入的 fake（settings／events 本套件不涉）。 */
function fakeSession(ds: SpeclinkDataSource, root: string, name: string): WorkspaceSession {
  return {
    id: `local:${root}`,
    locator: { kind: "local", root },
    descriptor: { name },
    dataSource: ds,
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

function makeStore(ds: SpeclinkDataSource, ws: WorkspaceAdapter) {
  return createAppStore({
    createSession: (root, name) => fakeSession(ds, root, name),
    workspace: ws,
  });
}

function tab(root: string, name = root): ProjectTab {
  return { locator: { kind: "local", root }, name };
}

function keys(tabs: ProjectTab[]): string[] {
  return tabs.map((t) => locatorKey(t.locator));
}

beforeEach(() => {
  localStorage.clear();
  toastError.mockClear();
});

describe("開啟專案 action 三態（design D3）", () => {
  it("命中專案：純探測成功後記入分頁、設 activeKey、重掛監看、整批 refresh", async () => {
    const ds = fakeDataSource();
    const ws = fakeWorkspace();
    const store = makeStore(ds, ws);
    await store.getState().openProjectAt("C:\\proj\\alpha");
    const s = store.getState();
    expect(ws.openProject).toHaveBeenCalledWith("C:\\proj\\alpha");
    expect(keys(s.tabs)).toEqual(["local:C:\\proj\\alpha"]);
    expect(s.activeKey).toBe("local:C:\\proj\\alpha");
    // 監看顯式跟隨活躍 session（決策 5）。
    expect(ws.watchWorkspace).toHaveBeenCalledWith("C:\\proj\\alpha");
    expect(ds.listChanges).toHaveBeenCalled();
    // 持久化：locator＋顯示名＋最後活躍 key。
    const persisted = readPersistedTabs();
    expect(persisted.tabs[0]).toEqual({
      locator: { kind: "local", root: "C:\\proj\\alpha" },
      name: "alpha",
    });
    expect(persisted.activeKey).toBe("local:C:\\proj\\alpha");
  });

  it("uninitialized：顯示初始化確認（pendingInit），分頁與資料不動", async () => {
    const ds = fakeDataSource();
    const ws = fakeWorkspace({
      openProject: vi.fn().mockResolvedValue({ status: "uninitialized", dir: "C:\\empty" }),
    });
    const store = makeStore(ds, ws);
    await store.getState().openProjectAt("C:\\empty");
    const s = store.getState();
    expect(s.pendingInit).toBe("C:\\empty");
    expect(s.tabs).toEqual([]);
    expect(ds.listChanges).not.toHaveBeenCalled();
  });

  it("取消初始化：畫面與狀態不動、不呼叫 init", async () => {
    const ws = fakeWorkspace({
      openProject: vi.fn().mockResolvedValue({ status: "uninitialized", dir: "C:\\empty" }),
    });
    const store = makeStore(fakeDataSource(), ws);
    await store.getState().openProjectAt("C:\\empty");
    store.getState().cancelInit();
    const s = store.getState();
    expect(s.pendingInit).toBeNull();
    expect(s.tabs).toEqual([]);
    expect(ws.initProject).not.toHaveBeenCalled();
  });

  it("確認初始化：以所選工具呼叫 init，成功後切入新專案", async () => {
    const ds = fakeDataSource();
    const ws = fakeWorkspace({
      openProject: vi.fn().mockResolvedValue({ status: "uninitialized", dir: "C:\\proj\\fresh" }),
    });
    const store = makeStore(ds, ws);
    await store.getState().openProjectAt("C:\\proj\\fresh");
    await store.getState().confirmInit(["claude", "codex"]);
    const s = store.getState();
    expect(ws.initProject).toHaveBeenCalledWith("C:\\proj\\fresh", ["claude", "codex"]);
    expect(s.pendingInit).toBeNull();
    expect(s.activeKey).toBe("local:C:\\proj\\fresh");
    expect(keys(s.tabs)).toContain("local:C:\\proj\\fresh");
  });

  // --- 第四態：未啟用（spec 需求「未啟用資料夾經確認後補齊啟用」；決策 3） ---

  it("unadopted：顯示啟用確認（pendingAdopt 錨定回報 root），分頁與資料不動", async () => {
    const ds = fakeDataSource();
    const ws = fakeWorkspace({
      openProject: vi.fn().mockResolvedValue({ status: "unadopted", root: "C:\\migrated" }),
    });
    const store = makeStore(ds, ws);
    // 自子目錄開啟：pendingAdopt 必須是探測回報的專案根，非使用者所選路徑。
    await store.getState().openProjectAt("C:\\migrated\\sub");
    const s = store.getState();
    expect(s.pendingAdopt).toBe("C:\\migrated");
    expect(s.tabs).toEqual([]);
    expect(ds.listChanges).not.toHaveBeenCalled();
  });

  it("取消啟用：狀態清空、無任何寫入呼叫", async () => {
    const ws = fakeWorkspace({
      openProject: vi.fn().mockResolvedValue({ status: "unadopted", root: "C:\\migrated" }),
    });
    const store = makeStore(fakeDataSource(), ws);
    await store.getState().openProjectAt("C:\\migrated");
    store.getState().cancelAdopt();
    const s = store.getState();
    expect(s.pendingAdopt).toBeNull();
    expect(s.tabs).toEqual([]);
    expect(ws.adoptProject).not.toHaveBeenCalled();
    expect(ws.initProject).not.toHaveBeenCalled();
  });

  it("確認啟用：以所選工具呼叫 adopt，成功後以回報 root 切入專案", async () => {
    const ds = fakeDataSource();
    const ws = fakeWorkspace({
      openProject: vi.fn().mockResolvedValue({ status: "unadopted", root: "C:\\migrated" }),
    });
    const store = makeStore(ds, ws);
    await store.getState().openProjectAt("C:\\migrated");
    await store.getState().confirmAdopt(["claude", "codex"]);
    const s = store.getState();
    expect(ws.adoptProject).toHaveBeenCalledWith("C:\\migrated", ["claude", "codex"]);
    expect(s.pendingAdopt).toBeNull();
    expect(s.activeKey).toBe("local:C:\\migrated");
    expect(keys(s.tabs)).toContain("local:C:\\migrated");
  });

  it("啟用失敗：單行錯誤 toast、不切換專案", async () => {
    const ws = fakeWorkspace({
      openProject: vi.fn().mockResolvedValue({ status: "unadopted", root: "C:\\migrated" }),
      adoptProject: vi.fn().mockRejectedValue("cannot adopt 'C:\\migrated': permission denied"),
    });
    const store = makeStore(fakeDataSource(), ws);
    await store.getState().openProjectAt("C:\\migrated");
    await store.getState().confirmAdopt(["claude"]);
    const s = store.getState();
    expect(s.tabs).toEqual([]);
    expect(s.activeKey).toBeNull();
    expect(s.pendingAdopt).toBeNull();
    expect(toastError).toHaveBeenCalledTimes(1);
    const [message] = toastError.mock.calls[0] as [string];
    expect(message).toContain("C:\\migrated");
    expect(message).toContain("permission denied");
  });

  it("開啟與初始化失敗皆以相同固定 id 發出含主詞與 core 錯誤的 toast", async () => {
    const openWs = fakeWorkspace({
      openProject: vi.fn().mockRejectedValue("cannot open 'C:\\gone': not an existing directory"),
    });
    const openStore = makeStore(fakeDataSource(), openWs);
    await openStore.getState().openProjectAt("C:\\gone");
    expect(openStore.getState().tabs).toEqual([]);
    expect(openStore.getState().activeKey).toBeNull();

    const initWs = fakeWorkspace({
      openProject: vi.fn().mockResolvedValue({ status: "uninitialized", dir: "C:\\fresh" }),
      initProject: vi.fn().mockRejectedValue("cannot initialize 'C:\\fresh': permission denied"),
    });
    const initStore = makeStore(fakeDataSource(), initWs);
    await initStore.getState().openProjectAt("C:\\fresh");
    await initStore.getState().confirmInit(["codex"]);
    expect(initStore.getState().tabs).toEqual([]);
    expect(initStore.getState().activeKey).toBeNull();

    expect(toastError).toHaveBeenCalledTimes(2);
    const [openMessage, openOptions] = toastError.mock.calls[0] as [string, { id?: string }];
    expect(openMessage).toContain("C:\\gone");
    expect(openMessage).toContain("cannot open");
    const [initMessage, initOptions] = toastError.mock.calls[1] as [string, { id?: string }];
    expect(initMessage).toContain("C:\\fresh");
    expect(initMessage).toContain("permission denied");
    expect(openOptions.id).toEqual(expect.any(String));
    expect(initOptions.id).toBe(openOptions.id);
  });
});

describe("分頁列（spec 需求「專案分頁列存於 app 本機」）", () => {
  it("依序開啟 A、B 再開 A：各一分頁、A 為 active（spec Scenario 去重）", async () => {
    const opened: Record<string, { status: string; root: string; name: string }> = {
      A: { status: "project", root: "A", name: "a" },
      B: { status: "project", root: "B", name: "b" },
    };
    const ws = fakeWorkspace({
      openProject: vi.fn().mockImplementation((p: string) => Promise.resolve(opened[p])),
    });
    const store = makeStore(fakeDataSource(), ws);
    await store.getState().openProjectAt("A");
    await store.getState().openProjectAt("B");
    await store.getState().openProjectAt("A");
    const s = store.getState();
    expect(keys(s.tabs)).toEqual(["local:A", "local:B"]);
    expect(s.activeKey).toBe("local:A");
    // 重啟還原：持久化內容一致。
    expect(readPersistedTabs().tabs.map((t) => locatorKey(t.locator))).toEqual([
      "local:A",
      "local:B",
    ]);
    expect(readPersistedTabs().activeKey).toBe("local:A");
  });

  it("點分頁切換走與開啟專案相同語意（spec Scenario 點擊分頁切換專案）", async () => {
    const ws = fakeWorkspace({
      openProject: vi.fn().mockResolvedValue({ status: "project", root: "B", name: "b" }),
    });
    const ds = fakeDataSource();
    const store = makeStore(ds, ws);
    store.setState({ tabs: [tab("A", "a"), tab("B", "b")], activeKey: "local:A" });
    await store.getState().activateTab("local:B");
    expect(ws.openProject).toHaveBeenCalledWith("B");
    expect(store.getState().activeKey).toBe("local:B");
    expect(ds.listChanges).toHaveBeenCalled();
  });

  it("關閉分頁即自持久化清單移除，session 一併回收", async () => {
    const store = makeStore(fakeDataSource(), fakeWorkspace());
    const tabs: ProjectTab[] = [tab("A", "a"), tab("B", "b")];
    store.setState({ tabs, activeKey: "local:A" });
    persistTabs(tabs, "local:A");
    store.getState().closeTab("local:B");
    expect(keys(store.getState().tabs)).toEqual(["local:A"]);
    expect(readPersistedTabs().tabs.map((t) => locatorKey(t.locator))).toEqual(["local:A"]);
    expect(store.getState().sessions["local:B"]).toBeUndefined();
  });

  it("restoreTabs：依持久化 activeKey 還原活躍分頁、背景 local 分頁各探測一次路徑有效性", async () => {
    persistTabs([tab("A", "a"), tab("B", "b")], "local:A");
    const ws = fakeWorkspace({
      openProject: vi
        .fn()
        .mockImplementation((p: string) =>
          Promise.resolve({ status: "project", root: p, name: p.toLowerCase() }),
        ),
      projectStats: vi.fn().mockResolvedValue({ pendingWrapUp: 2 }),
    });
    const store = makeStore(fakeDataSource(), ws);
    await store.getState().restoreTabs();
    const s = store.getState();
    expect(keys(s.tabs)).toEqual(["local:A", "local:B"]);
    expect(s.activeKey).toBe("local:A");
    // 背景分頁 B 探測一次路徑；active 分頁 A 不經 project_stats。分頁不攜帶計數徽章。
    expect(ws.projectStats).toHaveBeenCalledWith("B");
    expect(ws.projectStats).not.toHaveBeenCalledWith("A");
    expect(s.tabs.every((t) => !("badge" in t))).toBe(true);
  });

  it("首啟無持久化分頁：啟動目錄（cwd 探索）為專案時自動記入（決策 4 首啟路徑）", async () => {
    const ws = fakeWorkspace({
      startupDir: vi.fn().mockResolvedValue("C:\\proj\\alpha"),
    });
    const store = makeStore(fakeDataSource(), ws);
    await store.getState().restoreTabs();
    expect(ws.startupDir).toHaveBeenCalled();
    expect(ws.openProject).toHaveBeenCalledWith("C:\\proj\\alpha");
    expect(store.getState().activeKey).toBe("local:C:\\proj\\alpha");
  });

  it("首啟且啟動目錄非專案：維持零分頁空狀態", async () => {
    const ws = fakeWorkspace({
      startupDir: vi.fn().mockResolvedValue("C:\\nowhere"),
      openProject: vi.fn().mockRejectedValue("cannot open 'C:\\nowhere'"),
    });
    const store = makeStore(fakeDataSource(), ws);
    await store.getState().restoreTabs();
    expect(store.getState().tabs).toEqual([]);
    expect(store.getState().activeKey).toBeNull();
  });

  it("refresh 後分頁不攜帶計數徽章（spec「分頁不顯示計數徽章」）", async () => {
    const ds = fakeDataSource();
    // 契約範例：2 個已就緒變更＋1 份已結論未轉出討論 → 徽章 3。
    ds.listChanges = vi.fn().mockResolvedValue([
      { name: "ready-a", status: "in-progress", totalTasks: 5, completedTasks: 5 },
      { name: "ready-b", status: "in-progress", totalTasks: 3, completedTasks: 3 },
      { name: "started", status: "in-progress", totalTasks: 10, completedTasks: 2, startedAt: "2026-07-06" },
    ]);
    ds.listDiscussions = vi.fn().mockResolvedValue({
      active: [
        { slug: "alpha", topic: "a", status: "concluded", rounds: 1, created: "2026-01-02", promotedTo: [] },
        { slug: "beta", topic: "b", status: "open", rounds: 1, created: "2026-01-02", promotedTo: [] },
        { slug: "gamma", topic: "c", status: "promoted", rounds: 1, created: "2026-01-02", promotedTo: ["cut"] },
      ],
      archived: [],
    });
    const store = makeStore(ds, fakeWorkspace());
    store.setState({
      tabs: [tab("A", "a")],
      sessions: { "local:A": fakeSession(ds, "A", "a") },
      activeKey: "local:A",
    });
    await store.getState().refresh();
    expect("badge" in store.getState().tabs[0]).toBe(false);
  });

  it("切換以純探測回報值為準（決策 4：probe 即後端真相，無 current-root 全域）", async () => {
    const ws = fakeWorkspace({
      openProject: vi.fn().mockResolvedValue({ status: "project", root: "B", name: "fresh-name" }),
    });
    const store = makeStore(fakeDataSource(), ws);
    await store.getState().openProjectAt("B");
    expect(
      store.getState().tabs.find((t) => locatorKey(t.locator) === "local:B")?.name,
    ).toBe("fresh-name");
  });

  it("cycleTab 循環切至下一分頁（Ctrl+Tab，走開啟專案語意）", async () => {
    const ws = fakeWorkspace({
      openProject: vi
        .fn()
        .mockImplementation((p: string) =>
          Promise.resolve({ status: "project", root: p, name: p.toLowerCase() }),
        ),
    });
    const store = makeStore(fakeDataSource(), ws);
    store.setState({
      tabs: [tab("A", "a"), tab("B", "b"), tab("C", "c")],
      activeKey: "local:C",
    });
    await store.getState().cycleTab();
    expect(store.getState().activeKey).toBe("local:A"); // 尾端循環回首位
    await store.getState().cycleTab();
    expect(store.getState().activeKey).toBe("local:B");
  });

  it("gotoTab 直達第 N 個分頁（Ctrl+1..9，1-based；超界不動作）", async () => {
    const ws = fakeWorkspace({
      openProject: vi
        .fn()
        .mockImplementation((p: string) =>
          Promise.resolve({ status: "project", root: p, name: p.toLowerCase() }),
        ),
    });
    const store = makeStore(fakeDataSource(), ws);
    store.setState({
      tabs: [tab("A", "a"), tab("B", "b")],
      activeKey: "local:A",
    });
    await store.getState().gotoTab(2);
    expect(store.getState().activeKey).toBe("local:B");
    await store.getState().gotoTab(9);
    expect(store.getState().activeKey).toBe("local:B");
    expect(ws.openProject).toHaveBeenCalledTimes(1);
  });

  it("失效分頁：點擊轉錯誤態、顯示錯誤、可自分頁移除且不切換（spec Scenario）", async () => {
    const ws = fakeWorkspace({
      openProject: vi.fn().mockRejectedValue("cannot open 'B': not an existing directory"),
    });
    const store = makeStore(fakeDataSource(), ws);
    store.setState({
      tabs: [tab("A", "a"), tab("B", "b")],
      activeKey: "local:A",
    });
    await store.getState().activateTab("local:B");
    const s = store.getState();
    expect(s.activeKey).toBe("local:A");
    expect(s.tabErrors["local:B"]).toContain("not an existing directory");
    // 自分頁移除後持久化清單同步消失。
    store.getState().closeTab("local:B");
    expect(keys(store.getState().tabs)).toEqual(["local:A"]);
    expect(store.getState().tabErrors["local:B"]).toBeUndefined();
  });
});

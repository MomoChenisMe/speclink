import { beforeEach, describe, it, expect, vi } from "vitest";
import type { ChangeItem, SearchHit, SpeclinkDataSource, StatusReport } from "@speclink/ui";

import { createAppStore } from "../store";
import type { ConnectionsAdapter } from "../adapter/connections";
import type { WorkspaceAdapter } from "../adapter/workspace";
import { LOCAL_CAPABILITIES, type WorkspaceSession } from "../session";

const { toastError } = vi.hoisted(() => ({ toastError: vi.fn() }));
vi.mock("sonner", () => ({ toast: { error: toastError } }));

const STATUS: StatusReport = {
  changeName: "x",
  schemaName: "spec-driven",
  isComplete: false,
  applyRequires: ["tasks"],
  artifacts: [],
};

function fakeDataSource(over: Partial<SpeclinkDataSource> = {}): SpeclinkDataSource {
  return {
    listChanges: vi.fn().mockResolvedValue([
      { name: "desktop-shell-and-browser", status: "in-progress", totalTasks: 26, completedTasks: 24 },
    ]),
    listSpecs: vi.fn().mockResolvedValue([{ id: "desktop-app" }]),
    listArchived: vi.fn().mockResolvedValue([{ datedName: "2026-07-04-x", date: "2026-07-04", name: "x" }]),
    status: vi.fn().mockResolvedValue(STATUS),
    changeMeta: vi.fn().mockResolvedValue(null),
    deleteChange: vi.fn().mockResolvedValue(undefined),
    setTaskDone: vi.fn().mockResolvedValue(undefined),
    moveTask: vi.fn().mockResolvedValue(undefined),
    getDocument: vi.fn().mockResolvedValue("## Why\nhello"),
    getSpecDocument: vi.fn().mockResolvedValue("# spec"),
    searchWorkspace: vi.fn().mockResolvedValue([]),
    changeCapabilities: vi.fn().mockResolvedValue(["desktop-app"]),
    runVerb: vi.fn().mockResolvedValue({ valid: true }),
    getArchivedDocument: vi.fn().mockResolvedValue(null),
    archivedCapabilities: vi.fn().mockResolvedValue([]),
    listDiscussions: vi.fn().mockResolvedValue({ active: [], archived: [] }),
    getDiscussionDocument: vi.fn().mockResolvedValue(null),
    promoteDiscussion: vi.fn().mockResolvedValue({ change: "promoted-change" }),
    archiveDiscussion: vi.fn().mockResolvedValue(undefined),
    reorderCard: vi.fn().mockResolvedValue(undefined),
    ...over,
  };
}

/** 假 session（workspace-session 決策 6）：資料載入一律經活躍 session 的 dataSource。 */
function fakeSession(ds: SpeclinkDataSource, root = "A", name = "a"): WorkspaceSession {
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

/** 以活躍 session 預置 store（注入 session 工廠、無 workspace 探測面）。 */
function storeWith(ds: SpeclinkDataSource) {
  const store = createAppStore({ createSession: (root, name) => fakeSession(ds, root, name) });
  const session = fakeSession(ds);
  store.setState({ sessions: { [session.id]: session }, activeKey: session.id });
  return store;
}

function remoteSession(ds: SpeclinkDataSource, repoId: string): WorkspaceSession {
  const locator = { kind: "remote" as const, connectionId: "c1", projectId: "demo", repoId };
  return {
    id: `remote:c1/demo/${repoId}`,
    locator,
    descriptor: { name: `Demo/${repoId}` },
    dataSource: ds,
    settings: {
      kind: "remote",
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

function storeWithRemoteSessions(a: WorkspaceSession, b: WorkspaceSession) {
  const store = createAppStore({
    createSession: () => {
      throw new Error("remote session 測試不得建立 local session");
    },
    workspace: {} as WorkspaceAdapter,
  });
  store.setState({
    tabs: [
      { locator: a.locator, name: a.descriptor.name },
      { locator: b.locator, name: b.descriptor.name },
    ],
    sessions: { [a.id]: a, [b.id]: b },
    activeKey: a.id,
  });
  return store;
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

beforeEach(() => {
  toastError.mockClear();
  localStorage.clear();
});

describe("app store (Zustand)", () => {
  it("refresh loads changes, specs and archived", async () => {
    const store = storeWith(fakeDataSource());
    await store.getState().refresh();
    expect(store.getState().changes).toHaveLength(1);
    expect(store.getState().specs).toHaveLength(1);
    expect(store.getState().archived).toHaveLength(1);
    expect(store.getState().loaded).toBe(true);
  });

  it("refresh increments the refreshGen generation (initial 0)", async () => {
    // 刷新世代：內容元件據此重載已載入的文件（design D1）。
    const store = storeWith(fakeDataSource());
    expect(store.getState().refreshGen).toBe(0);
    await store.getState().refresh();
    expect(store.getState().refreshGen).toBe(1);
    await store.getState().refresh();
    expect(store.getState().refreshGen).toBe(2);
  });

  it("switches back to each remote session's own last-success snapshot when refresh fails", async () => {
    const aChanges = vi
      .fn()
      .mockResolvedValue([{ name: "a-change", status: "in-progress", totalTasks: 1, completedTasks: 0 }]);
    const bChanges = vi
      .fn()
      .mockResolvedValue([{ name: "b-change", status: "in-progress", totalTasks: 1, completedTasks: 0 }]);
    const a = remoteSession(fakeDataSource({ listChanges: aChanges }), "alpha");
    const b = remoteSession(fakeDataSource({ listChanges: bChanges }), "beta");
    const store = storeWithRemoteSessions(a, b);

    await store.getState().refresh();
    await store.getState().activateTab(b.id);
    expect(store.getState().changes.map((change) => change.name)).toEqual(["b-change"]);

    aChanges.mockRejectedValue(new Error("alpha offline"));
    bChanges.mockRejectedValue(new Error("beta offline"));
    await store.getState().activateTab(a.id);
    const afterReturningToA = store.getState().changes.map((change) => change.name);
    await store.getState().activateTab(b.id);
    const afterReturningToB = store.getState().changes.map((change) => change.name);

    expect([afterReturningToA, afterReturningToB]).toEqual([["a-change"], ["b-change"]]);
    const persisted = JSON.parse(localStorage.getItem("speclink.projectTabs") ?? "{}");
    expect(Object.keys(persisted).sort()).toEqual(["activeKey", "tabs", "version"]);
    expect(JSON.stringify(persisted)).not.toContain("a-change");
    expect(JSON.stringify(persisted)).not.toContain("b-change");
  });

  it("clears the previous workspace when a never-loaded needs-reauth session cannot refresh", async () => {
    const a = remoteSession(
      fakeDataSource({
        listChanges: vi
          .fn()
          .mockResolvedValue([{ name: "a-change", status: "in-progress", totalTasks: 1, completedTasks: 0 }]),
        listSpecs: vi.fn().mockResolvedValue([{ id: "a-spec" }]),
      }),
      "alpha",
    );
    const b = remoteSession(
      fakeDataSource({ listChanges: vi.fn().mockRejectedValue(new Error("login required")) }),
      "beta",
    );
    b.connectionState = {
      connectionId: "c1",
      state: "needs-reauth",
      message: "請重新登入",
    };
    const store = storeWithRemoteSessions(a, b);
    await store.getState().refresh();

    await store.getState().activateTab(b.id);

    const state = store.getState();
    expect(state.activeKey).toBe(b.id);
    expect(state.changes).toEqual([]);
    expect(state.specs).toEqual([]);
    expect(state.archived).toEqual([]);
    expect(state.discussions).toEqual({ active: [], archived: [] });
    expect(state.loaded).toBe(false);
  });

  it("clears a closed active session's content instead of retaining its runtime snapshot", async () => {
    const a = remoteSession(
      fakeDataSource({
        listChanges: vi
          .fn()
          .mockResolvedValue([{ name: "a-change", status: "in-progress", totalTasks: 1, completedTasks: 0 }]),
      }),
      "alpha",
    );
    const store = createAppStore({
      createSession: () => {
        throw new Error("remote session 測試不得建立 local session");
      },
      workspace: {} as WorkspaceAdapter,
    });
    store.setState({
      tabs: [{ locator: a.locator, name: a.descriptor.name }],
      sessions: { [a.id]: a },
      activeKey: a.id,
    });
    await store.getState().refresh();

    store.getState().closeTab(a.id);

    expect(store.getState().activeKey).toBeNull();
    expect(store.getState().changes).toEqual([]);
    expect(store.getState().loaded).toBe(false);
  });

  it("does not let a late refresh from workspace A overwrite active workspace B", async () => {
    const pendingA = deferred<ChangeItem[]>();
    const a = remoteSession(
      fakeDataSource({ listChanges: vi.fn(() => pendingA.promise) }),
      "alpha",
    );
    const b = remoteSession(
      fakeDataSource({
        listChanges: vi
          .fn()
          .mockResolvedValue([{ name: "b-change", status: "in-progress", totalTasks: 1, completedTasks: 0 }]),
      }),
      "beta",
    );
    const store = storeWithRemoteSessions(a, b);

    const aRefresh = store.getState().refresh();
    await store.getState().activateTab(b.id);
    expect(store.getState().changes.map((change) => change.name)).toEqual(["b-change"]);

    pendingA.resolve([
      { name: "a-late", status: "in-progress", totalTasks: 1, completedTasks: 0 },
    ]);
    await aRefresh;

    expect(store.getState().activeKey).toBe(b.id);
    expect(store.getState().changes.map((change) => change.name)).toEqual(["b-change"]);
  });

  it("keeps the latest refresh result when an older request for the same session finishes last", async () => {
    const older = deferred<ChangeItem[]>();
    const latest = deferred<ChangeItem[]>();
    const listChanges = vi
      .fn()
      .mockImplementationOnce(() => older.promise)
      .mockImplementationOnce(() => latest.promise);
    const b = remoteSession(fakeDataSource({ listChanges }), "beta");
    const other = remoteSession(fakeDataSource(), "other");
    const store = storeWithRemoteSessions(b, other);

    const olderRefresh = store.getState().refresh();
    const latestRefresh = store.getState().refresh();
    latest.resolve([
      { name: "newer", status: "in-progress", totalTasks: 1, completedTasks: 0 },
    ]);
    await latestRefresh;
    older.resolve([
      { name: "older", status: "in-progress", totalTasks: 1, completedTasks: 0 },
    ]);
    await olderRefresh;

    expect(store.getState().changes.map((change) => change.name)).toEqual(["newer"]);
  });

  it("invalidates workspace A search and detail state when switching to needs-reauth B", async () => {
    vi.useFakeTimers();
    const pendingSearch = deferred<SearchHit[]>();
    const a = remoteSession(
      fakeDataSource({
        listChanges: vi
          .fn()
          .mockResolvedValue([{ name: "a-change", status: "in-progress", totalTasks: 1, completedTasks: 0 }]),
        searchWorkspace: vi.fn(() => pendingSearch.promise),
      }),
      "alpha",
    );
    const b = remoteSession(
      fakeDataSource({ listChanges: vi.fn().mockRejectedValue(new Error("login required")) }),
      "beta",
    );
    b.connectionState = {
      connectionId: "c1",
      state: "needs-reauth",
      message: "請重新登入",
    };
    const store = storeWithRemoteSessions(a, b);

    try {
      await store.getState().refresh();
      store.getState().openDetail("a-change");
      store.setState({
        detailSpec: "a-spec",
        pendingArchive: "a-change",
        pendingDelete: "a-change",
        pendingArchiveDiscussion: "a-topic",
      });
      store.getState().setBoardQuery("a");
      await vi.advanceTimersByTimeAsync(200);

      await store.getState().activateTab(b.id);

      expect(store.getState()).toMatchObject({
        activeKey: b.id,
        searchHits: [],
        detailChange: null,
        detailDiscussion: null,
        detailSpec: null,
        detailArchived: null,
        pendingArchive: null,
        pendingDelete: null,
        pendingArchiveDiscussion: null,
        drawerVerb: null,
      });

      pendingSearch.resolve([
        { kind: "change", id: "a-change", artifact: "design.md", snippet: "A only" },
      ]);
      await Promise.resolve();
      await Promise.resolve();
      expect(store.getState().searchHits).toEqual([]);
      expect(store.getState().boardQuery).toBe("a");
    } finally {
      store.getState().disposeSearch();
      vi.useRealTimers();
    }
  });

  it("setView, setQuery and toggleExpand update UI state", () => {
    const store = storeWith(fakeDataSource());
    store.getState().setView("archived");
    store.getState().setQuery("desk");
    store.getState().toggleExpand("a");
    expect(store.getState().view).toBe("archived");
    expect(store.getState().query).toBe("desk");
    expect(store.getState().expandedName).toBe("a");
    // 再次 toggle 同一名稱收合
    store.getState().toggleExpand("a");
    expect(store.getState().expandedName).toBeNull();
  });

  it("boardView can switch to the specs page and keeps the specs list state", async () => {
    // spec「規格頁提供清單、搜尋與展開檢視」：主視圖新增 specs 態
    // （與看板、已封存、設定並列），切換不動已載入的 specs 清單。
    const store = storeWith(fakeDataSource());
    await store.getState().refresh();
    store.getState().setBoardView("specs");
    expect(store.getState().boardView).toBe("specs");
    expect(store.getState().specs).toHaveLength(1);
    store.getState().setBoardView("board");
    expect(store.getState().boardView).toBe("board");
  });

  it("boardQuery starts empty and setBoardQuery updates it", () => {
    // 看板搜尋字串：純 UI 狀態、不入任何 persist 機制（spec「不跨啟動保留」）。
    const store = storeWith(fakeDataSource());
    expect(store.getState().boardQuery).toBe("");
    store.getState().setBoardQuery("desk");
    expect(store.getState().boardQuery).toBe("desk");
  });

  it("boardQuery triggers the debounced full-text search (latest wins); clearing resets hits", async () => {
    // design D6：200ms 去抖、latest-wins；空 query 清命中並取消在途。
    vi.useFakeTimers();
    const hits = [{ kind: "change", id: "demo", artifact: "design.md", snippet: "…x…" }];
    const ds = fakeDataSource({ searchWorkspace: vi.fn().mockResolvedValue(hits) });
    const store = storeWith(ds);
    store.getState().setBoardQuery("d");
    store.getState().setBoardQuery("di");
    expect(ds.searchWorkspace).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(200);
    expect(ds.searchWorkspace).toHaveBeenCalledTimes(1);
    expect(ds.searchWorkspace).toHaveBeenCalledWith("di");
    expect(store.getState().searchHits).toEqual(hits);
    store.getState().setBoardQuery("");
    expect(store.getState().searchHits).toEqual([]);
    await vi.advanceTimersByTimeAsync(300);
    expect(ds.searchWorkspace).toHaveBeenCalledTimes(1);
    vi.useRealTimers();
  });

  it("full-text search failure falls back to field matching silently", async () => {
    // spec「全文查詢失敗靜默退回欄位比對」：不彈錯、不阻斷輸入。
    vi.useFakeTimers();
    const ds = fakeDataSource({ searchWorkspace: vi.fn().mockRejectedValue(new Error("ipc down")) });
    const store = storeWith(ds);
    store.getState().setBoardQuery("x");
    await vi.advanceTimersByTimeAsync(200);
    expect(store.getState().searchHits).toEqual([]);
    expect(toastError).not.toHaveBeenCalled();
    vi.useRealTimers();
  });

  it("disposeSearch cancels the pending debounced search（卸載後在途去抖不觸發）", async () => {
    // 生命週期清理：元件卸載時取消漏出的去抖 timer 並作廢在途回填，
    // 杜絕搜尋在擁有它的 store 卸載後才開火所致的未處理例外。
    vi.useFakeTimers();
    const ds = fakeDataSource({ searchWorkspace: vi.fn().mockResolvedValue([]) });
    const store = storeWith(ds);
    store.getState().setBoardQuery("di");
    store.getState().disposeSearch();
    await vi.advanceTimersByTimeAsync(300);
    expect(ds.searchWorkspace).not.toHaveBeenCalled();
    vi.useRealTimers();
  });

  it("boardQuery and the archived-page query are independent", () => {
    // spec「搜尋字串…與已封存頁獨立」：各自設值互不覆蓋。
    const store = storeWith(fakeDataSource());
    store.getState().setBoardQuery("kanban");
    store.getState().setQuery("archived");
    expect(store.getState().boardQuery).toBe("kanban");
    expect(store.getState().query).toBe("archived");
    store.getState().setQuery("other");
    expect(store.getState().boardQuery).toBe("kanban");
  });

  it("archive confirm flow: request sets pending, confirm runs archive and clears", async () => {
    const ds = fakeDataSource();
    const store = storeWith(ds);
    store.getState().requestArchive("desktop-shell-and-browser");
    expect(store.getState().pendingArchive).toBe("desktop-shell-and-browser");
    await store.getState().confirmArchive();
    expect(store.getState().pendingArchive).toBeNull();
    expect(ds.runVerb).toHaveBeenCalledWith("archive", "desktop-shell-and-browser");
  });

  it("cancelArchive clears pending without running", () => {
    const ds = fakeDataSource();
    const store = storeWith(ds);
    store.getState().requestArchive("x");
    store.getState().cancelArchive();
    expect(store.getState().pendingArchive).toBeNull();
    expect(ds.runVerb).not.toHaveBeenCalled();
  });

  it("刪除、封存變更與封存討論成功時皆不發出 toast", async () => {
    const ds = fakeDataSource({
      runVerb: vi.fn().mockResolvedValue({ datedName: "2026-07-17-desktop-shell-and-browser" }),
    });
    const store = storeWith(ds);

    store.getState().requestDelete("desktop-shell-and-browser");
    await store.getState().confirmDelete();
    await store.getState().runVerb("archive", "desktop-shell-and-browser");
    store.getState().requestArchiveDiscussion("desktop-feedback-surface");
    await store.getState().confirmArchiveDiscussion();

    expect(toastError).not.toHaveBeenCalled();
  });

  it("四條 store 失敗路徑皆發出含主詞、core 錯誤與相同固定 id 的 error toast", async () => {
    const ds = fakeDataSource({
      deleteChange: vi.fn().mockRejectedValue(new Error("delete locked")),
      runVerb: vi.fn().mockRejectedValue(new Error("archive blocked")),
      archiveDiscussion: vi.fn().mockRejectedValue(new Error("discussion locked")),
      reorderCard: vi.fn().mockRejectedValue(new Error("order locked")),
    });
    const store = storeWith(ds);

    store.getState().requestDelete("delete-me");
    await store.getState().confirmDelete();
    await store.getState().runVerb("archive", "archive-me");
    store.getState().requestArchiveDiscussion("discussion-me");
    await store.getState().confirmArchiveDiscussion();
    await store.getState().reorderCard("change", "reorder-me", null, "next-change");

    expect(toastError).toHaveBeenCalledTimes(4);
    const expected = [
      ["delete-me", "delete locked"],
      ["archive-me", "archive blocked"],
      ["discussion-me", "discussion locked"],
      ["reorder-me", "order locked"],
    ];
    for (const [index, [subject, coreError]] of expected.entries()) {
      const [message, options] = toastError.mock.calls[index] as [string, { id?: string }];
      expect(message).toContain(subject);
      expect(message).toContain(coreError);
      expect(options.id).toEqual(expect.any(String));
    }
    expect(new Set(toastError.mock.calls.map((call) => call[1]?.id)).size).toBe(1);
    expect(ds.listChanges).toHaveBeenCalled();
  });

  it("core 錯誤為空、超長或 HTML-like 字串時皆安全交給 sonner", async () => {
    const longError = "x".repeat(20_000);
    const htmlLikeError = '<img src="x" onerror="globalThis.pwned=true">';
    const deleteChange = vi
      .fn()
      .mockRejectedValueOnce("")
      .mockRejectedValueOnce(longError)
      .mockRejectedValueOnce(htmlLikeError);
    const store = storeWith(fakeDataSource({ deleteChange }));

    for (const subject of ["empty-error", "long-error", "html-error"]) {
      store.getState().requestDelete(subject);
      await store.getState().confirmDelete();
    }

    expect(toastError).toHaveBeenCalledTimes(3);
    const messages = toastError.mock.calls.map(([message]) => message as string);
    expect(messages[0]).toContain("empty-error");
    expect(messages[0]).toContain("✗ ");
    expect(messages[1]).toContain(longError);
    expect(messages[2]).toContain(htmlLikeError);
    expect(messages.every((message) => typeof message === "string")).toBe(true);
  });

  it("runVerb analyze runs validate and analyze together into one drawer result", async () => {
    // design D1：「分析」單鍵雙動詞——validate＋analyze 合併為單一結構化抽屜結果。
    const report = {
      change_id: "x",
      dimensions: [{ dimension: "Ambiguity", status: "1 issue(s) found", finding_count: 1 }],
      findings: [
        { id: "AMB-1", dimension: "Ambiguity", severity: "Suggestion", location: "specs", summary: "s", recommendation: "r" },
      ],
      artifacts_analyzed: [],
      artifacts_missing: [],
    };
    const runVerb = vi.fn().mockImplementation((verb: string) =>
      Promise.resolve(verb === "validate" ? { valid: true, errors: [] } : report),
    );
    const ds = fakeDataSource({ runVerb });
    const store = storeWith(ds);
    await store.getState().runVerb("analyze", "desktop-shell-and-browser");
    expect(runVerb).toHaveBeenCalledWith("validate", "desktop-shell-and-browser");
    expect(runVerb).toHaveBeenCalledWith("analyze", "desktop-shell-and-browser");
    expect(store.getState().drawerVerb).toMatchObject({
      change: "desktop-shell-and-browser",
      validate: { valid: true },
      analyze: { findings: [{ dimension: "Ambiguity" }] },
    });
    expect(toastError).not.toHaveBeenCalled();
    expect(ds.listChanges).toHaveBeenCalled();
  });

  it("runVerb analyze failure surfaces the error in the drawer result", async () => {
    // 任一動詞失敗不靜默：error 進抽屜結果呈現 core 訊息。
    const ds = fakeDataSource({ runVerb: vi.fn().mockRejectedValue(new Error("parse boom")) });
    const store = storeWith(ds);
    await store.getState().runVerb("analyze", "desktop-shell-and-browser");
    expect(store.getState().drawerVerb?.change).toBe("desktop-shell-and-browser");
    expect(store.getState().drawerVerb?.error).toContain("parse boom");
  });

  it("clearDrawerVerb closes the analysis result; switching change still clears it", async () => {
    // design D2：分析面板可關閉——clearDrawerVerb 收合；既有「換 change 清空」不回歸。
    const ds = fakeDataSource();
    const store = storeWith(ds);
    await store.getState().refresh();
    await store.getState().runVerb("analyze", "desktop-shell-and-browser");
    expect(store.getState().drawerVerb).not.toBeNull();
    store.getState().clearDrawerVerb();
    expect(store.getState().drawerVerb).toBeNull();
    // 換 change 清空行為保留
    await store.getState().runVerb("analyze", "desktop-shell-and-browser");
    store.getState().openDetail("desktop-shell-and-browser");
    expect(store.getState().drawerVerb).toBeNull();
  });

  it("runVerb archive 成功維持靜默", async () => {
    const ds = fakeDataSource({ runVerb: vi.fn().mockResolvedValue({ datedName: "2026-07-09-x" }) });
    const store = storeWith(ds);
    await store.getState().runVerb("archive", "desktop-shell-and-browser");
    expect(toastError).not.toHaveBeenCalled();
    expect(store.getState().drawerVerb).toBeNull();
    expect(ds.listChanges).toHaveBeenCalled();
  });

  it("詳情抽屜中的封存成功會關閉抽屜，失敗則保留 change 上下文", async () => {
    const successStore = storeWith(fakeDataSource());
    await successStore.getState().refresh();
    successStore.getState().openDetail("desktop-shell-and-browser");
    successStore.getState().requestArchive("desktop-shell-and-browser");
    await successStore.getState().confirmArchive();
    expect(successStore.getState().detailChange).toBeNull();

    const failureStore = storeWith(fakeDataSource({
      runVerb: vi.fn().mockRejectedValue(new Error("archive prerequisites missing")),
    }));
    await failureStore.getState().refresh();
    failureStore.getState().openDetail("desktop-shell-and-browser");
    failureStore.getState().requestArchive("desktop-shell-and-browser");
    await failureStore.getState().confirmArchive();
    expect(failureStore.getState().detailChange?.name).toBe("desktop-shell-and-browser");
  });

  it("reorderCard passes neighbor ids through and refreshes on success", async () => {
    // design D5：store 動作把 kind/id/prevId/nextId 原樣交給 data source，成功後整批 refresh。
    const ds = fakeDataSource();
    const store = storeWith(ds);
    await store.getState().reorderCard("discussion", "slug-a", "prev-s", null);
    expect(ds.reorderCard).toHaveBeenCalledWith("discussion", "slug-a", "prev-s", null);
    expect(ds.listChanges).toHaveBeenCalled();
    expect(toastError).not.toHaveBeenCalled();
  });

  it("remote reorder 在 server 寫回完成前先更新可見卡片順序", async () => {
    const writing = deferred<void>();
    const listed: ChangeItem[] = [
      { name: "alpha", status: "proposed", totalTasks: 0, completedTasks: 0 },
      { name: "beta", status: "proposed", totalTasks: 0, completedTasks: 0 },
    ];
    const ds = fakeDataSource({
      listChanges: vi.fn().mockResolvedValue(listed),
      reorderCard: vi.fn().mockReturnValue(writing.promise),
    });
    const active = remoteSession(ds, "backend");
    const other = remoteSession(fakeDataSource(), "other");
    const store = storeWithRemoteSessions(active, other);
    await store.getState().refresh();

    const saving = store.getState().reorderCard("change", "beta", null, "alpha");

    expect(store.getState().changes.map((change) => change.name)).toEqual(["beta", "alpha"]);
    writing.resolve();
    await saving;
  });

  it("remote reorder 寫回失敗且 refresh 失敗時仍立即還原最後成功順序", async () => {
    const writing = deferred<void>();
    const listed: ChangeItem[] = [
      { name: "alpha", status: "proposed", totalTasks: 0, completedTasks: 0 },
      { name: "beta", status: "proposed", totalTasks: 0, completedTasks: 0 },
    ];
    const listChanges = vi.fn().mockResolvedValueOnce(listed).mockRejectedValue(new Error("offline"));
    const ds = fakeDataSource({ listChanges, reorderCard: vi.fn().mockReturnValue(writing.promise) });
    const active = remoteSession(ds, "backend");
    const other = remoteSession(fakeDataSource(), "other");
    const store = storeWithRemoteSessions(active, other);
    await store.getState().refresh();

    const saving = store.getState().reorderCard("change", "beta", null, "alpha");
    expect(store.getState().changes.map((change) => change.name)).toEqual(["beta", "alpha"]);
    writing.reject(new Error("write failed"));
    await saving;

    expect(store.getState().changes.map((change) => change.name)).toEqual(["alpha", "beta"]);
    expect(toastError).toHaveBeenCalledTimes(1);
  });

  it("local reorder 維持既有流程，不在資料源寫回前改動可見順序", async () => {
    const writing = deferred<void>();
    const listed: ChangeItem[] = [
      { name: "alpha", status: "proposed", totalTasks: 0, completedTasks: 0 },
      { name: "beta", status: "proposed", totalTasks: 0, completedTasks: 0 },
    ];
    const ds = fakeDataSource({
      listChanges: vi.fn().mockResolvedValue(listed),
      reorderCard: vi.fn().mockReturnValue(writing.promise),
    });
    const store = storeWith(ds);
    await store.getState().refresh();

    const saving = store.getState().reorderCard("change", "beta", null, "alpha");

    expect(store.getState().changes.map((change) => change.name)).toEqual(["alpha", "beta"]);
    writing.resolve();
    await saving;
  });

  it("detailSpec 開閉 action 比照 detailChange（spec-archive-drawer design D2）", async () => {
    const store = storeWith(fakeDataSource());
    await store.getState().refresh();
    expect(store.getState().detailSpec).toBeNull();
    store.getState().openSpec("desktop-app");
    expect(store.getState().detailSpec).toBe("desktop-app");
    store.getState().closeSpec();
    expect(store.getState().detailSpec).toBeNull();
  });

  it("detailArchived 開閉 action：change 與 discussion 兩型 discriminated target", () => {
    const store = storeWith(fakeDataSource());
    expect(store.getState().detailArchived).toBeNull();
    store.getState().openArchived({ kind: "change", datedName: "2026-07-04-x" });
    expect(store.getState().detailArchived).toEqual({ kind: "change", datedName: "2026-07-04-x" });
    store.getState().openArchived({ kind: "discussion", slug: "old-topic" });
    expect(store.getState().detailArchived).toEqual({ kind: "discussion", slug: "old-topic" });
    store.getState().closeArchived();
    expect(store.getState().detailArchived).toBeNull();
  });

  it("detail 抽屜互斥：任一 open* 動作清除其他三個 detail 欄位（後開者取代先開者）", async () => {
    // 規格「detail 抽屜互斥」Example 表：討論→變更詳情→規格→封存→討論。
    const ds = fakeDataSource({
      listDiscussions: vi.fn().mockResolvedValue({
        active: [
          { slug: "topic-a", topic: "t", status: "open", rounds: 1, created: "2026-07-17", promotedTo: [] },
        ],
        archived: [],
      }),
    });
    const store = storeWith(ds);
    await store.getState().refresh();

    store.getState().openDiscussion("topic-a");
    expect(store.getState().detailDiscussion?.slug).toBe("topic-a");
    store.getState().openDetail("desktop-shell-and-browser");
    expect(store.getState().detailChange?.name).toBe("desktop-shell-and-browser");
    expect(store.getState().detailDiscussion).toBeNull();

    store.getState().openSpec("desktop-app");
    expect(store.getState().detailSpec).toBe("desktop-app");
    expect(store.getState().detailChange).toBeNull();

    store.getState().openArchived({ kind: "change", datedName: "2026-07-04-x" });
    expect(store.getState().detailArchived).toEqual({ kind: "change", datedName: "2026-07-04-x" });
    expect(store.getState().detailSpec).toBeNull();

    store.getState().openDiscussion("topic-a");
    expect(store.getState().detailDiscussion?.slug).toBe("topic-a");
    expect(store.getState().detailArchived).toBeNull();
  });

  it("detail 抽屜互斥：取代變更詳情抽屜時 drawerVerb 一併清空（比照 closeDetail）", async () => {
    const store = storeWith(fakeDataSource());
    await store.getState().refresh();
    store.getState().openDetail("desktop-shell-and-browser");
    await store.getState().runVerb("analyze", "desktop-shell-and-browser");
    expect(store.getState().drawerVerb).not.toBeNull();
    store.getState().openSpec("desktop-app");
    expect(store.getState().detailChange).toBeNull();
    expect(store.getState().drawerVerb).toBeNull();
  });

});

// ---- 系統匣樣式由平台決定（tray-macos-panel-only：規格「系統匣圖示與原生選單」平台分流） ----

describe("系統匣樣式由平台決定", () => {
  const setUA = (value: string) =>
    Object.defineProperty(window.navigator, "userAgent", { value, configurable: true });
  const MAC_UA = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)";
  const WIN_UA = "Mozilla/5.0 (Windows NT 10.0; Win64; x64)";

  it("macOS 初值為 panel、非 macOS 為 native-menu，且不讀 localStorage 舊偏好", () => {
    localStorage.setItem("speclink.trayStyle", "native-menu"); // 舊偏好殘留：不再被讀取
    setUA(MAC_UA);
    expect(storeWith(fakeDataSource()).getState().trayStyle).toBe("panel");
    setUA(WIN_UA);
    expect(storeWith(fakeDataSource()).getState().trayStyle).toBe("native-menu");
    localStorage.removeItem("speclink.trayStyle");
  });

  it("panelFallback 退回 native-menu、浮出單行錯誤，且不寫 localStorage（規格「面板樣式（macOS）」失敗退回）", () => {
    setUA(MAC_UA);
    const store = storeWith(fakeDataSource());
    store.getState().panelFallback("tray panel window creation failed: boom");
    expect(store.getState().trayStyle).toBe("native-menu");
    expect(store.getState().trayPanelError).toBe("tray panel window creation failed: boom");
    expect(localStorage.getItem("speclink.trayStyle")).toBeNull();
  });
});

// --- device login 分段輪詢（規格「device login 預設與 PAT fallback」的等待授權面；
// design 決策二：輪詢節奏歸前端 store） ---

describe("device login 分段輪詢", () => {
  const ORIGIN = "http://localhost:8080";
  const AUTH = {
    deviceCode: "dev-code-1",
    userCode: "ABCD-EFGH",
    verificationUri: "http://localhost:8080/activate",
    expiresIn: 900,
    interval: 5,
  };

  /** 假 connections adapter：啟動段回等待授權，觀測段由測試逐次餵狀態。 */
  function fakeConnections(over: Partial<ConnectionsAdapter> = {}): ConnectionsAdapter {
    return {
      list: vi.fn().mockResolvedValue([
        { id: "conn_1", origin: ORIGIN, name: "本地", loggedIn: false },
      ]),
      add: vi.fn(),
      remove: vi.fn(),
      deviceLoginStart: vi.fn().mockResolvedValue({
        status: "awaitingApproval",
        authorization: AUTH,
      }),
      deviceLoginObserve: vi.fn().mockResolvedValue({ status: "pending", slowDown: false }),
      patLogin: vi.fn(),
      logout: vi.fn(),
      scopes: vi.fn().mockResolvedValue({ projects: [] }),
      inspectCheckout: vi.fn(),
      bindCheckout: vi.fn(),
      ...over,
    };
  }

  function storeWithConnections(adapter: ConnectionsAdapter) {
    return createAppStore({
      createSession: () => ({}) as WorkspaceSession,
      connections: adapter,
    });
  }

  it("啟動段回等待授權即進入等待授權狀態，並依 server 間隔排程單次觀測", async () => {
    vi.useFakeTimers();
    const adapter = fakeConnections();
    const store = storeWithConnections(adapter);
    await store.getState().loginConnection(ORIGIN);

    const phase = store.getState().connectionPhases[ORIGIN];
    expect(phase).toMatchObject({
      kind: "awaitingApproval",
      userCode: AUTH.userCode,
      verificationUri: AUTH.verificationUri,
    });
    // 倒數以截止時刻為準（非累計計數）：睡眠後醒來仍判得出逾時。
    expect(phase.kind === "awaitingApproval" && phase.expiresAt).toBeGreaterThan(Date.now());

    // 排程尚未到點前不觀測；到點後恰觀測一次。
    expect(adapter.deviceLoginObserve).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(5000);
    expect(adapter.deviceLoginObserve).toHaveBeenCalledTimes(1);
    expect(adapter.deviceLoginObserve).toHaveBeenCalledWith(ORIGIN, AUTH.deviceCode);
    vi.useRealTimers();
  });

  it("收到 slow_down 即加大間隔", async () => {
    vi.useFakeTimers();
    const adapter = fakeConnections({
      deviceLoginObserve: vi
        .fn()
        .mockResolvedValueOnce({ status: "pending", slowDown: true })
        .mockResolvedValue({ status: "pending", slowDown: false }),
    });
    const store = storeWithConnections(adapter);
    await store.getState().loginConnection(ORIGIN);

    await vi.advanceTimersByTimeAsync(5000);
    expect(adapter.deviceLoginObserve).toHaveBeenCalledTimes(1);
    // 加大後，原本的間隔到點時還不該觀測。
    await vi.advanceTimersByTimeAsync(5000);
    expect(adapter.deviceLoginObserve).toHaveBeenCalledTimes(1);
    await vi.advanceTimersByTimeAsync(5000);
    expect(adapter.deviceLoginObserve).toHaveBeenCalledTimes(2);
    vi.useRealTimers();
  });

  it("觀測到核准即登入完成並停止排程", async () => {
    vi.useFakeTimers();
    const adapter = fakeConnections({
      list: vi
        .fn()
        .mockResolvedValue([{ id: "conn_1", origin: ORIGIN, name: "本地", loggedIn: true }]),
      deviceLoginObserve: vi.fn().mockResolvedValue({ status: "loggedIn", display: "Dev" }),
    });
    const store = storeWithConnections(adapter);
    await store.getState().loginConnection(ORIGIN);
    await vi.advanceTimersByTimeAsync(5000);

    expect(store.getState().connectionPhases[ORIGIN]).toEqual({ kind: "idle" });
    await vi.advanceTimersByTimeAsync(30000);
    expect(adapter.deviceLoginObserve).toHaveBeenCalledTimes(1);
    vi.useRealTimers();
  });

  it("取消即停止排程並回未登入，不留等待狀態", async () => {
    vi.useFakeTimers();
    const adapter = fakeConnections();
    const store = storeWithConnections(adapter);
    await store.getState().loginConnection(ORIGIN);

    store.getState().cancelLogin(ORIGIN);
    expect(store.getState().connectionPhases[ORIGIN]).toEqual({ kind: "idle" });
    await vi.advanceTimersByTimeAsync(30000);
    expect(adapter.deviceLoginObserve).not.toHaveBeenCalled();
    vi.useRealTimers();
  });

  it("觀測到拒絕與逾時各自浮為可讀狀態並停止排程", async () => {
    for (const [status, message] of [
      ["denied", "已在瀏覽器拒絕授權"],
      ["expired", "授權逾時"],
    ] as const) {
      vi.useFakeTimers();
      const adapter = fakeConnections({
        deviceLoginObserve: vi.fn().mockResolvedValue({ status }),
      });
      const store = storeWithConnections(adapter);
      await store.getState().loginConnection(ORIGIN);
      await vi.advanceTimersByTimeAsync(5000);

      const phase = store.getState().connectionPhases[ORIGIN];
      expect(phase.kind).toBe("error");
      expect(phase.kind === "error" && phase.message).toContain(message);
      await vi.advanceTimersByTimeAsync(30000);
      expect(adapter.deviceLoginObserve).toHaveBeenCalledTimes(1);
      vi.useRealTimers();
    }
  });

  it("倒數歸零即以逾時收場，不再觀測", async () => {
    vi.useFakeTimers();
    const adapter = fakeConnections({
      deviceLoginStart: vi.fn().mockResolvedValue({
        status: "awaitingApproval",
        // 有效期限比一個輪詢間隔還短：第一次到點時已過期。
        authorization: { ...AUTH, expiresIn: 3 },
      }),
    });
    const store = storeWithConnections(adapter);
    await store.getState().loginConnection(ORIGIN);

    await vi.advanceTimersByTimeAsync(5000);
    const phase = store.getState().connectionPhases[ORIGIN];
    expect(phase.kind).toBe("error");
    expect(phase.kind === "error" && phase.message).toContain("授權逾時");
    expect(adapter.deviceLoginObserve).not.toHaveBeenCalled();
    vi.useRealTimers();
  });
});

// --- 指令檔過期提示（desktop-instruction-staleness-prompt；規格「指令檔過期提示」） ---

const STALE_PROBE = {
  status: "stale" as const,
  currentVersion: "v1.3.0",
  tools: [{ tool: "claude", workspaceVersion: "v0.9.0", stale: true, missing: false }],
  differingFiles: ["CLAUDE.md", ".claude/skills/speclink-apply/SKILL.md"],
};

const MISSING_PROBE = {
  ...STALE_PROBE,
  status: "missing" as const,
  tools: [{ tool: "claude", workspaceVersion: null, stale: false, missing: true }],
};

function fakeInstructionWorkspace(over: Partial<WorkspaceAdapter> = {}) {
  return {
    openProject: vi.fn().mockResolvedValue({ status: "project", root: "A", name: "a" }),
    initProject: vi.fn(),
    adoptProject: vi.fn(),
    startupDir: vi.fn().mockRejectedValue("none"),
    projectStats: vi.fn().mockResolvedValue({ pendingWrapUp: 0 }),
    probeInstructions: vi.fn().mockResolvedValue(STALE_PROBE),
    updateInstructions: vi.fn().mockResolvedValue(undefined),
    watchWorkspace: vi.fn().mockResolvedValue(undefined),
    pickFolder: vi.fn().mockResolvedValue(null),
    ...over,
  } as unknown as WorkspaceAdapter;
}

/** 以單一 local session 預置 store 並注入 workspace 探測面。 */
function storeWithInstructionProbe(ws: WorkspaceAdapter, ds = fakeDataSource()) {
  const store = createAppStore({
    createSession: (root, name) => fakeSession(ds, root, name),
    workspace: ws,
  });
  const session = fakeSession(ds);
  store.setState({
    tabs: [{ locator: session.locator, name: session.descriptor.name }],
    sessions: { [session.id]: session },
    activeKey: session.id,
  });
  return store;
}

describe("指令檔過期提示的顯示裁決", () => {
  beforeEach(() => {
    localStorage.removeItem("speclink.instructionSkips");
  });

  it("過期且未略過：以更新語意提示並帶差異檔數", async () => {
    const store = storeWithInstructionProbe(fakeInstructionWorkspace());
    await store.getState().refreshInstructionPrompt();
    expect(store.getState().instructionPrompt).toEqual({
      kind: "stale",
      fileCount: 2,
      version: "v1.3.0",
    });
  });

  it("缺失且未略過：以安裝語意提示", async () => {
    const store = storeWithInstructionProbe(
      fakeInstructionWorkspace({ probeInstructions: vi.fn().mockResolvedValue(MISSING_PROBE) }),
    );
    await store.getState().refreshInstructionPrompt();
    expect(store.getState().instructionPrompt?.kind).toBe("missing");
  });

  it("保留現狀後同版不再提示，且不寫入專案內任何檔案", async () => {
    const ws = fakeInstructionWorkspace();
    const store = storeWithInstructionProbe(ws);
    await store.getState().refreshInstructionPrompt();
    store.getState().dismissInstructionPrompt();
    expect(store.getState().instructionPrompt).toBeNull();

    await store.getState().refreshInstructionPrompt();
    expect(store.getState().instructionPrompt).toBeNull();
    expect(ws.updateInstructions).not.toHaveBeenCalled();
  });

  it("缺失態的保留現狀與過期共用同一略過記憶", async () => {
    const store = storeWithInstructionProbe(
      fakeInstructionWorkspace({ probeInstructions: vi.fn().mockResolvedValue(MISSING_PROBE) }),
    );
    await store.getState().refreshInstructionPrompt();
    store.getState().dismissInstructionPrompt();
    await store.getState().refreshInstructionPrompt();
    expect(store.getState().instructionPrompt).toBeNull();
  });

  it("已略過舊版、產物層版號變動後重新提示", async () => {
    const probe = vi.fn().mockResolvedValue(STALE_PROBE);
    const store = storeWithInstructionProbe(fakeInstructionWorkspace({ probeInstructions: probe }));
    await store.getState().refreshInstructionPrompt();
    store.getState().dismissInstructionPrompt();

    probe.mockResolvedValue({ ...STALE_PROBE, currentVersion: "v1.4.0" });
    await store.getState().refreshInstructionPrompt();
    expect(store.getState().instructionPrompt?.version).toBe("v1.4.0");
  });

  it("無法判定：不提示且不記入略過（後續判過期仍提示）", async () => {
    const probe = vi.fn().mockResolvedValue({
      status: "unknown",
      currentVersion: "v1.3.0",
      tools: [],
      differingFiles: [],
    });
    const store = storeWithInstructionProbe(fakeInstructionWorkspace({ probeInstructions: probe }));
    await store.getState().refreshInstructionPrompt();
    expect(store.getState().instructionPrompt).toBeNull();
    expect(localStorage.getItem("speclink.instructionSkips")).toBeNull();

    probe.mockResolvedValue(STALE_PROBE);
    await store.getState().refreshInstructionPrompt();
    expect(store.getState().instructionPrompt?.kind).toBe("stale");
  });

  it("現版：不提示", async () => {
    const store = storeWithInstructionProbe(
      fakeInstructionWorkspace({
        probeInstructions: vi.fn().mockResolvedValue({
          status: "current",
          currentVersion: "v1.3.0",
          tools: [],
          differingFiles: [],
        }),
      }),
    );
    await store.getState().refreshInstructionPrompt();
    expect(store.getState().instructionPrompt).toBeNull();
  });

  it("remote 分頁不執行探測", async () => {
    const ws = fakeInstructionWorkspace();
    const ds = fakeDataSource();
    const store = createAppStore({
      createSession: () => {
        throw new Error("remote 分頁不建 local session");
      },
      workspace: ws,
    });
    const remote = remoteSession(ds, "repo-a");
    store.setState({
      tabs: [{ locator: remote.locator, name: remote.descriptor.name }],
      sessions: { [remote.id]: remote },
      activeKey: remote.id,
    });

    await store.getState().refreshInstructionPrompt();
    expect(ws.probeInstructions).not.toHaveBeenCalled();
    expect(store.getState().instructionPrompt).toBeNull();
  });

  it("更新成功：整套再生後重查，提示消失", async () => {
    const probe = vi.fn().mockResolvedValue(STALE_PROBE);
    const ws = fakeInstructionWorkspace({ probeInstructions: probe });
    const store = storeWithInstructionProbe(ws);
    await store.getState().refreshInstructionPrompt();

    probe.mockResolvedValue({
      status: "current",
      currentVersion: "v1.3.0",
      tools: [],
      differingFiles: [],
    });
    await store.getState().applyInstructionUpdate();

    expect(ws.updateInstructions).toHaveBeenCalledWith("A");
    expect(store.getState().instructionPrompt).toBeNull();
    expect(store.getState().instructionUpdateError).toBeNull();
  });

  it("更新失敗：錯誤留在提示原位、提示保持可重試", async () => {
    const ws = fakeInstructionWorkspace({
      updateInstructions: vi.fn().mockRejectedValue("CLAUDE.md: permission denied"),
    });
    const store = storeWithInstructionProbe(ws);
    await store.getState().refreshInstructionPrompt();
    await store.getState().applyInstructionUpdate();

    expect(store.getState().instructionUpdateError).toContain("permission denied");
    expect(store.getState().instructionPrompt).not.toBeNull();
  });

  it("外部 speclink update 後（workspace-changed 重查）提示自然消失", async () => {
    const probe = vi.fn().mockResolvedValue(STALE_PROBE);
    const store = storeWithInstructionProbe(fakeInstructionWorkspace({ probeInstructions: probe }));
    await store.getState().refreshInstructionPrompt();
    expect(store.getState().instructionPrompt).not.toBeNull();

    // 使用者於終端跑 speclink update：受管檔成為現版，重查即收合。
    probe.mockResolvedValue({
      status: "current",
      currentVersion: "v1.3.0",
      tools: [],
      differingFiles: [],
    });
    await store.getState().refreshInstructionPrompt();
    expect(store.getState().instructionPrompt).toBeNull();
  });
});

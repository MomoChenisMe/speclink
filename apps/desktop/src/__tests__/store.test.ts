import { afterEach, beforeEach, describe, it, expect, vi } from "vitest";
import type { ChangeItem, ManualIndex, SearchHit, SpeclinkDataSource, StatusReport } from "@speclink/ui";

import { createAppStore, openTicketStation } from "../store";
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

/** 兩站工單並存的 change——封存守門要連過兩站的測試共用這筆。 */
const BOTH_TICKETS_CHANGE = {
  name: "desktop-shell-and-browser",
  status: "in-progress",
  totalTasks: 26,
  completedTasks: 26,
  reviewStatus: "inReview" as const,
  verifyStatus: "inVerify" as const,
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
    listManualPages: vi
      .fn()
      .mockResolvedValue({ present: false, reason: null, pages: [], uncoveredNew: [], malformed: [] }),
    getManualPage: vi.fn().mockResolvedValue(null),
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
  const store = trackedAppStore({ createSession: (root, name) => fakeSession(ds, root, name) });
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
  const store = trackedAppStore({
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

// 追蹤本檔建立的每個 store，測後統一清掉搜尋去抖計時器：測試結束後才開火的
// 200ms 計時器會打在已拆除的 mock 上（searchWorkspace 回 undefined），成為整包
// 測試的 unhandled error flake——535 全過仍紅整個 run。
const trackedStores: ReturnType<typeof createAppStore>[] = [];
function trackedAppStore(...args: Parameters<typeof createAppStore>) {
  const store = createAppStore(...args);
  trackedStores.push(store);
  return store;
}
afterEach(() => {
  for (const store of trackedStores.splice(0)) store.getState().disposeSearch();
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
    const store = trackedAppStore({
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

  it("三選項「放棄審查」先刪工單再走既有封存流程", async () => {
    // spec「封存入口的未結工單三選項」：等同 review discard 後封存——工單先刪，
    // 之後照既有 archive 動詞走（含成功靜默與重載）。
    const discardReview = vi.fn().mockResolvedValue(undefined);
    const ds = fakeDataSource({ discardReview });
    const store = storeWith(ds);
    store.getState().requestArchive("desktop-shell-and-browser");
    await store.getState().confirmArchiveDiscardTicket("review");
    expect(discardReview).toHaveBeenCalledWith("desktop-shell-and-browser");
    expect(ds.runVerb).toHaveBeenCalledWith("archive", "desktop-shell-and-browser");
    expect(store.getState().pendingArchive).toBeNull();
    expect(toastError).not.toHaveBeenCalled();
  });

  it("「放棄審查」刪工單失敗即不封存，且提示說的是刪工單而非封存", async () => {
    const ds = fakeDataSource({
      discardReview: vi.fn().mockRejectedValue(new Error("offline")),
    });
    const store = storeWith(ds);
    store.getState().requestArchive("desktop-shell-and-browser");
    await store.getState().confirmArchiveDiscardTicket("review");
    expect(ds.runVerb).not.toHaveBeenCalledWith("archive", "desktop-shell-and-browser");
    expect(toastError).toHaveBeenCalledWith(
      expect.stringContaining("放棄審查失敗"),
      expect.anything(),
    );
  });

  it("後端不支援三選項處置時出聲，不靜默吞掉", async () => {
    // remote 等未實作的後端：對話框關掉卻什麼都沒發生，使用者無從得知。
    for (const action of ["confirmArchiveDiscardTicket", "confirmArchiveCarryTicket"] as const) {
      toastError.mockClear();
      const store = storeWith(fakeDataSource());
      store.getState().requestArchive("desktop-shell-and-browser");
      await store.getState()[action]("review");
      expect(store.getState().pendingArchive).toBeNull();
      expect(toastError).toHaveBeenCalled();
    }
  });

  it("不支援處置的提示不含工程方法名", async () => {
    // openspec/LANGUAGE.md：工程詞不出現在使用者可見文案。
    for (const [action, method] of [
      ["confirmArchiveDiscardTicket", "discardReview"],
      ["confirmArchiveCarryTicket", "archiveCarry"],
    ] as const) {
      toastError.mockClear();
      const store = storeWith(fakeDataSource());
      store.getState().requestArchive("desktop-shell-and-browser");
      await store.getState()[action]("review");
      const [message] = toastError.mock.calls[0] as [string];
      expect(message).not.toContain(method);
    }
  });

  it("「放棄審查」後封存失敗的提示點名審查已放棄，而非單純封存失敗", async () => {
    // 工單在 discard 已刪；封存此後被拒時，使用者必須知道審查紀錄已不在。
    const ds = fakeDataSource({
      discardReview: vi.fn().mockResolvedValue(undefined),
      runVerb: vi.fn().mockRejectedValue(new Error("tasks incomplete")),
    });
    const store = storeWith(ds);
    store.getState().requestArchive("desktop-shell-and-browser");
    await store.getState().confirmArchiveDiscardTicket("review");
    expect(toastError).toHaveBeenCalledWith(
      expect.stringContaining("審查已放棄"),
      expect.anything(),
    );
  });

  it("處置按鈕連點只發一次處置，不重複封存也不誤報失敗", async () => {
    // 對話框在 await 期間保持開啟且按鈕未鎖——連點的第二擊必須被忽略，否則
    // 第二次 discard 對已刪的工單失敗，錯誤提示配上「其實已封存」的矛盾。
    const discardReview = vi.fn().mockResolvedValue(undefined);
    const ds = fakeDataSource({ discardReview });
    const store = storeWith(ds);
    store.getState().requestArchive("desktop-shell-and-browser");
    await Promise.all([
      store.getState().confirmArchiveDiscardTicket("review"),
      store.getState().confirmArchiveDiscardTicket("review"),
    ]);
    expect(discardReview).toHaveBeenCalledTimes(1);
    const archiveCalls = (ds.runVerb as ReturnType<typeof vi.fn>).mock.calls.filter(
      ([verb]) => verb === "archive",
    );
    expect(archiveCalls).toHaveLength(1);
    expect(toastError).not.toHaveBeenCalled();
  });

  it("先放棄審查、驗證照樣帶走：封存失敗仍點名審查紀錄已不在", async () => {
    // 混合處置：review.md 已刪且不可回復——最終封存無論由哪站收尾，失敗提示
    // 都不得退化成單純「封存失敗」。
    const ds = fakeDataSource({
      discardReview: vi.fn().mockResolvedValue(undefined),
      archiveCarry: vi.fn().mockRejectedValue(new Error("refused")),
      listChanges: vi.fn().mockResolvedValue([BOTH_TICKETS_CHANGE]),
    });
    const store = storeWith(ds);
    await store.getState().refresh();
    store.getState().requestArchive("desktop-shell-and-browser");
    await store.getState().confirmArchiveDiscardTicket("review");
    await store.getState().confirmArchiveCarryTicket("verify");
    expect(toastError).toHaveBeenCalledWith(
      expect.stringContaining("審查已放棄"),
      expect.anything(),
    );
  });

  it("兩站都放棄後封存失敗：提示點名兩站紀錄都已不在", async () => {
    const ds = fakeDataSource({
      discardReview: vi.fn().mockResolvedValue(undefined),
      discardVerify: vi.fn().mockResolvedValue(undefined),
      runVerb: vi.fn().mockRejectedValue(new Error("refused")),
      listChanges: vi.fn().mockResolvedValue([BOTH_TICKETS_CHANGE]),
    });
    const store = storeWith(ds);
    await store.getState().refresh();
    store.getState().requestArchive("desktop-shell-and-browser");
    await store.getState().confirmArchiveDiscardTicket("review");
    await store.getState().confirmArchiveDiscardTicket("verify");
    expect(toastError).toHaveBeenCalledWith(
      expect.stringContaining("審查與驗證已放棄"),
      expect.anything(),
    );
  });

  it("處置途中工作區分頁已關閉：出聲而非靜默結束", async () => {
    // settleStation 的 dataSource 早退不得靜默——其他處置路徑此情境都會出聲。
    const store = storeWith(fakeDataSource());
    store.getState().requestArchive("desktop-shell-and-browser");
    store.setState({ sessions: {}, activeKey: null });
    await store.getState().confirmArchiveCarryTicket("review");
    expect(store.getState().pendingArchive).toBeNull();
    expect(toastError).toHaveBeenCalled();
  });

  it("「照樣帶走」封存成功會關閉抽屜，失敗則保留 change 上下文", async () => {
    // 三選項的第三條與既有 archive 同為封存終局——抽屜不得停在已封存的 change。
    const archiveCarry = vi.fn().mockResolvedValue(undefined);
    const successStore = storeWith(fakeDataSource({ archiveCarry }));
    await successStore.getState().refresh();
    successStore.getState().openDetail("desktop-shell-and-browser");
    successStore.getState().requestArchive("desktop-shell-and-browser");
    await successStore.getState().confirmArchiveCarryTicket("review");
    expect(archiveCarry).toHaveBeenCalledWith("desktop-shell-and-browser", true, false);
    expect(successStore.getState().pendingArchive).toBeNull();
    expect(successStore.getState().detailChange).toBeNull();

    const failureStore = storeWith(fakeDataSource({
      archiveCarry: vi.fn().mockRejectedValue(new Error("archive refused")),
    }));
    await failureStore.getState().refresh();
    failureStore.getState().openDetail("desktop-shell-and-browser");
    failureStore.getState().requestArchive("desktop-shell-and-browser");
    await failureStore.getState().confirmArchiveCarryTicket("review");
    expect(toastError).toHaveBeenCalled();
    expect(failureStore.getState().detailChange?.name).toBe("desktop-shell-and-browser");
  });

  it("雙工單並存時兩站分別處置後才封存（旗標一次帶齊）", async () => {
    // spec desktop-app Scenario「雙工單並存的封存」：處置第一站不封存，兩站
    // 都處置完才真的封存，且兩個帶走旗標一起上路。
    const archiveCarry = vi.fn().mockResolvedValue(undefined);
    const ds = fakeDataSource({
      archiveCarry,
      listChanges: vi.fn().mockResolvedValue([
        {
          name: "desktop-shell-and-browser",
          status: "in-progress",
          totalTasks: 26,
          completedTasks: 26,
          reviewStatus: "inReview",
          verifyStatus: "inVerify",
        },
      ]),
    });
    const store = storeWith(ds);
    await store.getState().refresh();
    store.getState().requestArchive("desktop-shell-and-browser");

    await store.getState().confirmArchiveCarryTicket("review");
    expect(archiveCarry).not.toHaveBeenCalled();
    expect(store.getState().pendingArchive).toBe("desktop-shell-and-browser");
    expect(
      openTicketStation(
        store.getState().changes,
        "desktop-shell-and-browser",
        store.getState().pendingArchiveSettled,
      ),
    ).toBe("verify");

    await store.getState().confirmArchiveCarryTicket("verify");
    expect(archiveCarry).toHaveBeenCalledWith("desktop-shell-and-browser", true, true);
    expect(store.getState().pendingArchive).toBeNull();
  });

  it("兩站都選「放棄」時不帶旗標，走一般封存動詞", async () => {
    const discardReview = vi.fn().mockResolvedValue(undefined);
    const discardVerify = vi.fn().mockResolvedValue(undefined);
    const ds = fakeDataSource({
      discardReview,
      discardVerify,
      listChanges: vi.fn().mockResolvedValue([
        {
          name: "desktop-shell-and-browser",
          status: "in-progress",
          totalTasks: 26,
          completedTasks: 26,
          reviewStatus: "inReview",
          verifyStatus: "inVerify",
        },
      ]),
    });
    const store = storeWith(ds);
    await store.getState().refresh();
    store.getState().requestArchive("desktop-shell-and-browser");
    await store.getState().confirmArchiveDiscardTicket("review");
    expect(ds.runVerb).not.toHaveBeenCalledWith("archive", "desktop-shell-and-browser");
    await store.getState().confirmArchiveDiscardTicket("verify");
    expect(discardReview).toHaveBeenCalledWith("desktop-shell-and-browser");
    expect(discardVerify).toHaveBeenCalledWith("desktop-shell-and-browser");
    expect(ds.runVerb).toHaveBeenCalledWith("archive", "desktop-shell-and-browser");
  });

  it("openTicketStation 的順序固定：審查站在前、驗證站在後", () => {
    const both = [
      {
        name: "c",
        status: "in-progress",
        totalTasks: 1,
        completedTasks: 1,
        reviewStatus: "inReview" as const,
        verifyStatus: "inVerify" as const,
      },
    ];
    const none = { review: false, verify: false };
    expect(openTicketStation(both, "c", none)).toBe("review");
    expect(openTicketStation(both, "c", { review: true, verify: false })).toBe("verify");
    expect(openTicketStation(both, "c", { review: true, verify: true })).toBeNull();
    expect(openTicketStation(both, "ghost", none)).toBeNull();
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

  // 回退 fallback 快照必須是完整的 WorkspaceSnapshot：缺 loadFailed 鍵的快照
  // 進了 Map，之後的翻頁 spread 不會覆蓋該欄位，前一個 workspace 的失敗記號
  // 就漏進來——沒失敗過的 workspace 顯示失敗提示。
  it("remote reorder 寫回失敗的回退快照保留 loadFailed 欄位，不受他 workspace 汙染", async () => {
    const gate = deferred<ChangeItem[]>();
    const dsA = fakeDataSource({
      listChanges: vi.fn(() => gate.promise),
      reorderCard: vi.fn().mockRejectedValue(new Error("write failed")),
    });
    const dsB = fakeDataSource({
      listChanges: vi.fn().mockRejectedValue(new Error("offline")),
    });
    const a = remoteSession(dsA, "repo-a");
    const b = remoteSession(dsB, "repo-b");
    const store = storeWithRemoteSessions(a, b);

    // A 的首批載入在途（尚無快照）時拖排失敗 → 回退走 fallback 字面量。
    void store.getState().refresh();
    const saving = store.getState().reorderCard("change", "alpha", null, null);

    // 翻到 B 且 B 載入失敗 → 現任 loadFailed 為 true。
    await store.getState().activateTab(b.id);
    expect(store.getState().loadFailed).toBe(true);

    // 切回 A：A 從未失敗過，翻頁當下不得顯示失敗記號。
    const back = store.getState().activateTab(a.id);
    expect(store.getState().loadFailed).toBe(false);

    gate.resolve([]);
    await Promise.all([saving, back]);
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

  it("openDetail 於已封存頁把底層頁面切回看板（規格「變更與討論抽屜開啟時底層落回看板」）", async () => {
    const store = storeWith(fakeDataSource());
    await store.getState().refresh();
    store.getState().setBoardView("archived");

    store.getState().openDetail("desktop-shell-and-browser");

    expect(store.getState().boardView).toBe("board");
    expect(store.getState().detailChange?.name).toBe("desktop-shell-and-browser");
  });

  it("openDiscussion 於規格頁把底層頁面切回看板", async () => {
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
    store.getState().setBoardView("specs");

    store.getState().openDiscussion("topic-a");

    expect(store.getState().boardView).toBe("board");
    expect(store.getState().detailDiscussion?.slug).toBe("topic-a");
  });

  it("openSpec 與 openArchived 不切頁——宿主頁面即規格頁與已封存頁", async () => {
    const store = storeWith(fakeDataSource());
    await store.getState().refresh();

    store.getState().setBoardView("specs");
    store.getState().openSpec("desktop-app");
    expect(store.getState().boardView).toBe("specs");

    store.getState().setBoardView("archived");
    store.getState().openArchived({ kind: "change", datedName: "2026-07-04-x" });
    expect(store.getState().boardView).toBe("archived");
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
    return trackedAppStore({
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
  tools: [{ tool: "claude", workspaceVersion: "v0.9.0", stale: true, newer: false, missing: false }],
  differingFiles: ["CLAUDE.md", ".claude/skills/speclink-apply/SKILL.md"],
};

const MISSING_PROBE = {
  ...STALE_PROBE,
  status: "missing" as const,
  tools: [{ tool: "claude", workspaceVersion: null, stale: false, newer: false, missing: true }],
};

/** 專案檔案領先引擎：app 本體是舊版，任何改寫動作都不該被提供。 */
const NEWER_PROBE = {
  ...STALE_PROBE,
  status: "newer" as const,
  tools: [{ tool: "claude", workspaceVersion: "v1.4.0", stale: false, newer: true, missing: false }],
};

function fakeInstructionWorkspace(over: Partial<WorkspaceAdapter> = {}) {
  return {
    openProject: vi.fn().mockResolvedValue({ status: "project", root: "A", name: "a" }),
    initProject: vi.fn(),
    adoptProject: vi.fn(),
    startupDir: vi.fn().mockRejectedValue("none"),
    projectStats: vi.fn().mockResolvedValue({ pendingWrapUp: 0 }),
    probeAssets: vi.fn().mockResolvedValue(STALE_PROBE),
    updateAssets: vi.fn().mockResolvedValue(undefined),
    watchWorkspace: vi.fn().mockResolvedValue(undefined),
    pickFolder: vi.fn().mockResolvedValue(null),
    ...over,
  } as unknown as WorkspaceAdapter;
}

/** 以單一 local session 預置 store 並注入 workspace 探測面。 */
function storeWithAssetProbe(ws: WorkspaceAdapter, ds = fakeDataSource()) {
  const store = trackedAppStore({
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

describe("監看重掛（rearmWatch）", () => {
  // worktree 增減會改變監看拓撲（副本 change 目錄、.git/worktrees 登記簿）；
  // workspace-changed 後由前端順手重掛，Rust 端目標集合不變時沿用原監看。
  it("活躍 local session → 對其 root 重掛監看", async () => {
    const ws = fakeInstructionWorkspace();
    const store = storeWithAssetProbe(ws);
    await store.getState().rearmWatch();
    expect(ws.watchWorkspace).toHaveBeenCalledWith("A");
  });

  it("無活躍 session → 不動", async () => {
    const ws = fakeInstructionWorkspace();
    const store = trackedAppStore({
      createSession: (root, name) => fakeSession(fakeDataSource(), root, name),
      workspace: ws,
    });
    await store.getState().rearmWatch();
    expect(ws.watchWorkspace).not.toHaveBeenCalled();
  });

  it("活躍 session 非 local → 不動", async () => {
    const ws = fakeInstructionWorkspace();
    const store = storeWithAssetProbe(ws);
    const remote = {
      ...fakeSession(fakeDataSource(), "R", "r"),
      id: "remote:R",
      locator: { kind: "remote", server: "s", projectKey: "p", repoKey: "r" },
    } as unknown as WorkspaceSession;
    store.setState({ sessions: { [remote.id]: remote }, activeKey: remote.id });
    await store.getState().rearmWatch();
    expect(ws.watchWorkspace).not.toHaveBeenCalled();
  });
});

describe("指令檔過期提示的顯示裁決", () => {
  beforeEach(() => {
    localStorage.removeItem("speclink.assetSkips");
  });

  it("過期且未略過：以更新語意提示並帶差異檔數", async () => {
    const store = storeWithAssetProbe(fakeInstructionWorkspace());
    await store.getState().refreshAssetPrompt();
    expect(store.getState().assetPrompt).toEqual({
      kind: "stale",
      fileCount: 2,
      version: "v1.3.0",
    });
  });

  it("缺失且未略過：以安裝語意提示", async () => {
    const store = storeWithAssetProbe(
      fakeInstructionWorkspace({ probeAssets: vi.fn().mockResolvedValue(MISSING_PROBE) }),
    );
    await store.getState().refreshAssetPrompt();
    expect(store.getState().assetPrompt?.kind).toBe("missing");
  });

  it("保留現狀後同版不再提示，且不寫入專案內任何檔案", async () => {
    const ws = fakeInstructionWorkspace();
    const store = storeWithAssetProbe(ws);
    await store.getState().refreshAssetPrompt();
    store.getState().dismissAssetPrompt();
    expect(store.getState().assetPrompt).toBeNull();

    await store.getState().refreshAssetPrompt();
    expect(store.getState().assetPrompt).toBeNull();
    expect(ws.updateAssets).not.toHaveBeenCalled();
  });

  it("領先引擎且未略過：以較新語意提示並帶差異檔數", async () => {
    const store = storeWithAssetProbe(
      fakeInstructionWorkspace({ probeAssets: vi.fn().mockResolvedValue(NEWER_PROBE) }),
    );
    await store.getState().refreshAssetPrompt();
    expect(store.getState().assetPrompt).toEqual({
      kind: "newer",
      fileCount: 2,
      version: "v1.3.0",
    });
  });

  it("較新態的保留現狀與過期共用同一略過記憶", async () => {
    const store = storeWithAssetProbe(
      fakeInstructionWorkspace({ probeAssets: vi.fn().mockResolvedValue(NEWER_PROBE) }),
    );
    await store.getState().refreshAssetPrompt();
    store.getState().dismissAssetPrompt();
    await store.getState().refreshAssetPrompt();
    expect(store.getState().assetPrompt).toBeNull();
  });

  it("缺失態的保留現狀與過期共用同一略過記憶", async () => {
    const store = storeWithAssetProbe(
      fakeInstructionWorkspace({ probeAssets: vi.fn().mockResolvedValue(MISSING_PROBE) }),
    );
    await store.getState().refreshAssetPrompt();
    store.getState().dismissAssetPrompt();
    await store.getState().refreshAssetPrompt();
    expect(store.getState().assetPrompt).toBeNull();
  });

  it("已略過舊版、產物層版號變動後重新提示", async () => {
    const probe = vi.fn().mockResolvedValue(STALE_PROBE);
    const store = storeWithAssetProbe(fakeInstructionWorkspace({ probeAssets: probe }));
    await store.getState().refreshAssetPrompt();
    store.getState().dismissAssetPrompt();

    probe.mockResolvedValue({ ...STALE_PROBE, currentVersion: "v1.4.0" });
    await store.getState().refreshAssetPrompt();
    expect(store.getState().assetPrompt?.version).toBe("v1.4.0");
  });

  it("無法判定：不提示且不記入略過（後續判過期仍提示）", async () => {
    const probe = vi.fn().mockResolvedValue({
      status: "unknown",
      currentVersion: "v1.3.0",
      tools: [],
      differingFiles: [],
    });
    const store = storeWithAssetProbe(fakeInstructionWorkspace({ probeAssets: probe }));
    await store.getState().refreshAssetPrompt();
    expect(store.getState().assetPrompt).toBeNull();
    expect(localStorage.getItem("speclink.assetSkips")).toBeNull();

    probe.mockResolvedValue(STALE_PROBE);
    await store.getState().refreshAssetPrompt();
    expect(store.getState().assetPrompt?.kind).toBe("stale");
  });

  it("現版：不提示", async () => {
    const store = storeWithAssetProbe(
      fakeInstructionWorkspace({
        probeAssets: vi.fn().mockResolvedValue({
          status: "current",
          currentVersion: "v1.3.0",
          tools: [],
          differingFiles: [],
        }),
      }),
    );
    await store.getState().refreshAssetPrompt();
    expect(store.getState().assetPrompt).toBeNull();
  });

  it("remote 分頁不執行探測", async () => {
    const ws = fakeInstructionWorkspace();
    const ds = fakeDataSource();
    const store = trackedAppStore({
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

    await store.getState().refreshAssetPrompt();
    expect(ws.probeAssets).not.toHaveBeenCalled();
    expect(store.getState().assetPrompt).toBeNull();
  });

  it("更新成功：整套再生後重查，提示消失", async () => {
    const probe = vi.fn().mockResolvedValue(STALE_PROBE);
    const ws = fakeInstructionWorkspace({ probeAssets: probe });
    const store = storeWithAssetProbe(ws);
    await store.getState().refreshAssetPrompt();

    probe.mockResolvedValue({
      status: "current",
      currentVersion: "v1.3.0",
      tools: [],
      differingFiles: [],
    });
    await store.getState().applyAssetUpdate();

    expect(ws.updateAssets).toHaveBeenCalledWith("A");
    expect(store.getState().assetPrompt).toBeNull();
    expect(store.getState().assetUpdateError).toBeNull();
  });

  it("更新失敗：錯誤留在提示原位、提示保持可重試", async () => {
    const ws = fakeInstructionWorkspace({
      updateAssets: vi.fn().mockRejectedValue("CLAUDE.md: permission denied"),
    });
    const store = storeWithAssetProbe(ws);
    await store.getState().refreshAssetPrompt();
    await store.getState().applyAssetUpdate();

    expect(store.getState().assetUpdateError).toContain("permission denied");
    expect(store.getState().assetPrompt).not.toBeNull();
  });

  it("外部 speclink update 後（workspace-changed 重查）提示自然消失", async () => {
    const probe = vi.fn().mockResolvedValue(STALE_PROBE);
    const store = storeWithAssetProbe(fakeInstructionWorkspace({ probeAssets: probe }));
    await store.getState().refreshAssetPrompt();
    expect(store.getState().assetPrompt).not.toBeNull();

    // 使用者於終端跑 speclink update：受管檔成為現版，重查即收合。
    probe.mockResolvedValue({
      status: "current",
      currentVersion: "v1.3.0",
      tools: [],
      differingFiles: [],
    });
    await store.getState().refreshAssetPrompt();
    expect(store.getState().assetPrompt).toBeNull();
  });
});

// spec「分頁切換中即時回饋」（design D1）：本地分頁的專案探測擋在翻頁之前，
// 期間 pendingTabKey 指向目標分頁——探測成功翻頁、失敗轉錯誤態，兩條路都清空。
describe("切換中分頁（pendingTabKey）", () => {
  /** 兩個本地分頁（活躍 A、待切 B），B 的探測以 deferred 控制完成時機。 */
  function storeWithTwoLocalTabs(probe: Promise<unknown>) {
    const ds = fakeDataSource();
    const ws = fakeInstructionWorkspace({
      openProject: vi.fn().mockReturnValue(probe),
    } as Partial<WorkspaceAdapter>);
    const store = trackedAppStore({
      createSession: (root, name) => fakeSession(ds, root, name),
      workspace: ws,
    });
    const a = fakeSession(ds, "A", "a");
    const b = fakeSession(ds, "B", "b");
    store.setState({
      tabs: [
        { locator: a.locator, name: a.descriptor.name },
        { locator: b.locator, name: b.descriptor.name },
      ],
      sessions: { [a.id]: a },
      activeKey: a.id,
    });
    return { store, ws, keyB: b.id };
  }

  it("初值為 null", () => {
    const store = storeWithAssetProbe(fakeInstructionWorkspace());
    expect(store.getState().pendingTabKey).toBeNull();
  });

  it("探測進行中 → 指向目標分頁，活躍分頁不變", async () => {
    const d = deferred<unknown>();
    const { store, keyB } = storeWithTwoLocalTabs(d.promise);
    const activeBefore = store.getState().activeKey;
    const activation = store.getState().activateTab(keyB);
    expect(store.getState().pendingTabKey).toBe(keyB);
    expect(store.getState().activeKey).toBe(activeBefore);
    d.resolve({ status: "project", root: "B", name: "b" });
    await activation;
  });

  it("探測成功翻頁後 → 清為 null", async () => {
    const d = deferred<unknown>();
    const { store, keyB } = storeWithTwoLocalTabs(d.promise);
    const activation = store.getState().activateTab(keyB);
    d.resolve({ status: "project", root: "B", name: "b" });
    await activation;
    expect(store.getState().pendingTabKey).toBeNull();
    expect(store.getState().activeKey).toBe(keyB);
  });

  // spec「探測成功後切頁」：spinner 只涵蓋探測本身。翻頁後的整批載入由 skeleton
  // 承擔——兩者並存會讓已切換完成的分頁持續掛「正在切換」。
  it("探測完成即清除 → 後續整批載入期間不再掛切換中", async () => {
    const probe = deferred<unknown>();
    const load = deferred<ChangeItem[]>();
    const ds = fakeDataSource({ listChanges: vi.fn(() => load.promise) });
    const ws = fakeInstructionWorkspace({
      openProject: vi.fn().mockReturnValue(probe.promise),
    } as Partial<WorkspaceAdapter>);
    const store = trackedAppStore({
      createSession: (root, name) => fakeSession(ds, root, name),
      workspace: ws,
    });
    const a = fakeSession(ds, "A", "a");
    const b = fakeSession(ds, "B", "b");
    store.setState({
      tabs: [
        { locator: a.locator, name: a.descriptor.name },
        { locator: b.locator, name: b.descriptor.name },
      ],
      sessions: { [a.id]: a },
      activeKey: a.id,
    });

    const activation = store.getState().activateTab(b.id);
    probe.resolve({ status: "project", root: "B", name: "b" });
    // 讓 enterProject 推進到整批載入的 await（清單仍未回）。
    await new Promise((r) => setTimeout(r, 0));

    expect(store.getState().activeKey).toBe(b.id);
    expect(store.getState().loaded).toBe(false);
    expect(store.getState().pendingTabKey).toBeNull();

    load.resolve([]);
    await activation;
  });

  it("探測回非專案 → 清為 null，分頁錯誤照舊寫入", async () => {
    const d = deferred<unknown>();
    const { store, keyB } = storeWithTwoLocalTabs(d.promise);
    const activation = store.getState().activateTab(keyB);
    d.resolve({ status: "uninitialized", dir: "B" });
    await activation;
    expect(store.getState().pendingTabKey).toBeNull();
    expect(store.getState().tabErrors[keyB]).toBeTruthy();
    expect(store.getState().activeKey).not.toBe(keyB);
  });

  it("探測上拋 → 清為 null，分頁錯誤照舊寫入", async () => {
    const d = deferred<unknown>();
    const { store, keyB } = storeWithTwoLocalTabs(d.promise);
    const activation = store.getState().activateTab(keyB);
    d.reject("boom");
    await activation;
    expect(store.getState().pendingTabKey).toBeNull();
    expect(store.getState().tabErrors[keyB]).toBeTruthy();
  });

  // 翻頁階段拋錯仍須轉分頁錯誤態——不得因 spinner 清除時機的重構而變成未處理的
  // rejection（呼叫端多為 void activateTab(...)，逸出即靜默）。
  it("翻頁階段拋錯 → 轉分頁錯誤態，不逸出", async () => {
    const ds = fakeDataSource();
    const ws = fakeInstructionWorkspace({
      openProject: vi.fn().mockResolvedValue({ status: "project", root: "B", name: "b" }),
    } as Partial<WorkspaceAdapter>);
    const store = trackedAppStore({
      createSession: () => {
        throw new Error("翻頁失敗");
      },
      workspace: ws,
    });
    const a = fakeSession(ds, "A", "a");
    const b = fakeSession(ds, "B", "b");
    store.setState({
      tabs: [
        { locator: a.locator, name: a.descriptor.name },
        { locator: b.locator, name: b.descriptor.name },
      ],
      sessions: { [a.id]: a },
      activeKey: a.id,
    });

    await expect(store.getState().activateTab(b.id)).resolves.toBeUndefined();
    expect(store.getState().tabErrors[b.id]).toBeTruthy();
    expect(store.getState().pendingTabKey).toBeNull();
  });

  // 快速連點兩個分頁：先發的收尾不得清掉後發的標記，否則後發切換全程無回饋。
  it("先發切換收尾 → 不清掉後發切換的標記", async () => {
    const first = deferred<unknown>();
    const second = deferred<unknown>();
    const ds = fakeDataSource();
    const ws = fakeInstructionWorkspace({
      openProject: vi
        .fn()
        .mockReturnValueOnce(first.promise)
        .mockReturnValueOnce(second.promise),
    } as Partial<WorkspaceAdapter>);
    const store = trackedAppStore({
      createSession: (root, name) => fakeSession(ds, root, name),
      workspace: ws,
    });
    const a = fakeSession(ds, "A", "a");
    const b = fakeSession(ds, "B", "b");
    const c = fakeSession(ds, "C", "c");
    store.setState({
      tabs: [
        { locator: a.locator, name: a.descriptor.name },
        { locator: b.locator, name: b.descriptor.name },
        { locator: c.locator, name: c.descriptor.name },
      ],
      sessions: { [a.id]: a },
      activeKey: a.id,
    });

    const toB = store.getState().activateTab(b.id);
    const toC = store.getState().activateTab(c.id);
    expect(store.getState().pendingTabKey).toBe(c.id);

    first.reject("B 探測失敗");
    await toB;
    // B 的收尾走完，標記仍指向 C——C 的切換還在進行中。
    expect(store.getState().pendingTabKey).toBe(c.id);

    second.resolve({ status: "project", root: "C", name: "c" });
    await toC;
    expect(store.getState().pendingTabKey).toBeNull();
  });
});

// 骨架的終止條件是 loadingActive，不是 loaded：讀不到不等於「確認是空的」，
// 所以失敗時 loaded 維持 false，但在途計數必須歸零讓骨架收掉。
describe("整批載入的進行中旗標", () => {
  it("載入進行中 → loadingActive 為 true；完成後落回 false", async () => {
    const d = deferred<ChangeItem[]>();
    const store = storeWith(fakeDataSource({ listChanges: vi.fn(() => d.promise) }));
    const pending = store.getState().refresh();
    expect(store.getState().loadingActive).toBe(true);
    d.resolve([]);
    await pending;
    expect(store.getState().loadingActive).toBe(false);
    expect(store.getState().loaded).toBe(true);
  });

  it("首訪無快取且讀取失敗 → loadingActive 落回 false，loaded 維持 false", async () => {
    const store = storeWith(
      fakeDataSource({ listChanges: vi.fn().mockRejectedValue(new Error("offline")) }),
    );
    await store.getState().refresh();
    expect(store.getState().loadingActive).toBe(false);
    expect(store.getState().loaded).toBe(false);
  });

  it("已有快取時讀取失敗 → 沿用最後一次成功快照，不覆蓋", async () => {
    const listChanges = vi
      .fn()
      .mockResolvedValue([{ name: "kept", status: "in-progress", totalTasks: 1, completedTasks: 0 }]);
    const store = storeWith(fakeDataSource({ listChanges }));
    await store.getState().refresh();
    expect(store.getState().changes.map((c) => c.name)).toEqual(["kept"]);

    listChanges.mockRejectedValue(new Error("offline"));
    await store.getState().refresh();
    expect(store.getState().changes.map((c) => c.name)).toEqual(["kept"]);
    expect(store.getState().loaded).toBe(true);
  });
});

// 讀不到 ≠ 確認是空的：首訪失敗要留下終態記號，看板與面板才能顯示「載入失敗」
// 而不是與真空 workspace 同貌的空態。記號隨快照走，切走再切回仍記得。
describe("首訪載入失敗終態", () => {
  it("首訪載入失敗 → loadFailed 為 true、loaded 維持 false", async () => {
    const store = storeWith(
      fakeDataSource({ listChanges: vi.fn().mockRejectedValue(new Error("offline")) }),
    );
    await store.getState().refresh();
    expect(store.getState().loadFailed).toBe(true);
    expect(store.getState().loaded).toBe(false);
  });

  it("失敗後成功載入 → loadFailed 落回 false", async () => {
    const listChanges = vi.fn().mockRejectedValueOnce(new Error("offline")).mockResolvedValue([]);
    const store = storeWith(fakeDataSource({ listChanges }));
    await store.getState().refresh();
    expect(store.getState().loadFailed).toBe(true);

    await store.getState().refresh();
    expect(store.getState().loadFailed).toBe(false);
    expect(store.getState().loaded).toBe(true);
  });

  it("過期世代的失敗回來 → 不得覆寫後發的成功", async () => {
    const first = deferred<ChangeItem[]>();
    const second = deferred<ChangeItem[]>();
    const listChanges = vi
      .fn()
      .mockReturnValueOnce(first.promise)
      .mockReturnValueOnce(second.promise);
    const store = storeWith(fakeDataSource({ listChanges }));

    const p1 = store.getState().refresh();
    const p2 = store.getState().refresh();
    second.resolve([]);
    await p2;
    first.reject(new Error("offline"));
    await p1;
    expect(store.getState().loadFailed).toBe(false);
    expect(store.getState().loaded).toBe(true);
  });

  it("切走再切回 → 失敗記錄隨快照存續（不重回骨架、不顯示空態）", async () => {
    const gate = deferred<ChangeItem[]>();
    const dsA = fakeDataSource({
      listChanges: vi
        .fn()
        .mockRejectedValueOnce(new Error("offline"))
        .mockReturnValueOnce(gate.promise),
    });
    const a = remoteSession(dsA, "repo-a");
    const b = remoteSession(fakeDataSource(), "repo-b");
    const store = storeWithRemoteSessions(a, b);

    await store.getState().refresh();
    expect(store.getState().loadFailed).toBe(true);

    await store.getState().activateTab(b.id);
    expect(store.getState().loadFailed).toBe(false);

    // 切回 A：重載才剛起跑，此刻的失敗記號來自快照而非這一發的結果。
    const back = store.getState().activateTab(a.id);
    expect(store.getState().loadFailed).toBe(true);
    expect(store.getState().loaded).toBe(false);

    gate.resolve([]);
    await back;
    expect(store.getState().loadFailed).toBe(false);
  });
});

// loadingActive 由 activeKey 的在途計數導出：計數按 key 記，導出值跟著活躍
// workspace 走。一旦兩者錯位就會卡在 true——那正是骨架永久掛著的老問題復發。
describe("整批載入旗標的記帳邊界", () => {
  it("載入途中關掉該分頁 → 旗標不卡在 true", async () => {
    const d = deferred<ChangeItem[]>();
    const ds = fakeDataSource({ listChanges: vi.fn(() => d.promise) });
    const store = trackedAppStore({ createSession: (root, name) => fakeSession(ds, root, name) });
    const a = fakeSession(ds, "A", "a");
    store.setState({
      tabs: [{ locator: a.locator, name: a.descriptor.name }],
      sessions: { [a.id]: a },
      activeKey: a.id,
    });

    const pending = store.getState().refresh();
    expect(store.getState().loadingActive).toBe(true);
    store.getState().closeTab(a.id);
    d.resolve([]);
    await pending;
    expect(store.getState().loadingActive).toBe(false);
  });

  it("別的 workspace 的在途載入結束 → 不得清掉現任的旗標", async () => {
    const aLoad = deferred<ChangeItem[]>();
    const bLoad = deferred<ChangeItem[]>();
    const dsA = fakeDataSource({ listChanges: vi.fn(() => aLoad.promise) });
    const dsB = fakeDataSource({ listChanges: vi.fn(() => bLoad.promise) });
    const a = fakeSession(dsA, "A", "a");
    const b = fakeSession(dsB, "B", "b");
    const store = trackedAppStore({ createSession: () => a });
    store.setState({
      tabs: [
        { locator: a.locator, name: a.descriptor.name },
        { locator: b.locator, name: b.descriptor.name },
      ],
      sessions: { [a.id]: a, [b.id]: b },
      activeKey: a.id,
    });

    const aPending = store.getState().refresh();
    // 翻到 B 並起 B 自己的載入：此時畫面上「正在載」的是 B。
    store.setState({ activeKey: b.id, loaded: false });
    const bPending = store.getState().refresh();
    expect(store.getState().loadingActive).toBe(true);

    // A 的在途載入這時才回來——它已不是現任，不得把 B 的旗標收掉。
    aLoad.resolve([]);
    await aPending;
    expect(store.getState().loadingActive).toBe(true);

    bLoad.resolve([]);
    await bPending;
    expect(store.getState().loadingActive).toBe(false);
  });

  it("同 key 重疊載入：先發成功回來 → 不得清掉後發在途的旗標", async () => {
    const first = deferred<ChangeItem[]>();
    const second = deferred<ChangeItem[]>();
    const listChanges = vi
      .fn()
      .mockReturnValueOnce(first.promise)
      .mockReturnValueOnce(second.promise);
    const store = storeWith(fakeDataSource({ listChanges }));

    const p1 = store.getState().refresh();
    const p2 = store.getState().refresh();
    expect(store.getState().loadingActive).toBe(true);

    // 先發此刻已是過期世代——回來時後發還在載，旗標不得歸零。
    first.resolve([]);
    await p1;
    expect(store.getState().loadingActive).toBe(true);

    second.resolve([]);
    await p2;
    expect(store.getState().loadingActive).toBe(false);
  });

  it("同 key 重疊載入：先發失敗 → 不得清掉後發在途的旗標", async () => {
    const first = deferred<ChangeItem[]>();
    const second = deferred<ChangeItem[]>();
    const listChanges = vi
      .fn()
      .mockReturnValueOnce(first.promise)
      .mockReturnValueOnce(second.promise);
    const store = storeWith(fakeDataSource({ listChanges }));

    const p1 = store.getState().refresh();
    const p2 = store.getState().refresh();

    first.reject(new Error("offline"));
    await p1;
    expect(store.getState().loadingActive).toBe(true);

    second.resolve([]);
    await p2;
    expect(store.getState().loadingActive).toBe(false);
  });

  it("開修復頁（後面不接整批載入）→ 不得標載入中", () => {
    const ds = fakeDataSource();
    const a = remoteSession(ds, "repo-a");
    const b = remoteSession(ds, "repo-b");
    const store = storeWithRemoteSessions(a, b);
    store.setState({
      remoteRecovery: { [b.id]: { status: "restoring", failure: null } },
    });

    store.getState().showRemoteWorkspaceRecovery(b.id);
    expect(store.getState().activeKey).toBe(b.id);
    // 這條翻頁路徑不會起整批載入——沒有在途就沒有載入中，入口無須表態。
    expect(store.getState().loadingActive).toBe(false);
  });
});

// 翻頁到首訪 workspace 的那一刻起就算載入中：翻頁入口不再自行表態，改由它
// 同步接上的 refresh() 計數 +1 導出——中間不得有「已翻頁、尚未起載入」的空窗，
// 那個窗口渲染的正是假空態。監看掛載慢時尤其明顯，故以卡住的 watch 把關。
describe("翻頁與載入中標記同批", () => {
  it("翻到首訪 workspace → activeKey 翻轉當下即為載入中", async () => {
    const load = deferred<ChangeItem[]>();
    const ds = fakeDataSource({ listChanges: vi.fn(() => load.promise) });
    let resolveWatch!: () => void;
    const watchGate = new Promise<void>((r) => {
      resolveWatch = r;
    });
    const ws = fakeInstructionWorkspace({
      openProject: vi.fn().mockResolvedValue({ status: "project", root: "B", name: "b" }),
      watchWorkspace: vi.fn(() => watchGate),
    } as Partial<WorkspaceAdapter>);
    const store = trackedAppStore({
      createSession: (root, name) => fakeSession(ds, root, name),
      workspace: ws,
    });
    const a = fakeSession(ds, "A", "a");
    const b = fakeSession(ds, "B", "b");
    store.setState({
      tabs: [
        { locator: a.locator, name: a.descriptor.name },
        { locator: b.locator, name: b.descriptor.name },
      ],
      sessions: { [a.id]: a },
      activeKey: a.id,
    });

    const activation = store.getState().activateTab(b.id);
    // 監看掛載卡住：此刻已翻頁但整批載入尚未開跑——不得出現三訊號全滅的空窗。
    await new Promise((r) => setTimeout(r, 0));
    expect(store.getState().activeKey).toBe(b.id);
    expect(store.getState().loaded).toBe(false);
    expect(store.getState().loadingActive).toBe(true);

    resolveWatch();
    load.resolve([]);
    await activation;
    expect(store.getState().loadingActive).toBe(false);
  });

  // 首批載入先於監看掛載起跑（消滅空窗）的代價：掛載完成前的檔案變動既無事件
  // 也不在首批結果內。掛載成功後必須補一發整批載入，蓋掉這個靜默過時窗口。
  it("監看掛載完成後補一發整批載入，涵蓋掛載窗口內的變動", async () => {
    const stale: ChangeItem[] = [];
    const fresh: ChangeItem[] = [
      { name: "landed-during-mount", status: "proposed", totalTasks: 0, completedTasks: 0 },
    ];
    const listChanges = vi
      .fn()
      .mockResolvedValueOnce(stale)
      .mockResolvedValue(fresh);
    const ds = fakeDataSource({ listChanges });
    let resolveWatch!: () => void;
    const watchGate = new Promise<void>((r) => {
      resolveWatch = r;
    });
    const ws = fakeInstructionWorkspace({
      openProject: vi.fn().mockResolvedValue({ status: "project", root: "B", name: "b" }),
      watchWorkspace: vi.fn(() => watchGate),
    } as Partial<WorkspaceAdapter>);
    const store = trackedAppStore({
      createSession: (root, name) => fakeSession(ds, root, name),
      workspace: ws,
    });
    const b = fakeSession(ds, "B", "b");
    store.setState({
      tabs: [{ locator: b.locator, name: b.descriptor.name }],
      sessions: {},
      activeKey: null,
    });

    const activation = store.getState().activateTab(b.id);
    await new Promise((r) => setTimeout(r, 0));
    // 首批（過時）已在途；此刻監看才掛載完成——窗口內的變動只有補讀看得到。
    resolveWatch();
    await activation;
    expect(listChanges.mock.calls.length).toBeGreaterThanOrEqual(2);
    expect(store.getState().changes.map((c) => c.name)).toEqual(["landed-during-mount"]);
  });
});

describe("認領撞 ownership 衝突的呈現", () => {
  // server 的 409 訊息本身就寫了「誰持有、該去找誰協調」——桌面端必須原樣
  // 轉印。退化成單純「認領失敗」,等於把撞工當下唯一有用的資訊丟掉。
  const HELD =
    "change 'demo' is already claimed by Alice Chen <alice@example.com> — coordinate with them, or ask them to release it";

  it("提示原樣帶出目前持有人與建議動作", async () => {
    const claim = vi.fn().mockRejectedValue(new Error(HELD));
    const store = storeWith(fakeDataSource({ claim }));
    await store.getState().claimChange("desktop-shell-and-browser");

    expect(claim).toHaveBeenCalledWith("desktop-shell-and-browser");
    const [message] = toastError.mock.calls[0] as [string];
    expect(message).toContain("Alice Chen <alice@example.com>");
    expect(message).toContain("coordinate with them");
  });

  it("認領成功不出提示,且重新載入清單", async () => {
    const ds = fakeDataSource({ claim: vi.fn().mockResolvedValue(undefined) });
    const store = storeWith(ds);
    await store.getState().claimChange("desktop-shell-and-browser");
    expect(toastError).not.toHaveBeenCalled();
    expect(ds.listChanges).toHaveBeenCalled();
  });
});

// desktop-manual-page design「手冊頁的外部變更重載沿用既有 watcher 事件」：store 收到
// 帶 root 的檔案變更事件（→ 整批 refresh）且手冊視圖活躍時重取索引；交錯回應以
// 最新為準；不新增監看目標（沿用 refresh 這條路）。
describe("手冊索引的重取（refreshManual）", () => {
  const INDEX: ManualIndex = { present: true, reason: null, pages: [], uncoveredNew: [], malformed: [] };

  it("切到手冊視圖即取索引；之後每次整批 refresh 在手冊視圖活躍時重取，非活躍不取", async () => {
    const ds = fakeDataSource({ listManualPages: vi.fn().mockResolvedValue(INDEX) });
    const store = storeWith(ds);
    await store.getState().refresh();
    expect(ds.listManualPages).not.toHaveBeenCalled();
    expect(store.getState().manual).toBeNull();

    store.getState().setBoardView("manual");
    await vi.waitFor(() => expect(store.getState().manual).toEqual(INDEX));
    expect(ds.listManualPages).toHaveBeenCalledTimes(1);

    // 兩筆假事件（workspace-changed → refresh）。
    await store.getState().refresh();
    await store.getState().refresh();
    expect(ds.listManualPages).toHaveBeenCalledTimes(3);

    store.getState().setBoardView("board");
    await store.getState().refresh();
    expect(ds.listManualPages).toHaveBeenCalledTimes(3);
  });

  it("交錯回應以最新為準：先發後到的索引不得覆蓋後發先到的", async () => {
    const first = deferred<ManualIndex>();
    const second = deferred<ManualIndex>();
    const listManualPages = vi
      .fn()
      .mockReturnValueOnce(first.promise)
      .mockReturnValueOnce(second.promise);
    const store = storeWith(fakeDataSource({ listManualPages }));
    store.getState().setBoardView("manual");
    await store.getState().refresh();
    expect(listManualPages).toHaveBeenCalledTimes(2);
    second.resolve({ ...INDEX, uncoveredNew: ["new"] });
    await vi.waitFor(() => expect(store.getState().manual?.uncoveredNew).toEqual(["new"]));
    first.resolve({ ...INDEX, uncoveredNew: ["old"] });
    await new Promise((r) => setTimeout(r, 0));
    expect(store.getState().manual?.uncoveredNew).toEqual(["new"]);
  });

  it("索引讀取失敗：無舊索引時落成尚無手冊的空索引，有舊索引時沿用舊索引", async () => {
    const listManualPages = vi.fn().mockRejectedValueOnce(new Error("io"));
    const store = storeWith(fakeDataSource({ listManualPages }));
    store.getState().setBoardView("manual");
    await vi.waitFor(() => expect(store.getState().manual?.present).toBe(false));
    listManualPages.mockResolvedValueOnce(INDEX).mockRejectedValueOnce(new Error("io"));
    await store.getState().refresh();
    await vi.waitFor(() => expect(store.getState().manual).toEqual(INDEX));
    await store.getState().refresh();
    await new Promise((r) => setTimeout(r, 0));
    expect(store.getState().manual).toEqual(INDEX);
  });

  it("手冊出處開規格抽屜：detailSpec 設定、boardView 維持手冊頁不切頁", () => {
    const store = storeWith(fakeDataSource());
    store.getState().setBoardView("manual");
    store.getState().openSpec("desktop-app");
    expect(store.getState().boardView).toBe("manual");
    expect(store.getState().detailSpec).toBe("desktop-app");
  });
});

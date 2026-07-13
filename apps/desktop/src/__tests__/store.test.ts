import { describe, it, expect, vi } from "vitest";
import type { SpeclinkDataSource, StatusReport } from "@speclink/ui";

import { createAppStore } from "../store";

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

describe("app store (Zustand)", () => {
  it("refresh loads changes, specs and archived", async () => {
    const store = createAppStore(fakeDataSource());
    await store.getState().refresh();
    expect(store.getState().changes).toHaveLength(1);
    expect(store.getState().specs).toHaveLength(1);
    expect(store.getState().archived).toHaveLength(1);
    expect(store.getState().loaded).toBe(true);
  });

  it("refresh increments the refreshGen generation (initial 0)", async () => {
    // 刷新世代：內容元件據此重載已載入的文件（design D1）。
    const store = createAppStore(fakeDataSource());
    expect(store.getState().refreshGen).toBe(0);
    await store.getState().refresh();
    expect(store.getState().refreshGen).toBe(1);
    await store.getState().refresh();
    expect(store.getState().refreshGen).toBe(2);
  });

  it("setView, setQuery and toggleExpand update UI state", () => {
    const store = createAppStore(fakeDataSource());
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
    const store = createAppStore(fakeDataSource());
    await store.getState().refresh();
    store.getState().setBoardView("specs");
    expect(store.getState().boardView).toBe("specs");
    expect(store.getState().specs).toHaveLength(1);
    store.getState().setBoardView("board");
    expect(store.getState().boardView).toBe("board");
  });

  it("boardQuery starts empty and setBoardQuery updates it", () => {
    // 看板搜尋字串：純 UI 狀態、不入任何 persist 機制（spec「不跨啟動保留」）。
    const store = createAppStore(fakeDataSource());
    expect(store.getState().boardQuery).toBe("");
    store.getState().setBoardQuery("desk");
    expect(store.getState().boardQuery).toBe("desk");
  });

  it("boardQuery triggers the debounced full-text search (latest wins); clearing resets hits", async () => {
    // design D6：200ms 去抖、latest-wins；空 query 清命中並取消在途。
    vi.useFakeTimers();
    const hits = [{ kind: "change", id: "demo", artifact: "design.md", snippet: "…x…" }];
    const ds = fakeDataSource({ searchWorkspace: vi.fn().mockResolvedValue(hits) });
    const store = createAppStore(ds);
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
    const store = createAppStore(ds);
    store.getState().setBoardQuery("x");
    await vi.advanceTimersByTimeAsync(200);
    expect(store.getState().searchHits).toEqual([]);
    expect(store.getState().verbResult).toBeNull();
    vi.useRealTimers();
  });

  it("disposeSearch cancels the pending debounced search（卸載後在途去抖不觸發）", async () => {
    // 生命週期清理：元件卸載時取消漏出的去抖 timer 並作廢在途回填，
    // 杜絕搜尋在擁有它的 store 卸載後才開火所致的未處理例外。
    vi.useFakeTimers();
    const ds = fakeDataSource({ searchWorkspace: vi.fn().mockResolvedValue([]) });
    const store = createAppStore(ds);
    store.getState().setBoardQuery("di");
    store.getState().disposeSearch();
    await vi.advanceTimersByTimeAsync(300);
    expect(ds.searchWorkspace).not.toHaveBeenCalled();
    vi.useRealTimers();
  });

  it("boardQuery and the archived-page query are independent", () => {
    // spec「搜尋字串…與已封存頁獨立」：各自設值互不覆蓋。
    const store = createAppStore(fakeDataSource());
    store.getState().setBoardQuery("kanban");
    store.getState().setQuery("archived");
    expect(store.getState().boardQuery).toBe("kanban");
    expect(store.getState().query).toBe("archived");
    store.getState().setQuery("other");
    expect(store.getState().boardQuery).toBe("kanban");
  });

  it("archive confirm flow: request sets pending, confirm runs archive and clears", async () => {
    const ds = fakeDataSource();
    const store = createAppStore(ds);
    store.getState().requestArchive("desktop-shell-and-browser");
    expect(store.getState().pendingArchive).toBe("desktop-shell-and-browser");
    await store.getState().confirmArchive();
    expect(store.getState().pendingArchive).toBeNull();
    expect(ds.runVerb).toHaveBeenCalledWith("archive", "desktop-shell-and-browser");
  });

  it("cancelArchive clears pending without running", () => {
    const ds = fakeDataSource();
    const store = createAppStore(ds);
    store.getState().requestArchive("x");
    store.getState().cancelArchive();
    expect(store.getState().pendingArchive).toBeNull();
    expect(ds.runVerb).not.toHaveBeenCalled();
  });

  it("runVerb analyze runs validate and analyze together into one drawer result", async () => {
    // design D1：「分析」單鍵雙動詞——validate＋analyze 合併為單一結構化抽屜結果，
    // 頂列 verbResult 保留給全域操作。
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
    const store = createAppStore(ds);
    await store.getState().runVerb("analyze", "desktop-shell-and-browser");
    expect(runVerb).toHaveBeenCalledWith("validate", "desktop-shell-and-browser");
    expect(runVerb).toHaveBeenCalledWith("analyze", "desktop-shell-and-browser");
    expect(store.getState().drawerVerb).toMatchObject({
      change: "desktop-shell-and-browser",
      validate: { valid: true },
      analyze: { findings: [{ dimension: "Ambiguity" }] },
    });
    expect(store.getState().verbResult).toBeNull();
    expect(ds.listChanges).toHaveBeenCalled();
  });

  it("runVerb analyze failure surfaces the error in the drawer result", async () => {
    // 任一動詞失敗不靜默：error 進抽屜結果呈現 core 訊息。
    const ds = fakeDataSource({ runVerb: vi.fn().mockRejectedValue(new Error("parse boom")) });
    const store = createAppStore(ds);
    await store.getState().runVerb("analyze", "desktop-shell-and-browser");
    expect(store.getState().drawerVerb?.change).toBe("desktop-shell-and-browser");
    expect(store.getState().drawerVerb?.error).toContain("parse boom");
  });

  it("clearDrawerVerb closes the analysis result; switching change still clears it", async () => {
    // design D2：分析面板可關閉——clearDrawerVerb 收合；既有「換 change 清空」不回歸。
    const ds = fakeDataSource();
    const store = createAppStore(ds);
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

  it("runVerb archive still surfaces in the top-bar verbResult", async () => {
    const ds = fakeDataSource({ runVerb: vi.fn().mockResolvedValue({ datedName: "2026-07-09-x" }) });
    const store = createAppStore(ds);
    await store.getState().runVerb("archive", "desktop-shell-and-browser");
    expect(store.getState().verbResult).toContain("archive");
    expect(store.getState().drawerVerb).toBeNull();
    expect(ds.listChanges).toHaveBeenCalled();
  });

  it("reorderCard passes neighbor ids through and refreshes on success", async () => {
    // design D5：store 動作把 kind/id/prevId/nextId 原樣交給 data source，成功後整批 refresh。
    const ds = fakeDataSource();
    const store = createAppStore(ds);
    await store.getState().reorderCard("discussion", "slug-a", "prev-s", null);
    expect(ds.reorderCard).toHaveBeenCalledWith("discussion", "slug-a", "prev-s", null);
    expect(ds.listChanges).toHaveBeenCalled();
    expect(store.getState().verbResult).toBeNull();
  });

  it("detailSpec 開閉 action 比照 detailChange（spec-archive-drawer design D2）", async () => {
    const store = createAppStore(fakeDataSource());
    await store.getState().refresh();
    expect(store.getState().detailSpec).toBeNull();
    store.getState().openSpec("desktop-app");
    expect(store.getState().detailSpec).toBe("desktop-app");
    store.getState().closeSpec();
    expect(store.getState().detailSpec).toBeNull();
  });

  it("detailArchived 開閉 action：change 與 discussion 兩型 discriminated target", () => {
    const store = createAppStore(fakeDataSource());
    expect(store.getState().detailArchived).toBeNull();
    store.getState().openArchived({ kind: "change", datedName: "2026-07-04-x" });
    expect(store.getState().detailArchived).toEqual({ kind: "change", datedName: "2026-07-04-x" });
    store.getState().openArchived({ kind: "discussion", slug: "old-topic" });
    expect(store.getState().detailArchived).toEqual({ kind: "discussion", slug: "old-topic" });
    store.getState().closeArchived();
    expect(store.getState().detailArchived).toBeNull();
  });

  it("reorderCard failure surfaces a one-line error and still refreshes", async () => {
    // spec「寫回失敗不留假象」：錯誤浮上 verbResult、看板刷新回磁碟現況。
    const ds = fakeDataSource({
      reorderCard: vi.fn().mockRejectedValue(new Error("file locked")),
    });
    const store = createAppStore(ds);
    await store.getState().reorderCard("change", "chg-a", null, "chg-b");
    expect(store.getState().verbResult).toContain("chg-a");
    expect(store.getState().verbResult).toContain("file locked");
    expect(ds.listChanges).toHaveBeenCalled();
  });
});

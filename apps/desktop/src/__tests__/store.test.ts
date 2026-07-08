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

  it("runVerb records a result and refreshes lists", async () => {
    const ds = fakeDataSource();
    const store = createAppStore(ds);
    await store.getState().runVerb("validate", "desktop-shell-and-browser");
    expect(store.getState().verbResult).toContain("validate");
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

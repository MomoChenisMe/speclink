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
});

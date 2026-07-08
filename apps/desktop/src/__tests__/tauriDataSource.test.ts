import { describe, it, expect, vi, beforeEach } from "vitest";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

import { createTauriDataSource } from "../adapter/tauriDataSource";

describe("createTauriDataSource", () => {
  beforeEach(() => invoke.mockReset());

  it("listChanges unwraps the { changes } envelope from the list_changes command", async () => {
    invoke.mockResolvedValueOnce({ changes: [{ name: "a", status: "s", totalTasks: 1, completedTasks: 0 }] });
    const ds = createTauriDataSource();
    const changes = await ds.listChanges();
    expect(invoke).toHaveBeenCalledWith("list_changes");
    expect(changes).toEqual([{ name: "a", status: "s", totalTasks: 1, completedTasks: 0 }]);
  });

  it("listSpecs unwraps the { specs } envelope and carries the optional modifiedAt field", async () => {
    // spec「桌面 app 呈現 change 與 spec 的清單與內容」呈現層輔助欄位（design D2）：
    // 清單項帶 modifiedAt（YYYY-MM-DD）；mtime 不可得時缺席。
    invoke.mockResolvedValueOnce({ specs: [{ id: "cap-x", modifiedAt: "2026-07-08" }, { id: "cap-y" }] });
    const ds = createTauriDataSource();
    const specs = await ds.listSpecs();
    expect(invoke).toHaveBeenCalledWith("list_specs");
    expect(specs[0].modifiedAt).toBe("2026-07-08");
    expect(specs[1].modifiedAt).toBeUndefined();
  });

  it("listArchived unwraps the { archived } envelope", async () => {
    invoke.mockResolvedValueOnce({ archived: [{ datedName: "2026-01-01-a", date: "2026-01-01", name: "a" }] });
    const ds = createTauriDataSource();
    const archived = await ds.listArchived();
    expect(invoke).toHaveBeenCalledWith("archived_changes");
    expect(archived[0].datedName).toBe("2026-01-01-a");
  });

  it("getDocument passes change + artifact to the document command", async () => {
    invoke.mockResolvedValueOnce("## Why");
    const ds = createTauriDataSource();
    const doc = await ds.getDocument("chg", "proposal.md");
    expect(invoke).toHaveBeenCalledWith("document", { change: "chg", artifact: "proposal.md" });
    expect(doc).toBe("## Why");
  });

  it("runVerb invokes the verb command by name with the change arg", async () => {
    invoke.mockResolvedValueOnce({ valid: true });
    const ds = createTauriDataSource();
    await ds.runVerb("validate", "chg");
    expect(invoke).toHaveBeenCalledWith("validate", { change: "chg" });
  });

  it("setAllTasks invokes set_all_tasks with change and done", async () => {
    // spec「任務分頁提供批次操作工具列」：全部已完成／重置任務走批次指令單次寫回。
    invoke.mockResolvedValue(undefined);
    const ds = createTauriDataSource();
    await ds.setAllTasks("chg", true);
    expect(invoke).toHaveBeenCalledWith("set_all_tasks", { change: "chg", done: true });
    await ds.setAllTasks("chg", false);
    expect(invoke).toHaveBeenCalledWith("set_all_tasks", { change: "chg", done: false });
  });

  it("reorderCard invokes reorder_card with kind, id and neighbor ids (null = column ends)", async () => {
    // design D5：以鄰居識別碼表達落點；null＝欄頂／欄底。
    invoke.mockResolvedValue(undefined);
    const ds = createTauriDataSource();
    await ds.reorderCard("change", "my-change", "prev-c", null);
    expect(invoke).toHaveBeenCalledWith("reorder_card", {
      kind: "change",
      id: "my-change",
      prevId: "prev-c",
      nextId: null,
    });
    await ds.reorderCard("discussion", "slug-x", null, "next-s");
    expect(invoke).toHaveBeenCalledWith("reorder_card", {
      kind: "discussion",
      id: "slug-x",
      prevId: null,
      nextId: "next-s",
    });
  });
});

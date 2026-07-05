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
});

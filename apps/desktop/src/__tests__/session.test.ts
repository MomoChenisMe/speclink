// WorkspaceLocator／locatorKey（workspace-session design 決策 1）：分頁去重、
// 持久化 activeKey 與 tray 選單識別的唯一身分函式。remote 變體本刀僅型別、
// 無任何建構路徑——key 規則先釘死，使持久化 schema 跨後續刀穩定。
import { describe, it, expect } from "vitest";

import { createLocalSession, locatorKey, type WorkspaceLocator } from "../session";
import { upsertTab, type ProjectTab } from "../tabs";

/** 假 invoke：記錄每筆 (cmd, args) 並依 cmd 回覆最小合法 payload。 */
function fakeInvoke() {
  const calls: Array<{ cmd: string; args?: Record<string, unknown> }> = [];
  const results: Record<string, unknown> = {
    list_changes: { changes: [] },
    list_specs: { specs: [] },
    archived_changes: { archived: [] },
    search_workspace: { hits: [] },
    list_discussions: { active: [], archived: [] },
    promote_discussion: { change: "chg" },
    change_capabilities: [],
    archived_capabilities: [],
    read_settings: {},
  };
  const invoke = async <T,>(cmd: string, args?: Record<string, unknown>): Promise<T> => {
    calls.push({ cmd, args });
    return (results[cmd] ?? null) as T;
  };
  return { calls, invoke };
}

describe("locatorKey（spec「分頁身分為 WorkspaceLocator 而非 root 路徑」）", () => {
  it("local locator maps to local:{root}", () => {
    expect(locatorKey({ kind: "local", root: "/proj/alpha" })).toBe("local:/proj/alpha");
  });

  it("remote locator maps to remote:{connectionId}/{projectId}/{repoId}", () => {
    const remote: WorkspaceLocator = {
      kind: "remote",
      connectionId: "c1",
      projectId: "p1",
      repoId: "r1",
    };
    expect(locatorKey(remote)).toBe("remote:c1/p1/r1");
  });

  it("checkoutRoot does not participate in the remote key", () => {
    expect(
      locatorKey({
        kind: "remote",
        connectionId: "c1",
        projectId: "p1",
        repoId: "r1",
        checkoutRoot: "/tmp/co",
      }),
    ).toBe(locatorKey({ kind: "remote", connectionId: "c1", projectId: "p1", repoId: "r1" }));
  });

  it("same root yields the same key; different roots differ (去重身分)", () => {
    expect(locatorKey({ kind: "local", root: "A" })).toBe(locatorKey({ kind: "local", root: "A" }));
    expect(locatorKey({ kind: "local", root: "A" })).not.toBe(
      locatorKey({ kind: "local", root: "B" }),
    );
  });
});

describe("createLocalSession（spec「每個 session 自帶 dataSource 且 Rust 側無 current-root 全域」）", () => {
  const ROOT = "/proj/a";

  it("derives id from locatorKey and binds the local locator", () => {
    const { invoke } = fakeInvoke();
    const session = createLocalSession(ROOT, { invoke });
    expect(session.id).toBe("local:/proj/a");
    expect(session.locator).toEqual({ kind: "local", root: ROOT });
  });

  it("every dataSource method carries the bound root", async () => {
    const { calls, invoke } = fakeInvoke();
    const ds = createLocalSession(ROOT, { invoke }).dataSource;
    await ds.listChanges();
    await ds.listSpecs();
    await ds.listArchived();
    await ds.status("chg");
    await ds.getDocument("chg", "proposal.md");
    await ds.getSpecDocument("cap");
    await ds.searchWorkspace("q");
    await ds.changeCapabilities("chg");
    await ds.changeMeta("chg");
    await ds.deleteChange("chg");
    await ds.setTaskDone("chg", "1", true);
    await ds.setAllTasks("chg", true);
    await ds.moveTask("chg", 1, 2, true);
    await ds.runVerb("validate", "chg");
    await ds.getArchivedDocument("2026-01-01-chg", "proposal.md");
    await ds.archivedCapabilities("2026-01-01-chg");
    await ds.listDiscussions();
    await ds.getDiscussionDocument("slug");
    await ds.promoteDiscussion("slug", "chg");
    await ds.archiveDiscussion("slug");
    await ds.reorderCard("change", "chg", null, null);
    expect(calls).toHaveLength(21);
    const missingRoot = calls.filter((c) => c.args?.root !== ROOT).map((c) => c.cmd);
    expect(missingRoot).toEqual([]);
  });

  it("every settings method carries the bound root", async () => {
    const { calls, invoke } = fakeInvoke();
    const settings = createLocalSession(ROOT, { invoke }).settings;
    await settings.readSettings();
    await settings.writeAppTools(["claude"]);
    await settings.writeWorkflowConfig({ locale: null, specLocale: null, tdd: false, audit: false });
    await settings.writeWorkflowContext("ctx");
    await settings.writeWorkflowRules([["proposal", ["rule"]]]);
    expect(calls).toHaveLength(5);
    const missingRoot = calls.filter((c) => c.args?.root !== ROOT).map((c) => c.cmd);
    expect(missingRoot).toEqual([]);
  });

  it("events source only fires when the payload root equals its own root", async () => {
    const { invoke } = fakeInvoke();
    let handler: ((e: { payload: unknown }) => void) | undefined;
    const listen = async (event: string, h: (e: { payload: unknown }) => void) => {
      expect(event).toBe("workspace-changed");
      handler = h;
      return () => {
        handler = undefined;
      };
    };
    const session = createLocalSession(ROOT, { invoke, listen });
    let fired = 0;
    const unsubscribe = session.events.subscribe(() => {
      fired += 1;
    });
    await Promise.resolve(); // listen 為非同步掛載
    handler?.({ payload: "/proj/b" });
    expect(fired).toBe(0);
    handler?.({ payload: ROOT });
    expect(fired).toBe(1);
    unsubscribe();
    await Promise.resolve(); // 解除訂閱經 listen promise 傳遞
    handler?.({ payload: ROOT });
    expect(fired).toBe(1);
  });
});

describe("upsertTab 以 locator key 去重（spec「同一專案重複開啟仍去重」）", () => {
  const local = (root: string): WorkspaceLocator => ({ kind: "local", root });
  const tab = (root: string, name = root): ProjectTab => ({
    locator: local(root),
    name,
    badge: null,
  });

  it("appends a tab with a distinct locator", () => {
    const tabs = upsertTab([tab("A")], { locator: local("B"), name: "B" });
    expect(tabs.map((t) => locatorKey(t.locator))).toEqual(["local:A", "local:B"]);
  });

  it("reopening the same locator updates in place without touching other tabs", () => {
    const tabs = upsertTab([tab("A", "old"), tab("B", "beta")], {
      locator: local("A"),
      name: "new",
    });
    expect(tabs.map((t) => locatorKey(t.locator))).toEqual(["local:A", "local:B"]);
    expect(tabs[0].name).toBe("new");
    expect(tabs[1].name).toBe("beta");
  });
});

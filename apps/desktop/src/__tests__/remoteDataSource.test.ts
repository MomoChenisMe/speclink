// RemoteDataSource＝薄 invoke 包裝（remote-data-source design 決策 7）：
// SpeclinkDataSource 全方法對 remote_* command 的參數映射（connectionId＋
// project＋repo）、不支援方法回拒絕（決策 1 (c)——server 缺什麼就停用什麼，
// 不在 client 偽造）；createRemoteSession 以 handshake 結果建 session、事件面
// 訂閱 remote-workspace-changed 並以 locator key 過濾。
import { describe, it, expect } from "vitest";

import { createRemoteDataSource } from "../adapter/remoteDataSource";
import {
  createLocalSession,
  createRemoteSession,
  type RemoteOpenInfo,
  type WorkspaceCapabilities,
} from "../session";

const CONN = "conn_abc";
const PROJECT = "demo";
const REPO = "backend";

/** 假 invoke：記錄每筆 (cmd, args) 並依 cmd 回覆最小合法 payload。 */
function fakeInvoke() {
  const calls: Array<{ cmd: string; args?: Record<string, unknown> }> = [];
  const results: Record<string, unknown> = {
    remote_list_changes: { changes: [{ name: "chg", status: "in-progress", completedTasks: 1, totalTasks: 2, summary: "s" }] },
    remote_list_specs: { specs: [{ id: "auth", path: "specs/auth/spec.md" }] },
    remote_status: {
      changeName: "chg",
      schemaName: "spec-driven",
      isComplete: false,
      applyRequires: ["tasks"],
      artifacts: [],
    },
    remote_document: "content",
    remote_archive: { specs: [] },
    remote_list_discussions: {
      active: [{ slug: "s1", topic: "T", status: "open", rounds: 1, created: "2026-01-01" }],
      archived: [],
    },
    remote_discussion_document: "discussion text",
    remote_promote_discussion: { change: "chg" },
  };
  const invoke = async <T,>(cmd: string, args?: Record<string, unknown>): Promise<T> => {
    calls.push({ cmd, args });
    return (results[cmd] ?? null) as T;
  };
  return { calls, invoke };
}

/** handshake 結果 fixture：直達／組合真、不支援假（決策 1 矩陣）。 */
function openInfo(): RemoteOpenInfo {
  const capabilities: WorkspaceCapabilities = {
    listChanges: true,
    listSpecs: true,
    listArchived: false,
    status: true,
    getDocument: true,
    getSpecDocument: false,
    searchWorkspace: false,
    changeCapabilities: false,
    changeMeta: false,
    deleteChange: false,
    setTaskDone: true,
    setAllTasks: true,
    moveTask: false,
    validate: false,
    analyze: false,
    archive: true,
    getArchivedDocument: false,
    archivedCapabilities: false,
    listDiscussions: true,
    getDiscussionDocument: true,
    promoteDiscussion: true,
    archiveDiscussion: true,
    reorderCard: false,
    liveUpdates: true,
  };
  return {
    projectKey: PROJECT,
    projectName: "Demo",
    repoKey: REPO,
    repoName: "backend",
    capabilities,
  };
}

describe("createRemoteDataSource（決策 7：薄 invoke 包裝）", () => {
  it("supported methods map to remote_* commands carrying connectionId + project + repo", async () => {
    const { calls, invoke } = fakeInvoke();
    const ds = createRemoteDataSource(CONN, PROJECT, REPO, invoke);

    await ds.listChanges();
    await ds.listSpecs();
    await ds.status("chg");
    await ds.getDocument("chg", "tasks");
    await ds.setTaskDone("chg", "1", true);
    await ds.setAllTasks("chg", true);
    await ds.runVerb("archive", "chg");
    await ds.listDiscussions();
    await ds.getDiscussionDocument("s1");
    await ds.promoteDiscussion("s1");
    await ds.archiveDiscussion("s1");

    expect(calls.map((c) => c.cmd)).toEqual([
      "remote_list_changes",
      "remote_list_specs",
      "remote_status",
      "remote_document",
      "remote_set_task_done",
      "remote_set_all_tasks",
      "remote_archive",
      "remote_list_discussions",
      "remote_discussion_document",
      "remote_promote_discussion",
      "remote_archive_discussion",
    ]);
    // 每一擊都帶 locator 識別。
    const missing = calls
      .filter(
        (c) =>
          c.args?.connectionId !== CONN || c.args?.project !== PROJECT || c.args?.repo !== REPO,
      )
      .map((c) => c.cmd);
    expect(missing).toEqual([]);
    // 方法專屬參數映射。
    expect(calls[2].args).toMatchObject({ change: "chg" });
    expect(calls[3].args).toMatchObject({ change: "chg", artifact: "tasks" });
    expect(calls[4].args).toMatchObject({ change: "chg", task: "1", done: true });
    expect(calls[5].args).toMatchObject({ change: "chg", done: true });
    expect(calls[6].args).toMatchObject({ change: "chg" });
    expect(calls[8].args).toMatchObject({ slug: "s1" });
    expect(calls[9].args).toMatchObject({ slug: "s1", name: null });
    expect(calls[10].args).toMatchObject({ slug: "s1" });
  });

  it("returns server payloads in the UI shapes", async () => {
    const { invoke } = fakeInvoke();
    const ds = createRemoteDataSource(CONN, PROJECT, REPO, invoke);
    const changes = await ds.listChanges();
    expect(changes[0]).toMatchObject({ name: "chg", totalTasks: 2 });
    const specs = await ds.listSpecs();
    expect(specs[0].id).toBe("auth");
    const status = await ds.status("chg");
    expect(status.schemaName).toBe("spec-driven");
    const discussions = await ds.listDiscussions();
    // server 不外露 promotedTo——以空清單補齊 UI 必填欄位（資料缺口，非偽造 affordance）。
    expect(discussions.active[0]).toMatchObject({ slug: "s1", promotedTo: [] });
  });

  it("unsupported methods reject without ever invoking", async () => {
    const { calls, invoke } = fakeInvoke();
    const ds = createRemoteDataSource(CONN, PROJECT, REPO, invoke);
    const rejections: Array<Promise<unknown>> = [
      ds.listArchived(),
      ds.getArchivedDocument("2026-01-01-chg", "proposal.md"),
      ds.archivedCapabilities("2026-01-01-chg"),
      ds.searchWorkspace("q"),
      ds.getSpecDocument("cap"),
      ds.changeCapabilities("chg"),
      ds.changeMeta("chg"),
      ds.deleteChange("chg"),
      ds.moveTask("chg", 1, 2),
      ds.reorderCard("change", "chg", null, null),
      ds.runVerb("validate", "chg"),
      ds.runVerb("analyze", "chg"),
    ];
    for (const p of rejections) {
      await expect(p).rejects.toThrow(/尚未提供/);
    }
    expect(calls).toHaveLength(0);
  });
});

describe("createRemoteSession（決策 6/7：handshake 結果建 session）", () => {
  it("derives id from the locator, names the tab Project/Repo, and carries capabilities", () => {
    const { invoke } = fakeInvoke();
    const session = createRemoteSession(CONN, openInfo(), { invoke });
    expect(session.id).toBe(`remote:${CONN}/${PROJECT}/${REPO}`);
    expect(session.locator).toEqual({
      kind: "remote",
      connectionId: CONN,
      projectId: PROJECT,
      repoId: REPO,
    });
    expect(session.descriptor.name).toBe("Demo/backend");
    expect(session.capabilities.searchWorkspace).toBe(false);
    expect(session.capabilities.listChanges).toBe(true);
  });

  it("events subscribe watches the stream, filters by locator key, and unwatches on teardown", async () => {
    const { calls, invoke } = fakeInvoke();
    let handler: ((e: { payload: unknown }) => void) | undefined;
    const listen = async (event: string, h: (e: { payload: unknown }) => void) => {
      expect(event).toBe("remote-workspace-changed");
      handler = h;
      return () => {
        handler = undefined;
      };
    };
    const session = createRemoteSession(CONN, openInfo(), { invoke, listen });
    let fired = 0;
    const unsubscribe = session.events.subscribe(() => {
      fired += 1;
    });
    await Promise.resolve();
    expect(calls.map((c) => c.cmd)).toContain("remote_watch");
    expect(calls.find((c) => c.cmd === "remote_watch")?.args).toMatchObject({
      connectionId: CONN,
      project: PROJECT,
      repo: REPO,
    });

    handler?.({ payload: "remote:other/x/y" });
    expect(fired).toBe(0);
    handler?.({ payload: session.id });
    expect(fired).toBe(1);

    unsubscribe();
    await Promise.resolve();
    expect(calls.map((c) => c.cmd)).toContain("remote_unwatch");
    handler?.({ payload: session.id });
    expect(fired).toBe(1);
  });

  it("local sessions carry an all-true capability description（決策 2：同一 UI 路徑）", () => {
    const { invoke } = fakeInvoke();
    const session = createLocalSession("/proj/a", { invoke });
    const caps = session.capabilities;
    expect(Object.values(caps).every((on) => on === true)).toBe(true);
  });
});

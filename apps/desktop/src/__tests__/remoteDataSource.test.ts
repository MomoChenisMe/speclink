// RemoteDataSource＝薄 invoke 包裝（remote-data-source design 決策 7）：
// SpeclinkDataSource 全方法對 remote_* command 的參數映射（connectionId＋
// project＋repo）、不支援方法回拒絕（決策 1 (c)——server 缺什麼就停用什麼，
// 不在 client 偽造）；createRemoteSession 以 handshake 結果建 session、事件面
// 訂閱 remote-workspace-changed 並以 locator key 過濾。
import { describe, it, expect } from "vitest";
import { changeStage, RevertBlockedError } from "@speclink/ui";

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
    remote_list_changes: { changes: [{ name: "chg", status: "in-progress", completedTasks: 1, totalTasks: 2, summary: "s", claimedBy: "Alice <a@example.com>" }] },
    remote_list_specs: { specs: [{ id: "auth", path: "specs/auth/spec.md" }] },
    remote_list_archived: {
      archived: [
        {
          datedName: "2026-01-01-old",
          date: "2026-01-01",
          name: "old",
          tasksTotal: 2,
          tasksDone: 2,
          specCount: 1,
          createdBy: "momo",
          fromDiscussions: ["source"],
        },
      ],
    },
    remote_status: {
      changeName: "chg",
      schemaName: "spec-driven",
      isComplete: false,
      applyRequires: ["tasks"],
      artifacts: [],
      created: "2026-07-29",
      createdBy: "Demo <d@e.com>",
      createdWith: "claude-code",
      startedAt: "2026-08-25T00:00:00Z",
      startedBy: "Demo <d@e.com>",
      fromDiscussions: ["auth-scope"],
      deltaCapabilities: ["auth", "user-profile"],
    },
    remote_document: "content",
    remote_spec_document: "# auth Specification",
    remote_search_workspace: {
      hits: [{ kind: "change", id: "chg", artifact: "proposal.md", snippet: "needle" }],
    },
    remote_archive: { specs: [] },
    remote_archived_document: "archived content",
    remote_archived_capabilities: ["auth"],
    remote_list_discussions: {
      active: [
        { slug: "s1", topic: "T", status: "open", rounds: 1, created: "2026-01-01" },
        {
          slug: "s2",
          topic: "Promoted",
          status: "promoted",
          rounds: 2,
          created: "2026-01-02",
          promotedTo: ["cut-a", "cut-b"],
          concluded: true,
        },
        {
          slug: "s3",
          topic: "Promoted unconcluded",
          status: "promoted",
          rounds: 1,
          created: "2026-01-03",
          promotedTo: ["cut-c"],
          concluded: false,
        },
      ],
      archived: [],
    },
    remote_discussion_document: "discussion text",
    remote_promote_discussion: { change: "chg" },
    remote_read_settings: {
      app: { tools: [], customTools: [], parseError: null },
      workflow: {
        locale: "tw",
        specLocale: null,
        tdd: false,
        audit: false,
        context: "Remote context",
        rules: {},
        schemaArtifacts: ["proposal", "design", "specs", "tasks"],
        parseError: null,
        revision: 7,
      },
    },
    remote_write_workflow_config: 8,
    remote_write_workflow_content: 8,
    remote_write_workflow_schema: 8,
    read_schemas: [
      {
        name: "spec-driven",
        source: "package",
        artifactIds: ["proposal", "design", "specs", "tasks"],
        artifacts: [],
        path: null,
        error: null,
      },
    ],
    remote_validate: { change: "chg", valid: true, errors: [], warnings: [] },
    remote_analyze: {
      change_id: "chg",
      dimensions: [],
      findings: [],
      artifacts_analyzed: [],
      artifacts_missing: [],
    },
    remote_delete_change: null,
    remote_move_task: null,
    remote_reorder_card: null,
    remote_revert_change_to_proposed: null,
    remote_claim: { claimedBy: "Tester <t@example.com>" },
  };
  const invoke = async <T,>(cmd: string, args?: Record<string, unknown>): Promise<T> => {
    calls.push({ cmd, args });
    return (results[cmd] ?? null) as T;
  };
  return { calls, invoke };
}

/** handshake 結果 fixture（editor）：全操作面直達，停用清單清空
 *（remote-board-order 決策 7）。 */
function openInfo(): RemoteOpenInfo {
  const capabilities: WorkspaceCapabilities = {
    listChanges: true,
    listSpecs: true,
    listArchived: true,
    status: true,
    getDocument: true,
    getSpecDocument: true,
    searchWorkspace: true,
    changeCapabilities: true,
    changeMeta: true,
    deleteChange: true,
    setTaskDone: true,
    setAllTasks: true,
    moveTask: true,
    validate: true,
    analyze: true,
    archive: true,
    getArchivedDocument: true,
    archivedCapabilities: true,
    listDiscussions: true,
    getDiscussionDocument: true,
    promoteDiscussion: true,
    archiveDiscussion: true,
    reorderCard: true,
    policyWrite: true,
    claim: true,
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

/** reader 的 handshake 結果：寫入面（含看板拖排）依 role 停用。 */
function readerInfo(): RemoteOpenInfo {
  const info = openInfo();
  return {
    ...info,
    capabilities: {
      ...info.capabilities,
      deleteChange: false,
      moveTask: false,
      reorderCard: false,
      policyWrite: false,
      claim: false,
    },
  };
}

describe("createRemoteDataSource（決策 7：薄 invoke 包裝）", () => {
  it("revertChangeToProposed maps to remote_revert_change_to_proposed and translates the blocked JSON", async () => {
    const { calls, invoke } = fakeInvoke();
    const ds = createRemoteDataSource(CONN, PROJECT, REPO, invoke);
    await ds.revertChangeToProposed("chg");
    const call = calls.find((c) => c.cmd === "remote_revert_change_to_proposed");
    expect(call?.args).toMatchObject({
      connectionId: CONN,
      project: PROJECT,
      repo: REPO,
      change: "chg",
    });

    // 守門 JSON → RevertBlockedError——與本地 adapter 同一結構化錯誤形狀
    //(spec Scenario「remote 模式行為一致」的 adapter 半邊)。
    const blocked = async () => {
      throw '{"kind":"revertBlocked","checkedTasks":1,"touchedFiles":["src/x.rs"]}';
    };
    const ds2 = createRemoteDataSource(CONN, PROJECT, REPO, blocked as never);
    const err = await ds2.revertChangeToProposed("chg").catch((e: unknown) => e);
    expect(err).toBeInstanceOf(RevertBlockedError);
    expect((err as RevertBlockedError).checkedTasks).toBe(1);
    expect((err as RevertBlockedError).touchedFiles).toEqual(["src/x.rs"]);
  });

  it("supported methods map to remote_* commands carrying connectionId + project + repo", async () => {
    const { calls, invoke } = fakeInvoke();
    const ds = createRemoteDataSource(CONN, PROJECT, REPO, invoke);

    await ds.listChanges();
    await ds.listSpecs();
    await ds.listArchived();
    await ds.status("chg");
    await ds.getDocument("chg", "tasks");
    await ds.getSpecDocument("auth");
    await ds.searchWorkspace("needle");
    await ds.setTaskDone("chg", "1", true);
    await ds.setAllTasks("chg", true);
    await ds.runVerb("archive", "chg");
    await ds.runVerb("validate", "chg");
    await ds.runVerb("analyze", "chg");
    await ds.deleteChange("chg");
    await ds.moveTask("chg", 1, 3);
    await ds.getArchivedDocument("2026-01-01-old", "proposal.md");
    await ds.archivedCapabilities("2026-01-01-old");
    await ds.listDiscussions();
    await ds.getDiscussionDocument("s1");
    await ds.promoteDiscussion("s1");
    await ds.archiveDiscussion("s1");
    await ds.reorderCard("change", "chg", "prev-card", "next-card");

    expect(calls.map((c) => c.cmd)).toEqual([
      "remote_list_changes",
      "remote_list_specs",
      "remote_list_archived",
      "remote_status",
      "remote_document",
      "remote_spec_document",
      "remote_search_workspace",
      "remote_set_task_done",
      "remote_set_all_tasks",
      "remote_archive",
      "remote_validate",
      "remote_analyze",
      "remote_delete_change",
      "remote_move_task",
      "remote_archived_document",
      "remote_archived_capabilities",
      "remote_list_discussions",
      "remote_discussion_document",
      "remote_promote_discussion",
      "remote_archive_discussion",
      "remote_reorder_card",
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
    expect(calls[3].args).toMatchObject({ change: "chg" });
    expect(calls[4].args).toMatchObject({ change: "chg", artifact: "tasks" });
    expect(calls[5].args).toMatchObject({ capability: "auth" });
    expect(calls[6].args).toMatchObject({ query: "needle" });
    expect(calls[7].args).toMatchObject({ change: "chg", task: "1", done: true });
    expect(calls[8].args).toMatchObject({ change: "chg", done: true });
    expect(calls[9].args).toMatchObject({ change: "chg" });
    expect(calls[10].args).toMatchObject({ change: "chg" });
    expect(calls[11].args).toMatchObject({ change: "chg" });
    // 桌面 remote 刪除帶 force=false（archive-readiness-gating D2 翻案）：與本地
    // discard 守門語意對齊，已開工 change 由 server 拒絕、走既有 deleteFailed toast。
    expect(calls[12].args).toMatchObject({ change: "chg", force: false });
    expect(calls[13].args).toMatchObject({ change: "chg", from: 1, to: 3, before: null });
    expect(calls[14].args).toMatchObject({
      datedName: "2026-01-01-old",
      artifact: "proposal.md",
    });
    expect(calls[15].args).toMatchObject({ datedName: "2026-01-01-old" });
    expect(calls[17].args).toMatchObject({ slug: "s1" });
    expect(calls[18].args).toMatchObject({ slug: "s1", name: null });
    expect(calls[19].args).toMatchObject({ slug: "s1" });
    // 看板拖排直達（remote-board-order）：鄰居定址與本地 reorder_card 同形。
    expect(calls[20].args).toMatchObject({
      kind: "change",
      id: "chg",
      prevId: "prev-card",
      nextId: "next-card",
    });
  });

  it("claim maps to remote_claim carrying the locator（remote-claim-ownership）", async () => {
    const { calls, invoke } = fakeInvoke();
    const ds = createRemoteDataSource(CONN, PROJECT, REPO, invoke);
    await ds.claim!("chg");
    const call = calls.find((c) => c.cmd === "remote_claim");
    expect(call?.args).toMatchObject({
      connectionId: CONN,
      project: PROJECT,
      repo: REPO,
      change: "chg",
    });
  });

  it("listChanges 帶出 claimedBy（認領人呈現的資料源）", async () => {
    const { invoke } = fakeInvoke();
    const ds = createRemoteDataSource(CONN, PROJECT, REPO, invoke);
    const [first] = await ds.listChanges();
    expect(first.claimedBy).toBe("Alice <a@example.com>");
  });

  it("returns server payloads in the UI shapes", async () => {
    const { invoke } = fakeInvoke();
    const ds = createRemoteDataSource(CONN, PROJECT, REPO, invoke);
    const changes = await ds.listChanges();
    expect(changes[0]).toMatchObject({ name: "chg", totalTasks: 2 });
    const specs = await ds.listSpecs();
    expect(specs[0].id).toBe("auth");
    const archived = await ds.listArchived();
    expect(archived[0]).toMatchObject({ datedName: "2026-01-01-old", specCount: 1 });
    const status = await ds.status("chg");
    expect(status.schemaName).toBe("spec-driven");
    await expect(ds.getSpecDocument("auth")).resolves.toBe("# auth Specification");
    await expect(ds.searchWorkspace("needle")).resolves.toEqual([
      { kind: "change", id: "chg", artifact: "proposal.md", snippet: "needle" },
    ]);
    await expect(
      ds.getArchivedDocument("2026-01-01-old", "proposal.md"),
    ).resolves.toBe("archived content");
    await expect(ds.archivedCapabilities("2026-01-01-old")).resolves.toEqual(["auth"]);
    const discussions = await ds.listDiscussions();
    // promotedTo 映射 wire 欄位（remote-read-parity）：缺席以空清單容錯、
    // 非空如實攜帶，不再以 client 端固定值補齊。
    expect(discussions.active[0]).toMatchObject({ slug: "s1", promotedTo: [] });
    expect(discussions.active[1]).toMatchObject({ slug: "s2", promotedTo: ["cut-a", "cut-b"] });
    // concluded 同律（conclusion-gated-discussion-archive）：映射 wire 值，
    // 缺席（舊 server）維持未知、不補成 false。
    expect(discussions.active[0].concluded).toBeUndefined();
    expect(discussions.active[1].concluded).toBe(true);
    expect(discussions.active[2]).toMatchObject({ slug: "s3", concluded: false });
  });

  it("wire 的 startedAt 進入 ChangeItem，changeStage 對開工零進度卡判進行中", async () => {
    // spec desktop-app「remote 已開工零進度列於進行中」：startedAt 隨清單
    // payload 進欄位推導，不以任務完成數替代開工判定。
    const invoke = async <T,>(): Promise<T> =>
      ({
        changes: [
          {
            name: "started-zero",
            summary: "",
            status: "in-progress",
            completedTasks: 0,
            totalTasks: 15,
            startedAt: "2026-07-30",
          },
          {
            name: "unstarted",
            summary: "",
            status: "in-progress",
            completedTasks: 0,
            totalTasks: 15,
          },
        ],
      }) as T;
    const ds = createRemoteDataSource(CONN, PROJECT, REPO, invoke);
    const changes = await ds.listChanges();
    expect(changes[0].startedAt).toBe("2026-07-30");
    expect(changeStage(changes[0])).toBe("in-progress");
    expect(changes[1].startedAt).toBeUndefined();
    expect(changeStage(changes[1])).toBe("proposed");
  });

  it("changeCapabilities 以既有 remote_status 路徑映射 deltaCapabilities", async () => {
    // remote-read-parity design D2：不開新 Tauri command、不另發 HTTP 請求，
    // 自 status payload 抽取既有欄位。
    const { calls, invoke } = fakeInvoke();
    const ds = createRemoteDataSource(CONN, PROJECT, REPO, invoke);
    await expect(ds.changeCapabilities("chg")).resolves.toEqual(["auth", "user-profile"]);
    expect(calls.map((c) => c.cmd)).toEqual(["remote_status"]);
    expect(calls[0].args).toMatchObject({
      connectionId: CONN,
      project: PROJECT,
      repo: REPO,
      change: "chg",
    });
  });

  it("changeMeta 以 status payload 組出 ChangeMetaInfo、缺席欄位誠實降級", async () => {
    const { calls, invoke } = fakeInvoke();
    const ds = createRemoteDataSource(CONN, PROJECT, REPO, invoke);
    await expect(ds.changeMeta("chg")).resolves.toEqual({
      schema: "spec-driven",
      created: "2026-07-29",
      createdBy: "Demo <d@e.com>",
      createdWith: "claude-code",
      fromDiscussions: ["auth-scope"],
      startedAt: "2026-08-25T00:00:00Z",
      startedBy: "Demo <d@e.com>",
    });
    expect(calls.map((c) => c.cmd)).toEqual(["remote_status"]);

    // 舊 server（無新欄位）：對應欄位為 null／缺席，不偽造預設值、不失敗。
    const legacyInvoke = async <T,>(): Promise<T> =>
      ({
        changeName: "chg",
        schemaName: "spec-driven",
        isComplete: false,
        applyRequires: ["tasks"],
        artifacts: [],
      }) as T;
    const legacy = createRemoteDataSource(CONN, PROJECT, REPO, legacyInvoke);
    const meta = await legacy.changeMeta("chg");
    expect(meta).toMatchObject({ schema: "spec-driven" });
    expect(meta?.createdBy ?? null).toBeNull();
    expect(meta?.createdWith ?? null).toBeNull();
    expect(meta?.startedAt ?? null).toBeNull();
    expect(meta?.startedBy ?? null).toBeNull();
    expect(meta?.created ?? null).toBeNull();
    expect(meta?.fromDiscussions ?? []).toEqual([]);
  });

  it("changeCapabilities 與 changeMeta 併發載入共用單次 remote_status 請求", async () => {
    // spec remote-workspace-data「以單 change 讀取回應既有 payload 映射實作、
    // 不另開請求」：抽屜同時載入 capability 清單與詮釋資料時只打一次端點；
    // 落定後的新一輪載入仍重新取值，不留 stale 快取。
    const { calls, invoke } = fakeInvoke();
    const ds = createRemoteDataSource(CONN, PROJECT, REPO, invoke);
    const [caps, meta] = await Promise.all([ds.changeCapabilities("chg"), ds.changeMeta("chg")]);
    expect(caps).toEqual(["auth", "user-profile"]);
    expect(meta?.createdBy).toBe("Demo <d@e.com>");
    expect(calls.filter((c) => c.cmd === "remote_status")).toHaveLength(1);
    await ds.changeMeta("chg");
    expect(calls.filter((c) => c.cmd === "remote_status")).toHaveLength(2);
  });
});

describe("createRemoteSession（決策 6/7：handshake 結果建 session）", () => {
  it("derives id from the locator, names the tab Project/Repo, and carries capabilities", () => {
    const { invoke } = fakeInvoke();
    const session = createRemoteSession(CONN, openInfo(), undefined, { invoke });
    expect(session.id).toBe(`remote:${CONN}/${PROJECT}/${REPO}`);
    expect(session.locator).toEqual({
      kind: "remote",
      connectionId: CONN,
      projectId: PROJECT,
      repoId: REPO,
    });
    expect(session.descriptor.name).toBe("Demo/backend");
    expect(session.capabilities.searchWorkspace).toBe(true);
    expect(session.capabilities.listArchived).toBe(true);
    expect(session.capabilities.getSpecDocument).toBe(true);
    expect(session.capabilities.listChanges).toBe(true);
    expect(session.settings.kind).toBe("remote");
    expect(session.settings.policyWrite).toBe(true);
  });

  it("capability 依 role：editor 的 reorderCard 真、reader 假（不偽造缺口）", () => {
    const { invoke } = fakeInvoke();
    const editor = createRemoteSession(CONN, openInfo(), undefined, { invoke });
    expect(editor.capabilities.reorderCard).toBe(true);
    const reader = createRemoteSession(CONN, readerInfo(), undefined, { invoke });
    expect(reader.capabilities.reorderCard).toBe(false);
    expect(reader.capabilities.listChanges).toBe(true);
  });

  it("remote settings 綁 locator，以讀得 revision 作 expectedRevision 寫回", async () => {
    const { calls, invoke } = fakeInvoke();
    const session = createRemoteSession(CONN, openInfo(), undefined, { invoke });

    const snap = await session.settings.readSettings();
    expect(snap.workflow.revision).toBe(7);
    await expect(
      session.settings.writeWorkflowConfig({
        locale: "tw",
        specLocale: "auto",
        tdd: true,
        audit: false,
        // remote 無 worktree 軸；欄位存在於完整目標狀態，寫入端忽略。
        worktree: false,
      }),
    ).resolves.toBe(8);
    await session.settings.writeWorkflowContext("updated");
    await session.settings.writeWorkflowRules([["tasks", ["test first"]]]);

    expect(calls.find((call) => call.cmd === "remote_read_settings")?.args).toEqual({
      connectionId: CONN,
      project: PROJECT,
      repo: REPO,
    });
    expect(calls.find((call) => call.cmd === "remote_write_workflow_config")?.args).toMatchObject({
      connectionId: CONN,
      project: PROJECT,
      repo: REPO,
      locale: "tw",
      specLocale: "auto",
      tdd: true,
      audit: false,
      expectedRevision: 7,
    });
    const contentWrites = calls.filter((call) => call.cmd === "remote_write_workflow_content");
    expect(contentWrites[0]?.args).toMatchObject({ context: "updated", expectedRevision: 8 });
    expect(contentWrites[1]?.args).toMatchObject({
      rules: [["tasks", ["test first"]]],
      expectedRevision: 8,
    });
  });

  it("remote 產出流程面：讀走本地內建組裝、切換帶 expectedRevision、fork 拒絕（desktop-schema-panel D2/D3）", async () => {
    const { calls, invoke } = fakeInvoke();
    const session = createRemoteSession(CONN, openInfo(), undefined, { invoke });
    await session.settings.readSettings(); // 取得 revision 7

    await session.settings.readSchemas();
    // 不打 server：由 desktop core 以內嵌內建組裝，也不帶任何本機 root。
    expect(calls.find((c) => c.cmd === "read_schemas")?.args).toEqual({});

    await expect(session.settings.writeWorkflowSchema("spec-driven")).resolves.toBe(8);
    expect(calls.find((c) => c.cmd === "remote_write_workflow_schema")?.args).toMatchObject({
      connectionId: CONN,
      project: PROJECT,
      repo: REPO,
      name: "spec-driven",
      expectedRevision: 7,
    });

    // remote 不支援 fork 與建立（spec「remote 模式無 fork 入口」「remote 模式無
    // 建立入口」的通道面防線）：rejected Promise 且不發任何 invoke。
    await expect(session.settings.forkSchema("spec-driven")).rejects.toThrow();
    expect(calls.some((c) => c.cmd === "fork_schema")).toBe(false);
    await expect(session.settings.createSchema("my-flow")).rejects.toThrow();
    expect(calls.some((c) => c.cmd === "init_schema")).toBe(false);
    await expect(session.settings.revealSchema("/anywhere")).rejects.toThrow();
    expect(calls.some((c) => c.cmd === "reveal_in_folder")).toBe(false);
    await expect(session.settings.deleteSchema("my-flow")).rejects.toThrow();
    expect(calls.some((c) => c.cmd === "delete_schema")).toBe(false);
  });

  it("events subscribe watches the stream, filters by locator key, and unwatches on teardown", async () => {
    const { calls, invoke } = fakeInvoke();
    const handlers = new Map<string, (e: { payload: unknown }) => void>();
    const listen = async (event: string, h: (e: { payload: unknown }) => void) => {
      handlers.set(event, h);
      return () => {
        handlers.delete(event);
      };
    };
    const session = createRemoteSession(CONN, openInfo(), undefined, { invoke, listen });
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

    handlers.get("remote-workspace-changed")?.({ payload: "remote:other/x/y" });
    expect(fired).toBe(0);
    handlers.get("remote-workspace-changed")?.({ payload: session.id });
    expect(fired).toBe(1);

    unsubscribe();
    await Promise.resolve();
    expect(calls.map((c) => c.cmd)).toContain("remote_unwatch");
    handlers.get("remote-workspace-changed")?.({ payload: session.id });
    expect(fired).toBe(1);
  });

  it("local sessions 除 RemoteOnly 的 claim 外全真（決策 2：同一 UI 路徑）", () => {
    const { invoke } = fakeInvoke();
    const session = createLocalSession("/proj/a", { invoke });
    const { claim, ...rest } = session.capabilities;
    expect(Object.values(rest).every((on) => on === true)).toBe(true);
    // claim 是 RemoteOnly 動詞：本地沒有共用者可撞工，能力為假而非偽造為真。
    expect(claim).toBe(false);
  });
});

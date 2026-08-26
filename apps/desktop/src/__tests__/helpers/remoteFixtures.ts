// remote 測試共用 fixtures：capability 矩陣（決策 1）、假 remote dataSource
// 與假 remote session——remoteOpen（store 面）與 remoteCapabilities（App 面）
// 測試共用。
import { vi } from "vitest";
import type { SpeclinkDataSource } from "@speclink/ui";

import type { WorkspaceCapabilities, WorkspaceSession } from "../../session";

export const REMOTE_KEY = "remote:c1/demo/backend";

/** remote capability 描述：直達／組合真、不支援假（決策 1 矩陣）。 */
export const REMOTE_CAPS: WorkspaceCapabilities = {
  listChanges: true,
  listSpecs: true,
  listArchived: true,
  status: true,
  getDocument: true,
  getSpecDocument: true,
  searchWorkspace: true,
  changeCapabilities: true,
  changeMeta: true,
  deleteChange: false,
  setTaskDone: true,
  setAllTasks: true,
  moveTask: false,
  validate: false,
  analyze: false,
  archive: true,
  getArchivedDocument: true,
  archivedCapabilities: true,
  listDiscussions: true,
  getDiscussionDocument: true,
  promoteDiscussion: true,
  archiveDiscussion: true,
  reorderCard: false,
  policyWrite: true,
  liveUpdates: true,
};

/** 假 remote dataSource：直達方法回真值樣本；其餘不支援方法拒絕（鏡射 remote adapter）。 */
export function fakeRemoteDs(over: Partial<SpeclinkDataSource> = {}): SpeclinkDataSource {
  const refuse = () => Promise.reject(new Error("此 server 尚未提供——功能已停用"));
  return {
    listChanges: vi.fn().mockResolvedValue([
      // 已就緒（2/2）：抽屜封存鈕的階段守門放行，capability 測試聚焦能力缺口本身。
      { name: "remote-change", status: "in-progress", totalTasks: 2, completedTasks: 2 },
    ]),
    listSpecs: vi.fn().mockResolvedValue([{ id: "auth" }]),
    listArchived: vi.fn().mockResolvedValue([
      {
        datedName: "2026-07-04-remote-old",
        date: "2026-07-04",
        name: "remote-old",
        tasksTotal: 1,
        tasksDone: 1,
        specCount: 1,
        // 與抽屜詮釋資料的 "Remote Creator" 錯開——負向斷言（舊 server 缺
        // 歸屬欄位）以 queryByText 驗缺席，封存卡字串不得撞名。
        createdBy: "Archived Author",
        fromDiscussions: [],
      },
    ]),
    status: vi.fn().mockResolvedValue({
      changeName: "remote-change",
      schemaName: "spec-driven",
      isComplete: false,
      applyRequires: ["tasks"],
      artifacts: [],
    }),
    getDocument: vi.fn().mockResolvedValue("- [ ] 1.1 First\n- [ ] 1.2 Second\n"),
    getSpecDocument: vi.fn().mockResolvedValue("# auth Specification\n\nRemote canonical truth.\n"),
    searchWorkspace: vi.fn().mockResolvedValue([]),
    // remote-read-parity：詮釋資料與 capability 清單以 status payload 映射直達。
    changeCapabilities: vi.fn().mockResolvedValue(["auth"]),
    changeMeta: vi.fn().mockResolvedValue({
      schema: "spec-driven",
      created: "2026-07-29",
      createdBy: "Remote Creator",
      createdWith: "claude-code",
      fromDiscussions: [],
      startedAt: "2026-08-25T00:00:00Z",
      startedBy: "Remote Creator",
    }),
    deleteChange: vi.fn(refuse),
    setTaskDone: vi.fn().mockResolvedValue(undefined),
    setAllTasks: vi.fn().mockResolvedValue(undefined),
    moveTask: vi.fn(refuse),
    runVerb: vi.fn().mockResolvedValue({}),
    getArchivedDocument: vi.fn().mockResolvedValue("## Why\n\nRemote archived truth.\n"),
    archivedCapabilities: vi.fn().mockResolvedValue(["auth"]),
    listDiscussions: vi.fn().mockResolvedValue({ active: [], archived: [] }),
    getDiscussionDocument: vi.fn().mockResolvedValue(null),
    promoteDiscussion: vi.fn().mockResolvedValue({ change: "chg" }),
    archiveDiscussion: vi.fn().mockResolvedValue(undefined),
    reorderCard: vi.fn(refuse),
    ...over,
  } as unknown as SpeclinkDataSource;
}

/** 假 remote session（handshake 成功後的形狀）。 */
export function fakeRemoteSession(ds: SpeclinkDataSource): WorkspaceSession {
  return {
    id: REMOTE_KEY,
    locator: { kind: "remote", connectionId: "c1", projectId: "demo", repoId: "backend" },
    descriptor: { name: "Demo/backend", badge: null },
    dataSource: ds,
    settings: {
      kind: "remote",
      policyWrite: true,
      readSettings: vi.fn(),
      writeAppTools: vi.fn(),
      writeWorkflowConfig: vi.fn(),
      writeWorkflowContext: vi.fn(),
      writeWorkflowRules: vi.fn(),
      readSchemas: vi.fn(),
      writeWorkflowSchema: vi.fn(),
      forkSchema: vi.fn(),
      createSchema: vi.fn(),
      revealSchema: vi.fn(),
      deleteSchema: vi.fn(),
    },
    events: { subscribe: () => () => {} },
    capabilities: REMOTE_CAPS,
  };
}

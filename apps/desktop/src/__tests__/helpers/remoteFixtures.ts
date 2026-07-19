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

/** 假 remote dataSource：直達方法回真值樣本；不支援方法一律拒絕（鏡射 remote adapter）。 */
export function fakeRemoteDs(over: Partial<SpeclinkDataSource> = {}): SpeclinkDataSource {
  const refuse = () => Promise.reject(new Error("此 server 尚未提供——功能已停用"));
  return {
    listChanges: vi.fn().mockResolvedValue([
      { name: "remote-change", status: "in-progress", totalTasks: 2, completedTasks: 0 },
    ]),
    listSpecs: vi.fn().mockResolvedValue([{ id: "auth" }]),
    listArchived: vi.fn(refuse),
    status: vi.fn().mockResolvedValue({
      changeName: "remote-change",
      schemaName: "spec-driven",
      isComplete: false,
      applyRequires: ["tasks"],
      artifacts: [],
    }),
    getDocument: vi.fn().mockResolvedValue("- [ ] 1.1 First\n- [ ] 1.2 Second\n"),
    getSpecDocument: vi.fn(refuse),
    searchWorkspace: vi.fn(refuse),
    changeCapabilities: vi.fn(refuse),
    changeMeta: vi.fn(refuse),
    deleteChange: vi.fn(refuse),
    setTaskDone: vi.fn().mockResolvedValue(undefined),
    setAllTasks: vi.fn().mockResolvedValue(undefined),
    moveTask: vi.fn(refuse),
    runVerb: vi.fn().mockResolvedValue({}),
    getArchivedDocument: vi.fn(refuse),
    archivedCapabilities: vi.fn(refuse),
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
      readSettings: vi.fn(),
      writeAppTools: vi.fn(),
      writeWorkflowConfig: vi.fn(),
      writeWorkflowContext: vi.fn(),
      writeWorkflowRules: vi.fn(),
    },
    events: { subscribe: () => () => {} },
    capabilities: REMOTE_CAPS,
  };
}

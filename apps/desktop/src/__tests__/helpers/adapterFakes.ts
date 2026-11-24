// 測試共用的假 adapter：每個方法都有安全的預設，個別測試只覆寫它在意的方法。介面新增方法時
// 只要補這裡，各測試的假物件不必逐一跟著改。
import { vi } from "vitest";

import type { ConnectionsAdapter } from "../../adapter/connections";
import type { WorkspaceAdapter } from "../../adapter/workspace";

/** 假 connections adapter：清單為空、連線檢查回 online、狀態訂閱不會送事件。 */
export function fakeConnectionsAdapter(over: Partial<ConnectionsAdapter> = {}): ConnectionsAdapter {
  return {
    list: vi.fn().mockResolvedValue([]),
    add: vi.fn(),
    remove: vi.fn().mockResolvedValue(undefined),
    loginStart: vi.fn(),
    loginCancel: vi.fn().mockResolvedValue(undefined),
    patLogin: vi.fn(),
    logout: vi.fn().mockResolvedValue({ revokedOnServer: true, patNotice: false }),
    scopes: vi.fn().mockResolvedValue({ projects: [] }),
    checkConnection: vi.fn(async (connectionId: string) => ({ connectionId, state: "online" as const, message: null })),
    onConnectionState: vi.fn().mockResolvedValue(() => {}),
    inspectCheckout: vi.fn(),
    bindCheckout: vi.fn(),
    ...over,
  };
}

/** 假 workspace adapter：不是專案目錄、沒有啟動目錄、背景探測成功。 */
export function fakeWorkspaceAdapter(over: Partial<WorkspaceAdapter> = {}): WorkspaceAdapter {
  return {
    openProject: vi.fn().mockRejectedValue("not a project"),
    initProject: vi.fn(),
    startupDir: vi.fn().mockRejectedValue("no startup dir"),
    projectStats: vi.fn().mockResolvedValue({ pendingWrapUp: 0 }),
    watchWorkspace: vi.fn().mockResolvedValue(undefined),
    pickFolder: vi.fn().mockResolvedValue(null),
    ...over,
  } as unknown as WorkspaceAdapter;
}

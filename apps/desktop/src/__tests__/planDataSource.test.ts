// 前置編輯的資料面（add-change-plan-desktop design D6）：Tauri 資料源以正確參數
// 委派 set_change_depends、remote 資料源在第三刀前拒絕、capability 旗標 local 真
// ／remote 假（離線遮罩亦為假）。
import { describe, it, expect, vi, beforeEach } from "vitest";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

import { createTauriDataSource } from "../adapter/tauriDataSource";
import { createRemoteDataSource } from "../adapter/remoteDataSource";
import {
  applyRemoteConnectionState,
  createRemoteSession,
  LOCAL_CAPABILITIES,
  type RemoteOpenInfo,
} from "../session";
import { REMOTE_CAPS } from "./helpers/remoteFixtures";

beforeEach(() => invoke.mockReset());

describe("setDepends 資料源委派", () => {
  it("tauri 資料源以 { root, change, on, remove } 呼叫 set_change_depends", async () => {
    invoke.mockResolvedValueOnce(undefined);
    const ds = createTauriDataSource("/r");
    await ds.setDepends("add-b", ["add-a"], false);
    expect(invoke).toHaveBeenCalledWith("set_change_depends", {
      root: "/r",
      change: "add-b",
      on: ["add-a"],
      remove: false,
    });
    invoke.mockResolvedValueOnce(undefined);
    await ds.setDepends("add-b", ["add-a"], true);
    expect(invoke).toHaveBeenLastCalledWith("set_change_depends", {
      root: "/r",
      change: "add-b",
      on: ["add-a"],
      remove: true,
    });
  });

  it("remote 資料源拒絕 setDepends 且不發任何請求（第三刀前不可用）", async () => {
    const remoteInvoke = vi.fn();
    const ds = createRemoteDataSource("c1", "demo", "backend", remoteInvoke);
    await expect(ds.setDepends("add-b", ["add-a"], false)).rejects.toThrow();
    expect(remoteInvoke).not.toHaveBeenCalled();
  });
});

describe("setDepends capability 旗標", () => {
  it("local 全真：setDepends 為 true", () => {
    expect(LOCAL_CAPABILITIES.setDepends).toBe(true);
  });

  it("remote 為 false，且離線遮罩後仍為 false", () => {
    const info: RemoteOpenInfo = {
      projectKey: "demo",
      projectName: "Demo",
      repoKey: "backend",
      repoName: "backend",
      capabilities: REMOTE_CAPS,
    };
    const session = createRemoteSession("c1", info, undefined, { invoke: vi.fn() });
    expect(session.capabilities.setDepends).toBe(false);
    // 寫入面在離線時一律遮罩：即使 handshake 曾宣告為真也不得殘留。
    const online = createRemoteSession(
      "c1",
      { ...info, capabilities: { ...REMOTE_CAPS, setDepends: true } },
      undefined,
      { invoke: vi.fn() },
    );
    const offline = applyRemoteConnectionState(online, {
      connectionId: "c1",
      state: "offline",
      message: null,
    });
    expect(offline.capabilities.setDepends).toBe(false);
  });
});

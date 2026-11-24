// remote-resilience「背景遠端分頁的連線定期檢查」的前端半邊：store 決定何時、對哪幾條
// connection 觸發檢查（去重、不重疊、只有本機分頁時不檢查），並把 Rust 判定的連線狀態
// （廣播的事件與檢查命令的回傳）記成每條 connection 一份、套到同 connection 的 session。
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { createAppStore } from "../store";
import { createConnectionsAdapter, type ConnectionsAdapter } from "../adapter/connections";
import type { InvokeFn, ListenFn } from "../session";
import type { ProjectTab } from "../tabs";
import { fakeConnectionsAdapter, fakeWorkspaceAdapter } from "./helpers/adapterFakes";
import { fakeRemoteDs, fakeRemoteSession, REMOTE_KEY } from "./helpers/remoteFixtures";

const online = (connectionId: string) => ({ connectionId, state: "online" as const, message: null });

function fakeConnections(checkConnection: ConnectionsAdapter["checkConnection"] = vi.fn(async (id: string) => online(id))) {
  return { adapter: fakeConnectionsAdapter({ checkConnection }), checkConnection };
}

const openRemote = vi.fn();

function storeWith(connections: ConnectionsAdapter) {
  return createAppStore({
    createSession: () => {
      throw new Error("不應建立 local session");
    },
    openRemote,
    workspace: fakeWorkspaceAdapter(),
    connections,
  });
}

const remoteTab = (connectionId: string, repoId: string): ProjectTab => ({
  locator: { kind: "remote", connectionId, projectId: "demo", repoId },
  name: `Demo/${repoId}`,
});
const localTab: ProjectTab = { locator: { kind: "local", root: "/p/local" }, name: "local" };

beforeEach(() => {
  localStorage.removeItem("speclink.projectTabs");
});

afterEach(() => {
  vi.useRealTimers();
});

describe("checkRemoteConnections", () => {
  it("同一條 connection 的兩個遠端分頁只檢查一次，本機分頁不檢查", async () => {
    const { adapter, checkConnection } = fakeConnections();
    const store = storeWith(adapter);
    store.setState({ tabs: [remoteTab("c1", "backend"), localTab, remoteTab("c1", "frontend")] });

    await store.getState().checkRemoteConnections();

    expect(checkConnection).toHaveBeenCalledTimes(1);
    expect(checkConnection).toHaveBeenCalledWith("c1");
  });

  it("只有本機分頁時不送檢查", async () => {
    const { adapter, checkConnection } = fakeConnections();
    const store = storeWith(adapter);
    store.setState({ tabs: [localTab] });

    await store.getState().checkRemoteConnections();

    expect(checkConnection).not.toHaveBeenCalled();
  });

  it("同一條 connection 的檢查還沒完成時不重複送", async () => {
    let finish!: () => void;
    const { adapter, checkConnection } = fakeConnections(
      vi.fn((id: string) => new Promise<ReturnType<typeof online>>((resolve) => (finish = () => resolve(online(id))))),
    );
    const store = storeWith(adapter);
    store.setState({ tabs: [remoteTab("c1", "backend")] });

    const first = store.getState().checkRemoteConnections();
    void store.getState().checkRemoteConnections();
    expect(checkConnection).toHaveBeenCalledTimes(1);

    finish();
    await first;
    void store.getState().checkRemoteConnections();
    expect(checkConnection).toHaveBeenCalledTimes(2);
  });

  it("命令回傳的目前狀態寫進 connectionStates（狀態沒變、沒有事件時也補得回來）", async () => {
    const { adapter } = fakeConnections(
      vi.fn(async (id: string) => ({ connectionId: id, state: "offline" as const, message: "離線" })),
    );
    const store = storeWith(adapter);
    store.setState({ tabs: [remoteTab("c1", "backend")] });

    await store.getState().checkRemoteConnections();

    expect(store.getState().connectionStates.c1?.state).toBe("offline");
  });

  it("檢查被拒時靜默略過，下一次照常檢查", async () => {
    const { adapter, checkConnection } = fakeConnections(
      vi.fn().mockRejectedValue("連線不存在——請重新設定 server 連線"),
    );
    const store = storeWith(adapter);
    store.setState({ tabs: [remoteTab("c1", "backend")] });

    await expect(store.getState().checkRemoteConnections()).resolves.toBeUndefined();
    await store.getState().checkRemoteConnections();

    expect(checkConnection).toHaveBeenCalledTimes(2);
  });
});

describe("startConnectionChecks", () => {
  it("每 60 秒檢查一次，停止後不再檢查", async () => {
    vi.useFakeTimers();
    const { adapter, checkConnection } = fakeConnections();
    const store = storeWith(adapter);
    store.setState({ tabs: [remoteTab("c1", "backend")] });

    const stop = store.getState().startConnectionChecks();
    await vi.advanceTimersByTimeAsync(60_000);
    expect(checkConnection).toHaveBeenCalledTimes(1);

    stop();
    await vi.advanceTimersByTimeAsync(60_000);
    expect(checkConnection).toHaveBeenCalledTimes(1);
  });
});

describe("restoreTabs", () => {
  it("還原出遠端分頁時觸發一次檢查", async () => {
    localStorage.setItem(
      "speclink.projectTabs",
      JSON.stringify({ version: 2, tabs: [remoteTab("c1", "backend"), localTab], activeKey: null }),
    );
    const { adapter, checkConnection } = fakeConnections();
    const store = storeWith(adapter);

    await store.getState().restoreTabs();

    expect(checkConnection).toHaveBeenCalledTimes(1);
    expect(checkConnection).toHaveBeenCalledWith("c1");
  });
});

describe("applyConnectionStateEvent", () => {
  it("記下 connection 的狀態，並把同 connection 的 session 標為 offline", () => {
    const { adapter } = fakeConnections();
    const store = storeWith(adapter);
    store.setState({ sessions: { [REMOTE_KEY]: fakeRemoteSession(fakeRemoteDs()) } });

    store.getState().applyConnectionStateEvent({ connectionId: "c1", state: "offline", message: "離線" });

    const state = store.getState();
    expect(state.connectionStates.c1?.state).toBe("offline");
    expect(state.sessions[REMOTE_KEY]?.connectionState?.state).toBe("offline");
  });

  it("同一則狀態再收到一次時不更新 store", () => {
    const { adapter } = fakeConnections();
    const store = storeWith(adapter);
    store.setState({ sessions: { [REMOTE_KEY]: fakeRemoteSession(fakeRemoteDs()) } });
    const event = { connectionId: "c1", state: "offline" as const, message: "離線" };
    store.getState().applyConnectionStateEvent(event);
    const listener = vi.fn();
    store.subscribe(listener);

    store.getState().applyConnectionStateEvent({ ...event });

    expect(listener).not.toHaveBeenCalled();
  });

  it("回到 online 只拿掉狀態：沒有 session 的分頁不建立 session、不 handshake", () => {
    const { adapter } = fakeConnections();
    const store = storeWith(adapter);
    store.setState({ tabs: [remoteTab("c1", "backend")] });
    store.getState().applyConnectionStateEvent({ connectionId: "c1", state: "offline", message: "離線" });

    store.getState().applyConnectionStateEvent(online("c1"));

    expect(store.getState().connectionStates.c1?.state).toBe("online");
    expect(store.getState().sessions).toEqual({});
    expect(openRemote).not.toHaveBeenCalled();
  });
});

describe("登出與移除連線清掉 connection 的狀態", () => {
  const connection = { id: "c1", origin: "http://server.test", name: "Server", loggedIn: true };

  it("logoutConnection 清掉同 origin 的 connection 狀態", async () => {
    const { adapter } = fakeConnections();
    const store = storeWith(adapter);
    store.setState({ connections: [connection] });
    store.getState().applyConnectionStateEvent({ connectionId: "c1", state: "needs-reauth", message: "請重新登入" });

    await store.getState().logoutConnection("http://server.test");

    expect(store.getState().connectionStates.c1).toBeUndefined();
  });

  it.each([
    ["logoutConnection", (store: ReturnType<typeof storeWith>) => store.getState().logoutConnection("http://server.test")],
    ["removeConnection", (store: ReturnType<typeof storeWith>) => store.getState().removeConnection("c1")],
  ])("%s 之前送出、之後才回來的檢查結果不寫回", async (_name, forget) => {
    let finish!: () => void;
    const { adapter } = fakeConnections(
      vi.fn(
        (id: string) =>
          new Promise<{ connectionId: string; state: "offline"; message: string }>(
            (resolve) => (finish = () => resolve({ connectionId: id, state: "offline", message: "離線" })),
          ),
      ),
    );
    const store = storeWith(adapter);
    store.setState({ connections: [connection], tabs: [remoteTab("c1", "backend")] });

    const checking = store.getState().checkRemoteConnections();
    await forget(store);
    finish();
    await checking;

    expect(store.getState().connectionStates.c1).toBeUndefined();
  });

  it("removeConnection 清掉該 connection 的狀態", async () => {
    const { adapter } = fakeConnections();
    const store = storeWith(adapter);
    store.setState({ connections: [connection] });
    store.getState().applyConnectionStateEvent({ connectionId: "c1", state: "offline", message: "離線" });

    await store.getState().removeConnection("c1");

    expect(store.getState().connectionStates.c1).toBeUndefined();
  });
});

describe("connections adapter", () => {
  it("checkConnection 以 connectionId 呼叫 remote_check_connection，回傳目前狀態", async () => {
    const invoke = vi.fn().mockResolvedValue(online("c1")) as unknown as InvokeFn;
    await expect(createConnectionsAdapter(invoke).checkConnection("c1")).resolves.toEqual(online("c1"));
    expect(invoke).toHaveBeenCalledWith("remote_check_connection", { connectionId: "c1" });
  });

  it("onConnectionState 只轉交形狀正確的 remote-connection-state 事件", async () => {
    let handler!: (e: { payload: unknown }) => void;
    const listen = vi.fn(async (_event: string, h: typeof handler) => {
      handler = h;
      return () => {};
    }) as unknown as ListenFn;
    const listener = vi.fn();
    await createConnectionsAdapter(vi.fn() as unknown as InvokeFn, listen).onConnectionState(listener);

    handler({ payload: { connectionId: "c1", state: "offline", message: null } });
    handler({ payload: { connectionId: "c1", state: "broken", message: null } });

    expect(listen).toHaveBeenCalledWith("remote-connection-state", expect.any(Function));
    expect(listener).toHaveBeenCalledTimes(1);
    expect(listener).toHaveBeenCalledWith({ connectionId: "c1", state: "offline", message: null });
  });
});

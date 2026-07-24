// remote-resilience：Rust 廣播是連線狀態單一真相；offline 時保留最後 snapshot、
// 呈現 stale/cloud-off 並以既有 capability 管線停用全部寫入。本地 session 不受影響。
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { App } from "../App";
import type { ConnectionsAdapter } from "../adapter/connections";
import {
  applyRemoteConnectionState,
  createLocalSession,
  createRemoteSession,
  type InvokeFn,
  type ListenFn,
  type RemoteOpenInfo,
  type WorkspaceSession,
} from "../session";
import { fakeRemoteDs, REMOTE_CAPS, REMOTE_KEY } from "./helpers/remoteFixtures";

vi.mock("@speclink/ui", async (importOriginal) => {
  const mod = await importOriginal<typeof import("@speclink/ui")>();
  return { ...mod, Toaster: () => <div data-testid="app-toaster" /> };
});

type EventHandler = (event: { payload: unknown }) => void;

function fakeEvents() {
  const handlers = new Map<string, Set<EventHandler>>();
  const listen = vi.fn(async (event: string, handler: EventHandler) => {
    const bucket = handlers.get(event) ?? new Set<EventHandler>();
    bucket.add(handler);
    handlers.set(event, bucket);
    return () => bucket.delete(handler);
  }) as unknown as ListenFn;
  return {
    listen,
    emit(event: string, payload: unknown) {
      for (const handler of handlers.get(event) ?? []) handler({ payload });
    },
    count(event: string) {
      return handlers.get(event)?.size ?? 0;
    },
  };
}

function fakeWorkspace() {
  return {
    openProject: vi.fn().mockRejectedValue("not a project"),
    initProject: vi.fn(),
    startupDir: vi.fn().mockRejectedValue("no startup dir"),
    projectStats: vi.fn().mockResolvedValue({ pendingWrapUp: 0 }),
    watchWorkspace: vi.fn().mockResolvedValue(undefined),
    pickFolder: vi.fn().mockResolvedValue(null),
  };
}

const INFO: RemoteOpenInfo = {
  projectKey: "demo",
  projectName: "Demo",
  repoKey: "backend",
  repoName: "Backend",
  capabilities: REMOTE_CAPS,
};

function remoteInvoke() {
  let reachable = true;
  let changes = [
    { name: "remote-change", status: "in-progress", totalTasks: 2, completedTasks: 0 },
  ];
  const invoke = vi.fn(async (command: string) => {
    if (command === "remote_watch" || command === "remote_unwatch") return undefined;
    if (!reachable) throw new Error("server unreachable");
    switch (command) {
      case "remote_list_changes":
        return { changes };
      case "remote_list_specs":
        return { specs: [{ id: "auth" }] };
      case "remote_list_archived":
        return { archived: [] };
      case "remote_list_discussions":
        return { active: [], archived: [] };
      case "remote_document":
        return "- [ ] 1.1 First\n- [ ] 1.2 Second\n";
      default:
        return null;
    }
  }) as unknown as InvokeFn;
  return {
    invoke,
    setReachable(value: boolean) {
      reachable = value;
    },
    addRecoveredChange() {
      changes = [
        ...changes,
        { name: "recovered-change", status: "proposed", totalTasks: 0, completedTasks: 0 },
      ];
    },
  };
}

beforeEach(() => {
  localStorage.removeItem("speclink.projectTabs");
  localStorage.setItem("speclink.uiLocale", "zh-TW");
});

function persistRemote() {
  localStorage.setItem(
    "speclink.projectTabs",
    JSON.stringify({
      version: 2,
      tabs: [
        {
          locator: { kind: "remote", connectionId: "c1", projectId: "demo", repoId: "backend" },
          name: "Demo/Backend",
        },
      ],
      activeKey: REMOTE_KEY,
    }),
  );
}

describe("remote offline stale snapshot", () => {
  it("啟動時 handshake 失敗會以作用中分頁顯示復原目的地，不洩漏上一個 workspace 資料", async () => {
    persistRemote();
    const openRemote = vi.fn().mockRejectedValue({
      message: "server unreachable — internal transport diagnostics",
      reason: null,
      status: null,
    });

    render(
      <App
        createSession={() => {
          throw new Error("remote 復原不得建立 local session");
        }}
        openRemote={openRemote}
        workspace={fakeWorkspace() as never}
      />,
    );

    expect((await screen.findByRole("alert")).textContent).toContain("無法連線到伺服器");
    expect(
      screen.getByRole("tab", { name: /Demo\/Backend/ }).getAttribute("aria-selected"),
    ).toBe("true");
    expect(screen.queryByText("remote-change")).toBeNull();
    expect(screen.queryByText("previous-workspace-change")).toBeNull();
  });

  it("保留清單、呈現 stale/cloud-off，並停用任務與封存寫入", async () => {
    persistRemote();
    const events = fakeEvents();
    const network = remoteInvoke();
    const session = createRemoteSession("c1", INFO, undefined, {
      invoke: network.invoke,
      listen: events.listen,
    });
    const openRemote = vi.fn(async (): Promise<WorkspaceSession> => session);
    expect(session.capabilities.setTaskDone).toBe(true);
    expect(
      applyRemoteConnectionState(session, {
        connectionId: "c1",
        state: "offline",
        message: "offline",
      }).capabilities.setTaskDone,
    ).toBe(false);
    render(
      <App
        createSession={() => {
          throw new Error("remote 流程不得建立 local session");
        }}
        openRemote={openRemote}
        workspace={fakeWorkspace() as never}
      />,
    );

    await screen.findByText("remote-change");
    await waitFor(() => expect(events.count("remote-connection-state")).toBe(1));
    fireEvent.click(screen.getByText("remote-change"));
    const archive = (await screen.findByRole("button", { name: "封存" })) as HTMLButtonElement;
    fireEvent.click(screen.getByRole("tab", { name: /任務/ }));
    await screen.findAllByRole("checkbox");
    expect(archive.disabled).toBe(false);

    network.setReachable(false);
    act(() => {
      events.emit("remote-connection-state", {
        connectionId: "c1",
        state: "offline",
        message: "此連線目前離線——顯示最後成功載入的內容",
      });
      events.emit("remote-workspace-changed", REMOTE_KEY);
    });

    expect((await screen.findByTestId("remote-stale-banner")).textContent).toContain("離線");
    expect(screen.getAllByText("remote-change").length).toBeGreaterThan(0);
    expect(document.querySelector(`[data-cloud-off="${REMOTE_KEY}"]`)).toBeTruthy();
    expect((screen.getByRole("button", { name: "封存" }) as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getAllByRole("checkbox")[0] as HTMLInputElement).disabled).toBe(true);
  });

  it("同一個 remote 狀態事件不改變本地分頁的內容與寫入能力", async () => {
    localStorage.setItem(
      "speclink.projectTabs",
      JSON.stringify({
        version: 2,
        tabs: [{ locator: { kind: "local", root: "A" }, name: "Local" }],
        activeKey: "local:A",
      }),
    );
    const events = fakeEvents();
    const workspace = fakeWorkspace();
    workspace.openProject = vi
      .fn()
      .mockResolvedValue({ status: "project", root: "A", name: "Local" });
    const ds = fakeRemoteDs({
      listChanges: vi.fn().mockResolvedValue([
        { name: "local-change", status: "in-progress", totalTasks: 2, completedTasks: 0 },
      ]),
      changeCapabilities: vi.fn().mockResolvedValue([]),
      changeMeta: vi.fn().mockResolvedValue(null),
    });
    render(
      <App
        createSession={(root, name) => {
          const local = createLocalSession(root, { name, invoke: vi.fn() as never, listen: events.listen });
          return { ...local, dataSource: ds };
        }}
        workspace={workspace as never}
      />,
    );
    await screen.findByText("local-change");
    act(() => {
      events.emit("remote-connection-state", {
        connectionId: "c1",
        state: "offline",
        message: "offline",
      });
    });
    expect(screen.queryByTestId("remote-stale-banner")).toBeNull();
    fireEvent.click(screen.getByText("local-change"));
    const archive = (await screen.findByRole("button", { name: "封存" })) as HTMLButtonElement;
    expect(archive.disabled).toBe(false);
  });

  it("online 事件會自動全量重查、載入恢復期間的新 change 並清除 stale", async () => {
    persistRemote();
    const events = fakeEvents();
    const network = remoteInvoke();
    const session = createRemoteSession("c1", INFO, undefined, {
      invoke: network.invoke,
      listen: events.listen,
    });
    render(
      <App
        createSession={() => {
          throw new Error("remote 流程不得建立 local session");
        }}
        openRemote={async () => session}
        workspace={fakeWorkspace() as never}
      />,
    );
    await screen.findByText("remote-change");
    await waitFor(() => expect(events.count("remote-connection-state")).toBe(1));

    act(() => {
      events.emit("remote-connection-state", {
        connectionId: "c1",
        state: "offline",
        message: "offline",
      });
    });
    await screen.findByTestId("remote-stale-banner");

    network.addRecoveredChange();
    act(() => {
      events.emit("remote-connection-state", {
        connectionId: "c1",
        state: "online",
        message: null,
      });
    });

    await screen.findByText("recovered-change");
    expect(screen.queryByTestId("remote-stale-banner")).toBeNull();
    expect(document.querySelector(`[data-cloud-off="${REMOTE_KEY}"]`)).toBeNull();
  });

  it("needs-reauth 導向聚焦登入，成功後依序 re-handshake 全部 remote sessions、重查並重掛 worker", async () => {
    const backendKey = "remote:c1/demo/backend";
    const frontendKey = "remote:c1/demo/frontend";
    localStorage.setItem(
      "speclink.projectTabs",
      JSON.stringify({
        version: 2,
        tabs: [
          {
            locator: { kind: "remote", connectionId: "c1", projectId: "demo", repoId: "backend" },
            name: "Demo/Backend",
          },
          {
            locator: { kind: "remote", connectionId: "c1", projectId: "demo", repoId: "frontend" },
            name: "Demo/Frontend",
          },
        ],
        activeKey: backendKey,
      }),
    );
    const events = fakeEvents();
    const order: string[] = [];
    const invoke = vi.fn(async (command: string, args: Record<string, unknown> = {}) => {
      const repo = typeof args.repo === "string" ? args.repo : "";
      order.push(repo ? `${command}:${repo}` : command);
      switch (command) {
        case "remote_watch":
        case "remote_unwatch":
          return undefined;
        case "remote_list_changes":
          return {
            changes: [
              {
                name: `${repo}-change`,
                status: "in-progress",
                totalTasks: 1,
                completedTasks: 0,
              },
            ],
          };
        case "remote_list_specs":
          return { specs: [] };
        case "remote_list_archived":
          return { archived: [] };
        case "remote_list_discussions":
          return { active: [], archived: [] };
        default:
          return null;
      }
    }) as unknown as InvokeFn;
    const openRemote = vi.fn(async (_connectionId: string, target: string) => {
      order.push(`handshake:${target}`);
      const repo = target.split("/")[1];
      return createRemoteSession(
        "c1",
        {
          ...INFO,
          repoKey: repo,
          repoName: repo === "backend" ? "Backend" : "Frontend",
        },
        undefined,
        { invoke, listen: events.listen },
      );
    });
    const connection = {
      id: "c1",
      origin: "http://server.test",
      name: "Server",
      loggedIn: true,
    };
    const connections: ConnectionsAdapter = {
      list: vi.fn(async () => {
        order.push("connection-list");
        return [connection];
      }),
      add: vi.fn(),
      remove: vi.fn(),
      deviceLogin: vi.fn(async () => {
        order.push("login");
        return { status: "loggedIn", display: "Dev" };
      }),
      patLogin: vi.fn(),
      logout: vi.fn(),
      scopes: vi.fn(),
      inspectCheckout: vi.fn(),
      bindCheckout: vi.fn(),
    } as ConnectionsAdapter;
    const createSession = vi.fn(() => {
      throw new Error("重新認證不得建立 local session");
    });
    render(
      <App
        createSession={createSession}
        openRemote={openRemote}
        workspace={fakeWorkspace() as never}
        connections={connections}
      />,
    );

    await screen.findByText("backend-change");
    fireEvent.click(document.querySelector(`[data-tab="${frontendKey}"]`) as HTMLElement);
    await screen.findByText("frontend-change");
    fireEvent.click(document.querySelector(`[data-tab="${backendKey}"]`) as HTMLElement);
    await screen.findByText("backend-change");
    order.length = 0;

    act(() => {
      events.emit("remote-connection-state", {
        connectionId: "c1",
        state: "needs-reauth",
        message: "登入已失效",
      });
    });
    fireEvent.click(await screen.findByRole("button", { name: "重新登入" }));
    const login = await screen.findByTestId("reauth-login-c1");
    await waitFor(() => expect(document.activeElement).toBe(login));
    expect(document.querySelectorAll("[data-tab]")).toHaveLength(2);

    fireEvent.click(login);
    await screen.findByText("backend-change");
    await waitFor(() => expect(screen.queryByTestId("remote-stale-banner")).toBeNull());
    await waitFor(() => expect(order).toContain("remote_watch:backend"));

    const loginIndex = order.indexOf("login");
    const backendHandshake = order.indexOf("handshake:demo/backend");
    const frontendHandshake = order.indexOf("handshake:demo/frontend");
    const reload = order.indexOf("remote_list_changes:backend");
    const unwatch = order.indexOf("remote_unwatch:backend");
    const watch = order.lastIndexOf("remote_watch:backend");
    expect(loginIndex).toBeGreaterThanOrEqual(0);
    expect(backendHandshake).toBeGreaterThan(loginIndex);
    expect(frontendHandshake).toBeGreaterThan(backendHandshake);
    expect(reload).toBeGreaterThan(frontendHandshake);
    expect(unwatch).toBeGreaterThan(reload);
    expect(watch).toBeGreaterThan(unwatch);
    expect(document.querySelectorAll("[data-tab]")).toHaveLength(2);
    expect(document.querySelector(`[data-tab="${backendKey}"]`)).toBeTruthy();
    expect(document.querySelector(`[data-tab="${frontendKey}"]`)).toBeTruthy();
    expect(createSession).not.toHaveBeenCalled();
  });
});

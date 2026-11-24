// remote-resilience：Rust 廣播是連線狀態單一真相；offline 時保留最後 snapshot、
// 呈現 stale/cloud-off 並以既有 capability 管線停用全部寫入。本地 session 不受影響。
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { App } from "../App";
import type { ConnectionsAdapter } from "../adapter/connections";
import { SEMANTIC_SURFACE } from "@speclink/ui";
import {
  applyRemoteAccessState,
  applyRemoteConnectionState,
  createLocalSession,
  createRemoteSession,
  remoteWorkspaceStatus,
  type InvokeFn,
  type ListenFn,
  type RemoteConnectionState,
  type RemoteOpenInfo,
  type WorkspaceCapabilities,
  type WorkspaceSession,
} from "../session";
import { changeList } from "./helpers/changeList";
import { fakeConnectionsAdapter, fakeWorkspaceAdapter } from "./helpers/adapterFakes";
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
    // 已就緒（2/2）：封存鈕的階段守門放行，離線測試聚焦連線狀態對寫入的影響。
    { name: "remote-change", status: "in-progress", totalTasks: 2, completedTasks: 2 },
  ];
  const invoke = vi.fn(async (command: string) => {
    if (command === "remote_watch" || command === "remote_unwatch") return undefined;
    if (!reachable) throw new Error("server unreachable");
    switch (command) {
      case "remote_list_changes":
        return { changes, planError: null, next: null, unpromotedCuts: [] };
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
        workspace={fakeWorkspaceAdapter() as never}
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
        workspace={fakeWorkspaceAdapter() as never}
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
    // 圖示列方塊：離線＝角落琥珀點。
    const square = Array.from(document.querySelectorAll("[data-tab]")).find(
      (el) => el.getAttribute("data-tab") === REMOTE_KEY,
    );
    expect(square?.querySelector('[data-tab-dot="warning"]')).toBeTruthy();
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
    const workspace = fakeWorkspaceAdapter();
    workspace.openProject = vi
      .fn()
      .mockResolvedValue({ status: "project", root: "A", name: "Local" });
    const ds = fakeRemoteDs({
      listChanges: vi.fn().mockResolvedValue(changeList([
        // 已就緒（2/2）：封存鈕的階段守門放行，聚焦 remote 事件不影響本地寫入能力。
        { name: "local-change", status: "in-progress", totalTasks: 2, completedTasks: 2 },
      ])),
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
        workspace={fakeWorkspaceAdapter() as never}
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
    const square = Array.from(document.querySelectorAll("[data-tab]")).find(
      (el) => el.getAttribute("data-tab") === REMOTE_KEY,
    );
    expect(square).toBeTruthy();
    expect(square?.querySelector("[data-tab-dot]")).toBeNull();
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
            planError: null,
            next: null,
            unpromotedCuts: [],
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
      ...fakeConnectionsAdapter(),
      list: vi.fn(async () => {
        order.push("connection-list");
        return [connection];
      }),
      add: vi.fn(),
      remove: vi.fn(),
      loginStart: vi.fn(async () => {
        order.push("login");
        return { status: "loggedIn" as const, display: "Dev" };
      }),
      loginCancel: vi.fn(),
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
        workspace={fakeWorkspaceAdapter() as never}
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
    const reauth = await screen.findByRole("button", { name: "重新登入" });
    // stale 橫幅整條已是琥珀警示；按鈕再塗琥珀只會兩片顏色互搶，改中性 outline。
    expect(reauth.className).not.toContain("status-warning");
    expect(reauth.className).toContain("border");
    fireEvent.click(reauth);
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

  it("session 回報 needs-reauth 後，從齒輪進伺服器頁也看得到該連線「登入過期」", async () => {
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
        activeKey: "remote:c1/demo/backend",
      }),
    );
    const events = fakeEvents();
    const invoke = vi.fn(async (command: string) => {
      switch (command) {
        case "remote_list_changes":
          return {
            changes: [{ name: "backend-change", status: "in-progress", totalTasks: 1, completedTasks: 0 }],
            planError: null,
            next: null,
            unpromotedCuts: [],
          };
        case "remote_list_specs":
          return { specs: [] };
        case "remote_list_archived":
          return { archived: [] };
        case "remote_list_discussions":
          return { active: [], archived: [] };
        default:
          return undefined;
      }
    }) as unknown as InvokeFn;
    const openRemote = vi.fn(async () =>
      createRemoteSession("c1", { ...INFO, repoKey: "backend", repoName: "Backend" }, undefined, {
        invoke,
        listen: events.listen,
      }),
    );
    const connections = fakeConnectionsAdapter({
      list: vi.fn(async () => [{ id: "c1", origin: "http://server.test", name: "Server", loggedIn: true }]),
    });
    render(
      <App
        createSession={vi.fn()}
        openRemote={openRemote}
        workspace={fakeWorkspaceAdapter() as never}
        connections={connections}
      />,
    );
    await screen.findByText("backend-change");
    act(() => {
      events.emit("remote-connection-state", { connectionId: "c1", state: "needs-reauth", message: "登入已失效" });
    });
    await screen.findByTestId("remote-stale-banner");

    fireEvent.click(
      (document.querySelector("[data-project-rail]") as HTMLElement).querySelector('[aria-label="設定"]') as HTMLElement,
    );
    fireEvent.click(screen.getByRole("button", { name: "伺服器" }));
    const row = await screen.findByTestId("server-row-http://server.test");
    expect(row.textContent).toContain("登入過期");
    expect(row.textContent).toContain("重新登入");
  });
});

// remote-resilience「存取被拒的狀態跟著分頁」：Rust 的事件 worker 以分頁（locator key）
// 為單位廣播 remote-access-state；前端只照事件更新，不由查詢失敗自行推斷。
describe("remote access denied", () => {
  const WRITE_CAPS: Array<keyof WorkspaceCapabilities> = [
    "deleteChange",
    "setTaskDone",
    "setAllTasks",
    "moveTask",
    "archive",
    "promoteDiscussion",
    "archiveDiscussion",
    "reorderCard",
    "setDepends",
    "claim",
    "policyWrite",
  ];
  /** 11 個寫入能力全開的握手結果：關掉與復原都看得出來。 */
  const ALL_WRITES: WorkspaceCapabilities = {
    ...REMOTE_CAPS,
    deleteChange: true,
    moveTask: true,
    reorderCard: true,
    setDepends: true,
  };
  const DENIED = { key: REMOTE_KEY, denied: true, message: "帳號已停用" };
  const REGAINED = { key: REMOTE_KEY, denied: false, message: null };

  function sessionOf(repoKey = "backend", capabilities = ALL_WRITES): WorkspaceSession {
    return createRemoteSession("c1", { ...INFO, repoKey, capabilities }, undefined, {
      invoke: vi.fn() as never,
      listen: fakeEvents().listen,
    });
  }

  it("多種狀態並存時只呈現一種：needs-reauth、offline、access-denied、ready", () => {
    // spec 的範例表逐列（connection 狀態、分頁是否存取被拒、呈現）。
    const rows: Array<[RemoteConnectionState, boolean, string]> = [
      ["online", false, "ready"],
      ["online", true, "access-denied"],
      ["offline", true, "offline"],
      ["needs-reauth", true, "needs-reauth"],
      ["offline", false, "offline"],
    ];
    for (const [state, denied, shown] of rows) {
      let session = applyRemoteConnectionState(sessionOf(), { connectionId: "c1", state, message: null });
      if (denied) session = applyRemoteAccessState(session, DENIED);
      expect(remoteWorkspaceStatus(session, undefined), `${state}／${denied}`).toBe(shown);
    }
  });

  it("被拒時關掉該分頁的 11 個寫入能力與流程政策的寫入開關，解除後復原；其他分頁不變", () => {
    const backend = sessionOf("backend");
    const frontend = sessionOf("frontend");

    const denied = applyRemoteAccessState(backend, DENIED);
    expect(denied.accessDenied).toEqual({ message: "帳號已停用" });
    for (const cap of WRITE_CAPS) expect(denied.capabilities[cap], cap).toBe(false);
    expect(denied.settings.policyWrite).toBe(false);
    // 讀取照常：已載入的內容仍可看。
    expect(denied.capabilities.listChanges).toBe(true);
    expect(denied.capabilities.getDocument).toBe(true);
    // 同一連線的另一個分頁：原樣回傳。
    expect(applyRemoteAccessState(frontend, DENIED)).toBe(frontend);

    const regained = applyRemoteAccessState(denied, REGAINED);
    expect(regained.accessDenied).toBeUndefined();
    expect(regained.capabilities).toEqual(ALL_WRITES);
    expect(regained.settings.policyWrite).toBe(true);
  });

  it("「可不可以寫」只有一個答案：離線與被拒任一成立就不能寫", () => {
    const offline = { connectionId: "c1", state: "offline" as const, message: null };
    const online = { connectionId: "c1", state: "online" as const, message: null };

    // 被拒期間連線回到 online：仍不能寫。
    let session = applyRemoteAccessState(applyRemoteConnectionState(sessionOf(), offline), DENIED);
    session = applyRemoteConnectionState(session, online);
    expect(session.capabilities.setTaskDone).toBe(false);
    expect(session.settings.policyWrite).toBe(false);

    // 離線期間解除被拒：仍不能寫，回到 online 才復原。
    session = applyRemoteAccessState(applyRemoteConnectionState(session, offline), REGAINED);
    expect(session.capabilities.setTaskDone).toBe(false);
    session = applyRemoteConnectionState(session, online);
    expect(session.capabilities).toEqual(ALL_WRITES);
    expect(session.settings.policyWrite).toBe(true);
  });

  async function openBoard() {
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
        workspace={fakeWorkspaceAdapter() as never}
      />,
    );
    await screen.findByText("remote-change");
    await waitFor(() => expect(events.count("remote-access-state")).toBe(1));
    return { events, network };
  }

  const railSquare = () =>
    Array.from(document.querySelectorAll<HTMLElement>("[data-tab]")).find(
      (el) => el.getAttribute("data-tab") === REMOTE_KEY,
    ) as HTMLElement;

  it("看板開著時被拒：紅色橫幅顯示 Server 的原因、圖示列紅點、內容保留、寫入停用", async () => {
    const { events } = await openBoard();
    fireEvent.click(screen.getByText("remote-change"));
    const archive = (await screen.findByRole("button", { name: "封存" })) as HTMLButtonElement;
    fireEvent.click(screen.getByRole("tab", { name: /任務/ }));
    await screen.findAllByRole("checkbox");
    expect(archive.disabled).toBe(false);

    act(() => {
      events.emit("remote-access-state", DENIED);
    });

    const banner = await screen.findByTestId("remote-stale-banner");
    expect(banner.getAttribute("data-connection-state")).toBe("access-denied");
    for (const token of SEMANTIC_SURFACE.danger.split(" ")) expect(banner.className).toContain(token);
    expect(banner.className).not.toContain("status-warning");
    expect(banner.textContent).toContain("沒有存取權限");
    expect(banner.textContent).toContain("帳號已停用");
    expect(banner.textContent).toContain("目前顯示最後成功載入的內容");
    expect(banner.textContent).toContain("stale");
    expect(banner.querySelector("button")).toBeNull();
    // 圖示列：錯誤色的點，無障礙名稱說明原因。
    expect(railSquare().querySelector("[data-tab-dot]")?.className).toContain("bg-destructive");
    expect(railSquare().getAttribute("aria-label")).toContain("存取遭拒");
    // 已載入的內容還在；寫入停用並說明原因。
    expect(screen.getAllByText("remote-change").length).toBeGreaterThan(0);
    expect((screen.getByRole("button", { name: "封存" }) as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getAllByRole("checkbox")[0] as HTMLInputElement).disabled).toBe(true);
    expect(document.querySelector('[title*="目前沒有存取權限"]')).toBeTruthy();
  });

  it("恢復存取後橫幅與紅點消失、寫入恢復、看板重新查詢", async () => {
    const { events, network } = await openBoard();
    act(() => {
      events.emit("remote-access-state", DENIED);
    });
    await screen.findByTestId("remote-stale-banner");

    network.addRecoveredChange();
    act(() => {
      events.emit("remote-access-state", REGAINED);
      // Rust 的 worker 在解除時同時發一次重新載入。
      events.emit("remote-workspace-changed", REMOTE_KEY);
    });

    await screen.findByText("recovered-change");
    expect(screen.queryByTestId("remote-stale-banner")).toBeNull();
    expect(railSquare().querySelector("[data-tab-dot]")).toBeNull();
    fireEvent.click(screen.getByText("remote-change"));
    const archive = (await screen.findByRole("button", { name: "封存" })) as HTMLButtonElement;
    expect(archive.disabled).toBe(false);
  });

  it("Server 沒有說明原因時，橫幅用固定的說明文案", async () => {
    const { events } = await openBoard();
    act(() => {
      events.emit("remote-access-state", { key: REMOTE_KEY, denied: true, message: null });
    });
    const banner = await screen.findByTestId("remote-stale-banner");
    expect(banner.textContent).toContain("目前帳號無法存取這個專案");
    expect(banner.textContent).toContain("目前顯示最後成功載入的內容");
  });

  it("別的分頁的事件與形狀不對的 payload 不影響這個分頁", async () => {
    const { events } = await openBoard();
    act(() => {
      events.emit("remote-access-state", { ...DENIED, key: "remote:c1/demo/frontend" });
      events.emit("remote-access-state", { key: REMOTE_KEY, denied: "yes", message: null });
      events.emit("remote-access-state", REMOTE_KEY);
    });
    expect(screen.queryByTestId("remote-stale-banner")).toBeNull();
    expect(railSquare().querySelector("[data-tab-dot]")).toBeNull();
  });

  it("被拒之後連線離線：只呈現離線橫幅；回到 online 而仍被拒時改回存取被拒", async () => {
    const { events } = await openBoard();
    act(() => {
      events.emit("remote-access-state", DENIED);
    });
    await screen.findByTestId("remote-stale-banner");

    act(() => {
      events.emit("remote-connection-state", { connectionId: "c1", state: "offline", message: "系統維護中——顯示最後成功載入的內容" });
    });
    expect(screen.getAllByTestId("remote-stale-banner")).toHaveLength(1);
    const offline = screen.getByTestId("remote-stale-banner");
    expect(offline.getAttribute("data-connection-state")).toBe("offline");
    expect(offline.textContent).toContain("系統維護中");
    expect(offline.textContent).not.toContain("沒有存取權限");
    expect(railSquare().querySelector("[data-tab-dot]")?.className).toContain("bg-status-warning");

    act(() => {
      events.emit("remote-connection-state", { connectionId: "c1", state: "online", message: null });
    });
    await waitFor(() =>
      expect(screen.getByTestId("remote-stale-banner").getAttribute("data-connection-state")).toBe("access-denied"),
    );
  });
});

// remote-resilience「背景遠端分頁的連線定期檢查」的 App 接線：掛載即全域監聽連線狀態，
// 還原分頁列後檢查一次，主視窗取得焦點時再檢查；卸載時解除監聽。
describe("背景遠端分頁的連線檢查接線", () => {
  it("還原後檢查一次、取得焦點再檢查，卸載解除監聽", async () => {
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
        activeKey: null,
      }),
    );
    const unlistenState = vi.fn();
    const connections = fakeConnectionsAdapter({
      list: vi.fn(async () => [{ id: "c1", origin: "http://server.test", name: "Server", loggedIn: true }]),
      onConnectionState: vi.fn(async () => unlistenState),
    });
    let onFocus: ((focused: boolean) => void) | null = null;
    const updater = {
      check: vi.fn().mockResolvedValue(null),
      relaunch: vi.fn(),
      onFocusChanged: vi.fn(async (h: (focused: boolean) => void) => {
        onFocus = h;
        return () => {};
      }),
    };
    const { unmount } = render(
      <App
        createSession={vi.fn()}
        workspace={fakeWorkspaceAdapter() as never}
        connections={connections}
        updater={updater as never}
      />,
    );

    await waitFor(() => expect(connections.checkConnection).toHaveBeenCalledTimes(1));
    expect(connections.checkConnection).toHaveBeenCalledWith("c1");
    expect(connections.onConnectionState).toHaveBeenCalledTimes(1);

    await waitFor(() => expect(onFocus).not.toBeNull());
    act(() => onFocus?.(false));
    expect(connections.checkConnection).toHaveBeenCalledTimes(1);
    act(() => onFocus?.(true));
    await waitFor(() => expect(connections.checkConnection).toHaveBeenCalledTimes(2));

    unmount();
    await waitFor(() => expect(unlistenState).toHaveBeenCalledTimes(1));
  });
});

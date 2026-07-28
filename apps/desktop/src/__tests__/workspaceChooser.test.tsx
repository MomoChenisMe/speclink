import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { I18nProvider } from "@speclink/ui";

import { WorkspaceChooser } from "../components/WorkspaceChooser";
import { RemoteConflictDialog } from "../components/RemoteConflictDialog";
import { APP_MESSAGES } from "../i18n/messages";
import { createAppStore, type AppState } from "../store";
import type { ConnectionsAdapter, ConnectionView } from "../adapter/connections";
import type { WorkspaceAdapter } from "../adapter/workspace";
import type { UseBoundStore, StoreApi } from "zustand";
import type { WorkspaceSession } from "../session";

const CONNECTION: ConnectionView = {
  id: "conn_1",
  origin: "https://spec.example.test",
  name: "團隊 Server",
  lastActorDisplay: "Momo",
  loggedIn: true,
};

const SCOPES = {
  projects: [
    {
      id: "prj_1",
      key: "speclink",
      name: "Speclink",
      repos: [
        { id: "repo_1", key: "desktop", name: "Desktop" },
        { id: "repo_2", key: "server", name: "Server" },
      ],
    },
  ],
};

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);

function fakeConnections(over: Partial<ConnectionsAdapter> = {}): ConnectionsAdapter {
  return {
    list: vi.fn().mockResolvedValue([CONNECTION]),
    add: vi.fn().mockResolvedValue(CONNECTION),
    remove: vi.fn().mockResolvedValue(undefined),
    deviceLoginStart: vi.fn().mockResolvedValue({ status: "loggedIn", display: "Momo" }),
    deviceLoginObserve: vi.fn(),
    patLogin: vi.fn().mockResolvedValue({ status: "loggedIn", display: "Momo" }),
    logout: vi.fn().mockResolvedValue({ revokedOnServer: true, patNotice: false }),
    scopes: vi.fn().mockResolvedValue(SCOPES),
    inspectCheckout: vi
      .fn()
      .mockImplementation(async (path: string) => ({ root: path, tools: ["claude"] })),
    bindCheckout: vi.fn().mockImplementation(async (path: string) => path),
    ...over,
  };
}

/** 走到 checkout 步驟並選好本機資料夾（inspect 完成後顯示工具 checkbox）。 */
async function reachCheckoutFolder() {
  fireEvent.click(screen.getByRole("button", { name: /選擇本機資料夾/ }));
  await waitFor(() =>
    expect(screen.getByRole("checkbox", { name: /claude/i })).toBeTruthy(),
  );
}

function fakeWorkspace(over: Partial<WorkspaceAdapter> = {}): WorkspaceAdapter {
  return {
    openProject: vi.fn(),
    initProject: vi.fn(),
    startupDir: vi.fn(),
    projectStats: vi.fn(),
    watchWorkspace: vi.fn(),
    pickFolder: vi.fn().mockResolvedValue(null),
    ...over,
  };
}

function renderChooser({
  adapter = fakeConnections(),
  workspace = fakeWorkspace(),
  connections = [CONNECTION],
  onOpenRemote = vi.fn().mockResolvedValue(undefined),
  onOpenLocal = vi.fn().mockResolvedValue(undefined),
  onRequestMigration = vi.fn().mockResolvedValue(undefined),
  onAddServer = vi.fn().mockResolvedValue(undefined),
  initialConnectionId,
  initialScope,
  initialCheckoutPath,
}: {
  adapter?: ConnectionsAdapter;
  workspace?: WorkspaceAdapter;
  connections?: ConnectionView[];
  onOpenRemote?: ReturnType<typeof vi.fn>;
  onOpenLocal?: ReturnType<typeof vi.fn>;
  onRequestMigration?: ReturnType<typeof vi.fn>;
  onAddServer?: ReturnType<typeof vi.fn>;
  initialConnectionId?: string;
  initialScope?: { projectKey: string; repoKey: string };
  initialCheckoutPath?: string;
} = {}) {
  const onOpenChange = vi.fn();
  render(
    <WorkspaceChooser
      open
      onOpenChange={onOpenChange}
      connections={connections}
      connectionAdapter={adapter}
      workspace={workspace}
      onOpenLocal={onOpenLocal}
      onRequestMigration={onRequestMigration}
      onAddServer={onAddServer}
      phases={{}}
      onCancelLogin={vi.fn()}
      onSubmitPat={vi.fn()}
      onRefreshConnections={vi.fn().mockResolvedValue(undefined)}
      onOpenRemote={onOpenRemote}
      initialConnectionId={initialConnectionId}
      initialScope={initialScope}
      initialCheckoutPath={initialCheckoutPath}
    />,
    { wrapper: zhWrapper },
  );
  return {
    adapter,
    workspace,
    onOpenRemote,
    onOpenLocal,
    onRequestMigration,
    onAddServer,
    onOpenChange,
  };
}

async function chooseDesktopRepo() {
  fireEvent.click(screen.getByRole("button", { name: /Speclink Server/ }));
  fireEvent.click(screen.getByRole("button", { name: /團隊 Server/ }));
  await waitFor(() => expect(screen.getByRole("radio", { name: /Desktop/ })).toBeTruthy());
  fireEvent.click(screen.getByRole("radio", { name: /Desktop/ }));
  fireEvent.click(screen.getByRole("button", { name: "下一步" }));
}

describe("WorkspaceChooser", () => {
  it("來源→已登入 server→scopes 分組單選→略過 checkout 開啟 spec-only 分頁", async () => {
    const open = vi.fn().mockResolvedValue(undefined);
    const { adapter, onOpenChange } = renderChooser({ onOpenRemote: open });

    await chooseDesktopRepo();
    fireEvent.click(screen.getByRole("button", { name: /略過（規格模式）/ }));
    fireEvent.click(screen.getByRole("button", { name: "開啟 Workspace" }));

    await waitFor(() =>
      expect(open).toHaveBeenCalledWith("conn_1", "speclink/desktop", undefined),
    );
    expect(adapter.scopes).toHaveBeenCalledWith("conn_1");
    expect(screen.queryByPlaceholderText("project 或 project/repo")).toBeNull();
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  // spec/design「Desktop IPC and UI contract」：先 inspect（零寫入的先檢查）
  it("選擇 checkout 會先走 inspect marker 驗證，拒絕訊息指出 marker 指向且不顯示工具選集", async () => {
    const inspect = vi.fn().mockRejectedValue(
      new Error(
        "此資料夾的 remote marker 指向 https://other.example.test / api，與所選不一致",
      ),
    );
    const bind = vi.fn();
    const adapter = fakeConnections({ inspectCheckout: inspect, bindCheckout: bind });
    const workspace = fakeWorkspace({ pickFolder: vi.fn().mockResolvedValue("/work/desktop") });
    const open = vi.fn().mockResolvedValue(undefined);
    renderChooser({ adapter, workspace, onOpenRemote: open });

    await chooseDesktopRepo();
    fireEvent.click(screen.getByRole("button", { name: /選擇本機資料夾/ }));

    await waitFor(() =>
      expect(screen.getByRole("alert").textContent).toContain(
        "https://other.example.test / api",
      ),
    );
    expect(inspect).toHaveBeenCalledWith(
      "/work/desktop",
      "https://spec.example.test",
      "speclink",
      "desktop",
    );
    expect(bind).not.toHaveBeenCalled();
    expect(screen.queryByRole("checkbox", { name: /claude/i })).toBeNull();
    expect(open).not.toHaveBeenCalled();
  });

  // spec/design「Desktop IPC and UI contract」：inspect 後顯示既有選集，
  // submit 依序 bind（帶選集）後才 openRemote（帶 checkoutRoot）。
  it("folder mode：inspect 預選既有工具，開啟時先 bind 後 openRemote", async () => {
    const inspect = vi
      .fn()
      .mockResolvedValue({ root: "/work/desktop", tools: ["codex"] });
    const bind = vi.fn().mockResolvedValue("/work/desktop");
    const adapter = fakeConnections({ inspectCheckout: inspect, bindCheckout: bind });
    const workspace = fakeWorkspace({ pickFolder: vi.fn().mockResolvedValue("/work/desktop") });
    const open = vi.fn().mockResolvedValue(undefined);
    const { onOpenChange } = renderChooser({ adapter, workspace, onOpenRemote: open });

    await chooseDesktopRepo();
    await reachCheckoutFolder();

    // inspect 回傳 codex → codex 勾選、claude 未勾。
    expect(
      (screen.getByRole("checkbox", { name: /codex/i }) as HTMLInputElement).getAttribute(
        "aria-checked",
      ),
    ).toBe("true");
    expect(
      (screen.getByRole("checkbox", { name: /claude/i }) as HTMLInputElement).getAttribute(
        "aria-checked",
      ),
    ).toBe("false");

    fireEvent.click(screen.getByRole("button", { name: "開啟 Workspace" }));

    await waitFor(() =>
      expect(bind).toHaveBeenCalledWith(
        "/work/desktop",
        "https://spec.example.test",
        "speclink",
        "desktop",
        ["codex"],
      ),
    );
    await waitFor(() =>
      expect(open).toHaveBeenCalledWith("conn_1", "speclink/desktop", "/work/desktop"),
    );
    // 同步成功才開啟：bind 先於 openRemote。
    expect(bind.mock.invocationCallOrder[0]).toBeLessThan(open.mock.invocationCallOrder[0]);
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  it("folder mode：空工具選集時「開啟 Workspace」停用且不 bind", async () => {
    const inspect = vi.fn().mockResolvedValue({ root: "/work/desktop", tools: [] });
    const bind = vi.fn();
    const adapter = fakeConnections({ inspectCheckout: inspect, bindCheckout: bind });
    const workspace = fakeWorkspace({ pickFolder: vi.fn().mockResolvedValue("/work/desktop") });
    renderChooser({ adapter, workspace });

    await chooseDesktopRepo();
    await reachCheckoutFolder();

    expect(
      (screen.getByRole("button", { name: "開啟 Workspace" }) as HTMLButtonElement).disabled,
    ).toBe(true);
    // 勾一個工具後啟用。
    fireEvent.click(screen.getByRole("checkbox", { name: /claude/i }));
    await waitFor(() =>
      expect(
        (screen.getByRole("button", { name: "開啟 Workspace" }) as HTMLButtonElement).disabled,
      ).toBe(false),
    );
    expect(bind).not.toHaveBeenCalled();
  });

  it("folder mode：bind 失敗保留 path 與工具選集且不 openRemote", async () => {
    const inspect = vi
      .fn()
      .mockResolvedValue({ root: "/work/desktop", tools: ["claude", "codex"] });
    const bind = vi.fn().mockRejectedValue(new Error("同步技能時發生檔案系統錯誤"));
    const adapter = fakeConnections({ inspectCheckout: inspect, bindCheckout: bind });
    const workspace = fakeWorkspace({ pickFolder: vi.fn().mockResolvedValue("/work/desktop") });
    const open = vi.fn().mockResolvedValue(undefined);
    const { onOpenChange } = renderChooser({ adapter, workspace, onOpenRemote: open });

    await chooseDesktopRepo();
    await reachCheckoutFolder();
    fireEvent.click(screen.getByRole("button", { name: "開啟 Workspace" }));

    await waitFor(() =>
      expect(screen.getByRole("alert").textContent).toContain("同步技能時發生檔案系統錯誤"),
    );
    expect(open).not.toHaveBeenCalled();
    expect(onOpenChange).not.toHaveBeenCalledWith(false);
    // 選集與 path 留存：checkbox 仍在、仍勾選，可重試。
    expect(
      (screen.getByRole("checkbox", { name: /claude/i }) as HTMLInputElement).getAttribute(
        "aria-checked",
      ),
    ).toBe("true");
  });

  // spec「缺少工具選集時導向 checkout 選擇」：intent 直達 checkout 步驟並預填 scope／path
  it("缺選集 intent：直達 checkout 步驟、依 scope inspect 預填 path 並要求明示選擇", async () => {
    const inspect = vi.fn().mockResolvedValue({ root: "/work/desktop", tools: [] });
    const adapter = fakeConnections({ inspectCheckout: inspect });
    const open = vi.fn().mockResolvedValue(undefined);
    renderChooser({
      adapter,
      onOpenRemote: open,
      initialConnectionId: "conn_1",
      initialScope: { projectKey: "speclink", repoKey: "desktop" },
      initialCheckoutPath: "/work/desktop",
    });

    // 直達 checkout 步驟並以預填 path inspect（不必再走 source→server→scopes）。
    await waitFor(() =>
      expect(screen.getByRole("checkbox", { name: /claude/i })).toBeTruthy(),
    );
    expect(inspect).toHaveBeenCalledWith(
      "/work/desktop",
      "https://spec.example.test",
      "speclink",
      "desktop",
    );
    // 缺選集 → Open 停用，需使用者明示勾選。
    expect(
      (screen.getByRole("button", { name: "開啟 Workspace" }) as HTMLButtonElement).disabled,
    ).toBe(true);
    expect(open).not.toHaveBeenCalled();
  });

  it("folder mode：bind 進行中不得重複送出", async () => {
    let release: (value: string) => void = () => {};
    const inspect = vi
      .fn()
      .mockResolvedValue({ root: "/work/desktop", tools: ["claude"] });
    const bind = vi.fn().mockImplementation(
      () =>
        new Promise<string>((resolve) => {
          release = resolve;
        }),
    );
    const adapter = fakeConnections({ inspectCheckout: inspect, bindCheckout: bind });
    const workspace = fakeWorkspace({ pickFolder: vi.fn().mockResolvedValue("/work/desktop") });
    renderChooser({ adapter, workspace });

    await chooseDesktopRepo();
    await reachCheckoutFolder();
    const openButton = screen.getByRole("button", { name: "開啟 Workspace" });
    fireEvent.click(openButton);
    await waitFor(() => expect((openButton as HTMLButtonElement).disabled).toBe(true));
    fireEvent.click(openButton);
    fireEvent.click(openButton);

    expect(bind).toHaveBeenCalledTimes(1);
    release("/work/desktop");
  });

  it("scopes 沒有 membership 時顯示繁中空清單說明", async () => {
    const adapter = fakeConnections({ scopes: vi.fn().mockResolvedValue({ projects: [] }) });
    renderChooser({ adapter });

    fireEvent.click(screen.getByRole("button", { name: /Speclink Server/ }));
    fireEvent.click(screen.getByRole("button", { name: /團隊 Server/ }));

    expect((await screen.findByTestId("scopes-empty")).textContent).toContain(
      "此帳號目前沒有任何 Project／Repo membership",
    );
    expect((screen.getByRole("button", { name: "下一步" }) as HTMLButtonElement).disabled).toBe(
      true,
    );
  });

  it("server 清單可就地新增並登入，完成後回到清單步驟", async () => {
    const add = vi.fn().mockResolvedValue(undefined);
    renderChooser({ connections: [], onAddServer: add });

    fireEvent.click(screen.getByRole("button", { name: /Speclink Server/ }));
    expect(screen.getByText(/目前沒有已登入的 server/)).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "新增 server" }));
    fireEvent.change(screen.getByLabelText("伺服器位址（http://…）"), {
      target: { value: "https://new.example.test" },
    });
    fireEvent.change(screen.getByLabelText("顯示名"), { target: { value: "新 Server" } });
    fireEvent.click(screen.getByRole("button", { name: "新增並登入" }));

    await waitFor(() =>
      expect(add).toHaveBeenCalledWith("https://new.example.test", "新 Server"),
    );
    // add 被呼叫≠完成：status 訊息在 promise resolve 後才渲染，須非同步等。
    expect((await screen.findByRole("status")).textContent).toContain("連線已新增");
  });

  it("本機資料夾分流沿用既有開啟流程", async () => {
    const local = vi.fn().mockResolvedValue(undefined);
    const workspace = fakeWorkspace({
      pickFolder: vi.fn().mockResolvedValue("/work/local"),
      openProject: vi.fn().mockResolvedValue({
        status: "project",
        root: "/work/local",
        name: "Local",
      }),
    });
    const { onOpenChange } = renderChooser({ workspace, onOpenLocal: local });

    fireEvent.click(screen.getByRole("button", { name: /本機資料夾/ }));
    fireEvent.click(await screen.findByRole("button", { name: "開啟本機" }));

    await waitFor(() => expect(local).toHaveBeenCalledWith("/work/local"));
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  it("選到含 openspec 的本機專案後提供遷移到 server 次要動作", async () => {
    const migrate = vi.fn().mockResolvedValue(undefined);
    const workspace = fakeWorkspace({
      pickFolder: vi.fn().mockResolvedValue("/work/local"),
      openProject: vi.fn().mockResolvedValue({
        status: "project",
        root: "/work/local",
        name: "Local",
      }),
    });
    const { onOpenChange } = renderChooser({
      workspace,
      onRequestMigration: migrate,
    });

    fireEvent.click(screen.getByRole("button", { name: /本機資料夾/ }));
    fireEvent.click(await screen.findByRole("button", { name: "遷移到 Server…" }));

    await waitFor(() => expect(migrate).toHaveBeenCalledWith("/work/local"));
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });
});

describe("remote marker 與本機 openspec 並存", () => {
  it("強制呈現繼續本機、以 server 為準、遷移本機內容三個出口", async () => {
    const continueLocal = vi.fn().mockResolvedValue(undefined);
    const useServer = vi.fn().mockResolvedValue(undefined);
    const migrateLocal = vi.fn();
    render(
      <RemoteConflictDialog
        conflict={{
          path: "/work/coexists",
          url: "https://spec.example.test/api/speclink/v1/projects/demo",
          repo: "backend",
        }}
        onContinueLocal={continueLocal}
        onUseServer={useServer}
        onMigrateLocal={migrateLocal}
      />,
      { wrapper: zhWrapper },
    );

    expect(screen.getByText(/備份後棄用本機內容.*不會上傳或合併/)).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: /^繼續本機/ }));
    fireEvent.click(screen.getByRole("button", { name: /^以 Server 為準/ }));
    await waitFor(() => expect(useServer).toHaveBeenCalledTimes(1));
    await waitFor(() =>
      expect(
        (screen.getByRole("button", { name: /^遷移本機內容/ }) as HTMLButtonElement).disabled,
      ).toBe(false),
    );
    fireEvent.click(screen.getByRole("button", { name: /^遷移本機內容/ }));

    await waitFor(() => expect(continueLocal).toHaveBeenCalledTimes(1));
    expect(migrateLocal).toHaveBeenCalledTimes(1);
    expect(screen.queryByRole("button", { name: "取消" })).toBeNull();
  });
});

// --- chooser 內就地登入回饋（規格「device login 預設與 PAT fallback」的
// 「從工作區選擇器發起登入同樣呈現等待授權面」；design 決策三）：真 store 驅動
// ——新增並登入後，等待授權面／PAT 輸入就在選擇器的 server 步驟渲染。 ---

describe("WorkspaceChooser 登入回饋", () => {
  const AUTH = {
    deviceCode: "dev-code-1",
    userCode: "ABCD-EFGH",
    verificationUri: "http://localhost:8080/activate",
    expiresIn: 900,
    interval: 1,
  };

  /** 假 adapter：in-memory registry；啟動段行為由測試覆寫。 */
  function storeAdapter(over: Partial<ConnectionsAdapter> = {}): ConnectionsAdapter {
    const entries: Array<Omit<ConnectionView, "loggedIn">> = [];
    const loggedIn = new Set<string>();
    return {
      list: async () => entries.map((e) => ({ ...e, loggedIn: loggedIn.has(e.origin) })),
      add: async (baseUrl, name) => {
        const origin = baseUrl.trim().replace(/\/+$/, "").toLowerCase();
        const entry = { id: `conn_${entries.length + 1}`, origin, name };
        entries.push(entry);
        return { ...entry, loggedIn: false };
      },
      remove: async () => {},
      deviceLoginStart: async () => ({ status: "awaitingApproval", authorization: AUTH }),
      deviceLoginObserve: async () => ({ status: "pending", slowDown: false }),
      patLogin: async (origin) => {
        loggedIn.add(origin);
        const entry = entries.find((e) => e.origin === origin);
        if (entry) entry.lastActorDisplay = "Momo";
        return { status: "loggedIn", display: "Momo" };
      },
      logout: async () => ({ revokedOnServer: true, patNotice: false }),
      scopes: async () => SCOPES,
      inspectCheckout: async (path) => ({ root: path, tools: [] }),
      bindCheckout: async (path) => path,
      ...over,
    };
  }

  function ChooserHarness({ useStore }: { useStore: UseBoundStore<StoreApi<AppState>> }) {
    const s = useStore();
    return (
      <WorkspaceChooser
        open
        onOpenChange={() => {}}
        connections={s.connections}
        phases={s.connectionPhases}
        connectionAdapter={fakeConnections()}
        workspace={fakeWorkspace()}
        onOpenLocal={async () => {}}
        onAddServer={s.addConnection}
        onCancelLogin={s.cancelLogin}
        onSubmitPat={s.submitPat}
        onRefreshConnections={s.refreshConnections}
        onOpenRemote={async () => {}}
      />
    );
  }

  function renderWithStore(adapter: ConnectionsAdapter) {
    const useStore = createAppStore({
      createSession: () => ({}) as WorkspaceSession,
      connections: adapter,
    });
    render(<ChooserHarness useStore={useStore} />, { wrapper: zhWrapper });
    return useStore;
  }

  /** 走到 server 步驟並送出新增並登入。 */
  function addAndLogin() {
    fireEvent.click(screen.getByRole("button", { name: /Speclink Server/ }));
    fireEvent.click(screen.getByRole("button", { name: "新增 server" }));
    fireEvent.change(screen.getByLabelText("伺服器位址（http://…）"), {
      target: { value: "http://localhost:8080" },
    });
    fireEvent.click(screen.getByRole("button", { name: "新增並登入" }));
  }

  it("新增並登入後選擇器內就地顯示等待授權面", async () => {
    renderWithStore(storeAdapter());
    addAndLogin();

    const waiting = await screen.findByTestId("awaiting-approval-http://localhost:8080");
    expect(waiting.textContent).toContain(AUTH.userCode);
    expect(waiting.textContent).toContain(AUTH.verificationUri);
    expect(waiting.textContent).toMatch(/\d+:\d{2}/);
    expect(screen.getByRole("button", { name: "取消登入" })).toBeTruthy();
  });

  it("取消即停止觀測、面消失且停留在 server 步驟", async () => {
    const observe = vi.fn(async () => ({ status: "pending" as const, slowDown: false }));
    renderWithStore(storeAdapter({ deviceLoginObserve: observe }));
    addAndLogin();
    await screen.findByTestId("awaiting-approval-http://localhost:8080");

    fireEvent.click(screen.getByRole("button", { name: "取消登入" }));
    await waitFor(() =>
      expect(screen.queryByTestId("awaiting-approval-http://localhost:8080")).toBeNull(),
    );
    // 仍在 server 步驟（新增 server 的開關還在）。
    expect(screen.getByRole("button", { name: "新增 server" })).toBeTruthy();
    observe.mockClear();
    await new Promise((resolve) => setTimeout(resolve, 1200));
    expect(observe).not.toHaveBeenCalled();
  });

  it("明確不支援時選擇器內就地現 PAT 輸入並可完成登入", async () => {
    renderWithStore(
      storeAdapter({ deviceLoginStart: async () => ({ status: "unsupported" }) }),
    );
    addAndLogin();

    const patInput = await screen.findByPlaceholderText("spk_pat_…");
    fireEvent.change(patInput, { target: { value: "spk_pat_good" } });
    fireEvent.click(screen.getByRole("button", { name: "以 PAT 登入" }));
    // 登入完成：連線出現在已登入清單（ChoiceCard 標題）。
    await waitFor(() => expect(screen.getByText(/Momo/)).toBeTruthy());
  });
});

import { describe, expect, it, vi } from "vitest";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
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
import type { RecentEntry } from "../recents";

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

async function renderChooser({
  adapter = fakeConnections(),
  workspace = fakeWorkspace(),
  connections = [CONNECTION],
  onOpenRemote = vi.fn().mockResolvedValue(undefined),
  onOpenLocal = vi.fn().mockResolvedValue(undefined),
  onRequestMigration = vi.fn().mockResolvedValue(undefined),
  onAddServer = vi.fn().mockResolvedValue(undefined),
  recents = [],
  onRemoveRecent = vi.fn(),
  onRefreshConnections = vi.fn().mockResolvedValue(true),
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
  recents?: RecentEntry[];
  onRemoveRecent?: ReturnType<typeof vi.fn>;
  onRefreshConnections?: ReturnType<typeof vi.fn>;
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
      onRefreshConnections={onRefreshConnections}
      onOpenRemote={onOpenRemote}
      recents={recents}
      onRemoveRecent={onRemoveRecent}
      initialConnectionId={initialConnectionId}
      initialScope={initialScope}
      initialCheckoutPath={initialCheckoutPath}
    />,
    { wrapper: zhWrapper },
  );
  // 掛載時的連線重整是唯一的非同步 setState——先沖乾淨，否則每個案例都噴
  // "not wrapped in act(...)"，且錯誤態的判定會在斷言之後才落定。
  await act(async () => {});
  return {
    adapter,
    workspace,
    onOpenRemote,
    onOpenLocal,
    onRequestMigration,
    onAddServer,
    onRemoveRecent,
    onOpenChange,
  };
}

/** 最近開啟列的開啟鈕（移除鈕帶 aria-label，開啟鈕沒有）。 */
function recentOpenButton(name: RegExp): HTMLButtonElement {
  const found = screen
    .getAllByRole("button", { name })
    .filter((el) => !el.getAttribute("aria-label"));
  expect(found).toHaveLength(1);
  return found[0] as HTMLButtonElement;
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
    const { adapter, onOpenChange } = await renderChooser({ onOpenRemote: open });

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
    await renderChooser({ adapter, workspace, onOpenRemote: open });

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
    const { onOpenChange } = await renderChooser({ adapter, workspace, onOpenRemote: open });

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
    await renderChooser({ adapter, workspace });

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
    const { onOpenChange } = await renderChooser({ adapter, workspace, onOpenRemote: open });

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
    await renderChooser({
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
    await renderChooser({ adapter, workspace });

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
    await renderChooser({ adapter });

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
    await renderChooser({ connections: [], onAddServer: add });

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
    const { onOpenChange } = await renderChooser({ workspace, onOpenLocal: local });

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
    const { onOpenChange } = await renderChooser({
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
    // 三個出口是平等的選擇，不是狀態：語彙統一為中性卡，顏色不暗示「哪個比較對」。
    for (const name of [/^繼續本機/, /^以 Server 為準/, /^遷移本機內容/]) {
      const cls = screen.getByRole("button", { name }).className;
      expect(cls).toContain("border-border");
      expect(cls).not.toContain("amber");
      expect(cls).not.toContain("bg-primary");
    }
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
        recents={[]}
        onRemoveRecent={vi.fn()}
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

describe("最近開啟清單（spec 需求「最近開啟清單」；design D3 顯示期過濾、D4 點擊開啟與失效錯誤態）", () => {
  const LOCAL_RECENT: RecentEntry = {
    locator: { kind: "local", root: "/work/speclink" },
    name: "speclink",
  };
  const REMOTE_RECENT: RecentEntry = {
    locator: {
      kind: "remote",
      connectionId: "conn_1",
      projectId: "prj_1",
      repoId: "repo_1",
      checkoutRoot: "/work/desktop",
    },
    name: "Speclink/Desktop",
  };

  it("recents 為空時第一步沒有「最近開啟」區段", () => {
    renderChooser();
    expect(screen.queryByText("最近開啟")).toBeNull();
    expect(screen.getByRole("button", { name: /本機資料夾/ })).toBeTruthy();
  });

  it("本機列顯示名稱與路徑、remote 列顯示連線名稱與顯示名", async () => {
    await renderChooser({ recents: [LOCAL_RECENT, REMOTE_RECENT] });
    expect(screen.getByText("最近開啟")).toBeTruthy();
    const local = screen.getByRole("button", { name: /\/work\/speclink/ });
    expect(local.textContent).toContain("speclink");
    const remote = screen.getByRole("button", { name: /團隊 Server/ });
    expect(remote.textContent).toContain("Speclink/Desktop");
    // 既有兩張來源卡仍在，且可及性名稱不變。
    expect(screen.getByRole("button", { name: /本機資料夾/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: /Speclink Server/ })).toBeTruthy();
  });

  it("點本機列先探測再沿既有本機開啟流程並關閉 chooser（spec Scenario 點本機條目直接開啟）", async () => {
    const workspace = fakeWorkspace({
      openProject: vi
        .fn()
        .mockResolvedValue({ status: "project", root: "/work/speclink", name: "speclink" }),
    });
    const { onOpenLocal, onOpenChange } = await renderChooser({
      workspace,
      recents: [LOCAL_RECENT],
    });
    fireEvent.click(screen.getByRole("button", { name: /\/work\/speclink/ }));
    await waitFor(() => expect(onOpenLocal).toHaveBeenCalledWith("/work/speclink"));
    expect(workspace.openProject).toHaveBeenCalledWith("/work/speclink");
    expect(onOpenChange).toHaveBeenCalledWith(false);
    // 不走首次選資料夾的「開啟本機／遷移」子畫面。
    expect(screen.queryByRole("button", { name: "開啟本機" })).toBeNull();
  });

  it("探測拋錯時該列轉錯誤態並顯示原因，不開啟（spec Scenario 本機資料夾已消失時轉錯誤態）", async () => {
    const workspace = fakeWorkspace({
      openProject: vi.fn().mockRejectedValue(new Error("cannot open '/work/speclink'")),
    });
    const { onOpenLocal, onOpenChange } = await renderChooser({
      workspace,
      recents: [LOCAL_RECENT],
    });
    fireEvent.click(screen.getByRole("button", { name: /\/work\/speclink/ }));
    await waitFor(() =>
      expect(screen.getByText(/cannot open '\/work\/speclink'/)).toBeTruthy(),
    );
    expect(onOpenLocal).not.toHaveBeenCalled();
    expect(onOpenChange).not.toHaveBeenCalledWith(false);
    expect(
      (screen.getByRole("button", { name: /\/work\/speclink/ }) as HTMLButtonElement).disabled,
    ).toBe(true);
    // 錯誤態的列仍可移除。
    expect(screen.getByRole("button", { name: "自最近開啟移除 speclink" })).toBeTruthy();
  });

  it("點 remote 列以原 connection、projectId/repoId 與 checkoutRoot 開啟（spec Scenario 點 remote 條目以原綁定開啟）", async () => {
    const { onOpenRemote } = await renderChooser({ recents: [REMOTE_RECENT] });
    fireEvent.click(screen.getByRole("button", { name: /團隊 Server/ }));
    await waitFor(() =>
      expect(onOpenRemote).toHaveBeenCalledWith("conn_1", "prj_1/repo_1", "/work/desktop"),
    );
  });

  it("remote handshake 拋錯時該列顯示原因", async () => {
    const onOpenRemote = vi.fn().mockRejectedValue(new Error("access denied — no access"));
    await renderChooser({ recents: [REMOTE_RECENT], onOpenRemote });
    fireEvent.click(screen.getByRole("button", { name: /團隊 Server/ }));
    await waitFor(() => expect(screen.getByText(/access denied/)).toBeTruthy());
  });

  it("連線已移除的 remote 列直接呈現錯誤態、開啟停用、可移除（spec Scenario remote 連線已移除時直接呈現錯誤態）", async () => {
    const gone: RecentEntry = {
      ...REMOTE_RECENT,
      locator: { ...REMOTE_RECENT.locator, connectionId: "conn_gone" } as RecentEntry["locator"],
    };
    const { onRemoveRecent, onOpenRemote } = await renderChooser({ recents: [gone] });
    await waitFor(() => expect(screen.getByText("連線已移除")).toBeTruthy());
    const row = screen.getByRole("button", { name: /連線已移除/ }) as HTMLButtonElement;
    expect(row.disabled).toBe(true);
    fireEvent.click(row);
    expect(onOpenRemote).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "自最近開啟移除 Speclink/Desktop" }));
    expect(onRemoveRecent).toHaveBeenCalledWith("remote:conn_gone/prj_1/repo_1");
  });

  it("移除鈕以 locator key 呼叫 onRemoveRecent（spec Scenario 移除條目後重啟不再出現）", async () => {
    const { onRemoveRecent, onOpenLocal } = await renderChooser({ recents: [LOCAL_RECENT] });
    fireEvent.click(screen.getByRole("button", { name: "自最近開啟移除 speclink" }));
    expect(onRemoveRecent).toHaveBeenCalledWith("local:/work/speclink");
    expect(onOpenLocal).not.toHaveBeenCalled();
  });
});

describe("最近開啟清單的補強（review／verify 第 1 輪 WARNING）", () => {
  const LOCAL: RecentEntry = {
    locator: { kind: "local", root: "/work/speclink" },
    name: "speclink",
  };
  const REMOTE: RecentEntry = {
    locator: {
      kind: "remote",
      connectionId: "conn_1",
      projectId: "prj_1",
      repoId: "repo_1",
      checkoutRoot: "/work/desktop",
    },
    name: "Speclink/Desktop",
  };

  it("點未初始化資料夾的條目仍轉交既有開啟流程（spec Scenario 點未初始化資料夾的條目仍走 init 確認）", async () => {
    const workspace = fakeWorkspace({
      openProject: vi.fn().mockResolvedValue({ status: "uninitialized", dir: "/work/speclink" }),
    });
    const { onOpenLocal, onOpenChange } = await renderChooser({ workspace, recents: [LOCAL] });
    fireEvent.click(screen.getByRole("button", { name: /\/work\/speclink/ }));
    // 探測成功即交給既有 openProjectAt——init 確認框由它負責，chooser 不自行寫入。
    await waitFor(() => expect(onOpenLocal).toHaveBeenCalledWith("/work/speclink"));
    expect(onOpenChange).toHaveBeenCalledWith(false);
    expect(screen.queryByRole("button", { name: "開啟本機" })).toBeNull();
  });

  it("連線已登出時該列以錯誤態呈現且停用開啟", async () => {
    const loggedOut: ConnectionView = { ...CONNECTION, loggedIn: false };
    await renderChooser({ recents: [REMOTE], connections: [loggedOut] });
    await waitFor(() => expect(screen.getByText("連線已登出")).toBeTruthy());
    expect(
      (screen.getByRole("button", { name: /連線已登出/ }) as HTMLButtonElement).disabled,
    ).toBe(true);
  });

  it("點 remote 列先驗證 checkout 綁定，失敗即轉錯誤態不開啟", async () => {
    const inspect = vi.fn().mockRejectedValue(new Error("找不到 checkout 資料夾 /work/desktop"));
    const adapter = fakeConnections({ inspectCheckout: inspect });
    const { onOpenRemote } = await renderChooser({ adapter, recents: [REMOTE] });
    fireEvent.click(screen.getByRole("button", { name: /團隊 Server/ }));
    await waitFor(() =>
      expect(screen.getByText(/找不到 checkout 資料夾/)).toBeTruthy(),
    );
    expect(inspect).toHaveBeenCalledWith(
      "/work/desktop",
      "https://spec.example.test",
      "prj_1",
      "repo_1",
    );
    expect(onOpenRemote).not.toHaveBeenCalled();
  });

  it("無 checkout 綁定的 remote 列不做 inspect，直接開啟", async () => {
    const specOnly: RecentEntry = {
      ...REMOTE,
      locator: { ...REMOTE.locator, checkoutRoot: undefined } as RecentEntry["locator"],
    };
    const adapter = fakeConnections();
    const { onOpenRemote } = await renderChooser({ adapter, recents: [specOnly] });
    fireEvent.click(screen.getByRole("button", { name: /團隊 Server/ }));
    await waitFor(() =>
      expect(onOpenRemote).toHaveBeenCalledWith("conn_1", "prj_1/repo_1", undefined),
    );
    expect(adapter.inspectCheckout).not.toHaveBeenCalled();
  });
});

describe("連線清單載入結果決定錯誤態（review 第 2 輪 WARNING）", () => {
  const REMOTE_ROW: RecentEntry = {
    locator: {
      kind: "remote",
      connectionId: "conn_1",
      projectId: "prj_1",
      repoId: "repo_1",
      checkoutRoot: "/work/desktop",
    },
    name: "Speclink/Desktop",
  };

  it("連線清單讀取失敗時不把 remote 列判成已移除", async () => {
    const onRefreshConnections = vi.fn().mockResolvedValue(false);
    await renderChooser({ recents: [REMOTE_ROW], connections: [], onRefreshConnections });
    await waitFor(() => expect(onRefreshConnections).toHaveBeenCalled());
    expect(screen.queryByText("連線已移除")).toBeNull();
    expect(recentOpenButton(/Speclink\/Desktop/).disabled).toBe(false);
  });

  it("讀取成功且清單真的空時才判成已移除", async () => {
    const onRefreshConnections = vi.fn().mockResolvedValue(true);
    await renderChooser({ recents: [REMOTE_ROW], connections: [], onRefreshConnections });
    await waitFor(() => expect(screen.getByText("連線已移除")).toBeTruthy());
    expect(
      (screen.getByRole("button", { name: /連線已移除/ }) as HTMLButtonElement).disabled,
    ).toBe(true);
  });

  it("重整仍在進行中時 remote 列不判為已移除（尚未 settle 的視窗）", async () => {
    let release: (value: boolean) => void = () => {};
    const pending = new Promise<boolean>((resolve) => {
      release = resolve;
    });
    await renderChooser({
      recents: [REMOTE_ROW],
      connections: [],
      onRefreshConnections: vi.fn().mockReturnValue(pending),
    });
    // renderChooser 的 act 沖洗不會讓一個尚未 resolve 的 promise 落定。
    expect(screen.queryByText("連線已移除")).toBeNull();
    expect(recentOpenButton(/Speclink\/Desktop/).disabled).toBe(false);
    await act(async () => {
      release(true);
    });
    // 讀取成功且清單真的空——此時才判定。
    await waitFor(() => expect(screen.getByText("連線已移除")).toBeTruthy());
  });

  it("清單未就緒時規格模式的 remote 列照樣開得起來", async () => {
    const specOnly: RecentEntry = {
      ...REMOTE_ROW,
      locator: { ...REMOTE_ROW.locator, checkoutRoot: undefined } as RecentEntry["locator"],
    };
    const onOpenRemote = vi.fn().mockResolvedValue(undefined);
    await renderChooser({
      recents: [specOnly],
      connections: [],
      onRefreshConnections: vi.fn().mockResolvedValue(false),
      onOpenRemote,
    });
    fireEvent.click(recentOpenButton(/Speclink\/Desktop/));
    await waitFor(() =>
      expect(onOpenRemote).toHaveBeenCalledWith("conn_1", "prj_1/repo_1", undefined),
    );
    expect(screen.queryByText("連線已移除")).toBeNull();
  });

  it("清單未就緒時點 checkout 綁定的 remote 列不留下帶 Error 前綴的訊息", async () => {
    const onRefreshConnections = vi.fn().mockResolvedValue(false);
    const onOpenRemote = vi.fn().mockResolvedValue(undefined);
    const adapter = fakeConnections();
    await renderChooser({
      adapter,
      recents: [REMOTE_ROW],
      connections: [],
      onRefreshConnections,
      onOpenRemote,
    });
    fireEvent.click(recentOpenButton(/Speclink\/Desktop/));
    await waitFor(() => expect(screen.getByText("連線已移除")).toBeTruthy());
    expect(screen.queryByText(/^Error: /)).toBeNull();
    expect(adapter.inspectCheckout).not.toHaveBeenCalled();
    expect(onOpenRemote).not.toHaveBeenCalled();
  });
});

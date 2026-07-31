import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";
import { render, screen, waitFor, fireEvent, within, act } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { App } from "../App";
import { APP_MESSAGES } from "../i18n/messages";
import { LOCAL_CAPABILITIES, type WorkspaceSession } from "../session";
import type { SpeclinkDataSource, StatusReport } from "@speclink/ui";

// 模擬 Tauri 事件層：捕捉 workspace-changed 的訂閱 handler，測試可手動觸發。
const { workspaceHandlers } = vi.hoisted(() => ({
  workspaceHandlers: [] as Array<() => void>,
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((event: string, handler: () => void) => {
    if (event === "workspace-changed") workspaceHandlers.push(handler);
    return Promise.resolve(() => {
      const i = workspaceHandlers.indexOf(handler);
      if (i >= 0) workspaceHandlers.splice(i, 1);
    });
  }),
}));

// app 版本查詢（側欄與設定頁的現版號）：jsdom 無 Tauri IPC，以固定版本模擬。
vi.mock("@tauri-apps/api/app", () => ({
  getVersion: vi.fn().mockResolvedValue("0.1.0"),
}));

// 兩個抽屜的 pass-through spy：捕捉 props（驗證刷新世代下發）後照常渲染原元件；
// Toaster 以 marker 驗證由 App 根層掛載，行為整合由 packages/ui 測試承載。
const { drawerSpy, toasterSpy } = vi.hoisted(() => ({
  drawerSpy: { rich: [] as Array<Record<string, unknown>>, disc: [] as Array<Record<string, unknown>> },
  toasterSpy: vi.fn(),
}));
vi.mock("@speclink/ui", async (importOriginal) => {
  const mod = await importOriginal<typeof import("@speclink/ui")>();
  return {
    ...mod,
    RichDetailDrawer: (props: never) => {
      drawerSpy.rich.push(props);
      return <mod.RichDetailDrawer {...(props as object) as Parameters<typeof mod.RichDetailDrawer>[0]} />;
    },
    DiscussionDrawer: (props: never) => {
      drawerSpy.disc.push(props);
      return <mod.DiscussionDrawer {...(props as object) as Parameters<typeof mod.DiscussionDrawer>[0]} />;
    },
    Toaster: () => {
      toasterSpy();
      return <div data-testid="app-toaster" />;
    },
  };
});

beforeEach(() => {
  workspaceHandlers.length = 0;
  drawerSpy.rich.length = 0;
  drawerSpy.disc.length = 0;
  toasterSpy.mockClear();
  // 各測試自行預置分頁持久化；先清掉避免跨測試洩漏。
  localStorage.removeItem("speclink.projectTabs");
  // jsdom 的 navigator.language 為 en-US；既有中文斷言以明示偏好 zh-TW 固定 UI 語言。
  localStorage.setItem("speclink.uiLocale", "zh-TW");
});

// workspace 探測面 mock（workspace-session 決策 6）：預設非專案語境——
// openProject 拒絕、startupDir 拒絕（首啟回退維持零分頁）。
function fakeWorkspace() {
  return {
    openProject: vi.fn().mockRejectedValue("not a project"),
    initProject: vi.fn(),
    adoptProject: vi.fn(),
    startupDir: vi.fn().mockRejectedValue("startup dir unavailable in this fake"),
    projectStats: vi.fn().mockResolvedValue({ pendingWrapUp: 0 }),
    watchWorkspace: vi.fn().mockResolvedValue(undefined),
    pickFolder: vi.fn().mockResolvedValue(null),
  };
}

/** 活躍 session 的設定面 mock（設定頁經 session.settings 讀寫）。 */
function fakeSettings() {
  return {
    readSettings: vi.fn().mockResolvedValue({
      app: { tools: [], customTools: [], parseError: null },
      workflow: {
        locale: null,
        specLocale: null,
        tdd: false,
        audit: false,
        context: null,
        rules: {},
        schemaArtifacts: ["proposal", "design", "specs", "tasks"],
        parseError: null,
      },
    }),
    writeAppTools: vi.fn(),
    writeWorkflowConfig: vi.fn().mockResolvedValue(undefined),
    writeWorkflowContext: vi.fn(),
    writeWorkflowRules: vi.fn(),
  };
}

/** session 工廠：dataSource 共用注入的 fake；events 訂閱者收進 workspaceHandlers，
 * 測試以 workspaceHandlers.forEach((h) => h()) 模擬 workspace-changed。 */
function makeSession(ds: SpeclinkDataSource, settings = fakeSettings()) {
  return (root: string, name: string): WorkspaceSession => ({
    id: `local:${root}`,
    locator: { kind: "local", root },
    descriptor: { name },
    dataSource: ds,
    settings: settings as never,
    events: {
      subscribe: (h: () => void) => {
        workspaceHandlers.push(h);
        return () => {
          const i = workspaceHandlers.indexOf(h);
          if (i >= 0) workspaceHandlers.splice(i, 1);
        };
      },
    },
    capabilities: LOCAL_CAPABILITIES,
  });
}

/** 預置單一專案分頁 A（v2 持久化、activeKey 指向）並渲染 App——
 * 資料經活躍 session 的 dataSource 載入（App 無全域 dataSource）。 */
function renderApp(
  ds: SpeclinkDataSource = fakeDataSource(),
  over: { ws?: ReturnType<typeof fakeWorkspace>; settings?: ReturnType<typeof fakeSettings> } = {},
) {
  localStorage.setItem(
    "speclink.projectTabs",
    JSON.stringify({
      version: 2,
      tabs: [{ locator: { kind: "local", root: "A" }, name: "proj-a" }],
      activeKey: "local:A",
    }),
  );
  const ws = over.ws ?? fakeWorkspace();
  if (!over.ws) {
    ws.openProject = vi.fn().mockResolvedValue({ status: "project", root: "A", name: "proj-a" });
  }
  const settings = over.settings ?? fakeSettings();
  render(<App createSession={makeSession(ds, settings)} workspace={ws as never} />);
  return { ds, ws, settings };
}

const STATUS: StatusReport = {
  changeName: "desktop-shell-and-browser",
  schemaName: "spec-driven",
  isComplete: false,
  applyRequires: ["tasks"],
  artifacts: [],
};

function fakeDataSource(over: Partial<SpeclinkDataSource> = {}): SpeclinkDataSource {
  return {
    listChanges: vi.fn().mockResolvedValue([
      { name: "desktop-shell-and-browser", status: "in-progress", totalTasks: 30, completedTasks: 30 },
    ]),
    listSpecs: vi.fn().mockResolvedValue([{ id: "desktop-app" }]),
    listArchived: vi.fn().mockResolvedValue([]),
    status: vi.fn().mockResolvedValue(STATUS),
    getDocument: vi.fn().mockResolvedValue("## Why\nhello body"),
    getSpecDocument: vi.fn().mockResolvedValue("# spec"),
    searchWorkspace: vi.fn().mockResolvedValue([]),
    changeCapabilities: vi.fn().mockResolvedValue(["desktop-app"]),
    changeMeta: vi.fn().mockResolvedValue({ created: "2026-07-05", createdBy: "MomoChen", createdWith: "claude" }),
    deleteChange: vi.fn().mockResolvedValue(undefined),
    setTaskDone: vi.fn().mockResolvedValue(undefined),
    setAllTasks: vi.fn().mockResolvedValue(undefined),
    moveTask: vi.fn().mockResolvedValue(undefined),
    runVerb: vi.fn().mockResolvedValue({ valid: true }),
    getArchivedDocument: vi.fn().mockResolvedValue(null),
    archivedCapabilities: vi.fn().mockResolvedValue([]),
    listDiscussions: vi.fn().mockResolvedValue({ active: [], archived: [] }),
    getDiscussionDocument: vi.fn().mockResolvedValue(null),
    promoteDiscussion: vi.fn().mockResolvedValue({ change: "promoted-change" }),
    archiveDiscussion: vi.fn().mockResolvedValue(undefined),
    reorderCard: vi.fn().mockResolvedValue(undefined),
    revertChangeToProposed: vi.fn().mockResolvedValue(undefined),
    ...over,
  };
}

describe("App (kanban primary + rich detail)", () => {
  it("根層掛載 Toaster，頂欄不含操作結果文字節點", async () => {
    renderApp();
    await waitFor(() => expect(screen.getByTestId("app-toaster")).toBeTruthy());
    expect(toasterSpy).toHaveBeenCalledTimes(1);

    const header = document.querySelector("header") as HTMLElement;
    expect(header).toBeTruthy();
    expect(header.querySelector(".font-mono")).toBeNull();
  });

  it("renders the kanban board by default with change cards", async () => {
    renderApp();
    await waitFor(() => expect(screen.getByText("desktop-shell-and-browser")).toBeTruthy());
    // 看板欄位存在
    expect(document.querySelector('[data-column="ready"]')).toBeTruthy();
  });

  it("opens the rich detail drawer with metadata when a card is clicked", async () => {
    const ds = fakeDataSource();
    renderApp(ds);
    await waitFor(() => screen.getByText("desktop-shell-and-browser"));
    fireEvent.click(screen.getByText("desktop-shell-and-browser"));
    await waitFor(() => expect(screen.getByText("MomoChen")).toBeTruthy());
    expect(ds.changeMeta).toHaveBeenCalledWith("desktop-shell-and-browser");
  });

  it("delete flow: drawer delete → confirm dialog → deleteChange called", async () => {
    // 階段守門後刪除鈕僅提案中可按（archive-readiness-gating）——改用提案中 fixture。
    const ds = fakeDataSource({
      listChanges: vi.fn().mockResolvedValue([
        { name: "desktop-shell-and-browser", status: "in-progress", totalTasks: 30, completedTasks: 0 },
      ]),
    });
    renderApp(ds);
    await waitFor(() => screen.getByText("desktop-shell-and-browser"));
    fireEvent.click(screen.getByText("desktop-shell-and-browser"));
    await waitFor(() => screen.getByRole("button", { name: /刪除/ }));
    fireEvent.click(screen.getByRole("button", { name: /刪除/ }));
    // 確認對話框
    await waitFor(() => screen.getByText("刪除變更？"));
    fireEvent.click(screen.getByRole("button", { name: "刪除" }));
    await waitFor(() => expect(ds.deleteChange).toHaveBeenCalledWith("desktop-shell-and-browser"));
  });

  it("revert flow: 進行中卡「退回提案中」→ 確認 → revertChangeToProposed called", async () => {
    // spec Scenario「零痕跡變更確認後退回提案中欄」的前半:點擊先出確認,
    // 確認後才呼叫 adapter(UI 不預判守門)。
    const ds = fakeDataSource({
      listChanges: vi.fn().mockResolvedValue([
        { name: "oops-started", status: "in-progress", totalTasks: 10, completedTasks: 0, startedAt: "2026-07-30" },
      ]),
    });
    renderApp(ds);
    const card = (await screen.findByText("oops-started")).closest("[data-change]") as HTMLElement;
    fireEvent.click(within(card).getByRole("button", { name: /退回提案中/ }));
    // 點擊先出確認——adapter 尚未被呼叫。
    expect(ds.revertChangeToProposed).not.toHaveBeenCalled();
    const confirm = await screen.findByRole("alertdialog");
    fireEvent.click(within(confirm).getByRole("button", { name: "退回" }));
    await waitFor(() => expect(ds.revertChangeToProposed).toHaveBeenCalledWith("oops-started"));
  });

  it("revert blocked: 守門拒絕開對話框列證據,無任何清理或強制退回按鈕", async () => {
    // spec Scenario「有工作痕跡時顯示守門對話框」。
    const { RevertBlockedError } = await import("@speclink/ui");
    const ds = fakeDataSource({
      listChanges: vi.fn().mockResolvedValue([
        { name: "oops-started", status: "in-progress", totalTasks: 10, completedTasks: 3, startedAt: "2026-07-30" },
      ]),
      revertChangeToProposed: vi
        .fn()
        .mockRejectedValue(
          new RevertBlockedError({ checkedTasks: 3, touchedFiles: ["src/a.rs", "src/b.ts"] }),
        ),
    });
    renderApp(ds);
    const card = (await screen.findByText("oops-started")).closest("[data-change]") as HTMLElement;
    fireEvent.click(within(card).getByRole("button", { name: /退回提案中/ }));
    const confirm = await screen.findByRole("alertdialog");
    fireEvent.click(within(confirm).getByRole("button", { name: "退回" }));
    // 守門對話框:列出已勾任務數與 touched 檔案清單。
    await waitFor(() => expect(screen.getByText(/src\/a\.rs/)).toBeTruthy());
    const dialog = screen.getByRole("alertdialog");
    expect(dialog.textContent).toContain("3");
    expect(dialog.textContent).toContain("src/b.ts");
    // 無清理/強制退回的機械出路——唯一按鈕只負責關閉。
    expect(within(dialog).getAllByRole("button")).toHaveLength(1);
    // 卡片停留在進行中欄。
    expect(document.querySelector('[data-column="in-progress"]')?.textContent).toContain(
      "oops-started",
    );
  });

  it("封存失敗時確認框關閉不會連帶關閉詳情抽屜", async () => {
    let rejectArchive!: (reason: unknown) => void;
    const ds = fakeDataSource({
      runVerb: vi.fn().mockImplementation(
        () => new Promise((_, reject) => {
          rejectArchive = reject;
        }),
      ),
    });
    renderApp(ds);
    fireEvent.click(await screen.findByText("desktop-shell-and-browser"));
    const drawer = await screen.findByRole("dialog");
    fireEvent.click(within(drawer).getByRole("button", { name: "封存" }));
    const confirm = await screen.findByRole("alertdialog");
    fireEvent.click(within(confirm).getByRole("button", { name: "封存" }));
    await waitFor(() => expect(ds.runVerb).toHaveBeenCalledWith("archive", "desktop-shell-and-browser"));
    // 原生 WebView 中 AlertDialog 收合會讓底下的 Sheet 收到 false；操作尚在途時須忽略。
    act(() => {
      (drawerSpy.rich[drawerSpy.rich.length - 1].onOpenChange as (open: boolean) => void)(false);
    });
    const stayedOpen = drawerSpy.rich[drawerSpy.rich.length - 1].open;
    await act(async () => {
      rejectArchive(new Error("archive prerequisites missing"));
    });
    expect(stayedOpen).toBe(true);
    expect(await screen.findByRole("dialog")).toBeTruthy();
  });

  it("passes an increasing refreshGen generation to both drawers", async () => {
    // design D1：世代自 store 經 props 下發，內容元件據此重載（重載行為由 packages/ui 測試承載）。
    renderApp();
    await waitFor(() => expect(workspaceHandlers.length).toBeGreaterThan(0));
    await waitFor(() => expect(drawerSpy.rich.length).toBeGreaterThan(0));
    await waitFor(() => expect(drawerSpy.disc.length).toBeGreaterThan(0));
    const before = drawerSpy.rich[drawerSpy.rich.length - 1].refreshGen;
    expect(typeof before).toBe("number");
    workspaceHandlers.forEach((h) => h());
    await waitFor(() => {
      expect(drawerSpy.rich[drawerSpy.rich.length - 1].refreshGen as number).toBeGreaterThan(before as number);
      expect(drawerSpy.disc[drawerSpy.disc.length - 1].refreshGen as number).toBeGreaterThan(before as number);
    });
  });

  it("workspace-changed event triggers a full refresh (external writers reflected)", async () => {
    const ds = fakeDataSource();
    renderApp(ds);
    await waitFor(() => screen.getByText("desktop-shell-and-browser"));
    await waitFor(() => expect(workspaceHandlers.length).toBeGreaterThan(0));
    const before = (ds.listChanges as Mock).mock.calls.length;
    // 模擬檔案監看發出的 Tauri 事件（外部 CLI/agent 寫入後）。
    workspaceHandlers.forEach((h) => h());
    await waitFor(() =>
      expect((ds.listChanges as Mock).mock.calls.length).toBeGreaterThan(before)
    );
    expect((ds.listArchived as Mock).mock.calls.length).toBeGreaterThan(1);
  });

  it("GUI 撤除 promote：concluded 討論卡無「轉為變更」，且無轉為變更確認框（D3）", async () => {
    const ds = fakeDataSource({
      listDiscussions: vi.fn().mockResolvedValue({
        active: [
          { slug: "settled", topic: "Settled topic", status: "concluded", rounds: 2, created: "2026-07-01", promotedTo: [] },
        ],
        archived: [],
      }),
    });
    renderApp(ds);
    await waitFor(() => screen.getByText("Settled topic"));
    // 轉為變更動詞與確認框皆已撤除；轉出改由 CLI／agent。
    expect(screen.queryByRole("button", { name: /轉為變更/ })).toBeNull();
    expect(screen.queryByText("轉為變更？")).toBeNull();
    expect(ds.promoteDiscussion).not.toHaveBeenCalled();
    // 封存動詞仍在（concluded 卡）。
    const card = screen.getByText("Settled topic").closest("[data-discussion]") as HTMLElement;
    expect(within(card).getByRole("button", { name: /封存/ })).toBeTruthy();
  });

  it("archive-discussion flow: 討論卡「封存」→ 確認（使用者語言）→ archiveDiscussion called", async () => {
    const ds = fakeDataSource({
      listDiscussions: vi.fn().mockResolvedValue({
        active: [
          { slug: "settled", topic: "Settled topic", status: "concluded", rounds: 2, created: "2026-07-01", promotedTo: [] },
        ],
        archived: [],
      }),
    });
    renderApp(ds);
    await waitFor(() => screen.getByText("Settled topic"));
    const card = screen.getByText("Settled topic").closest("[data-discussion]") as HTMLElement;
    fireEvent.click(within(card).getByRole("button", { name: /^封存$/ }));
    await waitFor(() => screen.getByText("封存討論？"));
    const dialog = screen.getByRole("alertdialog");
    expect(dialog.textContent).toContain("已封存頁");
    expect(dialog.textContent).not.toContain("discussions/archive");
    fireEvent.click(within(dialog).getByRole("button", { name: "封存" }));
    await waitFor(() => expect(ds.archiveDiscussion).toHaveBeenCalledWith("settled"));
  });

  it("零分頁（注入 workspace）：顯示空狀態引導頁，經 chooser 沿用本機開啟", async () => {
    const ws = fakeWorkspace();
    const ds = fakeDataSource({ listChanges: vi.fn().mockResolvedValue([]) });
    render(<App createSession={makeSession(ds)} workspace={ws as never} />);
    expect(await screen.findByText("開啟一個專案開始")).toBeTruthy();
    // 空狀態與頂列皆匯流至新增 Workspace chooser，再選本機資料夾。
    const openButtons = screen.getAllByText("新增 Workspace");
    fireEvent.click(openButtons[openButtons.length - 1]);
    const chooser = await screen.findByRole("alertdialog");
    fireEvent.click(within(chooser).getByRole("button", { name: /本機資料夾/ }));
    await waitFor(() => expect(ws.pickFolder).toHaveBeenCalled());
  });

  // spec「表單控制項與按鈕以主題化元件呈現」Scenario「初始化對話框工具多選主題化」
  it("初始化對話框的工具多選為主題化 checkbox：預設勾 claude、可獨立切換", async () => {
    const ws = fakeWorkspace();
    ws.pickFolder = vi.fn().mockResolvedValue("D:/newproj");
    ws.openProject = vi.fn().mockResolvedValue({ status: "uninitialized", dir: "D:/newproj" });
    const ds = fakeDataSource({ listChanges: vi.fn().mockResolvedValue([]) });
    render(<App createSession={makeSession(ds)} workspace={ws as never} />);
    const openButtons = await screen.findAllByText("新增 Workspace");
    fireEvent.click(openButtons[openButtons.length - 1]);
    const chooser = await screen.findByRole("alertdialog");
    fireEvent.click(within(chooser).getByRole("button", { name: /本機資料夾/ }));
    // chooser 關閉有離場動畫；等待初始化框的專屬控制項，避免抓到尚未卸載的 chooser。
    const claude = await screen.findByRole("checkbox", { name: "claude" });
    const dialog = claude.closest('[role="alertdialog"]') as HTMLElement;
    expect(dialog).toBeTruthy();
    const codex = within(dialog).getByRole("checkbox", { name: "codex" });
    // 主題化原語（button 元素）而非原生 input。
    expect(claude.tagName).not.toBe("INPUT");
    // 預設勾選狀態與替換前相同：claude 勾、codex 未勾。
    expect(claude.getAttribute("aria-checked")).toBe("true");
    expect(codex.getAttribute("aria-checked")).toBe("false");
    // 可獨立切換：勾 codex 不影響 claude；取消 claude 不影響 codex。
    fireEvent.click(codex);
    expect(codex.getAttribute("aria-checked")).toBe("true");
    expect(claude.getAttribute("aria-checked")).toBe("true");
    fireEvent.click(claude);
    expect(claude.getAttribute("aria-checked")).toBe("false");
    expect(codex.getAttribute("aria-checked")).toBe("true");
  });

  // spec「未啟用資料夾經確認後補齊啟用」：啟用確認框與初始化框同型、獨立狀態（決策 3）
  it("啟用確認對話框：unadopted 探測開框、預設勾 claude、確認以所選工具呼叫 adopt", async () => {
    const ws = fakeWorkspace();
    ws.pickFolder = vi.fn().mockResolvedValue("D:/migrated");
    ws.openProject = vi.fn().mockResolvedValue({ status: "unadopted", root: "D:/migrated" });
    ws.adoptProject = vi
      .fn()
      .mockResolvedValue({ status: "project", root: "D:/migrated", name: "migrated" });
    const ds = fakeDataSource({ listChanges: vi.fn().mockResolvedValue([]) });
    render(<App createSession={makeSession(ds)} workspace={ws as never} />);
    const openButtons = await screen.findAllByText("新增 Workspace");
    fireEvent.click(openButtons[openButtons.length - 1]);
    const chooser = await screen.findByRole("alertdialog");
    fireEvent.click(within(chooser).getByRole("button", { name: /本機資料夾/ }));
    // chooser 關閉有離場動畫；等待啟用框的專屬控制項。
    const claude = await screen.findByRole("checkbox", { name: "claude" });
    const dialog = claude.closest('[role="alertdialog"]') as HTMLElement;
    expect(dialog).toBeTruthy();
    // 啟用語意文案（非初始化文案）。
    expect(within(dialog).getByText("啟用 speclink？")).toBeTruthy();
    // 工具多選預設勾 claude、codex 未勾。
    expect(claude.getAttribute("aria-checked")).toBe("true");
    const codex = within(dialog).getByRole("checkbox", { name: "codex" });
    expect(codex.getAttribute("aria-checked")).toBe("false");
    // 確認 → 以所選工具呼叫 adopt（而非 init）。
    fireEvent.click(within(dialog).getByRole("button", { name: "啟用" }));
    await waitFor(() => expect(ws.adoptProject).toHaveBeenCalledWith("D:/migrated", ["claude"]));
    expect(ws.initProject).not.toHaveBeenCalled();
  });

  it("啟用確認對話框：取消關框且零寫入呼叫", async () => {
    const ws = fakeWorkspace();
    ws.pickFolder = vi.fn().mockResolvedValue("D:/migrated");
    ws.openProject = vi.fn().mockResolvedValue({ status: "unadopted", root: "D:/migrated" });
    const ds = fakeDataSource({ listChanges: vi.fn().mockResolvedValue([]) });
    render(<App createSession={makeSession(ds)} workspace={ws as never} />);
    const openButtons = await screen.findAllByText("新增 Workspace");
    fireEvent.click(openButtons[openButtons.length - 1]);
    const chooser = await screen.findByRole("alertdialog");
    fireEvent.click(within(chooser).getByRole("button", { name: /本機資料夾/ }));
    const claude = await screen.findByRole("checkbox", { name: "claude" });
    const dialog = claude.closest('[role="alertdialog"]') as HTMLElement;
    fireEvent.click(within(dialog).getByRole("button", { name: "取消" }));
    await waitFor(() => expect(screen.queryByRole("checkbox", { name: "claude" })).toBeNull());
    expect(ws.adoptProject).not.toHaveBeenCalled();
    expect(ws.initProject).not.toHaveBeenCalled();
  });

  it("分頁列取代頂欄「目前專案」佔位；點分頁切換後 active 標示更新", async () => {
    localStorage.setItem(
      "speclink.projectTabs",
      JSON.stringify({
        tabs: [
          { root: "A", name: "proj-a" },
          { root: "B", name: "proj-b" },
        ],
        activeRoot: "A",
      }),
    );
    const ws = fakeWorkspace();
    ws.openProject = vi
      .fn()
      .mockImplementation((p: string) =>
        Promise.resolve({ status: "project", root: p, name: p === "A" ? "proj-a" : "proj-b" }),
      );
    render(<App createSession={makeSession(fakeDataSource())} workspace={ws as never} />);
    const tabA = (await screen.findByText("proj-a")).closest("[data-tab]") as HTMLElement;
    expect(tabA.getAttribute("data-active")).toBe("true");
    // 佔位文字已被分頁列取代。
    expect(screen.queryByText("目前專案")).toBeNull();
    fireEvent.click(screen.getByText("proj-b"));
    await waitFor(() => {
      const tabB = screen.getByText("proj-b").closest("[data-tab]") as HTMLElement;
      expect(tabB.getAttribute("data-active")).toBe("true");
    });
  });

  it("切換 UI 語言即時全介面生效、持久化於本機且不觸碰 config.yaml（spec 互不影響）", async () => {
    localStorage.setItem(
      "speclink.projectTabs",
      JSON.stringify({ tabs: [{ root: "A", name: "proj-a" }], activeRoot: "A" }),
    );
    const ws = fakeWorkspace();
    ws.openProject = vi
      .fn()
      .mockResolvedValue({ status: "project", root: "A", name: "proj-a" });
    const settings = fakeSettings();
    render(<App createSession={makeSession(fakeDataSource(), settings)} workspace={ws as never} />);
    // 開應用程式設定頁 → 本機設定為預設簽 → 切 English。
    fireEvent.click(await screen.findByText("設定"));
    fireEvent.mouseDown(await screen.findByRole("tab", { name: "本機設定" }));
    const group = await screen.findByTestId("ui-locale");
    fireEvent.click(within(group).getByText("English"));
    // 即時全介面生效：側欄改為英文。
    expect(await screen.findByText("Settings")).toBeTruthy();
    expect(screen.getByText("Changes")).toBeTruthy();
    // 持久化於 app 本機；config.yaml 未被觸碰。
    expect(localStorage.getItem("speclink.uiLocale")).toBe("en");
    expect(settings.writeWorkflowConfig).not.toHaveBeenCalled();
  });

  it("寫入 config locale 不改 UI 語言（spec 互不影響的反向）", async () => {
    localStorage.setItem(
      "speclink.projectTabs",
      JSON.stringify({ tabs: [{ root: "A", name: "proj-a" }], activeRoot: "A" }),
    );
    const ws = fakeWorkspace();
    ws.openProject = vi
      .fn()
      .mockResolvedValue({ status: "project", root: "A", name: "proj-a" });
    const settings = fakeSettings();
    render(<App createSession={makeSession(fakeDataSource(), settings)} workspace={ws as never} />);
    fireEvent.click(await screen.findByText("專案設定"));
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await user.click(await screen.findByLabelText("locale"));
    await user.click(await screen.findByRole("option", { name: /^ja/ }));
    fireEvent.click(screen.getByTestId("save-workflow"));
    await waitFor(() =>
      expect(settings.writeWorkflowConfig).toHaveBeenCalledWith(
        expect.objectContaining({ locale: "ja" }),
      ),
    );
    // UI 語言不受影響：介面仍為 zh-TW、偏好鍵未被改動。
    expect(screen.getByText("設定")).toBeTruthy();
    expect(localStorage.getItem("speclink.uiLocale")).toBe("zh-TW");
  });

  it("archived entry in the sidebar jumps to the archived list", async () => {
    const ds = fakeDataSource({
      listArchived: vi.fn().mockResolvedValue([
        { datedName: "2026-07-04-old-change", date: "2026-07-04", name: "old-change" },
      ]),
    });
    renderApp(ds);
    await waitFor(() => screen.getByText("desktop-shell-and-browser"));
    fireEvent.click(screen.getByLabelText("已封存"));
    await waitFor(() => expect(screen.getByText("已封存的變更")).toBeTruthy());
    expect(screen.getByText("old-change")).toBeTruthy();
    expect(screen.getByText("2026-07-04")).toBeTruthy();
  });
});

describe("board search wiring（看板搜尋接線）", () => {
  it("kanban view renders a search input that filters cards and reflects boardQuery", async () => {
    renderApp();
    await waitFor(() => screen.getByText("desktop-shell-and-browser"));
    const input = screen.getByPlaceholderText("搜尋看板卡片…") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "zzz-no-match" } });
    // 受控輸入反映 store.boardQuery 的更新；卡片被過濾。
    expect(input.value).toBe("zzz-no-match");
    expect(screen.queryByText("desktop-shell-and-browser")).toBeNull();
    // 清空還原全量。
    fireEvent.change(input, { target: { value: "" } });
    expect(screen.getByText("desktop-shell-and-browser")).toBeTruthy();
  });

  it("the archived page search input does not contain the board query (independence)", async () => {
    // spec「搜尋字串不跨啟動保留且與已封存頁獨立」的獨立性半邊；
    // 不跨啟動由 store 無 persist 保證（store.test.ts 1.x）。
    renderApp();
    await waitFor(() => screen.getByText("desktop-shell-and-browser"));
    fireEvent.change(screen.getByPlaceholderText("搜尋看板卡片…"), {
      target: { value: "kanban-only" },
    });
    fireEvent.click(screen.getByLabelText("已封存"));
    await waitFor(() => expect(screen.getByText("已封存的變更")).toBeTruthy());
    const archInput = screen.getByPlaceholderText("搜尋已封存的變更與討論…") as HTMLInputElement;
    expect(archInput.value).toBe("");
  });
});

describe("sidebar navigation structure（側欄導覽結構）", () => {
  it("側欄頂部依序為變更/規格/已封存/專案設定，設定沉底，無備忘項且頂欄無已封存鈕", async () => {
    renderApp();
    await waitFor(() => screen.getByText("desktop-shell-and-browser"));
    const aside = document.querySelector("aside") as HTMLElement;
    // 已封存項以 aria-label 為無障礙名稱（徽章數字不污染），其餘取文字內容。
    const labels = within(aside)
      .getAllByRole("button")
      .map((b) => b.getAttribute("aria-label") ?? b.textContent ?? "");
    expect(labels).toEqual(["變更", "規格", "已封存", "專案設定", "設定"]);
    expect(screen.queryByText("備忘")).toBeNull();
    const header = document.querySelector("header") as HTMLElement;
    expect(within(header).queryByLabelText("已封存")).toBeNull();
    expect(within(header).queryByText("已封存")).toBeNull();
  });

  it("設定導覽項沉底：為側欄最末子元素、以自動上邊距與頂部四項彈性區隔，切頁與高亮語意不變", async () => {
    renderApp();
    await screen.findByText("desktop-shell-and-browser");
    const aside = document.querySelector("aside") as HTMLElement;
    const settingsNav = within(aside).getByRole("button", { name: "設定" });
    // 頂部四項維持依序；設定為側欄最末子元素。
    const labels = within(aside)
      .getAllByRole("button")
      .map((b) => b.getAttribute("aria-label") ?? b.textContent ?? "");
    expect(labels.slice(0, 4)).toEqual(["變更", "規格", "已封存", "專案設定"]);
    expect(aside.lastElementChild).toBe(settingsNav);
    // 彈性區隔：jsdom 無版面計算，以等效自動上邊距 class 斷言（design D5）。
    expect(settingsNav.className).toContain("mt-auto");
    // 切頁與高亮語意不變：點設定離開看板並高亮設定項。
    const changesNav = within(aside).getByRole("button", { name: "變更" });
    fireEvent.click(settingsNav);
    await waitFor(() => expect(document.querySelector('[data-column="ready"]')).toBeNull());
    expect(settingsNav.className).toContain("bg-primary");
    expect(changesNav.className).not.toContain("bg-primary");
  });

  it("點專案設定切至專案設定頁並轉移高亮", async () => {
    renderApp();
    await screen.findByText("desktop-shell-and-browser");
    const aside = document.querySelector("aside") as HTMLElement;
    const projectSettingsNav = within(aside).getByRole("button", { name: "專案設定" });
    const changesNav = within(aside).getByRole("button", { name: "變更" });

    fireEvent.click(projectSettingsNav);

    expect(await screen.findByRole("tab", { name: "config.yaml" })).toBeTruthy();
    expect(document.querySelector('[data-column="ready"]')).toBeNull();
    expect(projectSettingsNav.className).toContain("bg-primary");
    expect(changesNav.className).not.toContain("bg-primary");
  });

  it("零分頁時設定仍進入應用程式設定頁，專案設定則呈現空狀態引導頁", async () => {
    const ws = fakeWorkspace();
    render(<App createSession={makeSession(fakeDataSource())} workspace={ws as never} />);
    expect(await screen.findByText("開啟一個專案開始")).toBeTruthy();
    const aside = document.querySelector("aside") as HTMLElement;
    const settingsNav = within(aside).getByRole("button", { name: "設定" });

    fireEvent.click(settingsNav);
    expect(await screen.findByRole("tab", { name: "本機設定" })).toBeTruthy();
    expect(screen.queryByText("開啟一個專案開始")).toBeNull();
    expect(settingsNav.className).toContain("bg-primary");

    const projectSettingsNav = within(aside).getByRole("button", { name: "專案設定" });
    fireEvent.click(projectSettingsNav);
    expect(await screen.findByText("開啟一個專案開始")).toBeTruthy();
    expect(projectSettingsNav.className).toContain("bg-primary");
    expect(settingsNav.className).not.toContain("bg-primary");
  });

  it("已封存導覽項帶封存數量徽章，無障礙標籤為「已封存」", async () => {
    const ds = fakeDataSource({
      listArchived: vi.fn().mockResolvedValue([
        { datedName: "2026-07-04-old-change", date: "2026-07-04", name: "old-change" },
        { datedName: "2026-07-05-other-change", date: "2026-07-05", name: "other-change" },
      ]),
    });
    renderApp(ds);
    await waitFor(() => screen.getByText("desktop-shell-and-browser"));
    const aside = document.querySelector("aside") as HTMLElement;
    const nav = within(aside).getByRole("button", { name: "已封存" });
    await waitFor(() => expect(nav.textContent).toContain("2"));
  });

  it("封存清單變動後徽章即時更新（workspace-changed 觸發 refresh）", async () => {
    const archived: Array<{ datedName: string; date: string; name: string }> = [];
    const ds = fakeDataSource({
      listArchived: vi.fn().mockImplementation(() => Promise.resolve([...archived])),
    });
    renderApp(ds);
    await waitFor(() => screen.getByText("desktop-shell-and-browser"));
    const aside = document.querySelector("aside") as HTMLElement;
    const nav = within(aside).getByRole("button", { name: "已封存" });
    expect(nav.textContent).toContain("0");
    // 模擬外部終端封存一個變更：檔案監看發 workspace-changed → 整批 refresh。
    archived.push({ datedName: "2026-07-07-just-archived", date: "2026-07-07", name: "just-archived" });
    workspaceHandlers.forEach((h) => h());
    await waitFor(() => expect(nav.textContent).toContain("1"));
  });

  it("已封存導覽為切頁而非 toggle：再點停留在已封存頁，點變更才返回看板", async () => {
    renderApp();
    await waitFor(() => screen.getByText("desktop-shell-and-browser"));
    const aside = document.querySelector("aside") as HTMLElement;
    const archivedNav = within(aside).getByRole("button", { name: "已封存" });
    fireEvent.click(archivedNav);
    await waitFor(() => expect(screen.getByText("已封存的變更")).toBeTruthy());
    // 再點一次：停留（非 toggle 返回看板），現行項維持高亮。
    fireEvent.click(archivedNav);
    expect(screen.getByText("已封存的變更")).toBeTruthy();
    expect(document.querySelector('[data-column="ready"]')).toBeNull();
    expect(archivedNav.className).toContain("bg-primary");
    // 點「變更」返回看板：高亮轉移到變更項、已封存項恢復未選取樣式。
    const changesNav = within(aside).getByRole("button", { name: "變更" });
    fireEvent.click(changesNav);
    await waitFor(() => expect(document.querySelector('[data-column="ready"]')).toBeTruthy());
    expect(changesNav.className).toContain("bg-primary");
    expect(archivedNav.className).not.toContain("bg-primary");
  });

  it("點導覽「規格」進入規格頁：主內容出現規格清單、導覽項 active", async () => {
    // spec Scenario「進入規格頁顯示卡片清單」：切頁語意（與已封存頁同型），
    // 主內容渲染 SpecList（正典 spec 卡片＋搜尋列），返回看板點「變更」。
    renderApp();
    await waitFor(() => screen.getByText("desktop-shell-and-browser"));
    const aside = document.querySelector("aside") as HTMLElement;
    const specsNav = within(aside).getByRole("button", { name: "規格" });
    fireEvent.click(specsNav);
    await waitFor(() => expect(screen.getByText("desktop-app")).toBeTruthy());
    expect(screen.getByPlaceholderText("搜尋規格…")).toBeTruthy();
    expect(document.querySelector('[data-column="ready"]')).toBeNull();
    expect(specsNav.className).toContain("bg-primary");
    // 點「變更」返回看板。
    const changesNav = within(aside).getByRole("button", { name: "變更" });
    fireEvent.click(changesNav);
    await waitFor(() => expect(document.querySelector('[data-column="ready"]')).toBeTruthy());
    expect(specsNav.className).not.toContain("bg-primary");
  });

  it("規格頁點卡開唯讀規格抽屜，經 dataSource.getSpecDocument 載入正典全文", async () => {
    // spec Scenario「選定 spec 以抽屜顯示其正典內容」：App 掛載 SpecDrawer 並接線
    // store.detailSpec 與 dataSource.getSpecDocument（spec-archive-drawer design D2）。
    const ds = fakeDataSource({
      getSpecDocument: vi.fn().mockResolvedValue("# desktop-app Specification\n\n正典內文段落。"),
    });
    renderApp(ds);
    await waitFor(() => screen.getByText("desktop-shell-and-browser"));
    const aside = document.querySelector("aside") as HTMLElement;
    fireEvent.click(within(aside).getByRole("button", { name: "規格" }));
    await waitFor(() => screen.getByText("desktop-app"));
    expect(ds.getSpecDocument).not.toHaveBeenCalled();
    fireEvent.click(screen.getByText("desktop-app"));
    await waitFor(() => expect(screen.getByText("正典內文段落。")).toBeTruthy());
    expect(ds.getSpecDocument).toHaveBeenCalledWith("desktop-app");
    // 內容呈現在抽屜、非行內展開。
    expect(document.querySelector("[data-spec-drawer]")).toBeTruthy();
  });

  it("已封存頁點封存變更卡開四分頁唯讀抽屜（spec「已封存項目以抽屜檢視」）", async () => {
    const ds = fakeDataSource({
      listArchived: vi.fn().mockResolvedValue([
        {
          datedName: "2026-07-04-old",
          date: "2026-07-04",
          name: "old",
          tasksTotal: 2,
          tasksDone: 2,
          specCount: 1,
          createdBy: null,
          fromDiscussions: [],
        },
      ]),
      getArchivedDocument: vi.fn().mockResolvedValue("## Why\n\n封存提案內文。"),
    });
    renderApp(ds);
    await waitFor(() => screen.getByText("desktop-shell-and-browser"));
    const aside = document.querySelector("aside") as HTMLElement;
    fireEvent.click(within(aside).getByRole("button", { name: /已封存/ }));
    await waitFor(() => screen.getByText("old"));
    expect(ds.getArchivedDocument).not.toHaveBeenCalled();
    fireEvent.click(screen.getByText("old"));
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    expect(document.querySelector("[data-archived-drawer]")).toBeTruthy();
    expect(screen.getByRole("tab", { name: /提案/ })).toBeTruthy();
    expect(ds.getArchivedDocument).toHaveBeenCalledWith("2026-07-04-old", "proposal.md");
  });

  it("封存變更抽屜點來源討論 chip，同一抽屜切換為該討論的唯讀檢視", async () => {
    // spec Scenario「自封存變更抽屜跳轉來源討論」的接線面：fromDiscussions →
    // chips（topic 解析）→ openArchived({ kind: "discussion" })。
    const ds = fakeDataSource({
      listArchived: vi.fn().mockResolvedValue([
        {
          datedName: "2026-07-04-old",
          date: "2026-07-04",
          name: "old",
          specCount: 1,
          createdBy: null,
          fromDiscussions: ["old-topic"],
        },
      ]),
      listDiscussions: vi.fn().mockResolvedValue({
        active: [],
        archived: [
          { slug: "old-topic", topic: "Old topic", status: "promoted", rounds: 1, created: "2026-06-30", promotedTo: ["x"] },
        ],
      }),
      getArchivedDocument: vi.fn().mockResolvedValue("## Why\n\n封存提案內文。"),
      getDiscussionDocument: vi
        .fn()
        .mockResolvedValue(
          "---\ntopic: Old topic\nslug: old-topic\nstatus: promoted\ncreated: 2026-06-30\n---\n\n# Discussion: Old topic\n\n## Context\n\n封存背景內文。\n\n## Rounds\n\n## Conclusion\n\n**Decision**: 收工\n",
        ),
    });
    renderApp(ds);
    await waitFor(() => screen.getByText("desktop-shell-and-browser"));
    const aside = document.querySelector("aside") as HTMLElement;
    fireEvent.click(within(aside).getByRole("button", { name: /已封存/ }));
    await waitFor(() => screen.getByText("old"));
    fireEvent.click(screen.getByText("old"));
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    // chip 以 topic 顯示（自 discussions 兩節解析）。
    fireEvent.click(screen.getByRole("button", { name: "Old topic" }));
    await waitFor(() => expect(screen.getByText("封存背景內文。")).toBeTruthy());
    expect(ds.getDiscussionDocument).toHaveBeenCalledWith("old-topic");
    expect(screen.getByText("討論過程")).toBeTruthy();
  });

  it("已封存頁點封存討論卡開唯讀區段抽屜", async () => {
    const ds = fakeDataSource({
      listDiscussions: vi.fn().mockResolvedValue({
        active: [],
        archived: [
          { slug: "old-topic", topic: "Old topic", status: "promoted", rounds: 1, created: "2026-06-30", promotedTo: ["x"] },
        ],
      }),
      getDiscussionDocument: vi
        .fn()
        .mockResolvedValue(
          "---\ntopic: Old topic\nslug: old-topic\nstatus: promoted\ncreated: 2026-06-30\n---\n\n# Discussion: Old topic\n\n## Context\n\n封存背景內文。\n\n## Rounds\n\n## Conclusion\n\n**Decision**: 收工\n",
        ),
    });
    renderApp(ds);
    await waitFor(() => screen.getByText("desktop-shell-and-browser"));
    const aside = document.querySelector("aside") as HTMLElement;
    fireEvent.click(within(aside).getByRole("button", { name: /已封存/ }));
    // 已封存頁為「變更／討論」子頁籤（specs-archive-pagination design D3）：
    // 討論卡在「已封存的討論」子頁籤下。
    fireEvent.mouseDown(await screen.findByRole("tab", { name: /已封存的討論/ }));
    await waitFor(() => screen.getByText("Old topic"));
    fireEvent.click(screen.getByText("Old topic"));
    await waitFor(() => expect(screen.getByText("封存背景內文。")).toBeTruthy());
    expect(document.querySelector("[data-archived-drawer]")).toBeTruthy();
    expect(ds.getDiscussionDocument).toHaveBeenCalledWith("old-topic");
    expect(screen.getByText("討論過程")).toBeTruthy();
  });

  it("i18n 兩語系鍵集合相等，備忘鍵已自兩語系移除", () => {
    const zhKeys = Object.keys(APP_MESSAGES["zh-TW"]).sort();
    const enKeys = Object.keys(APP_MESSAGES.en).sort();
    expect(zhKeys).toEqual(enKeys);
    expect(zhKeys).not.toContain("app.navNotes");
  });
});

// spec 需求「清單最新在前與換頁瀏覽」（填滿高度增補）：規格頁與已封存頁的內部
// 捲動容器高度須受視窗約束——main 帶 overflow-hidden 而非整頁捲動；設定頁不受
// 影響、維持 overflow-y-auto 整頁捲動。
describe("main content scroll containment（主內容區捲動約束）", () => {
  it("看板/規格/已封存頁 main 為 overflow-hidden，設定頁維持 overflow-y-auto", async () => {
    renderApp();
    await screen.findByText("desktop-shell-and-browser");
    const main = () => document.querySelector("main") as HTMLElement;
    const aside = document.querySelector("aside") as HTMLElement;
    // 看板（預設）：既有 overflow-hidden。
    expect(main().className).toContain("overflow-hidden");
    // 規格頁：改 overflow-hidden，清單於內部容器捲動。
    fireEvent.click(within(aside).getByRole("button", { name: "規格" }));
    await waitFor(() => expect(screen.getByText("desktop-app")).toBeTruthy());
    expect(main().className).toContain("overflow-hidden");
    expect(main().className).not.toContain("overflow-y-auto");
    // 已封存頁：同上。
    fireEvent.click(within(aside).getByRole("button", { name: "已封存" }));
    await waitFor(() => expect(screen.getByText("已封存的變更")).toBeTruthy());
    expect(main().className).toContain("overflow-hidden");
    expect(main().className).not.toContain("overflow-y-auto");
    // 設定頁：維持整頁捲動。
    fireEvent.click(within(aside).getByRole("button", { name: "設定" }));
    await waitFor(() => expect(main().className).toContain("overflow-y-auto"));
    expect(main().className).not.toContain("overflow-hidden");
  });
});

describe("側欄無常駐版號（desktop-app 規格「側欄導覽結構」）", () => {
  it("注入 updater 面時側欄任何位置仍無版號文字；app 版號唯一住所為設定頁軟體更新卡", async () => {
    const ws = fakeWorkspace();
    ws.openProject = vi.fn().mockResolvedValue({ status: "project", root: "A", name: "proj-a" });
    render(
      <App
        createSession={makeSession(fakeDataSource())}
        workspace={ws as never}
        updater={{ check: vi.fn().mockResolvedValue(null), relaunch: vi.fn() }}
      />,
    );
    const aside = await screen.findByRole("complementary");
    await waitFor(() => expect(within(aside).getByRole("button", { name: "設定" })).toBeTruthy());
    expect(aside.textContent).not.toContain("v0.1.0");

    // 設定頁軟體更新卡仍顯示目前版本。
    fireEvent.click(within(aside).getByRole("button", { name: "設定" }));
    await waitFor(() => expect(screen.getByText(/目前版本\s*0\.1\.0/)).toBeTruthy());
  });

  it("未注入 updater 面時同樣無版號文字", async () => {
    renderApp();
    await waitFor(() => expect(screen.getByRole("complementary")).toBeTruthy());
    expect(screen.queryByText(/^v\d/)).toBeNull();
  });
});

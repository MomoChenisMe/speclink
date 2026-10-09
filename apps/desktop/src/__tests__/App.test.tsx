import { describe, it, expect, vi, beforeEach, afterEach, type Mock } from "vitest";
import { render, screen, waitFor, fireEvent, within, act, cleanup } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";
import { toast } from "sonner";

import { App } from "../App";
import { buildShortcutList } from "../appMenu";
import { APP_MESSAGES } from "../i18n/messages";
import { detectPlatform } from "../platform";
import { RELEASE_NOTES } from "../release-notes/release-notes";
import { LOCAL_CAPABILITIES, type WorkspaceSession } from "../session";
import { STALE_PROBE } from "./helpers/assetFixtures";
import { changeList, sharedBoardRequirement } from "./helpers/changeList";
import { REMOTE_KEY, fakeRemoteDs, fakeRemoteSession } from "./helpers/remoteFixtures";
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

// macOS 原生選單（desktop-native-menu）：攔下 installAppMenu，取得 App 每次安裝傳入的 deps
// （dispatch、t、hasProject）與取消函式的呼叫；選單模型本身由 appMenu.test 承載。
const { appMenuSpy, windowSpy } = vi.hoisted(() => ({
  appMenuSpy: {
    deps: [] as Array<Parameters<typeof import("../appMenu").installAppMenu>[0]>,
    cancel: vi.fn(),
  },
  windowSpy: { isFocused: vi.fn(), setFocus: vi.fn() },
}));
vi.mock("../appMenu", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../appMenu")>()),
  installAppMenu: (deps: (typeof appMenuSpy.deps)[number]) => {
    appMenuSpy.deps.push(deps);
    return appMenuSpy.cancel;
  },
}));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: vi.fn().mockResolvedValue(undefined) }));
// 選單「關閉專案」只在主視窗為焦點時動作：jsdom 無 Tauri 視窗，以可控的焦點狀態模擬。
vi.mock("@tauri-apps/api/window", async (importOriginal) => ({
  ...(await importOriginal<typeof import("@tauri-apps/api/window")>()),
  getCurrentWindow: () => windowSpy,
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
  appMenuSpy.deps.length = 0;
  appMenuSpy.cancel.mockClear();
  windowSpy.isFocused.mockReset().mockResolvedValue(true);
  // 各測試自行預置分頁持久化；先清掉避免跨測試洩漏。
  localStorage.removeItem("speclink.projectTabs");
  // 技能檔提示的「保留現狀」記憶也是 localStorage：清掉，免得一條測試按過
  // 保留現狀就讓後面期待提示出現的測試依執行順序逾時。
  localStorage.removeItem("speclink.instructionSkips");
  // 清單每頁筆數記憶（desktop-list-pages-reskin design D7）同樣跨測試洩漏。
  localStorage.removeItem("speclink.list.pageSizes");
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

/** 技能檔探測回報過期（3 個受管檔有異）的 workspace 面：store 帶 assetPrompt（kind stale、fileCount 3）。 */
function staleWorkspace() {
  return {
    ...fakeWorkspace(),
    openProject: vi.fn().mockResolvedValue({ status: "project", root: "A", name: "proj-a" }),
    probeAssets: vi.fn().mockResolvedValue({
      ...STALE_PROBE,
      differingFiles: [...STALE_PROBE.differingFiles, ".claude/skills/speclink-propose/SKILL.md"],
    }),
    updateAssets: vi.fn().mockResolvedValue(undefined),
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
        schemaName: "spec-driven",
        schemaKnown: true,
        parseError: null,
      },
    }),
    writeAppTools: vi.fn(),
    writeWorkflowConfig: vi.fn().mockResolvedValue(undefined),
    writeWorkflowContext: vi.fn(),
    writeWorkflowRules: vi.fn(),
    readSchemas: vi.fn().mockResolvedValue([]),
    writeWorkflowSchema: vi.fn().mockResolvedValue(undefined),
    forkSchema: vi.fn().mockResolvedValue("spec-driven-custom"),
    createSchema: vi.fn().mockResolvedValue(undefined),
    revealSchema: vi.fn().mockResolvedValue(undefined),
    deleteSchema: vi.fn().mockResolvedValue(undefined),
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
  over: {
    ws?: ReturnType<typeof fakeWorkspace>;
    settings?: ReturnType<typeof fakeSettings>;
    updater?: { check: Mock; relaunch: Mock };
  } = {},
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
  render(
    <App
      createSession={makeSession(ds, settings)}
      workspace={ws as never}
      updater={over.updater as never}
    />,
  );
  return { ds, ws, settings };
}

/** 殼（design D2）：專案欄是帶「專案導覽」標籤的 nav；設定入口是圖示列底部齒輪。 */
const projectColumn = () => screen.getByRole("navigation", { name: "專案導覽" });
const settingsGear = () => screen.getByRole("button", { name: "設定" });
/** 左側整欄（標題列＋圖示列＋專案欄）。 */
const leftSide = () => screen.getByTestId("left-titlebar").parentElement as HTMLElement;

const STATUS: StatusReport = {
  changeName: "desktop-shell-and-browser",
  schemaName: "spec-driven",
  isComplete: false,
  applyRequires: ["tasks"],
  artifacts: [],
};

function fakeDataSource(over: Partial<SpeclinkDataSource> = {}): SpeclinkDataSource {
  return {
    listChanges: vi.fn().mockResolvedValue(changeList([
      { name: "desktop-shell-and-browser", status: "in-progress", totalTasks: 30, completedTasks: 30 },
    ])),
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
    getStationTicket: vi.fn().mockResolvedValue(null),
    getArchivedStationTicket: vi.fn().mockResolvedValue(null),
    listDiscussions: vi.fn().mockResolvedValue({ active: [], archived: [] }),
    getDiscussionDocument: vi.fn().mockResolvedValue(null),
    promoteDiscussion: vi.fn().mockResolvedValue({ change: "promoted-change" }),
    archiveDiscussion: vi.fn().mockResolvedValue(undefined),
    reorderCard: vi.fn().mockResolvedValue(undefined),
    setDepends: vi.fn().mockResolvedValue(undefined),
    revertChangeToProposed: vi.fn().mockResolvedValue(undefined),
    listManualPages: vi
      .fn()
      .mockResolvedValue({ present: false, reason: null, pages: [], uncoveredNew: [], malformed: [] }),
    getManualPage: vi.fn().mockResolvedValue(null),
    ...over,
  };
}

/** 手冊索引 fixture（desktop-manual-page）：兩個分區、一頁 stale。 */
const MANUAL_INDEX = {
  present: true,
  reason: null,
  pages: [
    {
      slug: "index",
      title: "手冊",
      section: "開始使用",
      order: 10,
      keywords: [],
      sources: [],
      generated: "2026-09-01",
      stale: false,
    },
    {
      slug: "boards",
      title: "看板",
      section: "日常操作",
      order: 20,
      keywords: ["kanban"],
      sources: ["desktop-app"],
      generated: "2026-09-01",
      stale: true,
    },
  ],
  uncoveredNew: [],
  malformed: [],
};

describe("App (kanban primary + rich detail)", () => {
  it("根層掛載 Toaster，主區頂列不含操作結果文字節點", async () => {
    renderApp();
    await waitFor(() => expect(screen.getByTestId("app-toaster")).toBeTruthy());
    expect(toasterSpy).toHaveBeenCalledTimes(1);

    const bar = screen.getByTestId("main-titlebar");
    expect(bar.querySelector(".font-mono")).toBeNull();
  });

  // spec desktop-app「側欄導覽結構」：圖示列頂端是共用元件庫的品牌標記；殼上不再有橫式字標
  // （字標只在零專案空狀態）。
  it("圖示列頂端為共用 BrandMark，有專案時殼上無橫式字標", async () => {
    renderApp();
    await screen.findByText("desktop-shell-and-browser");
    const rail = document.querySelector("[data-project-rail]") as HTMLElement;
    expect(rail.firstElementChild?.getAttribute("aria-label")).toBe("Speclink");
    expect(document.querySelector('img[alt="Speclink"]')).toBeNull();
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

  it("清單 payload 帶 planError 時看板出現成環提示列（宿主自 payload 傳入）", async () => {
    // spec desktop-app「依賴成環時看板提示」Scenario「成環提示」。
    const ds = fakeDataSource({
      listChanges: vi.fn().mockResolvedValue(
        changeList(
          [{ name: "add-a", status: "in-progress", totalTasks: 3, completedTasks: 0 }],
          "dependency cycle: add-a -> add-b -> add-a",
        ),
      ),
    });
    renderApp(ds);
    await waitFor(() => screen.getByText("依賴成環：dependency cycle: add-a -> add-b -> add-a"));
    expect(screen.getByText("add-a")).toBeTruthy();
  });

  it("資料源提供 dependsCandidates 時，排程分頁的新增候選取自它（宿主接線）", async () => {
    // spec desktop-app Scenario「worktree 映射時候選來自副本名冊」的宿主面：看板清單有
    // add-auth 與 add-late，名冊查詢只回 add-auth——新增下拉只列 add-auth。
    const item = (name: string) => ({
      name,
      status: "in-progress",
      totalTasks: 3,
      completedTasks: 0,
      wave: 1,
      blockedBy: [],
      dependsOn: [],
      overlaps: [],
    });
    const dependsCandidates = vi.fn().mockResolvedValue(["add-auth"]);
    const ds = fakeDataSource({
      listChanges: vi
        .fn()
        .mockResolvedValue(changeList([item("add-dark-mode"), item("add-auth"), item("add-late")])),
      dependsCandidates,
    });
    renderApp(ds);
    fireEvent.click(await screen.findByText("add-dark-mode"));
    fireEvent.mouseDown(await screen.findByRole("tab", { name: /排程/ }));
    await waitFor(() => expect(dependsCandidates).toHaveBeenCalledWith("add-dark-mode"));
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await user.click(await screen.findByRole("combobox", { name: "新增前置" }));
    const options = (await screen.findAllByRole("option")).map((o) => o.textContent);
    expect(options).toEqual(["add-auth"]);
  });

  it("local 清單項帶齊排程欄位時，排程分頁的重疊卡與封存順序卡顯示其內容", async () => {
    // spec client-protocol「變更清單的排程欄位」的宿主面：list_changes payload 原樣進抽屜——
    // add-b 與 add-c 同動「看板與任務」，add-b 排在 add-c 之後封存。
    const item = (name: string, other: string, archiveAfter: string[]) => ({
      name,
      status: "in-progress",
      totalTasks: 3,
      completedTasks: 0,
      wave: 1,
      blockedBy: [],
      dependsOn: [],
      overlaps: [{ change: other, capabilities: ["desktop-app"] }],
      requirementOverlap: [sharedBoardRequirement(other)],
      archiveAfter,
    });
    const ds = fakeDataSource({
      listChanges: vi.fn().mockResolvedValue(changeList([item("add-c", "add-b", []), item("add-b", "add-c", ["add-c"])])),
    });
    renderApp(ds);
    fireEvent.click(await screen.findByText("add-b"));
    fireEvent.mouseDown(await screen.findByRole("tab", { name: /排程/ }));
    const section = (name: string) => document.querySelector(`[data-plan-section="${name}"]`) as HTMLElement;
    await waitFor(() => expect(section("archive")).toBeTruthy());
    const overlaps = section("overlaps");
    expect(within(overlaps).getByText("add-c")).toBeTruthy();
    expect(within(overlaps).getByText("desktop-app › 看板與任務")).toBeTruthy();
    expect(within(overlaps).getAllByText("修改")).toHaveLength(2);
    const archive = section("archive");
    expect(within(archive).getByText("add-c")).toBeTruthy();
    expect(within(archive).getByText("先封存它們，再對照正式規格重寫同名 requirement 後封存本變更")).toBeTruthy();
    // design Implementation Contract 行為 2：第 1 波、可以開工、尚無前置與新增下拉。
    expect(within(section("wave")).getByText("第 1 波")).toBeTruthy();
    const depends = section("depends");
    expect(within(depends).getByText("可以開工")).toBeTruthy();
    expect(within(depends).getByText("尚無前置")).toBeTruthy();
    expect(within(depends).getByRole("combobox", { name: "新增前置" })).toBeTruthy();
  });

  it("delete flow: drawer delete → confirm dialog → deleteChange called", async () => {
    // 階段守門後刪除鈕僅提案中可按（archive-readiness-gating）——改用提案中 fixture。
    const ds = fakeDataSource({
      listChanges: vi.fn().mockResolvedValue(changeList([
        { name: "desktop-shell-and-browser", status: "in-progress", totalTasks: 30, completedTasks: 0 },
      ])),
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
      listChanges: vi.fn().mockResolvedValue(changeList([
        { name: "oops-started", status: "in-progress", totalTasks: 10, completedTasks: 0, startedAt: "2026-07-30" },
      ])),
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
      listChanges: vi.fn().mockResolvedValue(changeList([
        { name: "oops-started", status: "in-progress", totalTasks: 10, completedTasks: 3, startedAt: "2026-07-30" },
      ])),
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
    const ds = fakeDataSource({ listChanges: vi.fn().mockResolvedValue(changeList([])) });
    render(<App createSession={makeSession(ds)} workspace={ws as never} />);
    expect(await screen.findByText("開啟一個專案開始")).toBeTruthy();
    // 空狀態與圖示列「＋」皆匯流至新增專案對話框，選本機卡後按「選擇資料夾…」。
    const openButtons = screen.getAllByText("新增專案");
    fireEvent.click(openButtons[openButtons.length - 1]);
    const chooser = await screen.findByRole("alertdialog");
    fireEvent.click(within(chooser).getByRole("button", { name: /本機資料夾/ }));
    fireEvent.click(within(chooser).getByRole("button", { name: "選擇資料夾…" }));
    await waitFor(() => expect(ws.pickFolder).toHaveBeenCalled());
  });

  it("零分頁空狀態「連線 Server」開對話框並直接呈現 Server 步驟（步驟條第 2 步）", async () => {
    const ws = fakeWorkspace();
    render(<App createSession={makeSession(fakeDataSource())} workspace={ws as never} />);
    fireEvent.click(await screen.findByRole("button", { name: "連線 Server" }));
    const chooser = await screen.findByRole("alertdialog");
    expect(within(chooser).getByText("選擇 Server")).toBeTruthy();
    expect(within(chooser).getByText("步驟 2 / 4")).toBeTruthy();
  });

  // spec「表單控制項與按鈕以主題化元件呈現」Scenario「初始化對話框工具多選主題化」
  it("初始化對話框的工具多選為主題化 checkbox：預設勾 claude、可獨立切換", async () => {
    const ws = fakeWorkspace();
    ws.pickFolder = vi.fn().mockResolvedValue("D:/newproj");
    ws.openProject = vi.fn().mockResolvedValue({ status: "uninitialized", dir: "D:/newproj" });
    const ds = fakeDataSource({ listChanges: vi.fn().mockResolvedValue(changeList([])) });
    render(<App createSession={makeSession(ds)} workspace={ws as never} />);
    const openButtons = await screen.findAllByText("新增專案");
    fireEvent.click(openButtons[openButtons.length - 1]);
    const chooser = await screen.findByRole("alertdialog");
    fireEvent.click(within(chooser).getByRole("button", { name: /本機資料夾/ }));
    fireEvent.click(within(chooser).getByRole("button", { name: "選擇資料夾…" }));
    // chooser 關閉有離場動畫；等待初始化框的專屬控制項，避免抓到尚未卸載的 chooser。
    const claude = await screen.findByRole("checkbox", { name: "claude" });
    const dialog = claude.closest('[role="alertdialog"]') as HTMLElement;
    expect(dialog).toBeTruthy();
    const codex = within(dialog).getByRole("checkbox", { name: "codex" });
    // spec「未初始化目錄經確認後自動初始化」：多選含 copilot，預設只勾 claude。
    const copilot = within(dialog).getByRole("checkbox", { name: "copilot" });
    // 主題化原語（button 元素）而非原生 input。
    expect(claude.tagName).not.toBe("INPUT");
    // 預設勾選狀態與替換前相同：claude 勾、codex 未勾。
    expect(claude.getAttribute("aria-checked")).toBe("true");
    expect(codex.getAttribute("aria-checked")).toBe("false");
    expect(copilot.getAttribute("aria-checked")).toBe("false");
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
    const ds = fakeDataSource({ listChanges: vi.fn().mockResolvedValue(changeList([])) });
    render(<App createSession={makeSession(ds)} workspace={ws as never} />);
    const openButtons = await screen.findAllByText("新增專案");
    fireEvent.click(openButtons[openButtons.length - 1]);
    const chooser = await screen.findByRole("alertdialog");
    fireEvent.click(within(chooser).getByRole("button", { name: /本機資料夾/ }));
    fireEvent.click(within(chooser).getByRole("button", { name: "選擇資料夾…" }));
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
    // 多選含 copilot、預設未勾。
    const copilot = within(dialog).getByRole("checkbox", { name: "copilot" });
    expect(copilot.getAttribute("aria-checked")).toBe("false");
    // 確認 → 以所選工具呼叫 adopt（而非 init）。
    fireEvent.click(within(dialog).getByRole("button", { name: "啟用" }));
    await waitFor(() => expect(ws.adoptProject).toHaveBeenCalledWith("D:/migrated", ["claude"]));
    expect(ws.initProject).not.toHaveBeenCalled();
  });

  it("啟用確認對話框：取消關框且零寫入呼叫", async () => {
    const ws = fakeWorkspace();
    ws.pickFolder = vi.fn().mockResolvedValue("D:/migrated");
    ws.openProject = vi.fn().mockResolvedValue({ status: "unadopted", root: "D:/migrated" });
    const ds = fakeDataSource({ listChanges: vi.fn().mockResolvedValue(changeList([])) });
    render(<App createSession={makeSession(ds)} workspace={ws as never} />);
    const openButtons = await screen.findAllByText("新增專案");
    fireEvent.click(openButtons[openButtons.length - 1]);
    const chooser = await screen.findByRole("alertdialog");
    fireEvent.click(within(chooser).getByRole("button", { name: /本機資料夾/ }));
    fireEvent.click(within(chooser).getByRole("button", { name: "選擇資料夾…" }));
    const claude = await screen.findByRole("checkbox", { name: "claude" });
    const dialog = claude.closest('[role="alertdialog"]') as HTMLElement;
    fireEvent.click(within(dialog).getByRole("button", { name: "取消" }));
    await waitFor(() => expect(screen.queryByRole("checkbox", { name: "claude" })).toBeNull());
    expect(ws.adoptProject).not.toHaveBeenCalled();
    expect(ws.initProject).not.toHaveBeenCalled();
  });

  it("圖示列方塊：點背景方塊切換後作用中標示、標題列專案名與麵包屑同步更新", async () => {
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
    const squareOf = (key: string) =>
      Array.from(document.querySelectorAll<HTMLElement>("[data-tab]")).find(
        (el) => el.getAttribute("data-tab") === key,
      ) as HTMLElement;
    await waitFor(() => expect(squareOf("local:A").getAttribute("data-active")).toBe("true"));
    expect(within(screen.getByTestId("left-titlebar")).getByText("proj-a")).toBeTruthy();
    fireEvent.click(squareOf("local:B"));
    await waitFor(() => expect(squareOf("local:B").getAttribute("data-active")).toBe("true"));
    expect(within(screen.getByTestId("left-titlebar")).getByText("proj-b")).toBeTruthy();
    expect(screen.getByTestId("main-titlebar").textContent).toBe("proj-b/變更");
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
    fireEvent.click(await screen.findByRole("button", { name: "設定" }));
    fireEvent.mouseDown(await screen.findByRole("tab", { name: "本機設定" }));
    const group = await screen.findByTestId("ui-locale");
    fireEvent.click(within(group).getByText("English"));
    // 即時全介面生效：標題列、齒輪與專案欄改為英文。
    expect((await screen.findAllByText("Settings")).length).toBeGreaterThan(0);
    expect(screen.getByRole("button", { name: "Settings" })).toBeTruthy();
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
    expect(settingsGear()).toBeTruthy();
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
  it("看板頁以頁標題區開頭：標題「變更」與說明，搜尋輸入在同一列", async () => {
    renderApp();
    await screen.findByText("desktop-shell-and-browser");
    const heading = screen.getByRole("heading", { level: 2, name: "變更" });
    const header = heading.closest("[data-page-header]") as HTMLElement;
    expect(header.textContent).toContain("依生命週期分欄；拖曳卡片調整順序，點卡片開詳情。");
    expect(within(header).getByPlaceholderText("搜尋看板卡片…")).toBeTruthy();
  });

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
  it("專案欄依序為變更/已封存/規格/手冊與底部專案設定，不含設定與備忘；設定在圖示列底部", async () => {
    renderApp();
    await waitFor(() => screen.getByText("desktop-shell-and-browser"));
    const aside = projectColumn();
    // 已封存項以 aria-label 為無障礙名稱（徽章數字不污染），其餘取文字內容。
    const labels = within(aside)
      .getAllByRole("button")
      .map((b) => b.getAttribute("aria-label") ?? b.textContent ?? "");
    expect(labels).toEqual(["變更", "已封存", "規格", "手冊", "專案設定"]);
    expect(leftSide().className).toContain("w-[256px]");
    expect(screen.queryByText("備忘")).toBeNull();
    const rail = document.querySelector("[data-project-rail]") as HTMLElement;
    expect(rail.contains(settingsGear())).toBe(true);
    // 齒輪是圖示列最後一個按鈕（沉底的一組：更新鈕＋齒輪，更新鈕無事時不渲染）。
    const railButtons = within(rail).getAllByRole("button");
    expect(railButtons[railButtons.length - 1]).toBe(settingsGear());
    expect(rail.lastElementChild?.contains(settingsGear())).toBe(true);
    // 左側標題列：作用中專案名與「⋯」專案動作鈕。
    const titleBar = screen.getByTestId("left-titlebar");
    expect(within(titleBar).getByText("proj-a").getAttribute("title")).toBe("A");
    expect(within(titleBar).getByRole("button", { name: "專案動作" })).toBeTruthy();
  });

  it("專案設定沉底（自動上邊距）；點齒輪進設定頁：左側標題列為「設定」、齒輪作用中、專案欄無作用中項", async () => {
    renderApp();
    await screen.findByText("desktop-shell-and-browser");
    const aside = projectColumn();
    const projectSettingsNav = within(aside).getByRole("button", { name: "專案設定" });
    // 彈性區隔：jsdom 無版面計算，以等效自動上邊距 class 斷言——沉底的是「提示卡＋專案設定」
    // 這一組（desktop-notice-relocation：技能檔提示卡落在彈性空白與專案設定之間）。
    expect(projectSettingsNav.parentElement?.className).toContain("mt-auto");
    const changesNav = within(aside).getByRole("button", { name: "變更" });
    expect(changesNav.className).toContain("bg-primary");

    fireEvent.click(settingsGear());
    await waitFor(() => expect(document.querySelector('[data-column="ready"]')).toBeNull());
    expect(settingsGear().className).toContain("bg-primary");
    for (const item of within(projectColumn()).getAllByRole("button")) {
      expect(item.className).not.toContain("bg-primary");
    }
    const titleBar = screen.getByTestId("left-titlebar");
    expect(titleBar.textContent).toBe("設定");
    expect(within(titleBar).queryByRole("button", { name: "專案動作" })).toBeNull();
    expect(screen.getByTestId("main-titlebar").textContent).toBe("設定");
  });

  it("標題列「⋯」開專案動作選單（與方塊右鍵同一份）；選「關閉專案」關掉作用中專案", async () => {
    renderApp();
    await screen.findByText("desktop-shell-and-browser");
    // Radix 選單開著時 body 為 pointer-events: none（內容層照常可點）。
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await user.click(
      within(screen.getByTestId("left-titlebar")).getByRole("button", { name: "專案動作" }),
    );
    const items = screen.getAllByRole("menuitem").map((i) => i.querySelector("span")?.textContent);
    expect(items).toEqual([
      "在檔案管理員顯示",
      "在終端機開啟",
      "以編輯器開啟",
      "複製路徑",
      "重新整理",
      "關閉專案",
    ]);
    await user.click(screen.getByRole("menuitem", { name: /^關閉專案/ }));
    expect(await screen.findByText("開啟一個專案開始")).toBeTruthy();
    expect(screen.queryByRole("navigation", { name: "專案導覽" })).toBeNull();
  });

  it("背景方塊右鍵選「重新整理」：切到該專案（切換即重新載入），不是重載作用中專案", async () => {
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
    const squareOf = (key: string) =>
      Array.from(document.querySelectorAll<HTMLElement>("[data-tab]")).find(
        (el) => el.getAttribute("data-tab") === key,
      ) as HTMLElement;
    await waitFor(() => expect(squareOf("local:A").getAttribute("data-active")).toBe("true"));
    fireEvent.contextMenu(squareOf("local:B"));
    fireEvent.click(await screen.findByRole("menuitem", { name: /^重新整理/ }));
    await waitFor(() => expect(squareOf("local:B").getAttribute("data-active")).toBe("true"));
  });

  it("Ctrl+R 重新整理作用中專案、Ctrl+W 關閉作用中專案（Windows／Linux 的選單快捷鍵）", async () => {
    const { ds } = renderApp();
    await screen.findByText("desktop-shell-and-browser");
    const before = vi.mocked(ds.listChanges).mock.calls.length;
    fireEvent.keyDown(window, { key: "r", ctrlKey: true });
    await waitFor(() => expect(vi.mocked(ds.listChanges).mock.calls.length).toBeGreaterThan(before));
    fireEvent.keyDown(window, { key: "w", ctrlKey: true });
    expect(await screen.findByText("開啟一個專案開始")).toBeTruthy();
  });

  it("macOS：⌘R 重新整理；⌘W 不在網頁層接（交給原生選單），專案仍開著", async () => {
    const ua = vi
      .spyOn(navigator, "userAgent", "get")
      .mockReturnValue("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15");
    try {
      const { ds } = renderApp();
      await screen.findByText("desktop-shell-and-browser");
      const before = vi.mocked(ds.listChanges).mock.calls.length;
      fireEvent.keyDown(window, { key: "r", metaKey: true });
      await waitFor(() =>
        expect(vi.mocked(ds.listChanges).mock.calls.length).toBeGreaterThan(before),
      );
      fireEvent.keyDown(window, { key: "w", metaKey: true });
      fireEvent.keyDown(window, { key: "w", ctrlKey: true });
      expect(screen.queryByText("開啟一個專案開始")).toBeNull();
      expect(screen.getByTestId("left-titlebar").textContent).toContain("proj-a");
    } finally {
      ua.mockRestore();
    }
  });

  it("有新版本時視窗頂端無更新橫幅：主區頂列之下直接是主內容（spec「視窗頂端 SHALL NOT 出現更新橫幅」）", async () => {
    const ws = fakeWorkspace();
    ws.openProject = vi.fn().mockResolvedValue({ status: "project", root: "A", name: "proj-a" });
    render(
      <App
        createSession={makeSession(fakeDataSource())}
        workspace={ws as never}
        updater={{
          check: vi.fn().mockResolvedValue({ version: "9.9.9", download: vi.fn(), install: vi.fn() }),
          relaunch: vi.fn(),
        }}
      />,
    );
    await screen.findByRole("button", { name: /有新版本 9\.9\.9/ });
    const titleBar = screen.getByTestId("main-titlebar");
    expect(titleBar.parentElement?.firstElementChild).toBe(titleBar);
    expect(titleBar.nextElementSibling?.tagName).toBe("MAIN");
    expect(screen.queryByRole("status")).toBeNull();
  });

  it("有新版本時圖示列齒輪上方出現更新鈕（提示帶版號）；點擊進設定頁、齒輪作用中（spec「桌面自動更新」）", async () => {
    const ws = fakeWorkspace();
    ws.openProject = vi.fn().mockResolvedValue({ status: "project", root: "A", name: "proj-a" });
    render(
      <App
        createSession={makeSession(fakeDataSource())}
        workspace={ws as never}
        updater={{
          check: vi.fn().mockResolvedValue({ version: "9.9.9", download: vi.fn(), install: vi.fn() }),
          relaunch: vi.fn(),
        }}
      />,
    );
    const button = await screen.findByRole("button", { name: /有新版本 9\.9\.9/ });
    const rail = document.querySelector("[data-project-rail]") as HTMLElement;
    expect(rail.contains(button)).toBe(true);
    // 齒輪上方：更新鈕在文件順序上先於齒輪。
    expect(button.compareDocumentPosition(settingsGear()) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    fireEvent.click(button);
    await waitFor(() => expect(screen.getByTestId("updater-card")).toBeTruthy());
    expect(settingsGear().className).toContain("bg-primary");
  });

  it("麵包屑隨頁面切換，點專案名回看板", async () => {
    renderApp();
    await screen.findByText("desktop-shell-and-browser");
    const crumbs = screen.getByTestId("main-titlebar");
    expect(crumbs.textContent).toBe("proj-a/變更");
    fireEvent.click(within(projectColumn()).getByRole("button", { name: "規格" }));
    await waitFor(() => expect(crumbs.textContent).toBe("proj-a/規格"));
    fireEvent.click(within(crumbs).getByRole("button", { name: "proj-a" }));
    await waitFor(() => expect(document.querySelector('[data-column="ready"]')).toBeTruthy());
    expect(crumbs.textContent).toBe("proj-a/變更");
    fireEvent.click(within(projectColumn()).getByRole("button", { name: "手冊" }));
    await waitFor(() => expect(crumbs.textContent).toBe("proj-a/手冊"));
  });

  it("麵包屑隨專案切換：A / 規格 → B / 規格（切專案保留所在頁）", async () => {
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
    const crumbs = await screen.findByTestId("main-titlebar");
    await waitFor(() => expect(crumbs.textContent).toBe("proj-a/變更"));
    fireEvent.click(within(projectColumn()).getByRole("button", { name: "規格" }));
    await waitFor(() => expect(crumbs.textContent).toBe("proj-a/規格"));
    const squareB = Array.from(document.querySelectorAll<HTMLElement>("[data-tab]")).find(
      (el) => el.getAttribute("data-tab") === "local:B",
    ) as HTMLElement;
    fireEvent.click(squareB);
    await waitFor(() => expect(crumbs.textContent).toBe("proj-b/規格"));
  });

  it("點專案設定切至專案設定頁並轉移高亮", async () => {
    renderApp();
    await screen.findByText("desktop-shell-and-browser");
    const aside = projectColumn();
    const projectSettingsNav = within(aside).getByRole("button", { name: "專案設定" });
    const changesNav = within(aside).getByRole("button", { name: "變更" });

    fireEvent.click(projectSettingsNav);

    expect(await screen.findByRole("tab", { name: "config.yaml" })).toBeTruthy();
    expect(document.querySelector('[data-column="ready"]')).toBeNull();
    expect(projectSettingsNav.className).toContain("bg-primary");
    expect(changesNav.className).not.toContain("bg-primary");
  });

  it("零分頁：左側只有圖示列（無專案欄、無專案設定）；齒輪仍進入應用程式設定頁", async () => {
    const ws = fakeWorkspace();
    render(<App createSession={makeSession(fakeDataSource())} workspace={ws as never} />);
    expect(await screen.findByText("開啟一個專案開始")).toBeTruthy();
    expect(screen.queryByRole("navigation", { name: "專案導覽" })).toBeNull();
    expect(screen.queryByRole("button", { name: "專案設定" })).toBeNull();
    // 左側只剩 56px 圖示列（macOS 標題列的紅綠燈留白不得把它撐寬）。
    expect(leftSide().className).toContain("w-14");
    expect(screen.getByTestId("left-titlebar").textContent).toBe("");
    expect(screen.getByTestId("main-titlebar").textContent).toBe("");

    fireEvent.click(settingsGear());
    expect(await screen.findByRole("tab", { name: "本機設定" })).toBeTruthy();
    expect(screen.queryByText("開啟一個專案開始")).toBeNull();
    expect(settingsGear().className).toContain("bg-primary");
    // 56px 圖示列放不下標題文字（macOS 紅綠燈也佔住這段）：左側留白，「設定」由主區麵包屑呈現。
    expect(screen.getByTestId("left-titlebar").textContent).toBe("");
    expect(screen.getByTestId("main-titlebar").textContent).toBe("設定");
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
    const aside = projectColumn();
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
    const aside = projectColumn();
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
    const aside = projectColumn();
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
    // 主內容渲染 SpecList（正式規格卡片＋搜尋列），返回看板點「變更」。
    renderApp();
    await waitFor(() => screen.getByText("desktop-shell-and-browser"));
    const aside = projectColumn();
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

  it("規格頁改每頁 50 後重開 app 仍為每頁 50，已封存變更節仍為 20（desktop-list-pages-reskin design D7）", async () => {
    // spec scenario「每頁筆數跨啟動記住」：記憶存在 app 本機，重建 App（新 store）後讀回。
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    renderApp();
    await waitFor(() => screen.getByText("desktop-shell-and-browser"));
    fireEvent.click(within(projectColumn()).getByRole("button", { name: "規格" }));
    await waitFor(() => expect(screen.getByText("desktop-app")).toBeTruthy());
    await user.click(screen.getByRole("combobox", { name: "每頁 20 個" }));
    await user.click(await screen.findByRole("option", { name: "每頁 50 個" }));
    expect(screen.getByRole("combobox", { name: "每頁 50 個" })).toBeTruthy();
    expect(JSON.parse(localStorage.getItem("speclink.list.pageSizes")!).specs).toBe(50);
    // 重開 app：卸載後以新 store 重建（給一筆封存變更，已封存頁才有工具列）。
    cleanup();
    renderApp(
      fakeDataSource({
        listArchived: vi.fn().mockResolvedValue([
          { datedName: "2026-07-04-old-change", date: "2026-07-04", name: "old-change" },
        ]),
      }),
    );
    await waitFor(() => screen.getByText("desktop-shell-and-browser"));
    fireEvent.click(within(projectColumn()).getByRole("button", { name: "規格" }));
    await waitFor(() => expect(screen.getByText("desktop-app")).toBeTruthy());
    expect(screen.getByRole("combobox", { name: "每頁 50 個" })).toBeTruthy();
    fireEvent.click(within(projectColumn()).getByRole("button", { name: "已封存" }));
    await waitFor(() => expect(screen.getByText("old-change")).toBeTruthy());
    expect(screen.getByRole("combobox", { name: "每頁 20 個" })).toBeTruthy();
  });

  it("切到規格頁與已封存頁各有對應頁標題 h2 與全圓搜尋框，列表卡填滿主區（desktop-list-pages-reskin）", async () => {
    // spec「規格頁提供清單、搜尋與展開檢視」「已封存頁含討論節」：App 傳 title／description，
    // 頁標題區在列表卡之上；已封存頁的列表卡以卡片標頭式分頁接在頂部。
    renderApp();
    await waitFor(() => screen.getByText("desktop-shell-and-browser"));
    const aside = projectColumn();
    fireEvent.click(within(aside).getByRole("button", { name: "規格" }));
    await waitFor(() => expect(screen.getByText("desktop-app")).toBeTruthy());
    const specsHeader = document.querySelector("main [data-page-header]") as HTMLElement;
    expect(within(specsHeader).getByRole("heading", { level: 2 }).textContent).toBe("規格");
    expect(within(specsHeader).getByPlaceholderText("搜尋規格…").className).toContain("rounded-full");
    expect(document.querySelector("main [data-list-card]")).toBeTruthy();
    expect(document.querySelector("main")!.className).toContain("p-5");
    fireEvent.click(within(aside).getByRole("button", { name: "已封存" }));
    await waitFor(() => expect(screen.getByText("已封存的變更")).toBeTruthy());
    const archivedHeader = document.querySelector("main [data-page-header]") as HTMLElement;
    expect(within(archivedHeader).getByRole("heading", { level: 2 }).textContent).toBe("已封存");
    expect(within(archivedHeader).getByPlaceholderText("搜尋已封存的變更與討論…").className).toContain("rounded-full");
    expect(document.querySelector("main [data-list-card]")!.className).toContain("rounded-t-none");
    expect(document.querySelector("main")!.className).toContain("p-5");
  });

  it("規格頁點卡開唯讀規格抽屜，經 dataSource.getSpecDocument 載入正式規格全文", async () => {
    // spec Scenario「選定 spec 以抽屜顯示其正式規格內容」：App 掛載 SpecDrawer 並接線
    // store.detailSpec 與 dataSource.getSpecDocument（spec-archive-drawer design D2）。
    const ds = fakeDataSource({
      getSpecDocument: vi.fn().mockResolvedValue("# desktop-app Specification\n\n正典內文段落。"),
    });
    renderApp(ds);
    await waitFor(() => screen.getByText("desktop-shell-and-browser"));
    const aside = projectColumn();
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
    const aside = projectColumn();
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
    const aside = projectColumn();
    fireEvent.click(within(aside).getByRole("button", { name: /已封存/ }));
    await waitFor(() => screen.getByText("old"));
    fireEvent.click(screen.getByText("old"));
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    // chip 以 slug 直出（change-drawer-header-redesign；topic 降為提示）。
    fireEvent.click(screen.getByRole("button", { name: /old-topic/ }));
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
    const aside = projectColumn();
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
    const aside = projectColumn();
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
    fireEvent.click(settingsGear());
    await waitFor(() => expect(main().className).toContain("overflow-y-auto"));
    expect(main().className).not.toContain("overflow-hidden");
  });

  // spec「指令檔過期提示」「指令檔過期提示捲動釘選」（desktop-notice-relocation）：提示卡住在
  // 專案欄、不屬主區任何捲動容器；跨頁常駐；主區無舊橫幅、無 sticky 包裹層、捲動約束不因提示
  // 改變。jsdom 無版面計算，這裡釘的是 DOM 歸屬與 class 契約。
  it("技能檔提示卡渲染在專案欄內、主區無舊橫幅與釘選包裹層；切規格、已封存與專案設定頁仍在", async () => {
    const ws = staleWorkspace();
    renderApp(fakeDataSource(), { ws });
    await screen.findByText("desktop-shell-and-browser");
    const card = await screen.findByTestId("asset-notice-card");
    const main = () => document.querySelector("main") as HTMLElement;
    const aside = projectColumn();
    expect(aside.contains(card)).toBe(true);
    expect(card.textContent).toContain("3 個檔案");
    // 主區：舊橫幅退場、無釘選包裹層、捲動約束照舊。
    expect(main().querySelector('[data-testid="asset-notice-card"]')).toBeNull();
    expect(main().querySelector(".sticky")).toBeNull();
    expect(main().classList.contains("overflow-hidden")).toBe(true);
    // Scenario「提示卡跨頁常駐」：規格、手冊、已封存、專案設定依序切（手冊頁是舊實作有特例的頁）。
    const crumbs = screen.getByTestId("main-titlebar");
    for (const label of ["規格", "手冊", "已封存"] as const) {
      fireEvent.click(within(aside).getByRole("button", { name: label }));
      await waitFor(() => expect(crumbs.textContent).toBe(`proj-a/${label}`));
      expect(projectColumn().contains(screen.getByTestId("asset-notice-card"))).toBe(true);
      expect(main().classList.contains("overflow-hidden")).toBe(true);
    }
    fireEvent.click(within(aside).getByRole("button", { name: "專案設定" }));
    await waitFor(() => expect(main().className).toContain("overflow-y-auto"));
    expect(projectColumn().contains(screen.getByTestId("asset-notice-card"))).toBe(true);
    expect(main().querySelector(".sticky")).toBeNull();
  });

  it("點提示卡開確認框；開啟變更抽屜時確認框關閉（可取消浮層互斥）", async () => {
    renderApp(fakeDataSource(), { ws: staleWorkspace() });
    await screen.findByText("desktop-shell-and-browser");
    fireEvent.click(await screen.findByTestId("asset-notice-card"));
    const dialog = await screen.findByTestId("asset-notice-dialog");
    expect(within(dialog).getByRole("button", { name: "更新技能檔" })).toBeTruthy();
    // 看板卡片走 store openDetail；fireEvent 不送 pointerdown，不會誤走對話框的點外關閉。
    fireEvent.click(screen.getByText("desktop-shell-and-browser"));
    await waitFor(() => expect(screen.queryByTestId("asset-notice-dialog")).toBeNull());
    expect(drawerSpy.rich[drawerSpy.rich.length - 1].open).toBe(true);
    // 被抽屜頂掉等同「稍後」：提示卡仍在（抽屜開啟時 Radix 把其餘區域標為 aria-hidden，
    // 角色查詢找不到專案欄，改以 testid 斷言存在）。
    expect(screen.getByTestId("asset-notice-card")).toBeTruthy();
  });

  it("remote 分頁不探測技能檔、專案欄無提示卡", async () => {
    localStorage.setItem(
      "speclink.projectTabs",
      JSON.stringify({
        version: 2,
        tabs: [
          {
            locator: { kind: "remote", connectionId: "c1", projectId: "demo", repoId: "backend" },
            name: "Demo/backend",
          },
        ],
        activeKey: REMOTE_KEY,
      }),
    );
    const ws = staleWorkspace();
    const ds = fakeRemoteDs();
    render(
      <App
        createSession={() => {
          throw new Error("remote 流程不應觸發 local 工廠");
        }}
        openRemote={vi.fn(async () => fakeRemoteSession(ds))}
        workspace={ws as never}
      />,
    );
    await screen.findByText("remote-change");
    expect(projectColumn()).toBeTruthy();
    expect(ws.probeAssets).not.toHaveBeenCalled();
    expect(screen.queryByTestId("asset-notice-card")).toBeNull();
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
    const aside = (await screen.findByTestId("left-titlebar")).parentElement as HTMLElement;
    await waitFor(() => expect(settingsGear()).toBeTruthy());
    expect(aside.textContent).not.toContain("v0.1.0");

    // 設定頁軟體更新卡仍顯示目前版本。
    fireEvent.click(settingsGear());
    await waitFor(() => expect(screen.getByText(/目前版本\s*0\.1\.0/)).toBeTruthy());
  });

  it("未注入 updater 面時同樣無版號文字", async () => {
    renderApp();
    await waitFor(() => expect(leftSide()).toBeTruthy());
    expect(screen.queryByText(/^v\d/)).toBeNull();
  });
});

// 前景重檢（desktop-app「回到前景逾一小時即重檢」「回到前景未滿一小時不重檢」；
// design D3）：App 於啟動 effect 訂閱主視窗焦點，只對 focused=true 反應、經 1 小時
// 節流；卸載取消訂閱。焦點事件經 updater adapter 的可選方法注入，這裡以假 adapter
// 捕獲 handler 手動觸發。
describe("主視窗回到前景時重檢更新", () => {
  const T0 = Date.UTC(2026, 0, 1, 9, 0, 0);
  const HOUR = 60 * 60 * 1000;

  afterEach(() => {
    vi.useRealTimers();
  });

  function focusAdapter() {
    let handler: ((focused: boolean) => void) | null = null;
    const unlisten = vi.fn();
    const adapter = {
      check: vi.fn().mockResolvedValue(null),
      relaunch: vi.fn(),
      onFocusChanged: vi.fn((h: (focused: boolean) => void) => {
        handler = h;
        return Promise.resolve(unlisten);
      }),
    };
    return { adapter, unlisten, focus: (focused: boolean) => act(() => handler?.(focused)) };
  }

  function renderWith(adapter: object) {
    const ws = fakeWorkspace();
    ws.openProject = vi.fn().mockResolvedValue({ status: "project", root: "A", name: "proj-a" });
    return render(
      <App
        createSession={makeSession(fakeDataSource())}
        workspace={ws as never}
        updater={adapter as never}
      />,
    );
  }

  it("啟動檢查後：focused=false 不查、未滿 1 小時的 focused=true 不查、滿 1 小時才再查一次；卸載取消訂閱", async () => {
    // 只假 Date：setSystemTime 控制節流判定，setTimeout 等維持真實以免卡住 waitFor。
    vi.useFakeTimers({ toFake: ["Date"] });
    vi.setSystemTime(T0);
    const { adapter, unlisten, focus } = focusAdapter();
    const { unmount } = renderWith(adapter);

    await waitFor(() => expect(adapter.check).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(adapter.onFocusChanged).toHaveBeenCalledTimes(1));

    vi.setSystemTime(T0 + 30 * 60 * 1000);
    await focus(true);
    expect(adapter.check).toHaveBeenCalledTimes(1);

    // 滿 1 小時後先送失焦：節流已放行，仍不可查——證明只對取得焦點反應。
    vi.setSystemTime(T0 + HOUR);
    await focus(false);
    expect(adapter.check).toHaveBeenCalledTimes(1);

    await focus(true);
    await waitFor(() => expect(adapter.check).toHaveBeenCalledTimes(2));

    unmount();
    await waitFor(() => expect(unlisten).toHaveBeenCalledTimes(1));
  });

  it("假 adapter 未提供 onFocusChanged：啟動檢查照常一次、不訂閱、無錯誤", async () => {
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
    try {
      const adapter = { check: vi.fn().mockResolvedValue(null), relaunch: vi.fn() };
      renderWith(adapter);
      await waitFor(() => expect(adapter.check).toHaveBeenCalledTimes(1));
      await waitFor(() => expect(leftSide()).toBeTruthy());
      expect(adapter.check).toHaveBeenCalledTimes(1);
      expect(consoleError).not.toHaveBeenCalled();
    } finally {
      consoleError.mockRestore();
    }
  });
});

// 看板骨架只在「活躍 workspace 正在載入」時出現。持久化分頁探測失敗時
// activeKey 恆 null、loaded 恆 false，整批載入根本不會發生——若只看 loaded，
// 看板會永久停在骨架卡，比改動前可辨識的空看板更糟。
describe("看板骨架的終止條件", () => {
  it("有分頁但探測失敗（無活躍 workspace）→ 不出骨架卡", async () => {
    const ws = fakeWorkspace();
    ws.openProject = vi.fn().mockRejectedValue("目錄已不存在");
    renderApp(fakeDataSource(), { ws });
    await waitFor(() => expect(ws.openProject).toHaveBeenCalled());
    await waitFor(() => expect(document.querySelector('[aria-busy="true"]')).toBeNull());
  });
});

// 看板失敗終態的接線閘門 loadFailed={!s.loaded && s.loadFailed}：首訪失敗顯示
// 提示；已有真值的重載失敗維持舊快照靜默呈現，不得誤掛提示。
describe("看板首訪失敗終態的接線", () => {
  it("首訪整批載入失敗 → 骨架收掉，卡片區顯示載入失敗提示", async () => {
    const ds = fakeDataSource({
      listChanges: vi.fn().mockRejectedValue(new Error("offline")),
    });
    renderApp(ds);
    await waitFor(() =>
      expect(document.querySelectorAll('[data-testid="column-load-failed"]').length).toBeGreaterThan(0),
    );
    expect(document.querySelector('[aria-busy="true"]')).toBeNull();
    expect(screen.queryByText("尚無討論")).toBeNull();
  });

  it("已有舊快取的重載失敗 → 照常顯示舊資料，無失敗提示", async () => {
    const ds = fakeDataSource({
      listChanges: vi
        .fn()
        .mockResolvedValueOnce(changeList([
          { name: "kept-change", status: "in-progress", totalTasks: 2, completedTasks: 1 },
        ]))
        .mockRejectedValue(new Error("offline")),
    });
    renderApp(ds);
    await waitFor(() => expect(screen.getByText("kept-change")).toBeTruthy());
    // 掛載後補讀那一發以失敗收場——舊快照仍在，不得出現失敗提示。
    await waitFor(() => expect(vi.mocked(ds.listChanges).mock.calls.length).toBeGreaterThan(1));
    expect(screen.getByText("kept-change")).toBeTruthy();
    expect(document.querySelector('[data-testid="column-load-failed"]')).toBeNull();
  });
});

// desktop-app「側欄導覽結構」Scenario「手冊導覽項切頁與零分頁空狀態」＋
// desktop-manual-page「內頁渲染與出處跳規格」「手冊頁隨外部變更即時更新」的 App 半邊。
describe("手冊頁接線（desktop-manual-page）", () => {
  it("點導覽「手冊」切至手冊頁且高亮：主內容出現側欄樹與內文；點「變更」返回看板", async () => {
    const ds = fakeDataSource({
      listManualPages: vi.fn().mockResolvedValue(MANUAL_INDEX),
      getManualPage: vi.fn().mockResolvedValue("# 手冊\n\n歡迎使用。"),
    });
    renderApp(ds);
    await screen.findByText("desktop-shell-and-browser");
    // 進手冊頁前不讀索引（只在手冊視圖活躍時讀）。
    expect(ds.listManualPages).not.toHaveBeenCalled();
    const aside = projectColumn();
    const manualNav = within(aside).getByRole("button", { name: "手冊" });
    const changesNav = within(aside).getByRole("button", { name: "變更" });
    fireEvent.click(manualNav);
    await screen.findByText("歡迎使用。");
    expect(screen.getByPlaceholderText("搜尋手冊…")).toBeTruthy();
    expect(document.querySelector('[data-manual-page="boards"]')?.textContent).toContain("可能過期");
    expect(document.querySelector('[data-column="ready"]')).toBeNull();
    expect(manualNav.className).toContain("bg-primary");
    expect(changesNav.className).not.toContain("bg-primary");
    expect(ds.getManualPage).toHaveBeenCalledWith("index");
    // 再點手冊停留（切頁而非 toggle）；點「變更」才返回看板。
    fireEvent.click(manualNav);
    expect(screen.getByText("歡迎使用。")).toBeTruthy();
    fireEvent.click(changesNav);
    await waitFor(() => expect(document.querySelector('[data-column="ready"]')).toBeTruthy());
    expect(manualNav.className).not.toContain("bg-primary");
  });

  it("無手冊目錄時主內容為尚無手冊空狀態，手冊項高亮、無錯誤彈窗", async () => {
    renderApp();
    await screen.findByText("desktop-shell-and-browser");
    const aside = projectColumn();
    const manualNav = within(aside).getByRole("button", { name: "手冊" });
    fireEvent.click(manualNav);
    expect(await screen.findByText("尚無手冊")).toBeTruthy();
    expect(manualNav.className).toContain("bg-primary");
    expect(screen.queryByRole("alertdialog")).toBeNull();
  });

  it("零分頁時手冊不可達：無專案欄、不讀手冊索引，主區為空狀態", async () => {
    const ws = fakeWorkspace();
    const ds = fakeDataSource();
    render(<App createSession={makeSession(ds)} workspace={ws as never} />);
    expect(await screen.findByText("開啟一個專案開始")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "手冊" })).toBeNull();
    expect(ds.listManualPages).not.toHaveBeenCalled();
  });

  it("點手冊出處在手冊頁上開規格抽屜、不切頁", async () => {
    // spec Scenario「點出處開啟規格抽屜」：共用規格頁的唯讀抽屜（SpecDrawer），側欄不切頁。
    const ds = fakeDataSource({
      listManualPages: vi.fn().mockResolvedValue(MANUAL_INDEX),
      getManualPage: vi
        .fn()
        .mockResolvedValue("# 看板\n\n看板說明。\n\n**出處**：`desktop-app`、`ghost-cap`"),
      getSpecDocument: vi.fn().mockResolvedValue("# desktop-app Specification\n\n正典內文段落。"),
    });
    renderApp(ds);
    await screen.findByText("desktop-shell-and-browser");
    const aside = projectColumn();
    const manualNav = within(aside).getByRole("button", { name: "手冊" });
    const specsNav = within(aside).getByRole("button", { name: "規格" });
    fireEvent.click(manualNav);
    await screen.findByText("看板說明。");
    // 出處列以索引的 sources 為準：首頁 sources 為空，切到帶 sources 的「看板」頁。
    fireEvent.click(document.querySelector('[data-manual-page="boards"]') as HTMLElement);
    const sources = await waitFor(() => {
      const el = document.querySelector("[data-manual-sources]") as HTMLElement;
      expect(el).toBeTruthy();
      return el;
    });
    // 不在索引 sources 的名稱不進出處列（內文出處行只剝掉、不重解）。
    expect(within(sources).queryByRole("button", { name: "ghost-cap" })).toBeNull();
    expect(within(sources).queryByText("ghost-cap")).toBeNull();
    fireEvent.click(within(sources).getByRole("button", { name: "desktop-app" }));
    await waitFor(() => expect(screen.getByText("正典內文段落。")).toBeTruthy());
    // 不切頁：手冊項仍高亮、規格項未高亮，手冊內文仍在畫面，規格抽屜疊在其上。
    expect(manualNav.className).toContain("bg-primary");
    expect(specsNav.className).not.toContain("bg-primary");
    expect(screen.getByText("看板說明。")).toBeTruthy();
    expect(document.querySelector("[data-spec-drawer]")).toBeTruthy();
    expect(ds.getSpecDocument).toHaveBeenCalledWith("desktop-app");
  });

  it("手冊頁開著時 workspace-changed 觸發索引重取與目前頁內文重載", async () => {
    // spec Scenario「外部重生一頁後內容更新」：沿用既有監看事件，不新增監看目標。
    const ds = fakeDataSource({
      listManualPages: vi.fn().mockResolvedValue(MANUAL_INDEX),
      getManualPage: vi.fn().mockResolvedValue("# 手冊\n\n第一版內文。"),
    });
    renderApp(ds);
    await screen.findByText("desktop-shell-and-browser");
    await waitFor(() => expect(workspaceHandlers.length).toBeGreaterThan(0));
    const aside = projectColumn();
    fireEvent.click(within(aside).getByRole("button", { name: "手冊" }));
    await screen.findByText("第一版內文。");
    const indexCalls = (ds.listManualPages as Mock).mock.calls.length;
    const pageCalls = (ds.getManualPage as Mock).mock.calls.length;
    (ds.getManualPage as Mock).mockResolvedValue("# 手冊\n\n重生後的內文。");
    workspaceHandlers.forEach((h) => h());
    await screen.findByText("重生後的內文。");
    expect((ds.listManualPages as Mock).mock.calls.length).toBeGreaterThan(indexCalls);
    expect((ds.getManualPage as Mock).mock.calls.length).toBeGreaterThan(pageCalls);
  });
});

// drawer-provenance-links：規格抽屜出身列的溯源籤→封存變更抽屜（底層不切頁）；封存討論抽屜
// 衍生列三態（已封存→封存抽屜、活躍→詳情抽屜且落回看板、已刪除→不可點）的 App 接線面。
describe("抽屜溯源籤接線（drawer-provenance-links）", () => {
  const DISCUSSION_DOC =
    "---\ntopic: Old topic\nslug: old-topic\nstatus: promoted\ncreated: 2026-06-30\n---\n\n# Discussion: Old topic\n\n## Context\n\n封存背景內文。\n\n## Rounds\n\n## Conclusion\n\n**Decision**: 收工\n";
  const overflowList = () =>
    waitFor(() => {
      const el = document.querySelector("[data-source-overflow-list]") as HTMLElement | null;
      expect(el).toBeTruthy();
      return el!;
    });

  it("手冊頁開規格抽屜後點溯源籤：開該封存變更抽屜、規格抽屜關閉、底層仍留手冊頁", async () => {
    // spec Scenario「點擊溯源籤開啟封存變更抽屜且底層不切頁」。
    const ds = fakeDataSource({
      listManualPages: vi.fn().mockResolvedValue(MANUAL_INDEX),
      getManualPage: vi.fn().mockResolvedValue("# 看板\n\n看板說明。\n\n**出處**：`desktop-app`"),
      getSpecDocument: vi
        .fn()
        .mockResolvedValue(
          "# desktop-app Specification\n\n正典內文段落。\n\n<!-- @trace\nsource: old\nupdated: 2026-07-04\n-->\n",
        ),
      listArchived: vi.fn().mockResolvedValue([
        { datedName: "2026-07-04-old", date: "2026-07-04", name: "old", createdBy: null, fromDiscussions: [] },
      ]),
      getArchivedDocument: vi.fn().mockResolvedValue("## Why\n\n封存提案內文。"),
    });
    renderApp(ds);
    await screen.findByText("desktop-shell-and-browser");
    const aside = projectColumn();
    const manualNav = within(aside).getByRole("button", { name: "手冊" });
    fireEvent.click(manualNav);
    await screen.findByText("看板說明。");
    fireEvent.click(document.querySelector('[data-manual-page="boards"]') as HTMLElement);
    const sources = await waitFor(() => {
      const el = document.querySelector("[data-manual-sources]") as HTMLElement;
      expect(el).toBeTruthy();
      return el;
    });
    fireEvent.click(within(sources).getByRole("button", { name: "desktop-app" }));
    await waitFor(() => expect(screen.getByText("正典內文段落。")).toBeTruthy());
    const row = await waitFor(() => {
      const el = document.querySelector("[data-spec-drawer] [data-provenance-row]") as HTMLElement | null;
      expect(el).toBeTruthy();
      return el!;
    });
    expect(row.textContent).toContain("來自");
    fireEvent.click(within(row).getByRole("button", { name: /old/ }));
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    expect(ds.getArchivedDocument).toHaveBeenCalledWith("2026-07-04-old", "proposal.md");
    expect(document.querySelector("[data-archived-drawer]")).toBeTruthy();
    // detail 抽屜互斥：規格抽屜關閉；封存抽屜不切底層頁：手冊項仍高亮、看板欄不存在。
    await waitFor(() => expect(document.querySelector("[data-spec-drawer]")).toBeNull());
    expect(manualNav.className).toContain("bg-primary");
    expect(document.querySelector('[data-column="ready"]')).toBeNull();
  });

  it("封存討論抽屜衍生列三態：首籤（已封存）開封存變更抽屜、浮層列出活躍者階段詞與已刪除者「無封存記錄」", async () => {
    // spec Scenario「封存討論抽屜列出衍生變更並跳轉封存變更」「衍生變更籤的三態」。
    const ds = fakeDataSource({
      listChanges: vi.fn().mockResolvedValue(changeList([
        { name: "desktop-shell-and-browser", status: "in-progress", totalTasks: 30, completedTasks: 30 },
        { name: "live-child", status: "in-progress", totalTasks: 3, completedTasks: 1 },
      ])),
      listArchived: vi.fn().mockResolvedValue([
        { datedName: "2026-09-02-arch-child", date: "2026-09-02", name: "arch-child", createdBy: null, fromDiscussions: ["old-topic"] },
      ]),
      listDiscussions: vi.fn().mockResolvedValue({
        active: [],
        archived: [
          {
            slug: "old-topic",
            topic: "Old topic",
            status: "promoted",
            rounds: 1,
            created: "2026-06-30",
            promotedTo: ["arch-child", "live-child", "gone-child"],
          },
        ],
      }),
      getArchivedDocument: vi.fn().mockResolvedValue("## Why\n\n封存提案內文。"),
      getDiscussionDocument: vi.fn().mockResolvedValue(DISCUSSION_DOC),
    });
    renderApp(ds);
    await screen.findByText("desktop-shell-and-browser");
    const aside = projectColumn();
    const archivedNav = within(aside).getByRole("button", { name: /已封存/ });
    fireEvent.click(archivedNav);
    fireEvent.mouseDown(await screen.findByRole("tab", { name: /已封存的討論/ }));
    await screen.findByText("Old topic");
    fireEvent.click(screen.getByText("Old topic"));
    await waitFor(() => expect(screen.getByText("封存背景內文。")).toBeTruthy());
    const row = document.querySelector("[data-archived-drawer] [data-promoted-row]") as HTMLElement | null;
    expect(row).toBeTruthy();
    expect(row!.textContent).toContain("衍生");
    expect(within(row!).getByRole("button", { name: /arch-child/ })).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: /其餘 2 份/ }));
    const popover = await overflowList();
    const live = within(popover).getByRole("button", { name: /live-child/ });
    expect(live.textContent).toContain("進行中");
    expect(live.getAttribute("aria-disabled")).toBeNull();
    const gone = within(popover).getByRole("button", { name: /gone-child/ });
    expect(gone.getAttribute("aria-disabled")).toBe("true");
    expect(gone.textContent).toContain("無封存記錄");
    // 點首籤（已封存子變更）：同一封存抽屜切換為該變更；底層仍留已封存頁。
    fireEvent.click(within(row!).getByRole("button", { name: /arch-child/ }));
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    expect(ds.getArchivedDocument).toHaveBeenCalledWith("2026-09-02-arch-child", "proposal.md");
    expect(archivedNav.className).toContain("bg-primary");
    expect(document.querySelector('[data-column="ready"]')).toBeNull();
  });

  it("封存討論抽屜點活躍子變更籤：開其詳情抽屜且底層落回看板", async () => {
    const ds = fakeDataSource({
      listChanges: vi.fn().mockResolvedValue(changeList([
        { name: "desktop-shell-and-browser", status: "in-progress", totalTasks: 30, completedTasks: 30 },
        { name: "live-child", status: "in-progress", totalTasks: 3, completedTasks: 1 },
      ])),
      listDiscussions: vi.fn().mockResolvedValue({
        active: [],
        archived: [
          { slug: "old-topic", topic: "Old topic", status: "promoted", rounds: 1, created: "2026-06-30", promotedTo: ["live-child"] },
        ],
      }),
      getDiscussionDocument: vi.fn().mockResolvedValue(DISCUSSION_DOC),
    });
    renderApp(ds);
    await screen.findByText("desktop-shell-and-browser");
    const aside = projectColumn();
    const archivedNav = within(aside).getByRole("button", { name: /已封存/ });
    const changesNav = within(aside).getByRole("button", { name: "變更" });
    fireEvent.click(archivedNav);
    fireEvent.mouseDown(await screen.findByRole("tab", { name: /已封存的討論/ }));
    await screen.findByText("Old topic");
    fireEvent.click(screen.getByText("Old topic"));
    await waitFor(() => expect(screen.getByText("封存背景內文。")).toBeTruthy());
    const row = document.querySelector("[data-archived-drawer] [data-promoted-row]") as HTMLElement;
    fireEvent.click(within(row).getByRole("button", { name: /live-child/ }));
    // 變更詳情抽屜開啟（載入其提案），封存抽屜關閉，底層落回看板。
    await waitFor(() => expect(ds.getDocument).toHaveBeenCalledWith("live-child", "proposal.md"));
    await waitFor(() => expect(document.querySelector("[data-archived-drawer]")).toBeNull());
    await waitFor(() => expect(document.querySelector('[data-column="ready"]')).toBeTruthy());
    expect(changesNav.className).toContain("bg-primary");
    expect(archivedNav.className).not.toContain("bg-primary");
  });
});

// 更新日誌彈窗的 App 層接線（desktop-app 規格「更新日誌彈窗」）：把純判定、對話框與
// 已看過記錄接起來的膠水——模式切換、關閉才寫版號、dev 不彈不寫、瀏覽模式不寫。
describe("更新日誌彈窗接線（desktop-app「更新日誌彈窗」）", () => {
  const KEY = "speclink.lastSeenVersion";
  const TOP = RELEASE_NOTES[0].version;

  beforeEach(() => {
    localStorage.removeItem(KEY);
    vi.mocked(getVersion).mockResolvedValue(TOP);
    vi.stubEnv("DEV", false);
  });
  afterEach(() => {
    vi.unstubAllEnvs();
    vi.mocked(getVersion).mockResolvedValue("0.1.0");
  });

  function renderWithUpdater(ds: SpeclinkDataSource = fakeDataSource()) {
    const ws = fakeWorkspace();
    ws.openProject = vi.fn().mockResolvedValue({ status: "project", root: "A", name: "proj-a" });
    render(
      <App
        createSession={makeSession(ds)}
        workspace={ws as never}
        updater={{ check: vi.fn().mockResolvedValue(null), relaunch: vi.fn() }}
      />,
    );
  }

  /** 預置專案分頁 A，讓看板有卡片可點（抽屜相關案例用）。 */
  function seedProjectTab() {
    localStorage.setItem(
      "speclink.projectTabs",
      JSON.stringify({
        version: 2,
        tabs: [{ locator: { kind: "local", root: "A" }, name: "proj-a" }],
        activeKey: "local:A",
      }),
    );
  }

  it("版號變更後首次啟動彈出「X.Y.Z 更新內容」，按「知道了」關閉並記下現版號", async () => {
    localStorage.setItem(KEY, "0.1.0");
    renderWithUpdater();
    const dialog = await screen.findByTestId("release-notes-dialog");
    expect(within(dialog).getByText(`${TOP} 更新內容`)).toBeTruthy();
    fireEvent.click(within(dialog).getByRole("button", { name: "知道了" }));
    await waitFor(() => expect(screen.queryByTestId("release-notes-dialog")).toBeNull());
    expect(localStorage.getItem(KEY)).toBe(TOP);
  });

  it("macOS 原生選單的「更新日誌」開瀏覽模式", async () => {
    localStorage.setItem(KEY, TOP);
    renderWithUpdater();
    await screen.findByTestId("left-titlebar");
    expect(screen.queryByTestId("release-notes-dialog")).toBeNull();
    act(() => appMenuSpy.deps[appMenuSpy.deps.length - 1].dispatch("releaseNotes"));
    const dialog = await screen.findByTestId("release-notes-dialog");
    expect(within(dialog).getByText("更新日誌")).toBeTruthy();
  });

  it("寫入已看過記錄失敗時對話框仍關得掉", async () => {
    localStorage.setItem(KEY, "0.1.0");
    // vitest.setup 掛的 localStorage 來自另一個 jsdom window，全域 Storage.prototype 攔不到；
    // 對實例直接賦值又會被 jsdom 當成存一個叫 setItem 的 key——spy 要落在該實例的 prototype。
    const proto = Object.getPrototypeOf(localStorage) as Storage;
    const setItem = proto.setItem;
    const spy = vi.spyOn(proto, "setItem").mockImplementation(function (this: Storage, key, value) {
      if (key === KEY) throw new Error("storage disabled");
      setItem.call(this, key, value);
    });
    try {
      renderWithUpdater();
      const dialog = await screen.findByTestId("release-notes-dialog");
      fireEvent.click(within(dialog).getByRole("button", { name: "知道了" }));
      await waitFor(() => expect(screen.queryByTestId("release-notes-dialog")).toBeNull());
      expect(spy).toHaveBeenCalledWith(KEY, TOP);
      // app 沒有整個炸掉：左側還在。
      expect(leftSide()).toBeTruthy();
    } finally {
      spy.mockRestore();
    }
  });

  it("首次安裝（無記錄）不彈出，直接記下現版號", async () => {
    renderWithUpdater();
    await waitFor(() => expect(localStorage.getItem(KEY)).toBe(TOP));
    expect(screen.queryByTestId("release-notes-dialog")).toBeNull();
  });

  it("dev 建置不彈出也不寫記錄", async () => {
    vi.stubEnv("DEV", true);
    renderWithUpdater();
    await waitFor(() => expect(screen.getByText(/目前版本/)).toBeTruthy(), { timeout: 100 }).catch(() => {});
    await screen.findByTestId("left-titlebar");
    expect(screen.queryByTestId("release-notes-dialog")).toBeNull();
    expect(localStorage.getItem(KEY)).toBeNull();
  });

  it("設定頁「更新日誌」開瀏覽模式，關閉後已看過記錄不變", async () => {
    localStorage.setItem(KEY, TOP);
    renderWithUpdater();
    const aside = (await screen.findByTestId("left-titlebar")).parentElement as HTMLElement;
    await waitFor(() => expect(settingsGear()).toBeTruthy());
    fireEvent.click(settingsGear());
    fireEvent.click(await screen.findByRole("button", { name: "更新日誌" }));
    const dialog = await screen.findByTestId("release-notes-dialog");
    expect(within(dialog).getByText("更新日誌")).toBeTruthy();
    fireEvent.click(within(dialog).getByRole("button", { name: "關閉" }));
    await waitFor(() => expect(screen.queryByTestId("release-notes-dialog")).toBeNull());
    expect(localStorage.getItem(KEY)).toBe(TOP);
  });

  it("抽屜開啟時更新日誌關閉且不記為已看過（規格「detail 抽屜互斥」）", async () => {
    localStorage.setItem(KEY, "0.1.0");
    seedProjectTab();
    renderWithUpdater();
    await screen.findByTestId("release-notes-dialog");

    // 看板卡片走 store openDetail；fireEvent 不送 pointerdown，不會誤走對話框的點外關閉。
    fireEvent.click(await screen.findByText("desktop-shell-and-browser"));

    await waitFor(() => expect(screen.queryByTestId("release-notes-dialog")).toBeNull());
    expect(drawerSpy.rich[drawerSpy.rich.length - 1].open).toBe(true);
    expect(localStorage.getItem(KEY)).toBe("0.1.0");
  });

  it("whatsNew 開啟中開啟討論：對話框關閉、討論抽屜開啟且不記為已看過", async () => {
    // 規格 Example「更新日誌（whatsNew）｜開啟討論」。
    localStorage.setItem(KEY, "0.1.0");
    seedProjectTab();
    renderWithUpdater(
      fakeDataSource({
        listDiscussions: vi.fn().mockResolvedValue({
          active: [
            { slug: "topic-a", topic: "Topic A", status: "open", rounds: 1, created: "2026-07-17", promotedTo: [] },
          ],
          archived: [],
        }),
      }),
    );
    await screen.findByTestId("release-notes-dialog");

    fireEvent.click(await screen.findByText("Topic A"));

    await waitFor(() => expect(screen.queryByTestId("release-notes-dialog")).toBeNull());
    expect(drawerSpy.disc[drawerSpy.disc.length - 1].open).toBe(true);
    expect(localStorage.getItem(KEY)).toBe("0.1.0");
  });

  it("抽屜已開著時 whatsNew 才彈出：換開另一個 change 仍收掉更新日誌", async () => {
    localStorage.setItem(KEY, "0.1.0");
    seedProjectTab();
    let resolveVersion!: (version: string) => void;
    vi.mocked(getVersion).mockReturnValueOnce(new Promise((resolve) => (resolveVersion = resolve)));
    renderWithUpdater(
      fakeDataSource({
        listChanges: vi.fn().mockResolvedValue(
          changeList([
            { name: "change-a", status: "in-progress", totalTasks: 3, completedTasks: 1 },
            { name: "change-b", status: "in-progress", totalTasks: 3, completedTasks: 1 },
          ]),
        ),
      }),
    );
    fireEvent.click(await screen.findByText("change-a"));
    await waitFor(() => expect(drawerSpy.rich[drawerSpy.rich.length - 1].open).toBe(true));
    await act(async () => resolveVersion(TOP));
    await screen.findByTestId("release-notes-dialog");

    fireEvent.click(screen.getByText("change-b"));

    await waitFor(() => expect(screen.queryByTestId("release-notes-dialog")).toBeNull());
    expect((drawerSpy.rich[drawerSpy.rich.length - 1].change as { name: string }).name).toBe("change-b");
    expect(localStorage.getItem(KEY)).toBe("0.1.0");
  });
});

// spec desktop-app「詳情抽屜的工單分頁」「已封存抽屜的工單分頁」的宿主接線
// （drawer-quality-ticket-tab D3／D5）：兩個抽屜的 loadStationTicket 各接資料源的
// getStationTicket／getArchivedStationTicket，與 loadDocument 同款包裝。
describe("工單分頁接線（drawer-quality-ticket-tab）", () => {
  const HASH = `sha256:${"a".repeat(64)}`;
  const TICKET = {
    rounds: [
      {
        index: 1,
        phase: "discovery",
        patchHash: HASH,
        scope: ["src/a.rs"],
        findings: [{ severity: "WARNING", path: "src/a.rs", text: "Correctness: 未處理空清單" }],
      },
    ],
  };

  it("inReview 變更的抽屜出現「審查」分頁，工單經 dataSource.getStationTicket 載入", async () => {
    const ds = fakeDataSource({
      listChanges: vi.fn().mockResolvedValue(
        changeList([
          { name: "desktop-shell-and-browser", status: "in-progress", totalTasks: 30, completedTasks: 30, reviewStatus: "inReview" },
        ]),
      ),
      getStationTicket: vi.fn().mockResolvedValue(TICKET),
    });
    renderApp(ds);
    await waitFor(() => screen.getByText("desktop-shell-and-browser"));
    expect(ds.getStationTicket).not.toHaveBeenCalled();
    fireEvent.click(screen.getByText("desktop-shell-and-browser"));
    await waitFor(() => screen.getByRole("tab", { name: /審查/ }));
    await waitFor(() => expect(ds.getStationTicket).toHaveBeenCalledWith("desktop-shell-and-browser", "review"));
    fireEvent.mouseDown(screen.getByRole("tab", { name: /審查/ }));
    await waitFor(() =>
      expect((document.querySelector("[data-ticket-header]")?.textContent ?? "").replace(/\s+/g, " ")).toBe(
        "審查 · 第 1 輪（首輪）CRITICAL 0 · WARNING 1 · SUGGESTION 0",
      ),
    );
  });

  it("reviewedNotPassed 的封存抽屜出現「審查」分頁，工單經 dataSource.getArchivedStationTicket 載入", async () => {
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
          reviewStatus: "reviewedNotPassed",
          verifyStatus: "none",
        },
      ]),
      getArchivedDocument: vi.fn().mockResolvedValue("## Why\n\n封存提案內文。"),
      getArchivedStationTicket: vi.fn().mockResolvedValue(TICKET),
    });
    renderApp(ds);
    await waitFor(() => screen.getByText("desktop-shell-and-browser"));
    const aside = projectColumn();
    fireEvent.click(within(aside).getByRole("button", { name: /已封存/ }));
    await waitFor(() => screen.getByText("old"));
    fireEvent.click(screen.getByText("old"));
    await waitFor(() => screen.getByRole("tab", { name: /審查/ }));
    await waitFor(() => expect(ds.getArchivedStationTicket).toHaveBeenCalledWith("2026-07-04-old", "review"));
    fireEvent.mouseDown(screen.getByRole("tab", { name: /審查/ }));
    await waitFor(() => expect(document.querySelector("[data-ticket-header]")).toBeTruthy());
  });
});

// desktop-board-reskin design D2：App 把 store 的 promotedExpanded／setPromotedExpanded 接給看板；
// 點討論欄底的收合列即寫回 app 本機鍵，重啟（鍵已存在）時啟動即展開。App 的 store 是
// 內部建立的，store 狀態以其可觀察結果斷言：收合列 aria-expanded 與 localStorage 鍵。
describe("看板討論欄收合列的展開記憶接線（desktop-board-reskin）", () => {
  const KEY = "speclink.board.promotedExpanded";
  const promotedOnly = () =>
    fakeDataSource({
      listDiscussions: vi.fn().mockResolvedValue({
        active: [
          {
            slug: "fanout",
            topic: "Fanout topic",
            status: "promoted",
            rounds: 4,
            created: "2026-07-03",
            promotedTo: ["desktop-shell-and-browser"],
          },
        ],
        archived: [],
      }),
    });

  afterEach(() => {
    localStorage.removeItem(KEY);
  });

  it('點收合列後 store 的 promotedExpanded 為 true：細列展開且鍵寫為 "true"', async () => {
    localStorage.removeItem(KEY);
    renderApp(promotedOnly());
    const bar = await screen.findByRole("button", { name: /已轉出/ });
    expect(bar.getAttribute("aria-expanded")).toBe("false");
    expect(screen.queryByText("Fanout topic")).toBeNull();
    fireEvent.click(bar);
    expect(bar.getAttribute("aria-expanded")).toBe("true");
    expect(screen.getByText("Fanout topic")).toBeTruthy();
    expect(localStorage.getItem(KEY)).toBe("true");
  });

  it('鍵已為 "true" 時啟動即展開（跨啟動保留）', async () => {
    localStorage.setItem(KEY, "true");
    renderApp(promotedOnly());
    const bar = await screen.findByRole("button", { name: /已轉出/ });
    expect(bar.getAttribute("aria-expanded")).toBe("true");
    expect(screen.getByText("Fanout topic")).toBeTruthy();
  });
});

// desktop-app「macOS 原生選單」的 App 層接線（desktop-native-menu design D3）：語言與
// 專案有無改變時 rebuild、選單動作派發到 store 既有動作或外部瀏覽器。
describe("macOS 原生選單接線（desktop-app「macOS 原生選單」）", () => {
  const menuDeps = () => appMenuSpy.deps[appMenuSpy.deps.length - 1];
  const searchInput = () => screen.getByPlaceholderText("搜尋看板卡片…");

  it("切換 UI 語言後以新語言重裝選單（取消上一次），切回 zh-TW 變回繁中", async () => {
    renderApp();
    await screen.findByText("desktop-shell-and-browser");
    expect(menuDeps().t("menu.file.title")).toBe("檔案");
    fireEvent.click(settingsGear());
    fireEvent.mouseDown(await screen.findByRole("tab", { name: "本機設定" }));
    const cancelsBefore = appMenuSpy.cancel.mock.calls.length;
    fireEvent.click(within(await screen.findByTestId("ui-locale")).getByText("English"));
    await waitFor(() => expect(menuDeps().t("menu.file.title")).toBe("File"));
    expect(appMenuSpy.cancel.mock.calls.length).toBeGreaterThan(cancelsBefore);
    fireEvent.click(within(screen.getByTestId("ui-locale")).getByText("繁體中文"));
    await waitFor(() => expect(menuDeps().t("menu.file.title")).toBe("檔案"));
  });

  it("dispatch(\"viewSpecs\") 切到規格頁且專案欄「規格」為作用中；dispatch(\"viewBoard\") 回到看板", async () => {
    renderApp();
    await screen.findByText("desktop-shell-and-browser");
    act(() => menuDeps().dispatch("viewSpecs"));
    await waitFor(() => expect(screen.getByTestId("main-titlebar").textContent).toBe("proj-a/規格"));
    expect(within(projectColumn()).getByRole("button", { name: "規格" }).className).toContain(
      "bg-primary",
    );
    act(() => menuDeps().dispatch("viewBoard"));
    await waitFor(() => expect(screen.getByTestId("main-titlebar").textContent).toBe("proj-a/變更"));
    expect(document.querySelector('[data-column="ready"]')).toBeTruthy();
  });

  it("零專案時安裝收到 hasProject=false；dispatch(\"openProject\") 開新增專案對話框", async () => {
    render(<App createSession={makeSession(fakeDataSource())} workspace={fakeWorkspace() as never} />);
    expect(await screen.findByText("開啟一個專案開始")).toBeTruthy();
    expect(menuDeps().hasProject).toBe(false);
    act(() => menuDeps().dispatch("openProject"));
    const chooser = await screen.findByRole("alertdialog");
    expect(within(chooser).getByRole("button", { name: /本機資料夾/ })).toBeTruthy();
  });

  it("dispatch(\"closeProject\") 關掉作用中專案後重裝收到 hasProject=false", async () => {
    renderApp();
    await screen.findByText("desktop-shell-and-browser");
    expect(menuDeps().hasProject).toBe(true);
    act(() => menuDeps().dispatch("closeProject"));
    expect(await screen.findByText("開啟一個專案開始")).toBeTruthy();
    await waitFor(() => expect(menuDeps().hasProject).toBe(false));
  });

  it("主視窗不在焦點（系統匣面板在前景）時 dispatch(\"closeProject\") 不關專案", async () => {
    windowSpy.isFocused.mockResolvedValue(false);
    renderApp();
    await screen.findByText("desktop-shell-and-browser");
    act(() => menuDeps().dispatch("closeProject"));
    await waitFor(() => expect(windowSpy.isFocused).toHaveBeenCalled());
    await act(async () => {});
    expect(screen.queryByText("開啟一個專案開始")).toBeNull();
    expect(menuDeps().hasProject).toBe(true);
  });

  it("看板顯示時 dispatch(\"focusSearch\") 讓看板搜尋輸入取得焦點", async () => {
    renderApp();
    await screen.findByText("desktop-shell-and-browser");
    act(() => menuDeps().dispatch("focusSearch"));
    expect(document.activeElement).toBe(searchInput());
  });

  it("規格頁 dispatch(\"focusSearch\") 先切回看板再聚焦搜尋輸入", async () => {
    renderApp();
    await screen.findByText("desktop-shell-and-browser");
    act(() => menuDeps().dispatch("viewSpecs"));
    await waitFor(() => expect(screen.getByTestId("main-titlebar").textContent).toBe("proj-a/規格"));
    act(() => menuDeps().dispatch("focusSearch"));
    await waitFor(() => expect(document.activeElement).toBe(searchInput()));
    expect(screen.getByTestId("main-titlebar").textContent).toBe("proj-a/變更");
  });

  it("dispatch(\"checkUpdates\") 進設定頁並執行手動檢查", async () => {
    const updater = { check: vi.fn().mockResolvedValue(null), relaunch: vi.fn() };
    renderApp(fakeDataSource(), { updater });
    await screen.findByText("desktop-shell-and-browser");
    const checksBefore = updater.check.mock.calls.length;
    act(() => menuDeps().dispatch("checkUpdates"));
    await waitFor(() => expect(updater.check.mock.calls.length).toBeGreaterThan(checksBefore));
    expect(screen.getByTestId("main-titlebar").textContent).toBe("設定");
    expect(await screen.findByTestId("updater-card")).toBeTruthy();
  });

  it("dispatch(\"github\")／dispatch(\"reportIssue\") 以 openUrl 開 repo 網址", async () => {
    renderApp();
    await screen.findByText("desktop-shell-and-browser");
    vi.mocked(openUrl).mockClear();
    act(() => menuDeps().dispatch("github"));
    act(() => menuDeps().dispatch("reportIssue"));
    expect(vi.mocked(openUrl).mock.calls).toEqual([
      ["https://github.com/MomoChenisMe/speclink"],
      ["https://github.com/MomoChenisMe/speclink/issues/new"],
    ]);
  });

  it("openUrl 失敗時 toast 錯誤原文", async () => {
    const error = vi.spyOn(toast, "error");
    vi.mocked(openUrl).mockRejectedValueOnce("no default browser");
    renderApp();
    await screen.findByText("desktop-shell-and-browser");
    act(() => menuDeps().dispatch("github"));
    await waitFor(() => expect(error).toHaveBeenCalledWith("no default browser"));
    error.mockRestore();
  });
});

// 快捷鍵卡與實際按鍵不漂移（desktop-native-menu design D5）：非 macOS 卡片的每一列，按下去
// 都要有 App 的 keydown 或看板搜尋列接手（preventDefault）。改了按鍵卻沒改卡片時這裡會紅。
describe("非 macOS 快捷鍵卡與 keydown 一致", () => {
  const zhT = (key: string) => APP_MESSAGES["zh-TW"][key] ?? key;
  const platform = detectPlatform();
  /** 卡片按鍵字串轉成要按的鍵：`Ctrl+W` → w、`Ctrl+Tab` → Tab、`Ctrl+1–9` → 1 與 9。 */
  const keysOf = (keys: string) => {
    const key = keys.replace(/^Ctrl\+/, "");
    if (key.includes("–")) return key.split("–");
    return [key.length === 1 ? key.toLowerCase() : key];
  };

  it("jsdom 以非 macOS 平台渲染", () => {
    expect(platform).not.toBe("macos");
  });

  it.each(buildShortcutList({ t: zhT, platform }).map((row) => [row.text, row.keys] as const))(
    "「%s」%s 有人接手",
    async (_text, keys) => {
      renderApp();
      await screen.findByText("desktop-shell-and-browser");
      for (const key of keysOf(keys)) {
        expect(fireEvent.keyDown(window, { key, ctrlKey: true })).toBe(false);
      }
    },
  );
});

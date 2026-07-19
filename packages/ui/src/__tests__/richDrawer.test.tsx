import { describe, it, expect, vi, type Mock } from "vitest";
import { render as rtlRender, screen, waitFor, fireEvent, act, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";

// 既有中文斷言包 I18nProvider locale zh-TW 後照舊斷言（i18n 抽 key 的回歸保護）。
const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

import { RichDetailDrawer } from "../components/RichDetailDrawer";
import { DELTA_COLORS } from "../components/DeltaBadges";
import type { ChangeItem } from "../adapter";

// 攔截 TaskList props 以驗證抽屜→拖放回呼的接線（jsdom 無法真拖）。
const { taskListProps } = vi.hoisted(() => ({
  taskListProps: [] as Array<Record<string, unknown>>,
}));
vi.mock("../components/TaskList", () => ({
  TaskList: (props: Record<string, unknown>) => {
    taskListProps.push(props);
    return <div data-testid="tasklist-stub" />;
  },
}));

const change: ChangeItem = {
  name: "desktop-shell-and-browser",
  status: "in-progress",
  totalTasks: 30,
  completedTasks: 30,
};

const SPEC_MD = "## ADDED Requirements\n\n### Requirement: a\nb\n\n### Requirement: c\nd\n";

function makeProps(over: Record<string, unknown> = {}) {
  return {
    open: true,
    onOpenChange: vi.fn(),
    change,
    loadDocument: vi.fn(async (_c: string, artifact: string) =>
      artifact.startsWith("specs/") ? SPEC_MD : `# doc for ${artifact}`,
    ),
    loadCapabilities: vi.fn(async () => ["desktop-app"]),
    loadMeta: vi.fn(async () => ({ created: "2026-07-05", createdBy: "MomoChen", createdWith: "claude" })),
    onRunVerb: vi.fn(),
    onDelete: vi.fn(),
    ...over,
  };
}

// spec 需求「規格分頁 delta 區段以色標呈現」的渲染面（design D4 色標區段、配色對齊 DeltaBadges）。
describe("規格分頁 delta 區段色標", () => {
  const DELTA_MD =
    "## ADDED Requirements\n\n### Requirement: a\nb\n\n## MODIFIED Requirements\n\n### Requirement: c\nd\n\n## REMOVED Requirements\n\n### Requirement: e\nf\n\n## RENAMED Requirements\n\n### Requirement: g\nh\n";

  async function openSpecsTab(specMd: string) {
    const props = makeProps({
      loadDocument: vi.fn(async (_c: string, artifact: string) =>
        artifact.startsWith("specs/") ? specMd : `# doc for ${artifact}`,
      ),
    });
    const result = render(<RichDetailDrawer {...(props as never)} />);
    await waitFor(() => screen.getByRole("tab", { name: /規格/ }));
    fireEvent.mouseDown(screen.getByRole("tab", { name: /規格/ }));
    return result;
  }

  it("delta 區段呈現色標標頭（配色與 DeltaBadges 同一常數來源），原始標題文字不直出", async () => {
    const { baseElement } = await openSpecsTab(DELTA_MD);
    await waitFor(() =>
      expect(baseElement.querySelector('[data-delta-section="added"]')).toBeTruthy(),
    );
    const added = baseElement.querySelector('[data-delta-section="added"]') as HTMLElement;
    const modified = baseElement.querySelector('[data-delta-section="modified"]') as HTMLElement;
    expect(within(added).getByText("新增")).toBeTruthy();
    expect(within(modified).getByText("修改")).toBeTruthy();
    expect(added.className).toContain(DELTA_COLORS.added);
    expect(modified.className).toContain(DELTA_COLORS.modified);
    // spec「標籤為大標題且字級大於內文」：色標標頭同大標題款、保留各色（design D6）。
    expect(added.className).toContain("text-xl");
    expect(added.className).toContain("font-bold");
    expect(added.className).not.toContain("text-xs");
    // Example「四種 delta 區段的色標對應」其餘兩列：移除紅、更名藍。
    const removed = baseElement.querySelector('[data-delta-section="removed"]') as HTMLElement;
    const renamed = baseElement.querySelector('[data-delta-section="renamed"]') as HTMLElement;
    expect(within(removed).getByText("移除")).toBeTruthy();
    expect(within(renamed).getByText("更名")).toBeTruthy();
    expect(removed.className).toContain(DELTA_COLORS.removed);
    expect(renamed.className).toContain(DELTA_COLORS.renamed);
    expect(screen.queryByText(/REMOVED Requirements/)).toBeNull();
    expect(screen.queryByText(/RENAMED Requirements/)).toBeNull();
    // 原始機器標題不以標題文字直出。
    expect(screen.queryByText(/ADDED Requirements/)).toBeNull();
    expect(screen.queryByText(/MODIFIED Requirements/)).toBeNull();
    // requirement 內文照 prose 排版呈現。
    expect(screen.getByText(/Requirement: a/)).toBeTruthy();
    expect(screen.getByText(/Requirement: c/)).toBeTruthy();
  });

  it("無 delta 標記的規格整篇照常渲染（無色標標頭）", async () => {
    const { baseElement } = await openSpecsTab("# 正典規格\n\n### Requirement: plain\n本文。\n");
    await waitFor(() => expect(screen.getByText(/Requirement: plain/)).toBeTruthy());
    expect(baseElement.querySelector("[data-delta-section]")).toBeNull();
  });
});

// spec 需求「提案與設計章節以中文標籤呈現」的接線面（design D3）。
describe("提案／設計分頁章節標籤", () => {
  it("提案分頁呈現中文章節標籤，英文模板標題不直出", async () => {
    const props = makeProps({
      loadDocument: vi.fn(async (_c: string, artifact: string) =>
        artifact === "proposal.md" ? "## Why\n\n動機內文。\n\n## What Changes\n\n- 變更項\n" : SPEC_MD,
      ),
    });
    render(<RichDetailDrawer {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("動機內文。")).toBeTruthy());
    expect(screen.getByText("為什麼")).toBeTruthy();
    expect(screen.getByText("變更內容")).toBeTruthy();
    expect(screen.queryByText("Why")).toBeNull();
    expect(screen.queryByText("What Changes")).toBeNull();
  });
});

// remote-data-source 手動驗證回歸：空殼 change（server 上尚無 proposal 文件）
// loadDocument 回 null——「不存在」須呈現尚無文案，不得與「載入中」共用同一字樣。
describe("提案分頁載入中／不存在分流", () => {
  it("loadDocument 回 null 時顯示尚無提案文案，而非載入中", async () => {
    const props = makeProps({
      loadDocument: vi.fn(async () => null),
      loadCapabilities: vi.fn(async () => []),
    });
    render(<RichDetailDrawer {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("（此 change 尚無提案內容）")).toBeTruthy());
    expect(screen.queryByText("載入中…")).toBeNull();
  });

  it("載入未完成（promise 未解決）時顯示載入中", async () => {
    const props = makeProps({
      loadDocument: vi.fn(() => new Promise<string>(() => {})),
      loadCapabilities: vi.fn(async () => []),
      loadMeta: vi.fn(() => new Promise<never>(() => {})),
    });
    render(<RichDetailDrawer {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("載入中…")).toBeTruthy());
    expect(screen.queryByText("（此 change 尚無提案內容）")).toBeNull();
  });
});

describe("RichDetailDrawer", () => {
  it("renders metadata row (author, agent, task count) and progress", async () => {
    render(<RichDetailDrawer {...(makeProps() as never)} />);
    await waitFor(() => expect(screen.getByText("MomoChen")).toBeTruthy());
    expect(screen.getByText(/claude/)).toBeTruthy();
    expect(screen.getAllByText(/30\/30/).length).toBeGreaterThan(0);
  });

  it("shows spec delta counts on the specs tab", async () => {
    render(<RichDetailDrawer {...(makeProps() as never)} />);
    await waitFor(() => expect(screen.getByText(/\+2/)).toBeTruthy());
  });

  it("forwards a drag-reorder drop to onMoveTask; reload rides the refreshGen bump", async () => {
    const props = makeProps({ onMoveTask: vi.fn().mockResolvedValue(undefined) });
    const { rerender } = render(<RichDetailDrawer {...(props as never)} refreshGen={0} />);
    // TaskList 掛在任務分頁下——先切分頁（Radix TabsTrigger 以 mousedown 觸發）。
    await waitFor(() => screen.getByRole("tab", { name: /任務/ }));
    fireEvent.mouseDown(screen.getByRole("tab", { name: /任務/ }));
    await waitFor(() => expect(taskListProps.length).toBeGreaterThan(0));
    const latest = taskListProps[taskListProps.length - 1] as {
      onReorder?: (from: number, to: number, before?: boolean) => Promise<void> | void;
    };
    expect(typeof latest.onReorder).toBe("function");
    const tasksReads = () =>
      (props.loadDocument as Mock).mock.calls.filter((c) => c[1] === "tasks.md").length;
    const before = tasksReads();
    // onReorder 直呼繞過事件系統，內部 state 更新須明確包 act。
    await act(async () => {
      await latest.onReorder!(2, 5);
    });
    // 一次到位轉發 from/to（側別未指定）。
    expect((props as { onMoveTask: Mock }).onMoveTask).toHaveBeenCalledWith(
      "desktop-shell-and-browser",
      2,
      5,
      undefined,
    );
    // design D2：獨立局部重讀已移除——重載由宿主 refresh 後的世代遞增驅動。
    expect(tasksReads()).toBe(before);
    rerender(<RichDetailDrawer {...(props as never)} refreshGen={1} />);
    await waitFor(() => expect(tasksReads()).toBeGreaterThan(before));
    // 標題落點（design D7 moveTask 側別）：before=true 轉發至第四參數。
    await act(async () => {
      await latest.onReorder!(1, 3, true);
    });
    expect((props as { onMoveTask: Mock }).onMoveTask).toHaveBeenCalledWith(
      "desktop-shell-and-browser",
      1,
      3,
      true,
    );
  });

  it("reloads documents and meta in place when refreshGen advances (external change)", async () => {
    const props = makeProps();
    const { rerender } = render(<RichDetailDrawer {...(props as never)} refreshGen={0} />);
    await waitFor(() => expect(screen.getByText("MomoChen")).toBeTruthy());
    expect(screen.queryByText(/開工/)).toBeNull();
    const docCalls = () => (props.loadDocument as Mock).mock.calls.length;
    const metaCalls = () => (props.loadMeta as Mock).mock.calls.length;
    const d0 = docCalls();
    const m0 = metaCalls();
    // 外部寫者蓋開工章後 watcher 觸發 refresh → 世代遞增 → 開著的抽屜重載至磁碟現況。
    (props.loadMeta as Mock).mockResolvedValue({
      created: "2026-07-05",
      createdBy: "MomoChen",
      createdWith: "claude",
      startedAt: "2026-07-07",
      startedBy: "Worker <w@example.com>",
    });
    rerender(<RichDetailDrawer {...(props as never)} refreshGen={1} />);
    await waitFor(() => expect(docCalls()).toBeGreaterThan(d0));
    await waitFor(() => expect(metaCalls()).toBeGreaterThan(m0));
    await waitFor(() => expect(screen.getByText(/2026-07-07 開工/)).toBeTruthy());
  });

  it("defers the refreshGen reload while a task interaction is in flight, catches up after", async () => {
    let release!: () => void;
    const gate = new Promise<void>((r) => {
      release = r;
    });
    const props = makeProps({ onToggleTask: vi.fn(() => gate) });
    const { rerender } = render(<RichDetailDrawer {...(props as never)} refreshGen={0} />);
    await waitFor(() => screen.getByRole("tab", { name: /任務/ }));
    fireEvent.mouseDown(screen.getByRole("tab", { name: /任務/ }));
    await waitFor(() => expect(taskListProps.length).toBeGreaterThan(0));
    const latest = taskListProps[taskListProps.length - 1] as {
      onToggle?: (ordinal: number, done: boolean) => void;
    };
    const tasksReads = () =>
      (props.loadDocument as Mock).mock.calls.filter((c) => c[1] === "tasks.md").length;
    const before = tasksReads();
    act(() => latest.onToggle!(1, true)); // 互動進行中（onToggleTask 未 resolve）
    rerender(<RichDetailDrawer {...(props as never)} refreshGen={1} />);
    await new Promise((r) => setTimeout(r, 25));
    // 讓路：互動未結束前外部世代不觸發重載，不打斷進行中操作。
    expect(tasksReads()).toBe(before);
    await act(async () => release()); // 互動結束 → 補載一次
    await waitFor(() => expect(tasksReads()).toBeGreaterThan(before));
  });

  it("defers the refreshGen reload while a drag gesture is active, catches up after drop", async () => {
    // spec：讓路涵蓋「拖曳排序進行中」（按住～放開），不只放開後的寫回等待。
    const props = makeProps();
    const { rerender } = render(<RichDetailDrawer {...(props as never)} refreshGen={0} />);
    await waitFor(() => screen.getByRole("tab", { name: /任務/ }));
    fireEvent.mouseDown(screen.getByRole("tab", { name: /任務/ }));
    await waitFor(() => expect(taskListProps.length).toBeGreaterThan(0));
    const latest = taskListProps[taskListProps.length - 1] as {
      onDragActiveChange?: (active: boolean) => void;
    };
    expect(typeof latest.onDragActiveChange).toBe("function");
    const tasksReads = () =>
      (props.loadDocument as Mock).mock.calls.filter((c) => c[1] === "tasks.md").length;
    const before = tasksReads();
    act(() => latest.onDragActiveChange!(true)); // 拖曳手勢開始（尚未放開）
    rerender(<RichDetailDrawer {...(props as never)} refreshGen={1} />);
    await new Promise((r) => setTimeout(r, 25));
    expect(tasksReads()).toBe(before); // 拖曳中外部世代不重載
    act(() => latest.onDragActiveChange!(false)); // 放開
    await waitFor(() => expect(tasksReads()).toBeGreaterThan(before)); // 補載一次
  });

  it("discards stale responses when a newer refreshGen load raced ahead (latest-wins)", async () => {
    const pendingProposal: Array<(v: string) => void> = [];
    const props = makeProps({
      loadDocument: vi.fn(async (_c: string, artifact: string) => {
        if (artifact === "proposal.md") return new Promise<string>((r) => pendingProposal.push(r));
        return artifact.startsWith("specs/") ? SPEC_MD : `# doc for ${artifact}`;
      }),
    });
    const { rerender } = render(<RichDetailDrawer {...(props as never)} refreshGen={0} />);
    await waitFor(() => expect(pendingProposal.length).toBe(1)); // 初載請求（gen 0）
    rerender(<RichDetailDrawer {...(props as never)} refreshGen={1} />);
    await waitFor(() => expect(pendingProposal.length).toBe(2)); // 世代重載請求（gen 1）
    pendingProposal[1]("# proposal v2"); // 新世代回應先到
    await waitFor(() => expect(screen.getByText(/proposal v2/)).toBeTruthy());
    pendingProposal[0]("# proposal v1-stale"); // 舊世代回應後到——必須被丟棄
    await new Promise((r) => setTimeout(r, 25));
    expect(screen.queryByText(/v1-stale/)).toBeNull();
    expect(screen.getByText(/proposal v2/)).toBeTruthy();
  });

  it("toggle completion updates tasks and meta together via the refreshGen path only", async () => {
    const props = makeProps({ onToggleTask: vi.fn().mockResolvedValue(undefined) });
    const { rerender } = render(<RichDetailDrawer {...(props as never)} refreshGen={0} />);
    await waitFor(() => screen.getByRole("tab", { name: /任務/ }));
    fireEvent.mouseDown(screen.getByRole("tab", { name: /任務/ }));
    await waitFor(() => expect(taskListProps.length).toBeGreaterThan(0));
    const latest = taskListProps[taskListProps.length - 1] as {
      onToggle?: (ordinal: number, done: boolean) => void;
    };
    const tasksReads = () =>
      (props.loadDocument as Mock).mock.calls.filter((c) => c[1] === "tasks.md").length;
    const metaCalls = () => (props.loadMeta as Mock).mock.calls.length;
    const t0 = tasksReads();
    const m0 = metaCalls();
    act(() => latest.onToggle!(1, true));
    await waitFor(() => expect((props as { onToggleTask: Mock }).onToggleTask).toHaveBeenCalled());
    await new Promise((r) => setTimeout(r, 25));
    // 獨立局部重讀路徑已移除（design D2 單一資料流）。
    expect(tasksReads()).toBe(t0);
    // 宿主 onToggleTask 內 refresh → 世代遞增（此處由測試模擬）→ tasks 與 meta 一併重載。
    rerender(<RichDetailDrawer {...(props as never)} refreshGen={1} />);
    await waitFor(() => expect(tasksReads()).toBeGreaterThan(t0));
    await waitFor(() => expect(metaCalls()).toBeGreaterThan(m0));
  });

  // spec「勾選任務即時回饋」（desktop-task-interactions）：樂觀更新＋失敗回滾＋不鎖清單。
  describe("勾選樂觀更新", () => {
    const TASKS_MD = "## 1. G\n\n- [ ] 1.1 a\n- [ ] 1.2 b\n";
    const tasksLoader = () =>
      vi.fn(async (_c: string, artifact: string) => {
        if (artifact === "tasks.md") return TASKS_MD;
        return artifact.startsWith("specs/") ? SPEC_MD : `# doc for ${artifact}`;
      });
    const latestTL = () =>
      taskListProps[taskListProps.length - 1] as {
        markdown?: string | null;
        busy?: boolean;
        onToggle?: (o: number, d: boolean) => void;
      };
    const openTasksTab = async () => {
      await waitFor(() => screen.getByRole("tab", { name: /任務/ }));
      fireEvent.mouseDown(screen.getByRole("tab", { name: /任務/ }));
      await waitFor(() => expect(latestTL().markdown).toContain("- [ ] 1.1 a"));
    };

    // spec「任務寫回非阻塞且序列化」Scenario「舊載入回應不覆蓋樂觀狀態」（design D4）
    it("更早發起的舊載入回應到達時不覆蓋樂觀勾選狀態", async () => {
      const pending: Array<(v: string | null) => void> = [];
      let hangTasks = false;
      const loadDocument = vi.fn((_c: string, artifact: string) => {
        if (artifact === "tasks.md" && hangTasks) {
          return new Promise<string | null>((r) => pending.push(r));
        }
        if (artifact === "tasks.md") return Promise.resolve<string | null>(TASKS_MD);
        return Promise.resolve<string | null>(
          artifact.startsWith("specs/") ? SPEC_MD : `# doc for ${artifact}`,
        );
      });
      const props = makeProps({ loadDocument, onToggleTask: vi.fn().mockResolvedValue(undefined) });
      const { rerender } = render(<RichDetailDrawer {...(props as never)} refreshGen={0} />);
      await openTasksTab();
      // 外部世代重載：tasks.md 載入懸掛（在途回應）。
      hangTasks = true;
      rerender(<RichDetailDrawer {...(props as never)} refreshGen={1} />);
      await waitFor(() => expect(pending.length).toBe(1));
      // 在途期間樂觀勾選任務 1。
      act(() => latestTL().onToggle!(1, true));
      await waitFor(() => expect(latestTL().markdown).toContain("- [x] 1.1 a"));
      // 舊回應（未勾選內容）此刻才到達——不得覆蓋樂觀狀態。
      await act(async () => {
        pending[0](TASKS_MD);
        await new Promise((r) => setTimeout(r, 10));
      });
      expect(latestTL().markdown).toContain("- [x] 1.1 a");
    });

    it("optimistically flips the checkbox before the write resolves", async () => {
      const gate = new Promise<void>(() => {}); // 永不 resolve＝寫回進行中
      const props = makeProps({ loadDocument: tasksLoader(), onToggleTask: vi.fn(() => gate) });
      render(<RichDetailDrawer {...(props as never)} refreshGen={0} />);
      await openTasksTab();
      act(() => latestTL().onToggle!(1, true));
      await waitFor(() => expect(latestTL().markdown).toContain("- [x] 1.1 a"));
      expect(latestTL().markdown).toContain("- [ ] 1.2 b");
    });

    it("rolls back and surfaces a one-line error when the write fails", async () => {
      const props = makeProps({
        loadDocument: tasksLoader(),
        onToggleTask: vi.fn().mockRejectedValue(new Error("disk full")),
      });
      render(<RichDetailDrawer {...(props as never)} refreshGen={0} />);
      await openTasksTab();
      act(() => latestTL().onToggle!(1, true));
      await waitFor(() => expect(screen.getByText(/寫回失敗/)).toBeTruthy());
      expect(screen.getByText(/disk full/)).toBeTruthy();
      expect(latestTL().markdown).toContain("- [ ] 1.1 a"); // 回滾至磁碟現況
    });

    it("keeps the list interactive during an in-flight single toggle", async () => {
      const releases: Array<() => void> = [];
      const props = makeProps({
        loadDocument: tasksLoader(),
        onToggleTask: vi.fn(() => new Promise<void>((r) => releases.push(r))),
      });
      render(<RichDetailDrawer {...(props as never)} refreshGen={0} />);
      await openTasksTab();
      act(() => latestTL().onToggle!(1, true));
      await waitFor(() => expect(latestTL().markdown).toContain("- [x] 1.1 a"));
      // 單發寫回進行中不鎖清單（busy 僅批次／拖放例外）。
      expect(latestTL().busy).toBe(false);
      // 第二勾不被擋、同樣立即反映。
      act(() => latestTL().onToggle!(2, true));
      expect(props.onToggleTask as Mock).toHaveBeenCalledTimes(2);
      await waitFor(() => expect(latestTL().markdown).toContain("- [x] 1.2 b"));
    });
  });

  it("shows who started the work and when, once the change is started", async () => {
    const props = makeProps({
      loadMeta: vi.fn(async () => ({
        created: "2026-07-05",
        createdBy: "MomoChen",
        createdWith: "claude",
        startedAt: "2026-07-06",
        startedBy: "Worker <w@example.com>",
        startedWith: "claude",
      })),
    });
    render(<RichDetailDrawer {...(props as never)} />);
    await waitFor(() => expect(screen.getByText(/Worker <w@example\.com>/)).toBeTruthy());
    expect(screen.getByText(/2026-07-06 開工/)).toBeTruthy();
  });

  it("hides the started row when the change has not been started", async () => {
    render(<RichDetailDrawer {...(makeProps() as never)} />);
    await waitFor(() => expect(screen.getByText("MomoChen")).toBeTruthy());
    expect(screen.queryByText(/開工/)).toBeNull();
  });

  it("fires onDelete when the delete action is clicked", async () => {
    const props = makeProps();
    render(<RichDetailDrawer {...(props as never)} />);
    // 先等初始 async 載入落地（loadMeta 的 MomoChen 為完成標記），避免 act 警告。
    await screen.findByText("MomoChen");
    fireEvent.click(screen.getByRole("button", { name: /刪除/ }));
    expect(props.onDelete).toHaveBeenCalledWith("desktop-shell-and-browser");
  });

  it("fires onRunVerb for analyze / archive actions", async () => {
    const props = makeProps();
    render(<RichDetailDrawer {...(props as never)} />);
    await screen.findByText("MomoChen");
    fireEvent.click(screen.getByRole("button", { name: /分析/ }));
    expect(props.onRunVerb).toHaveBeenCalledWith("analyze", "desktop-shell-and-browser");
    fireEvent.click(screen.getByRole("button", { name: /封存/ }));
    expect(props.onRunVerb).toHaveBeenCalledWith("archive", "desktop-shell-and-browser");
  });

  it("unavailable 停用分析與刪除並附繁中說明；封存照常（remote capability 缺口）", async () => {
    // Radix Sheet 會在 jsdom body 設 pointer-events:none；停用這項環境檢查，
    // 仍以實際 pointer enter/leave 驗證 tooltip trigger。
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    const props = makeProps({
      unavailable: {
        analyze: "此 server 尚未提供 validate/analyze——功能已停用",
        delete: "此 server 尚未提供刪除變更——功能已停用",
      },
    });
    render(<RichDetailDrawer {...(props as never)} />);
    await screen.findByText("MomoChen");
    const analyze = screen.getByRole("button", { name: /分析/ }) as HTMLButtonElement;
    expect(analyze.disabled).toBe(true);
    expect(analyze.title).toContain("validate/analyze");
    const analyzeTrigger = analyze.parentElement!;
    await user.hover(analyzeTrigger);
    await waitFor(() => expect(analyzeTrigger.getAttribute("data-state")).toContain("open"));
    expect(document.querySelector("[data-radix-popper-content-wrapper]")?.textContent).toContain("validate/analyze");
    const del = screen.getByRole("button", { name: /刪除/ }) as HTMLButtonElement;
    expect(del.disabled).toBe(true);
    expect(del.title).toContain("刪除變更");
    // archive 是直達端點——照常可點。
    fireEvent.click(screen.getByRole("button", { name: /封存/ }));
    expect(props.onRunVerb).toHaveBeenCalledWith("archive", "desktop-shell-and-browser");
  });

  it("delete unavailable 的繁中說明會顯示為可見 tooltip", async () => {
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    render(<RichDetailDrawer {...(makeProps({ unavailable: { delete: "此 server 尚未提供刪除變更——功能已停用" } }) as never)} />);
    await screen.findByText("MomoChen");
    const del = screen.getByRole("button", { name: /刪除/ }) as HTMLButtonElement;
    const deleteTrigger = del.parentElement!;
    await user.hover(deleteTrigger);
    await waitFor(() => expect(deleteTrigger.getAttribute("data-state")).toContain("open"));
    expect(document.querySelector("[data-radix-popper-content-wrapper]")?.textContent).toContain("刪除變更");
  });

  it("onMoveTask 缺席時 TaskList 收到 onReorder undefined（把手整段停用）", async () => {
    taskListProps.length = 0;
    const props = makeProps({ onToggleTask: vi.fn(), onSetAllTasks: vi.fn() });
    render(<RichDetailDrawer {...(props as never)} />);
    await screen.findByText("MomoChen");
    fireEvent.mouseDown(screen.getByRole("tab", { name: /任務/ }));
    await waitFor(() => expect(taskListProps.length).toBeGreaterThan(0));
    const latest = taskListProps.at(-1)!;
    expect(latest.onReorder).toBeUndefined();
    expect(latest.onToggle).toBeDefined();
    expect(latest.onSetAll).toBeDefined();
  });
});

// spec 需求「桌面 app 提供動詞操作面」：「分析」一鍵雙動詞的合併結果於抽屜內、
// 動作列近處呈現（design D1）；動作列不再提供獨立驗證鈕。
describe("抽屜內分析結果呈現", () => {
  const region = () => document.querySelector("[data-verb-result]") as HTMLElement | null;
  const report = {
    change_id: "x",
    dimensions: [{ dimension: "Ambiguity", status: "1 issue(s) found", finding_count: 1 }],
    findings: [
      { id: "AMB-1", dimension: "Ambiguity", severity: "Suggestion", location: "specs", summary: "缺具體範例的情境", recommendation: "r" },
    ],
    artifacts_analyzed: [],
    artifacts_missing: [],
  };

  it("動作列不提供獨立驗證鈕，僅分析／封存／刪除", async () => {
    render(<RichDetailDrawer {...(makeProps() as never)} />);
    await screen.findByText("MomoChen");
    expect(screen.queryByRole("button", { name: "驗證" })).toBeNull();
    expect(screen.getByRole("button", { name: /分析/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: /封存/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: /刪除/ })).toBeTruthy();
  });

  it("合併結果呈現結構驗證列、繁中維度摘要卡與發現卡", async () => {
    const props = makeProps({
      drawerVerb: {
        change: "desktop-shell-and-browser",
        validate: { valid: true, errors: [] },
        analyze: report,
      },
    });
    render(<RichDetailDrawer {...(props as never)} />);
    // 先等初始 async 載入落地（loadMeta 的 MomoChen 為完成標記），避免 act 警告。
    await screen.findByText("MomoChen");
    const r = region();
    expect(r).toBeTruthy();
    expect(within(r!).getByText("結構驗證通過")).toBeTruthy();
    expect(within(r!).getByText("模糊度")).toBeTruthy();
    expect(within(r!).getByText(/缺具體範例的情境/)).toBeTruthy();
  });

  it("結構驗證失敗於面板逐條呈現錯誤", async () => {
    const props = makeProps({
      drawerVerb: {
        change: "desktop-shell-and-browser",
        validate: { valid: false, errors: ["tasks.md: missing", "second error"] },
        analyze: report,
      },
    });
    render(<RichDetailDrawer {...(props as never)} />);
    await screen.findByText("MomoChen");
    const r = region();
    expect(within(r!).getByText(/結構驗證 2 個錯誤/)).toBeTruthy();
    expect(within(r!).getByText(/tasks\.md: missing/)).toBeTruthy();
    expect(within(r!).getByText(/second error/)).toBeTruthy();
  });

  it("執行失敗呈現 core 的單行錯誤", async () => {
    const props = makeProps({
      drawerVerb: { change: "desktop-shell-and-browser", error: "parse boom" },
    });
    render(<RichDetailDrawer {...(props as never)} />);
    await screen.findByText("MomoChen");
    expect(within(region()!).getByText(/parse boom/)).toBeTruthy();
  });

  it("動詞結果屬於別的 change 時不呈現", async () => {
    const props = makeProps({
      drawerVerb: { change: "some-other-change", validate: { valid: true, errors: [] } },
    });
    render(<RichDetailDrawer {...(props as never)} />);
    await screen.findByText("MomoChen");
    expect(region()).toBeNull();
  });

  // design D2：分析鈕為切換——結果開啟時再點收合（onClearVerb）、不重跑動詞。
  it("結果開啟時分析鈕 aria-pressed，再點呼叫 onClearVerb 而非重跑", async () => {
    const props = makeProps({
      drawerVerb: {
        change: "desktop-shell-and-browser",
        validate: { valid: true, errors: [] },
        analyze: report,
      },
      onClearVerb: vi.fn(),
    });
    render(<RichDetailDrawer {...(props as never)} />);
    await screen.findByText("MomoChen");
    const btn = screen.getByRole("button", { name: "分析" });
    expect(btn.getAttribute("aria-pressed")).toBe("true");
    fireEvent.click(btn);
    expect(props.onClearVerb).toHaveBeenCalledTimes(1);
    expect(props.onRunVerb).not.toHaveBeenCalled();
  });

  it("結果未開啟時分析鈕 aria-pressed=false，點按執行動詞", async () => {
    const props = makeProps({ onClearVerb: vi.fn() });
    render(<RichDetailDrawer {...(props as never)} />);
    await screen.findByText("MomoChen");
    const btn = screen.getByRole("button", { name: "分析" });
    expect(btn.getAttribute("aria-pressed")).toBe("false");
    fireEvent.click(btn);
    expect(props.onRunVerb).toHaveBeenCalledWith("analyze", "desktop-shell-and-browser");
    expect(props.onClearVerb).not.toHaveBeenCalled();
  });

  it("面板關閉鈕呼叫 onClearVerb 收合", async () => {
    const props = makeProps({
      drawerVerb: {
        change: "desktop-shell-and-browser",
        validate: { valid: true, errors: [] },
        analyze: report,
      },
      onClearVerb: vi.fn(),
    });
    render(<RichDetailDrawer {...(props as never)} />);
    await screen.findByText("MomoChen");
    fireEvent.click(screen.getByRole("button", { name: "關閉分析結果" }));
    expect(props.onClearVerb).toHaveBeenCalledTimes(1);
  });
});

// spec 需求「markdown 文件內容行寬有上限」（design D4）：抽屜分頁內容的捲動容器
// 內存在共用置中容器（行寬上限＋水平置中），包住分頁全部內容——SectionedDoc
// 區段標籤、任務清單與內文同欄對齊置中。
describe("變更抽屜閱讀欄置中", () => {
  it("捲動容器內有置中容器（w-full＋max-w-[96ch]＋mx-auto）且包住分頁內容", async () => {
    const props = makeProps();
    const { baseElement } = render(<RichDetailDrawer {...(props as never)} />);
    await waitFor(() => expect(screen.getByText(/doc for proposal.md/)).toBeTruthy());
    const col = baseElement.querySelector("[data-reading-column]") as HTMLElement;
    expect(col).toBeTruthy();
    expect(col.className).toContain("w-full");
    expect(col.className).toContain("max-w-[96ch]");
    expect(col.className).toContain("mx-auto");
    // 位於分頁內容的捲動容器內側。
    expect(col.parentElement?.className).toContain("overflow-y-auto");
    // 包住分頁全部內容：提案分頁內文在欄內。
    expect(within(col).getByText(/doc for proposal.md/)).toBeTruthy();
    // 任務分頁：任務清單同欄（TaskList stub 在欄內）。
    fireEvent.mouseDown(screen.getByRole("tab", { name: /任務/ }));
    expect(within(col).getByTestId("tasklist-stub")).toBeTruthy();
  });
});

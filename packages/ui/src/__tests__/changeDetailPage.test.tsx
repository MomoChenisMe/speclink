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

import { ChangeDetailPage } from "../components/ChangeDetailPage";
import type { DocumentTarget } from "../components/DetailHeader";
import { DELTA_COLORS } from "../components/DeltaBadges";
import type { ChangeItem } from "../adapter";

// 攔截 TaskList props 以驗證詳情頁→拖放回呼的接線（jsdom 無法真拖）。
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

function makeProps<T extends Record<string, unknown> = Record<never, never>>(over: T = {} as T) {
  return {
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

/** 規格文件裡需求段的標題（delta 檢視的需求段標題為 h3）。 */
const requirementTitles = (root: HTMLElement) =>
  Array.from(root.querySelectorAll("[data-requirement] h3")).map((h) => h.textContent);

/** 展開標題列右端的「⋯」選單（封存／刪除兩項）。 */
async function openMoreMenu() {
  const user = userEvent.setup({ pointerEventsCheck: 0 });
  await user.click(screen.getByRole("button", { name: "更多動作" }));
  await screen.findByRole("menu");
  return user;
}

/** 「⋯」選單項的停用原因（項目下方灰字）。 */
const menuReason = (item: HTMLElement) => item.querySelector("[data-menu-reason]")?.textContent ?? "";

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
    const result = render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
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
    // spec「章節標題為 16px 粗體」：色標標頭與章節標題同款、保留各色（desktop-change-doc-views design D6）。
    expect(added.className).toContain("text-base");
    expect(added.className).toContain("font-semibold");
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
    // ADDED／MODIFIED 的需求成需求段；REMOVED／RENAMED 照 prose。
    expect(requirementTitles(baseElement)).toEqual(["a", "c"]);
    // prose 的 h3 同時是本頁目錄的錨點：斷言限定在閱讀卡內文。
    const body = baseElement.querySelector("[data-doc-body]") as HTMLElement;
    expect(within(body).getByText(/Requirement: e/)).toBeTruthy();
  });

  it("無 delta 標記的規格整篇照常渲染（無色標標頭）", async () => {
    const { baseElement } = await openSpecsTab("# 正式規格\n\n### Requirement: plain\n本文。\n");
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
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await waitFor(() => expect(screen.getByText("動機內文。")).toBeTruthy());
    // 章節標籤同時是本頁目錄的錨點：斷言限定在閱讀卡內文。
    const body = within(document.querySelector("[data-doc-body]") as HTMLElement);
    expect(body.getByText("為什麼")).toBeTruthy();
    expect(body.getByText("變更內容")).toBeTruthy();
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
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await waitFor(() => expect(screen.getByText("（此 change 尚無提案內容）")).toBeTruthy());
    expect(screen.queryByText("載入中…")).toBeNull();
  });

  // 原意圖不變（載入中 ≠ 不存在）；呈現面自「載入中…」文字改為文件骨架
  // （spec「抽屜文件載入以 skeleton 呈現」，desktop-loading-skeleton-ux）。
  it("載入未完成（promise 未解決）時顯示文件骨架", async () => {
    const props = makeProps({
      loadDocument: vi.fn(() => new Promise<string>(() => {})),
      loadCapabilities: vi.fn(async () => []),
      loadMeta: vi.fn(() => new Promise<never>(() => {})),
    });
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await waitFor(() => expect(document.querySelector('[aria-busy="true"]')).toBeTruthy());
    expect(screen.queryByText("（此 change 尚無提案內容）")).toBeNull();
  });
});

describe("ChangeDetailPage", () => {
  it("renders metadata row (author, agent, task count) and progress", async () => {
    render(<ChangeDetailPage {...(makeProps() as Parameters<typeof ChangeDetailPage>[0])} />);
    await waitFor(() => expect(screen.getByText("MomoChen")).toBeTruthy());
    expect(screen.getByText(/claude/)).toBeTruthy();
    expect(screen.getAllByText(/30\/30/).length).toBeGreaterThan(0);
  });

  it("標頭複製名稱鈕寫入變更名，成功以 status 宣告「已複製」（共用 CopyButton）", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", { value: { writeText }, configurable: true });
    render(<ChangeDetailPage {...(makeProps() as Parameters<typeof ChangeDetailPage>[0])} />);
    fireEvent.click(screen.getByRole("button", { name: "複製名稱" }));
    expect(writeText).toHaveBeenCalledWith("desktop-shell-and-browser");
    await waitFor(() => expect(screen.getByRole("status").textContent).toBe("已複製"));
  });

  it("shows spec delta counts on the specs tab", async () => {
    render(<ChangeDetailPage {...(makeProps() as Parameters<typeof ChangeDetailPage>[0])} />);
    await waitFor(() => expect(screen.getByText(/\+2/)).toBeTruthy());
  });

  it("forwards a drag-reorder drop to onMoveTask; reload rides the refreshGen bump", async () => {
    const props = makeProps({ onMoveTask: vi.fn().mockResolvedValue(undefined) });
    const { rerender } = render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} refreshGen={0} />);
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
    rerender(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} refreshGen={1} />);
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
    const { rerender } = render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} refreshGen={0} />);
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
    rerender(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} refreshGen={1} />);
    await waitFor(() => expect(docCalls()).toBeGreaterThan(d0));
    await waitFor(() => expect(metaCalls()).toBeGreaterThan(m0));
    await waitFor(() => expect(screen.getByText(/開工 2026-07-07/)).toBeTruthy());
  });

  it("defers the refreshGen reload while a task interaction is in flight, catches up after", async () => {
    let release!: () => void;
    const gate = new Promise<void>((r) => {
      release = r;
    });
    const props = makeProps({ onToggleTask: vi.fn(() => gate) });
    const { rerender } = render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} refreshGen={0} />);
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
    rerender(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} refreshGen={1} />);
    await new Promise((r) => setTimeout(r, 25));
    // 讓路：互動未結束前外部世代不觸發重載，不打斷進行中操作。
    expect(tasksReads()).toBe(before);
    await act(async () => release()); // 互動結束 → 補載一次
    await waitFor(() => expect(tasksReads()).toBeGreaterThan(before));
  });

  it("defers the refreshGen reload while a drag gesture is active, catches up after drop", async () => {
    // spec：讓路涵蓋「拖曳排序進行中」（按住～放開），不只放開後的寫回等待。
    const props = makeProps();
    const { rerender } = render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} refreshGen={0} />);
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
    rerender(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} refreshGen={1} />);
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
    const { rerender } = render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} refreshGen={0} />);
    await waitFor(() => expect(pendingProposal.length).toBe(1)); // 初載請求（gen 0）
    rerender(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} refreshGen={1} />);
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
    const { rerender } = render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} refreshGen={0} />);
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
    rerender(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} refreshGen={1} />);
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
      const { rerender } = render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} refreshGen={0} />);
      await openTasksTab();
      // 外部世代重載：tasks.md 載入懸掛（在途回應）。
      hangTasks = true;
      rerender(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} refreshGen={1} />);
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
      render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} refreshGen={0} />);
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
      render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} refreshGen={0} />);
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
      render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} refreshGen={0} />);
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

  it("shows the started date once the change is started; starter identity lives in the tooltip", async () => {
    // change-drawer-header-redesign：出身列單行讓位——開工僅顯日期，開工者（含 email）收提示。
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
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await waitFor(() => expect(screen.getByText(/開工 2026-07-06/)).toBeTruthy());
    expect(screen.queryByText(/w@example\.com/)).toBeNull();
  });

  it("hides the started row when the change has not been started", async () => {
    render(<ChangeDetailPage {...(makeProps() as Parameters<typeof ChangeDetailPage>[0])} />);
    await waitFor(() => expect(screen.getByText("MomoChen")).toBeTruthy());
    expect(screen.queryByText(/開工/)).toBeNull();
  });

  it("fires onDelete when the delete item of the ⋯ menu is chosen", async () => {
    // 階段守門後刪除項僅提案中可選（archive-readiness-gating）——改用提案中 fixture。
    const props = makeProps({
      change: { name: "desktop-shell-and-browser", status: "in-progress", totalTasks: 30, completedTasks: 0 },
    });
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    // 先等初始 async 載入落地（loadMeta 的 MomoChen 為完成標記），避免 act 警告。
    await screen.findByText("MomoChen");
    const user = await openMoreMenu();
    await user.click(screen.getByRole("menuitem", { name: /刪除/ }));
    expect(props.onDelete).toHaveBeenCalledWith("desktop-shell-and-browser");
  });

  it("fires onRunVerb for analyze / archive actions", async () => {
    const props = makeProps();
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    fireEvent.click(screen.getByRole("button", { name: /分析/ }));
    expect(props.onRunVerb).toHaveBeenCalledWith("analyze", "desktop-shell-and-browser");
    fireEvent.click(screen.getByRole("button", { name: /封存/ }));
    expect(props.onRunVerb).toHaveBeenCalledWith("archive", "desktop-shell-and-browser");
  });

  it("unavailable 停用分析與刪除並附繁中說明；封存照常（remote capability 缺口）", async () => {
    // 以實際 pointer enter/leave 驗證 tooltip trigger。
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    const props = makeProps({
      unavailable: {
        analyze: "此 server 尚未提供 validate/analyze——功能已停用",
        delete: "此 server 尚未提供刪除變更——功能已停用",
      },
    });
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    const analyze = screen.getByRole("button", { name: /分析/ }) as HTMLButtonElement;
    expect(analyze.disabled).toBe(true);
    expect(analyze.title).toContain("validate/analyze");
    const analyzeTrigger = analyze.parentElement!;
    await user.hover(analyzeTrigger);
    await waitFor(() => expect(analyzeTrigger.getAttribute("data-state")).toContain("open"));
    expect(document.querySelector("[data-radix-popper-content-wrapper]")?.textContent).toContain("validate/analyze");
    // archive 是直達端點——已就緒時標題列主要鈕照常可點。
    fireEvent.click(screen.getByRole("button", { name: /封存/ }));
    expect(props.onRunVerb).toHaveBeenCalledWith("archive", "desktop-shell-and-browser");
    await openMoreMenu();
    const del = screen.getByRole("menuitem", { name: /刪除/ });
    expect(del.getAttribute("aria-disabled")).toBe("true");
    expect(menuReason(del)).toContain("刪除變更");
  });

  it("未認領的 change 呈現認領操作，點擊回呼帶 change 名（remote-claim-ownership D4）", async () => {
    const props = makeProps({ onClaim: vi.fn() });
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    fireEvent.click(screen.getByRole("button", { name: /認領/ }));
    expect(props.onClaim).toHaveBeenCalledWith("desktop-shell-and-browser");
  });

  it("已認領的 change 呈現認領人、不再提供認領操作", async () => {
    const props = makeProps({
      change: { ...change, claimedBy: "Alice <a@example.com>" },
      onClaim: vi.fn(),
    });
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    expect(screen.getByText(/Alice/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: /認領/ })).toBeNull();
  });

  it("onClaim 缺席時不渲染認領操作（本地分頁無認領面）", async () => {
    render(<ChangeDetailPage {...(makeProps() as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    expect(screen.queryByRole("button", { name: /認領/ })).toBeNull();
  });

  it("claim unavailable 停用認領鈕並附繁中說明（reader）", async () => {
    const props = makeProps({
      onClaim: vi.fn(),
      unavailable: { claim: "此帳號為唯讀成員，無法認領變更" },
    });
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    const claim = screen.getByRole("button", { name: /認領/ }) as HTMLButtonElement;
    expect(claim.disabled).toBe(true);
    expect(claim.title).toContain("唯讀成員");
    fireEvent.click(claim);
    expect(props.onClaim).not.toHaveBeenCalled();
  });

  it("onMoveTask 缺席時 TaskList 收到 onReorder undefined（把手整段停用）", async () => {
    taskListProps.length = 0;
    const props = makeProps({ onToggleTask: vi.fn(), onSetAllTasks: vi.fn() });
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    fireEvent.mouseDown(screen.getByRole("tab", { name: /任務/ }));
    await waitFor(() => expect(taskListProps.length).toBeGreaterThan(0));
    const latest = taskListProps.at(-1)!;
    expect(latest.onReorder).toBeUndefined();
    expect(latest.onToggle).toBeDefined();
    expect(latest.onSetAll).toBeDefined();
  });
});

// spec 需求「詳情頁的封存與刪除依階段守門」（archive-readiness-gating D4）：
// 「⋯」選單的封存項僅已就緒可選、刪除項僅提案中可選，停用時項目下方灰字說明原因；
// 已就緒時標題列另有「封存」主要鈕；unavailable（remote 能力缺失）原因優先於階段原因。
describe("詳情頁封存/刪除的階段守門", () => {
  const proposed: ChangeItem = { name: "prop-c", status: "in-progress", totalTasks: 11, completedTasks: 0 };
  const inProgress: ChangeItem = {
    name: "wip-c",
    status: "in-progress",
    totalTasks: 19,
    completedTasks: 5,
    startedAt: "2026-07-06",
  };
  const ready: ChangeItem = { name: "ready-c", status: "done", totalTasks: 30, completedTasks: 30 };

  it("非已就緒的封存項停用附任務進度與出路、標題列無封存鈕；已就緒時選單項與主要鈕照常可按", async () => {
    // spec scenario：提案中(0/11)與進行中(5/19)皆停用，項目下方載明進度。
    for (const [c, progress] of [
      [proposed, "0/11"],
      [inProgress, "5/19"],
    ] as const) {
      const r = render(<ChangeDetailPage {...(makeProps({ change: c }) as Parameters<typeof ChangeDetailPage>[0])} />);
      await screen.findByText("MomoChen");
      expect(screen.queryByRole("button", { name: /^封存$/ })).toBeNull();
      await openMoreMenu();
      const item = screen.getByRole("menuitem", { name: /封存/ });
      expect(item.getAttribute("aria-disabled")).toBe("true");
      expect(menuReason(item)).toContain(progress);
      expect(menuReason(item)).toContain("完成後才能封存");
      r.unmount();
    }
    const p3 = makeProps({ change: ready });
    render(<ChangeDetailPage {...(p3 as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    const primary = screen.getByRole("button", { name: /^封存$/ }) as HTMLButtonElement;
    expect(primary.disabled).toBe(false);
    fireEvent.click(primary);
    expect(p3.onRunVerb).toHaveBeenCalledWith("archive", "ready-c");
    const user = await openMoreMenu();
    const item = screen.getByRole("menuitem", { name: /封存/ });
    expect(item.getAttribute("aria-disabled")).toBeNull();
    expect(menuReason(item)).toBe("");
    await user.click(item);
    expect(p3.onRunVerb).toHaveBeenCalledTimes(2);
  });

  it("非提案中的刪除項停用附開工痕跡與退回出路；提案中照常可選；刪除項置底且為警示色", async () => {
    for (const c of [inProgress, ready]) {
      const r = render(<ChangeDetailPage {...(makeProps({ change: c }) as Parameters<typeof ChangeDetailPage>[0])} />);
      await screen.findByText("MomoChen");
      await openMoreMenu();
      const del = screen.getByRole("menuitem", { name: /刪除/ });
      expect(del.getAttribute("aria-disabled")).toBe("true");
      expect(menuReason(del)).toContain("開工痕跡");
      expect(menuReason(del)).toContain("退回提案中");
      r.unmount();
    }
    const p3 = makeProps({ change: proposed });
    render(<ChangeDetailPage {...(p3 as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    const user = await openMoreMenu();
    const items = screen.getAllByRole("menuitem");
    const del = items.at(-1)!;
    expect(del.textContent).toContain("刪除");
    expect(del.className).toContain("text-destructive");
    expect(screen.getByRole("menu").querySelector('[role="separator"]')).toBeTruthy();
    await user.click(del);
    expect(p3.onDelete).toHaveBeenCalledWith("prop-c");
  });

  it("unavailable 原因優先於階段原因（能力缺失比階段更硬）", async () => {
    const p = makeProps({
      change: inProgress,
      unavailable: {
        archive: "此 server 尚未提供封存——功能已停用",
        delete: "此 server 尚未提供刪除變更——功能已停用",
      },
    });
    render(<ChangeDetailPage {...(p as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    await openMoreMenu();
    const arch = screen.getByRole("menuitem", { name: /封存/ });
    expect(arch.getAttribute("aria-disabled")).toBe("true");
    expect(menuReason(arch)).toContain("server");
    expect(menuReason(arch)).not.toContain("5/19");
    const del = screen.getByRole("menuitem", { name: /刪除/ });
    expect(del.getAttribute("aria-disabled")).toBe("true");
    expect(menuReason(del)).toContain("server");
    expect(menuReason(del)).not.toContain("開工痕跡");
  });
});

// spec 需求「桌面 app 提供動詞操作面」：「分析」一鍵雙動詞的合併結果於詳情頁內、
// 「狀態」卡之下的分析結果卡呈現；動作區不提供獨立驗證鈕。
describe("詳情頁內分析結果呈現", () => {
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

  it("動作區不提供獨立驗證鈕：分析＋依階段鈕＋「⋯」（封存／刪除）", async () => {
    render(<ChangeDetailPage {...(makeProps() as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    expect(screen.queryByRole("button", { name: "驗證" })).toBeNull();
    expect(screen.getByRole("button", { name: /分析/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: /^封存$/ })).toBeTruthy();
    await openMoreMenu();
    expect(screen.getAllByRole("menuitem").map((el) => el.querySelector("[data-menu-label]")?.textContent)).toEqual([
      "封存",
      "刪除",
    ]);
  });

  it("分析結果卡渲染於閱讀區、閱讀卡之上；狀態列在左欄", async () => {
    const props = makeProps({
      detailVerb: {
        change: "desktop-shell-and-browser",
        validate: { valid: true, errors: [] },
        analyze: report,
      },
    });
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    const tree = document.querySelector("[data-detail-tree]")!;
    const reading = document.querySelector("[data-detail-reading]")!;
    const result = region()!;
    expect(tree.querySelector("[data-status-card]")).toBeTruthy();
    expect(reading.contains(result)).toBe(true);
    const header = reading.querySelector("[data-doc-header]")!;
    expect(result.compareDocumentPosition(header) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  });

  it("合併結果呈現結構驗證列、繁中維度摘要卡與發現卡", async () => {
    const props = makeProps({
      detailVerb: {
        change: "desktop-shell-and-browser",
        validate: { valid: true, errors: [] },
        analyze: report,
      },
    });
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
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
      detailVerb: {
        change: "desktop-shell-and-browser",
        validate: { valid: false, errors: ["tasks.md: missing", "second error"] },
        analyze: report,
      },
    });
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    const r = region();
    expect(within(r!).getByText(/結構驗證 2 個錯誤/)).toBeTruthy();
    expect(within(r!).getByText(/tasks\.md: missing/)).toBeTruthy();
    expect(within(r!).getByText(/second error/)).toBeTruthy();
  });

  it("執行失敗呈現 core 的單行錯誤", async () => {
    const props = makeProps({
      detailVerb: { change: "desktop-shell-and-browser", error: "parse boom" },
    });
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    expect(within(region()!).getByText(/parse boom/)).toBeTruthy();
  });

  it("動詞結果屬於別的 change 時不呈現", async () => {
    const props = makeProps({
      detailVerb: { change: "some-other-change", validate: { valid: true, errors: [] } },
    });
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    expect(region()).toBeNull();
  });

  // design D2：分析鈕為切換——結果開啟時再點收合（onClearVerb）、不重跑動詞。
  it("結果開啟時分析鈕 aria-pressed，再點呼叫 onClearVerb 而非重跑", async () => {
    const props = makeProps({
      detailVerb: {
        change: "desktop-shell-and-browser",
        validate: { valid: true, errors: [] },
        analyze: report,
      },
      onClearVerb: vi.fn(),
    });
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    const btn = screen.getByRole("button", { name: "分析" });
    expect(btn.getAttribute("aria-pressed")).toBe("true");
    fireEvent.click(btn);
    expect(props.onClearVerb).toHaveBeenCalledTimes(1);
    expect(props.onRunVerb).not.toHaveBeenCalled();
  });

  it("結果未開啟時分析鈕 aria-pressed=false，點按執行動詞", async () => {
    const props = makeProps({ onClearVerb: vi.fn() });
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    const btn = screen.getByRole("button", { name: "分析" });
    expect(btn.getAttribute("aria-pressed")).toBe("false");
    fireEvent.click(btn);
    expect(props.onRunVerb).toHaveBeenCalledWith("analyze", "desktop-shell-and-browser");
    expect(props.onClearVerb).not.toHaveBeenCalled();
  });

  it("面板關閉鈕呼叫 onClearVerb 收合", async () => {
    const props = makeProps({
      detailVerb: {
        change: "desktop-shell-and-browser",
        validate: { valid: true, errors: [] },
        analyze: report,
      },
      onClearVerb: vi.fn(),
    });
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    fireEvent.click(screen.getByRole("button", { name: "關閉分析結果" }));
    expect(props.onClearVerb).toHaveBeenCalledTimes(1);
  });
});

// spec 需求「markdown 文件內容行寬有上限」（design D4）與「詳情頁的文件樹、閱讀卡與本頁目錄」：
// 閱讀卡內文為共用置中容器（行寬上限＋水平置中），包住文件全部內容；閱讀區內容欄置中、寬 768px。
describe("變更詳情頁閱讀欄置中", () => {
  // spec「markdown 文件內容行寬有上限」（desktop-change-doc-views design D10）：閱讀卡內以 768px 卡欄為上限，不再另套 96ch。
  it("閱讀卡內文撐滿卡欄（w-full、無 96ch）且包住文件內容；閱讀區內容欄 768px 置中", async () => {
    const props = makeProps();
    const { baseElement } = render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await waitFor(() => expect(screen.getByText(/doc for proposal.md/)).toBeTruthy());
    const col = baseElement.querySelector("[data-doc-body]") as HTMLElement;
    expect(col).toBeTruthy();
    expect(col.className).toContain("w-full");
    expect(col.className).not.toContain("max-w-[96ch]");
    const reading = baseElement.querySelector("[data-detail-reading]") as HTMLElement;
    const frame = reading.firstElementChild as HTMLElement;
    expect(frame.className).toContain("max-w-[768px]");
    expect(frame.className).toContain("mx-auto");
    expect(frame.contains(col)).toBe(true);
    // 包住分頁全部內容：提案分頁內文在欄內。
    expect(within(col).getByText(/doc for proposal.md/)).toBeTruthy();
    // 任務分頁：任務清單同欄（TaskList stub 在欄內）。
    fireEvent.mouseDown(screen.getByRole("tab", { name: /任務/ }));
    expect(within(col).getByTestId("tasklist-stub")).toBeTruthy();
  });
});

// spec 需求「抽屜標頭標記受寬度約束且抽屜不產生水平捲軸」（change-drawer-header-redesign
// 改寫）：來源討論標記以 slug 直出（等寬字型、寬度上限截斷）、topic 降為主題化提示、
// 抽屜面板關閉水平捲動。
describe("變更抽屜來源討論標記（slug 直出）", () => {
  const LONG_TOPIC =
    "目前前端專案的設計，是完全純文字的web服務，我希望可以設計一下，因為有好幾個端點都要自己打URL才能夠進去，完全不符合人性，所以請你幫我重新設計一下";

  it("標記面為 slug 而非 topic：等寬字型、寬度上限、截斷", async () => {
    const props = makeProps({
      sourceDiscussions: [{ slug: "web-service-navigation-redesign", topic: LONG_TOPIC }],
      onOpenDiscussion: vi.fn(),
    });
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    const label = await waitFor(() => {
      const el = document.querySelector("[data-source-discussion-label]") as HTMLElement | null;
      expect(el).toBeTruthy();
      return el!;
    });
    // 籤面文字是 slug，topic 全文不出現在可視文字。
    expect(label.textContent).toBe("web-service-navigation-redesign");
    expect(screen.queryByText(LONG_TOPIC)).toBeNull();
    expect(label.className).toContain("truncate");
    const chip = label.closest("button") as HTMLElement;
    expect(chip.className).toContain("font-mono");
    expect(chip.className).toContain("max-w-[140px]");
  });

  it("主題化提示呈現 slug 與 topic（取代原生 title）", async () => {
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    const props = makeProps({
      sourceDiscussions: [{ slug: "alpha-ux", topic: "Alpha UX 討論" }],
      onOpenDiscussion: vi.fn(),
    });
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    const chip = await screen.findByRole("button", { name: /alpha-ux/ });
    await user.hover(chip);
    await waitFor(() => {
      const tip = document.querySelector("[data-radix-popper-content-wrapper]");
      expect(tip?.textContent).toContain("alpha-ux");
      expect(tip?.textContent).toContain("Alpha UX 討論");
    });
  });

  it("閱讀區可壓縮（min-w-0），出身行溢出裁切、來自列不撐寬左欄——頁不產生水平捲軸", async () => {
    const props = makeProps({
      sourceDiscussions: [{ slug: "web-service-navigation-redesign", topic: LONG_TOPIC }],
    });
    const { baseElement } = render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await waitFor(() =>
      expect(document.querySelector("[data-source-discussion-label]")).toBeTruthy(),
    );
    const reading = baseElement.querySelector("[data-detail-reading]") as HTMLElement;
    expect(reading.className).toContain("min-w-0");
    const row = baseElement.querySelector("[data-identity-provenance]") as HTMLElement;
    expect(row.className).toContain("overflow-hidden");
    expect(row.className).toContain("min-w-0");
    const from = baseElement.querySelector("[data-relation-from]") as HTMLElement;
    expect(from.className).toContain("min-w-0");
    expect(from.querySelector("[data-source-discussion-label]")).toBeTruthy();
  });

  it("點擊籤開啟對應討論", async () => {
    const onOpenDiscussion = vi.fn();
    const props = makeProps({
      sourceDiscussions: [{ slug: "alpha-ux", topic: "Alpha UX 討論" }],
      onOpenDiscussion,
    });
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    const chip = await screen.findByRole("button", { name: /alpha-ux/ });
    fireEvent.click(chip);
    expect(onOpenDiscussion).toHaveBeenCalledWith("alpha-ux");
  });
});

// spec 需求「變更的來源討論多值呈現」（change-drawer-header-redesign 改寫）：
// 固定顯示出身（清單第一份）討論籤，其餘收「+N」數字籤，點擊開浮層（slug 主行＋
// topic 副行）可跳討論；同源籤比照；單筆無 +N。
describe("來源討論多值呈現（+N 浮層）", () => {
  const FOUR = [
    { slug: "code-review-stage", topic: "審查站落地" },
    { slug: "tray-station-badges", topic: "系統匣站章" },
    { slug: "code-review-convergence-boundary", topic: "審查收斂邊界" },
    { slug: "apply-provenance-scope", topic: "apply 溯源範圍" },
  ];

  it("四筆來源討論：僅出身籤直出，+3 籤呈現溢出數，其餘 slug 不直出", async () => {
    const props = makeProps({ sourceDiscussions: FOUR, onOpenDiscussion: vi.fn() });
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByRole("button", { name: /code-review-stage/ });
    const overflow = screen.getByRole("button", { name: /其餘 3 份/ });
    expect(overflow.getAttribute("data-source-overflow")).not.toBeNull();
    expect(overflow.textContent).toBe("+3");
    // 其餘三筆不直接渲染於標頭。
    expect(screen.queryByRole("button", { name: /tray-station-badges/ })).toBeNull();
    expect(screen.queryByRole("button", { name: /apply-provenance-scope/ })).toBeNull();
  });

  it("點 +N 浮層列出其餘討論（slug 主行＋topic 副行），點項目跳討論並關閉浮層", async () => {
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    const onOpenDiscussion = vi.fn();
    const props = makeProps({ sourceDiscussions: FOUR, onOpenDiscussion });
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByRole("button", { name: /code-review-stage/ });
    await user.click(screen.getByRole("button", { name: /其餘 3 份/ }));
    const popover = await waitFor(() => {
      const el = document.querySelector("[data-source-overflow-list]") as HTMLElement | null;
      expect(el).toBeTruthy();
      return el!;
    });
    // slug 主行＋topic 副行。
    expect(popover.textContent).toContain("tray-station-badges");
    expect(popover.textContent).toContain("系統匣站章");
    expect(popover.textContent).toContain("code-review-convergence-boundary");
    expect(popover.textContent).toContain("apply-provenance-scope");
    await user.click(within(popover).getByRole("button", { name: /tray-station-badges/ }));
    expect(onOpenDiscussion).toHaveBeenCalledWith("tray-station-badges");
    await waitFor(() =>
      expect(document.querySelector("[data-source-overflow-list]")).toBeNull(),
    );
  });

  it("單筆來源討論無 +N 籤", async () => {
    const props = makeProps({
      sourceDiscussions: [{ slug: "alpha-ux", topic: "Alpha UX 討論" }],
    });
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByRole("button", { name: /alpha-ux/ });
    expect(document.querySelector("[data-source-overflow]")).toBeNull();
  });

  it("同源籤比照：首籤直出、其餘收 +N，浮層項點擊互跳", async () => {
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    const onOpenSibling = vi.fn();
    const props = makeProps({
      sourceDiscussions: [{ slug: "alpha-ux", topic: "Alpha UX 討論" }],
      siblingChanges: ["sib-one", "sib-two", "sib-three"],
      onOpenSibling,
    });
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByRole("button", { name: /sib-one/ });
    expect(screen.queryByRole("button", { name: /sib-two/ })).toBeNull();
    await user.click(screen.getByRole("button", { name: /其餘 2 份/ }));
    const popover = await waitFor(() => {
      const el = document.querySelector("[data-source-overflow-list]") as HTMLElement | null;
      expect(el).toBeTruthy();
      return el!;
    });
    await user.click(within(popover).getByRole("button", { name: /sib-two/ }));
    expect(onOpenSibling).toHaveBeenCalledWith("sib-two");
  });
});

// spec 需求「變更詳情頁的標頭與狀態卡」：左欄身分區（名稱＋複製＋籤、單行出身、關聯列）／
// 「文件」節點／「狀態」列三列（任務進度、品質關卡、排程）；變更層動作在標題列（無宿主時原地）。
describe("標頭與狀態卡", () => {
  const identity = () => document.querySelector("[data-detail-identity]") as HTMLElement;
  const actions = () => document.querySelector("[data-detail-actions]") as HTMLElement;
  const statusRow = (name: "tasks" | "quality" | "plan") =>
    document.querySelector(`[data-status-${name}]`) as HTMLElement;
  const flat = (el: Element | null) => (el?.textContent ?? "").replace(/\s+/g, " ").trim();

  it("左欄由上而下為身分區、「文件」節點、「狀態」列；身分區無任務計數", async () => {
    render(<ChangeDetailPage {...(makeProps() as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    const tree = document.querySelector("[data-detail-tree]") as HTMLElement;
    const order = [identity(), within(tree).getByRole("tablist"), within(tree).getByText("狀態"), tree.querySelector("[data-status-card]")!];
    for (let i = 1; i < order.length; i++) {
      expect(order[i - 1].compareDocumentPosition(order[i]) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    }
    expect(within(tree).getByText("文件")).toBeTruthy();
    expect(flat(identity())).not.toMatch(/30\/30/);
    // 階段籤：已就緒。
    expect(within(identity()).getByText("已就緒")).toBeTruthy();
    // 任務進度由「狀態」列與「任務」節點徽章承載。
    expect(flat(statusRow("tasks"))).toContain("30/30");
    expect(screen.getByRole("tab", { name: /任務/ }).textContent).toContain("30/30");
  });

  it("狀態列三列：任務進度 16/17、品質關卡兩枚帶日期章籤、排程第 2 波等 1 項", async () => {
    const c: ChangeItem = {
      ...change,
      totalTasks: 17,
      completedTasks: 16,
      reviewStatus: "reviewed",
      reviewedAt: "2026-08-04",
      reviewedBy: "MomoChen <momochenisme@gmail.com>",
      verifyStatus: "verified",
      verifiedAt: "2026-08-06",
      verifiedBy: "MomoChen <momochenisme@gmail.com>",
      wave: 2,
      blockedBy: ["add-a"],
    };
    render(<ChangeDetailPage {...(makeProps({ change: c }) as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    expect(within(statusRow("tasks")).getByText("任務進度")).toBeTruthy();
    expect(flat(statusRow("tasks"))).toContain("16/17");
    expect(flat(statusRow("tasks"))).not.toContain("94%");
    expect(statusRow("tasks").querySelector("[data-progress-track]")).toBeTruthy();
    const quality = flat(statusRow("quality"));
    expect(quality).toContain("品質關卡");
    expect(quality).toMatch(/已審查.*2026-08-04.*已驗證.*2026-08-06/);
    expect(quality).not.toContain("momochenisme@gmail.com");
    const plan = flat(statusRow("plan"));
    expect(plan).toContain("排程");
    expect(plan).toContain("第 2 波");
    expect(plan).toContain("等 1 項");
  });

  it("兩站皆 none：全頁無章籤、品質關卡列顯示「尚未經過品質關卡」；可以開工與缺 wave 的說明詞條", async () => {
    const r = render(
      <ChangeDetailPage {...(makeProps({ change: { ...change, wave: 1, blockedBy: [] } }) as Parameters<typeof ChangeDetailPage>[0])} />,
    );
    await screen.findByText("MomoChen");
    expect(document.querySelector("[data-review-row]")).toBeNull();
    expect(document.querySelector("[data-verify-row]")).toBeNull();
    expect(flat(statusRow("quality"))).toContain("尚未經過品質關卡");
    expect(flat(statusRow("plan"))).toContain("可以開工");
    r.unmount();
    // wave 缺席：沿用排程分頁的說明詞條（remote 無 plan／成環兩種）。
    const r2 = render(<ChangeDetailPage {...(makeProps() as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    expect(flat(statusRow("plan"))).toContain("此模式尚未提供排程資訊");
    r2.unmount();
    render(<ChangeDetailPage {...(makeProps({ change: { ...change, dependsOn: ["add-a"] } }) as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    expect(flat(statusRow("plan"))).toContain("依賴成環");
  });

  // spec Scenario「章籤與品質關卡列」：身分區只有階段籤；章籤只在「品質關卡」列，
  // 蓋章日期與含 email 完整識別收進該章籤的提示。
  it("身分區只有階段籤；品質關卡列的章籤提示帶日期與完整識別", async () => {
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    const stamped: ChangeItem = {
      ...change,
      reviewStatus: "reviewed",
      reviewedAt: "2026-08-04",
      reviewedBy: "MomoChen <momochenisme@gmail.com>",
      verifyStatus: "verified",
      verifiedAt: "2026-08-06",
      verifiedBy: "MomoChen <momochenisme@gmail.com>",
    };
    render(<ChangeDetailPage {...(makeProps({ change: stamped }) as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    const row = identity();
    const text = row.textContent ?? "";
    expect(within(row).getByText("已就緒")).toBeTruthy();
    expect(text).not.toContain("已審查");
    expect(text).not.toContain("已驗證");
    expect(row.querySelector("[data-review-row]")).toBeNull();
    expect(row.querySelector("[data-verify-row]")).toBeNull();
    expect(row.className).toContain("min-w-0");

    await user.hover(within(statusRow("quality")).getByText("已審查"));
    await waitFor(() => {
      const tip = document.querySelector("[data-radix-popper-content-wrapper]");
      expect(tip?.textContent).toContain("2026-08-04");
      expect(tip?.textContent).toContain("momochenisme@gmail.com");
    });
  });

  it("驗證章籤的提示帶該站自己的蓋章日期與完整識別", async () => {
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    const stamped: ChangeItem = {
      ...change,
      reviewStatus: "reviewed",
      reviewedAt: "2026-08-04",
      reviewedBy: "MomoChen <momochenisme@gmail.com>",
      verifyStatus: "verified",
      verifiedAt: "2026-08-06",
      verifiedBy: "Reviewer Two <two@example.com>",
    };
    render(<ChangeDetailPage {...(makeProps({ change: stamped }) as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    await user.hover(within(statusRow("quality")).getByText("已驗證"));
    await waitFor(() => {
      const tip = document.querySelector("[data-radix-popper-content-wrapper]");
      expect(tip?.textContent).toContain("2026-08-06");
      expect(tip?.textContent).toContain("two@example.com");
    });
  });

  it("有提示的章籤可用鍵盤聚焦，聚焦即出提示", async () => {
    const stamped: ChangeItem = {
      ...change,
      reviewStatus: "reviewed",
      reviewedAt: "2026-08-04",
      reviewedBy: "MomoChen <momochenisme@gmail.com>",
    };
    render(<ChangeDetailPage {...(makeProps({ change: stamped }) as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    const stamp = statusRow("quality").querySelector("[data-review-row]") as HTMLElement;
    // 提示觸發點必帶 data-state（進行中題以其缺席判定無提示）。
    expect(stamp.getAttribute("data-state")).toBe("closed");
    act(() => stamp.focus());
    expect(document.activeElement).toBe(stamp);
    await waitFor(() => {
      const tip = document.querySelector("[data-radix-popper-content-wrapper]");
      expect(tip?.textContent).toContain("2026-08-04");
      expect(tip?.textContent).toContain("momochenisme@gmail.com");
    });
  });

  it("進行中兩站（inReview／inVerify）：身分區無章籤；品質關卡列僅狀態詞、無日期、無提示", async () => {
    const running: ChangeItem = {
      ...change,
      reviewStatus: "inReview",
      verifyStatus: "inVerify",
    };
    render(<ChangeDetailPage {...(makeProps({ change: running }) as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    expect(identity().textContent).not.toContain("審查中");
    expect(identity().textContent).not.toContain("驗證中");
    const quality = statusRow("quality");
    expect(quality.textContent).toContain("審查中");
    expect(quality.textContent).toContain("驗證中");
    expect(quality.textContent).not.toMatch(/\d{4}-\d{2}-\d{2}/);
    // 進行中無戳記：兩站章籤都不是提示觸發點（無 data-state），也不進 Tab 序。
    for (const mark of ["[data-review-row]", "[data-verify-row]"]) {
      const stamp = quality.querySelector(mark)!;
      expect(stamp.hasAttribute("data-state")).toBe(false);
      expect(stamp.hasAttribute("tabindex")).toBe(false);
    }
  });

  it("已就緒時動作區有「封存」主要鈕；進行中為「退回提案中」", async () => {
    const r = render(<ChangeDetailPage {...(makeProps({ onRevert: vi.fn() }) as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    const archive = within(actions()).getByRole("button", { name: /^封存$/ });
    expect(archive.className).toContain("bg-primary");
    expect(within(actions()).queryByRole("button", { name: /退回提案中/ })).toBeNull();
    r.unmount();
    render(
      <ChangeDetailPage
        {...(makeProps({
          change: { ...change, completedTasks: 3, startedAt: "2026-07-30" },
          onRevert: vi.fn(),
        }) as Parameters<typeof ChangeDetailPage>[0])}
      />,
    );
    await screen.findByText("MomoChen");
    expect(within(actions()).getByRole("button", { name: /退回提案中/ })).toBeTruthy();
    expect(within(actions()).queryByRole("button", { name: /^封存$/ })).toBeNull();
  });

  // spec Scenario「標題列的變更層動作」：無宿主插槽時原地渲染；順序為分析、主要鈕、⋯，「⋯」為 outline。
  it("動作鈕在無 provider 時原地渲染且「⋯」為 outline、順序為分析／封存／⋯", async () => {
    render(<ChangeDetailPage {...(makeProps() as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    const buttons = within(actions()).getAllByRole("button");
    expect(buttons.map((b) => b.textContent?.trim() || b.getAttribute("aria-label"))).toEqual(["分析", "封存", "更多動作"]);
    const more = within(actions()).getByRole("button", { name: "更多動作" });
    expect(more.className).toContain("border");
    expect(more.className).not.toContain("bg-muted");
    // 動作鈕不在左欄與閱讀卡內。
    expect(document.querySelector("[data-detail-tree]")!.contains(more)).toBe(false);
    expect(document.querySelector("[data-doc-header]")!.contains(more)).toBe(false);
  });

  it("建立者 email 不直出，僅顯名字；提示保完整識別", async () => {
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    const props = makeProps({
      loadMeta: vi.fn(async () => ({
        created: "2026-07-05",
        createdBy: "MomoChen <momochenisme@gmail.com>",
        createdWith: "claude",
      })),
    });
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    const name = await screen.findByText("MomoChen");
    expect(screen.queryByText(/momochenisme@gmail\.com/)).toBeNull();
    // 首字母圓標為靜態 metadata：走中性（五處頭像同款）。
    const avatar = name.querySelector("span") as HTMLElement;
    expect(avatar.textContent).toBe("M");
    expect(avatar.className).toContain("bg-muted");
    expect(avatar.className).not.toContain("bg-primary");
    await user.hover(screen.getByText("MomoChen"));
    await waitFor(() => {
      const tip = document.querySelector("[data-radix-popper-content-wrapper]");
      expect(tip?.textContent).toContain("momochenisme@gmail.com");
    });
  });

  it("worktree 映射時出身列顯示分支與路徑，無映射時整段缺席", async () => {
    // spec worktree-overlay「desktop 看板的 worktree 呈現」：抽屜顯示分支名與
    // worktree 路徑（OS 原生形式）。
    const wtChange: ChangeItem = {
      ...change,
      worktree: { branch: "speclink/demo", path: "/work/speclink.worktrees/demo" },
    };
    const { baseElement, unmount } = render(
      <ChangeDetailPage {...(makeProps({ change: wtChange }) as Parameters<typeof ChangeDetailPage>[0])} />,
    );
    await screen.findByText("MomoChen");
    const row = baseElement.querySelector("[data-identity-provenance]") as HTMLElement;
    const branch = within(row).getByText(/speclink\/demo/);
    expect(branch).toBeTruthy();
    expect(within(row).getByTitle("/work/speclink.worktrees/demo")).toBeTruthy();
    // spec「worktree 標示以藍呈現」下半句：抽屜的分支與路徑維持中性——掃視層
    // （卡片）搶眼、閱讀層（抽屜出身列）安靜。
    expect(row.className).toContain("text-muted-foreground");
    expect(branch.className).not.toContain("status-progress");
    unmount();

    const plain = render(<ChangeDetailPage {...(makeProps() as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    const plainRow = plain.baseElement.querySelector("[data-identity-provenance]") as HTMLElement;
    expect(within(plainRow).queryByText(/speclink\//)).toBeNull();
  });

  it("出身行為不折行單行容器、關聯列各自一行（4 筆來源討論＋開工＋同源）", async () => {
    const props = makeProps({
      loadMeta: vi.fn(async () => ({
        created: "2026-07-05",
        createdBy: "MomoChen <momochenisme@gmail.com>",
        createdWith: "claude",
        startedAt: "2026-08-03",
        startedBy: "MomoChen <momochenisme@gmail.com>",
      })),
      sourceDiscussions: [
        { slug: "code-review-stage", topic: "審查站落地" },
        { slug: "tray-station-badges", topic: "系統匣站章" },
        { slug: "code-review-convergence-boundary", topic: "審查收斂邊界" },
        { slug: "apply-provenance-scope", topic: "apply 溯源範圍" },
      ],
      siblingChanges: ["sib-one", "sib-two"],
    });
    const { baseElement } = render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    const row = baseElement.querySelector("[data-identity-provenance]") as HTMLElement;
    expect(row).toBeTruthy();
    // 恆定單行：不折行、溢出裁切兜底。
    expect(row.className).not.toContain("flex-wrap");
    expect(row.className).toContain("overflow-hidden");
    expect(row.className).toContain("whitespace-nowrap");
    expect(within(row).getByText("MomoChen")).toBeTruthy();
    expect(within(row).getByText(/開工 2026-08-03/)).toBeTruthy();
    // 關聯列各自一行：「來自」首籤＋「+3」、「同源」首籤＋「+1」、「規格」能力籤。
    const from = baseElement.querySelector("[data-relation-from]") as HTMLElement;
    expect(within(from).getByText("來自")).toBeTruthy();
    expect(within(from).getByRole("button", { name: /code-review-stage/ })).toBeTruthy();
    expect(within(from).getByRole("button", { name: /其餘 3 份/ }).textContent).toBe("+3");
    const sibling = baseElement.querySelector("[data-relation-sibling]") as HTMLElement;
    expect(within(sibling).getByText("同源")).toBeTruthy();
    expect(within(sibling).getByRole("button", { name: /sib-one/ })).toBeTruthy();
    const specs = await waitFor(() => {
      const el = baseElement.querySelector("[data-relation-specs]") as HTMLElement | null;
      expect(el).toBeTruthy();
      return el!;
    });
    expect(within(specs).getByRole("button", { name: "desktop-app" })).toBeTruthy();
  });
});

// spec「進行中變更可自看板退回提案中」的抽屜半邊:退回動作僅於派生進行中出現。
describe("退回提案中動作(標題列動作區)", () => {
  it("派生進行中的變更顯示退回動作,點擊觸發 onRevert", async () => {
    const onRevert = vi.fn();
    const inProgress: ChangeItem = {
      name: "oops-started",
      status: "in-progress",
      totalTasks: 10,
      completedTasks: 0,
      startedAt: "2026-07-30",
    };
    render(<ChangeDetailPage {...(makeProps({ change: inProgress, onRevert }) as Parameters<typeof ChangeDetailPage>[0])} />);
    const btn = await screen.findByRole("button", { name: /退回提案中/ });
    fireEvent.click(btn);
    expect(onRevert).toHaveBeenCalledWith("oops-started");
  });

  it("提案中的變更不顯示退回動作", async () => {
    const proposed: ChangeItem = {
      name: "fresh-proposal",
      status: "in-progress",
      totalTasks: 10,
      completedTasks: 0,
    };
    render(<ChangeDetailPage {...(makeProps({ change: proposed, onRevert: vi.fn() }) as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    expect(screen.queryByRole("button", { name: /退回提案中/ })).toBeNull();
  });

  it("已就緒(全完成)的變更不顯示退回動作", async () => {
    // 既有 fixture change 為 30/30 全完成——派生已就緒。
    render(<ChangeDetailPage {...(makeProps({ onRevert: vi.fn() }) as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    expect(screen.queryByRole("button", { name: /退回提案中/ })).toBeNull();
  });
});

// spec「抽屜文件載入以 skeleton 呈現」（design D3）：三態分流——載入中畫骨架、
// 檔案不存在才出空態文案、有內容走既有渲染。載入中不得顯示「沒有文件」。
describe("變更抽屜文件三態", () => {
  /** 永不完成的載入：停在「載入中」態供斷言。 */
  const pending = () => new Promise<never>(() => {});

  async function openTab(name: RegExp) {
    await waitFor(() => screen.getByRole("tab", { name }));
    fireEvent.mouseDown(screen.getByRole("tab", { name }));
  }

  it("載入中 → 四個分頁皆為文件骨架，且無任何空態文案", async () => {
    render(
      <ChangeDetailPage
        {...(makeProps({
          loadDocument: vi.fn(pending),
          loadCapabilities: vi.fn(pending),
        }) as Parameters<typeof ChangeDetailPage>[0])}
      />,
    );
    // 提案分頁（預設）
    await waitFor(() => expect(document.querySelector('[aria-busy="true"]')).toBeTruthy());
    expect(screen.queryByText("（此 change 尚無提案內容）")).toBeNull();

    for (const [tab, emptyText] of [
      [/設計/, "（此 change 無設計文件）"],
      [/規格/, "（此 change 無 delta 規格）"],
    ] as const) {
      await openTab(tab);
      expect(document.querySelector('[aria-busy="true"]')).toBeTruthy();
      expect(screen.queryByText(emptyText)).toBeNull();
    }
  });

  it("載入完成且檔案不存在 → 各分頁顯示既有空態文案，無骨架", async () => {
    render(
      <ChangeDetailPage
        {...(makeProps({
          loadDocument: vi.fn(async () => null),
          loadCapabilities: vi.fn(async () => []),
        }) as Parameters<typeof ChangeDetailPage>[0])}
      />,
    );
    await screen.findByText("（此 change 尚無提案內容）");
    expect(document.querySelector('[aria-busy="true"]')).toBeNull();

    await openTab(/設計/);
    expect(screen.getByText("（此 change 無設計文件）")).toBeTruthy();

    await openTab(/規格/);
    expect(screen.getByText("（此 change 無 delta 規格）")).toBeTruthy();
  });

  it("載入完成且有內容 → 顯示內容，無骨架", async () => {
    render(<ChangeDetailPage {...(makeProps() as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText(/doc for proposal.md/);
    expect(document.querySelector('[aria-busy="true"]')).toBeNull();
  });
});

// 三態的 undefined 是「還在載」，不是「載不到」。capability 載入失敗時若停在
// undefined，規格分頁會永久掛骨架——改動前停在 {} 至少會誠實說「無 delta 規格」。
describe("規格分頁載入失敗的收斂", () => {
  it("loadCapabilities 失敗 → 收斂為空態文案，不永久掛骨架", async () => {
    render(
      <ChangeDetailPage
        {...(makeProps({ loadCapabilities: vi.fn(async () => Promise.reject("讀不到")) }) as Parameters<typeof ChangeDetailPage>[0])}
      />,
    );
    await waitFor(() => screen.getByRole("tab", { name: /規格/ }));
    fireEvent.mouseDown(screen.getByRole("tab", { name: /規格/ }));
    await waitFor(() => expect(screen.getByText("（此 change 無 delta 規格）")).toBeTruthy());
    expect(document.querySelector('[aria-busy="true"]')).toBeNull();
  });

  it("文件首載失敗 → 落到既有空態文案，不停在骨架", async () => {
    render(
      <ChangeDetailPage
        {...(makeProps({ loadDocument: vi.fn().mockRejectedValue(new Error("offline")) }) as Parameters<typeof ChangeDetailPage>[0])}
      />,
    );
    await waitFor(() => expect(screen.getByText("（此 change 尚無提案內容）")).toBeTruthy());
    expect(document.querySelector('[aria-busy="true"]')).toBeNull();
  });

  it("規格分頁重載失敗 → 保留已載內容，不抹成假空態", async () => {
    let fail = false;
    const loadCapabilities = vi.fn(async () => {
      if (fail) throw new Error("offline");
      return ["desktop-app"];
    });
    const props = makeProps({ loadCapabilities });
    const { rerender } = render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} refreshGen={0} />);
    await waitFor(() => screen.getByRole("tab", { name: /規格/ }));
    fireEvent.mouseDown(screen.getByRole("tab", { name: /規格/ }));
    await waitFor(() => expect(requirementTitles(document.body)).toContain("a"));

    fail = true;
    rerender(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} refreshGen={1} />);
    await new Promise((r) => setTimeout(r, 25));
    // 抽屜開著時一次短暫失敗不得把已載好的內容抹成「無 delta 規格」。
    expect(requirementTitles(document.body)).toContain("a");
    expect(screen.queryByText("（此 change 無 delta 規格）")).toBeNull();
  });
});

// spec desktop-app「詳情抽屜的工單分頁」：條件式「審查」「驗證」分頁（排程之後、
// 審查在前）、分頁出現時以 (change, station) 載入、隨 refreshGen 重載、狀態翻為
// 非進行中即退場並切回「提案」、載入失敗與無工單皆為空態文案而非錯誤。
describe("詳情抽屜的工單分頁", () => {
  const HASH = `sha256:${"a".repeat(64)}`;
  const SCOPE = ["src/a.rs", "src/b.rs", "src/c.rs"];
  const REVIEW_TICKET = {
    rounds: [
      {
        index: 1,
        phase: "discovery",
        patchHash: HASH,
        scope: SCOPE,
        findings: [
          { severity: "WARNING", path: "src/a.rs", text: "Correctness: 未處理空清單" },
          { severity: "WARNING", path: "src/b.rs", text: "Standards: 命名不一致" },
        ],
      },
      {
        index: 2,
        phase: "validation",
        patchHash: HASH,
        scope: SCOPE,
        findings: [
          { severity: "WARNING", path: "src/b.rs", text: "Standards: 命名不一致 (accepted)" },
          { severity: "SUGGESTION", path: "src/c.rs", text: "Style: 可簡化" },
        ],
      },
    ],
  };
  const VERIFY_TICKET = {
    rounds: [
      {
        index: 1,
        phase: "discovery",
        patchHash: HASH,
        scope: ["src/a.rs"],
        findings: [{ severity: "CRITICAL", path: "src/a.rs", text: "Spec: 缺 Scenario 覆蓋" }],
      },
    ],
  };
  const x = (over: Partial<ChangeItem>): ChangeItem => ({
    name: "x",
    status: "in-progress",
    totalTasks: 2,
    completedTasks: 1,
    ...over,
  });
  const tabNames = () => screen.getAllByRole("tab").map((el) => (el.textContent ?? "").replace(/\s+/g, " ").trim());
  // 段標題列已撤（desktop-change-doc-views design D8）：站名在閱讀卡標題、輪數與階段詞在籤槽、計數在數字磚。
  const flat = (el: Element | null) => (el?.textContent ?? "").replace(/\s+/g, " ").trim();
  const docHead = () => ({
    title: flat(document.querySelector("[data-doc-header] h2")),
    badges: flat(document.querySelector("[data-doc-badges]")),
    tiles: Array.from(document.querySelectorAll("[data-doc-tile]")).map(
      (tile) => `${flat(tile.lastElementChild)} ${flat(tile.firstElementChild)}`,
    ),
  });
  async function openTab(name: RegExp) {
    await waitFor(() => screen.getByRole("tab", { name }));
    fireEvent.mouseDown(screen.getByRole("tab", { name }));
  }

  it("inReview → 「審查」分頁排在排程之後，工單以 (x, review) 載入並結構化呈現", async () => {
    const loadStationTicket = vi.fn(async () => REVIEW_TICKET);
    render(
      <ChangeDetailPage
        {...(makeProps({ change: x({ reviewStatus: "inReview", verifyStatus: "none" }), loadStationTicket }) as Parameters<typeof ChangeDetailPage>[0])}
      />,
    );
    await waitFor(() => screen.getByRole("tab", { name: /審查/ }));
    const names = tabNames();
    expect(names).toHaveLength(6);
    expect(names[4]).toMatch(/排程/);
    expect(names[5]).toMatch(/審查/);
    expect(screen.queryByRole("tab", { name: /驗證/ })).toBeNull();
    // 分頁出現即載入（不必等使用者切過去）。
    await waitFor(() => expect(loadStationTicket).toHaveBeenCalledWith("x", "review"));

    await openTab(/審查/);
    await waitFor(() =>
      expect(docHead()).toEqual({
        title: "審查",
        badges: "第 2 輪複驗",
        tiles: ["輪 2", "CRITICAL（末輪） 0", "WARNING（末輪） 1", "SUGGESTION（末輪） 1"],
      }),
    );
    // 末輪仍有 WARNING：不出「可以蓋章」提示框。
    expect(document.querySelector("[data-doc-callout]")).toBeNull();
    const r2 = document.querySelector('[data-ticket-round="2"]') as HTMLElement;
    expect(r2.querySelectorAll("[data-ticket-finding]")).toHaveLength(2);
    expect(within(r2).getByText("已接受")).toBeTruthy();
    expect(within(r2).queryByText(/\(accepted\)/)).toBeNull();
    const r1 = document.querySelector('[data-ticket-round="1"]') as HTMLElement;
    expect(r1.querySelector("[data-ticket-round-toggle]")?.getAttribute("aria-expanded")).toBe("false");
    expect((r1.querySelector("[data-ticket-round-header]")?.textContent ?? "").replace(/\s+/g, " ")).toContain(
      "第 1 輪 · 首輪 · 範圍 3 檔 · 2 條",
    );
  });

  it("審查／驗證節點的徽章為末輪序號；排程節點有 wave 時帶「第 N 波」", async () => {
    const loadStationTicket = vi.fn(async (_c: string, station: string) =>
      station === "review" ? REVIEW_TICKET : VERIFY_TICKET,
    );
    render(
      <ChangeDetailPage
        {...(makeProps({
          change: x({ reviewStatus: "inReview", verifyStatus: "inVerify", wave: 3, blockedBy: [] }),
          loadStationTicket,
        }) as Parameters<typeof ChangeDetailPage>[0])}
      />,
    );
    await waitFor(() => expect(screen.getByRole("tab", { name: /審查/ }).textContent).toContain("第 2 輪"));
    expect(screen.getByRole("tab", { name: /驗證/ }).textContent).toContain("第 1 輪");
    expect(screen.getByRole("tab", { name: /排程/ }).textContent).toContain("第 3 波");
  });

  it("兩站皆進行中 → 「審查」「驗證」依序出現、各載各站、內容互不混入", async () => {
    const loadStationTicket = vi.fn(async (_c: string, station: string) =>
      station === "review" ? REVIEW_TICKET : VERIFY_TICKET,
    );
    render(
      <ChangeDetailPage
        {...(makeProps({ change: x({ reviewStatus: "inReview", verifyStatus: "inVerify" }), loadStationTicket }) as Parameters<typeof ChangeDetailPage>[0])}
      />,
    );
    await waitFor(() => screen.getByRole("tab", { name: /驗證/ }));
    const names = tabNames();
    expect(names).toHaveLength(7);
    expect(names[4]).toMatch(/排程/);
    expect(names[5]).toMatch(/審查/);
    expect(names[6]).toMatch(/驗證/);
    await waitFor(() => expect(loadStationTicket).toHaveBeenCalledTimes(2));
    expect(loadStationTicket).toHaveBeenCalledWith("x", "review");
    expect(loadStationTicket).toHaveBeenCalledWith("x", "verify");

    await openTab(/驗證/);
    await waitFor(() =>
      expect(docHead()).toEqual({
        title: "驗證",
        badges: "第 1 輪首輪",
        tiles: ["輪 1", "CRITICAL（末輪） 1", "WARNING（末輪） 0", "SUGGESTION（末輪） 0"],
      }),
    );
    expect(document.querySelectorAll("[data-ticket-view]")).toHaveLength(1);
    await openTab(/審查/);
    await waitFor(() => expect(docHead()).toMatchObject({ title: "審查", badges: "第 2 輪複驗" }));
  });

  it("reviewed／none → 兩分頁缺席，載入器未被呼叫", async () => {
    const loadStationTicket = vi.fn(async () => REVIEW_TICKET);
    render(
      <ChangeDetailPage
        {...(makeProps({ change: x({ reviewStatus: "reviewed", verifyStatus: "none" }), loadStationTicket }) as Parameters<typeof ChangeDetailPage>[0])}
      />,
    );
    await waitFor(() => screen.getByRole("tab", { name: /排程/ }));
    await screen.findByText(/doc for proposal.md/);
    expect(tabNames()).toHaveLength(5);
    expect(screen.queryByRole("tab", { name: /審查/ })).toBeNull();
    expect(screen.queryByRole("tab", { name: /驗證/ })).toBeNull();
    expect(loadStationTicket).not.toHaveBeenCalled();
  });

  it("停在「審查」分頁時狀態翻為 reviewed → 分頁消失、退回「提案」", async () => {
    const loadStationTicket = vi.fn(async () => REVIEW_TICKET);
    const props = makeProps({ change: x({ reviewStatus: "inReview", verifyStatus: "none" }), loadStationTicket });
    const { rerender } = render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await openTab(/審查/);
    await waitFor(() => expect(docHead()).toMatchObject({ title: "審查", badges: "第 2 輪複驗" }));
    expect(screen.getByRole("tab", { name: /審查/ }).getAttribute("data-state")).toBe("active");

    rerender(
      <ChangeDetailPage
        {...(props as Parameters<typeof ChangeDetailPage>[0])}
        change={x({ reviewStatus: "reviewed", verifyStatus: "none", reviewedAt: "2026-09-22", reviewedBy: "Rev <r@example.com>" })}
      />,
    );
    await waitFor(() => expect(screen.queryByRole("tab", { name: /審查/ })).toBeNull());
    expect(screen.getByRole("tab", { name: /提案/ }).getAttribute("data-state")).toBe("active");
    await screen.findByText(/doc for proposal.md/);
    expect(document.querySelector("[data-ticket-view]")).toBeNull();
    // 「品質關卡」列的章籤改為「已審查」。
    expect(document.querySelector("[data-status-quality] [data-review-row]")!.textContent).toContain("已審查");
  });

  it("refreshGen 遞增 → 工單重載（追加輪後段標題更新）", async () => {
    let current = VERIFY_TICKET;
    const loadStationTicket = vi.fn(async () => current);
    const props = makeProps({ change: x({ reviewStatus: "inReview", verifyStatus: "none" }), loadStationTicket });
    const { rerender } = render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await openTab(/審查/);
    await waitFor(() => expect(docHead().badges).toMatch(/^第 1 輪/));
    expect(loadStationTicket).toHaveBeenCalledTimes(1);

    current = REVIEW_TICKET;
    rerender(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} refreshGen={1} />);
    await waitFor(() => expect(loadStationTicket).toHaveBeenCalledTimes(2));
    await waitFor(() => expect(docHead()).toMatchObject({ badges: "第 2 輪複驗", tiles: expect.arrayContaining(["輪 2"]) }));
    // spec Scenario「工單追加輪後分頁更新」THEN：新末輪展開、舊輪收合。
    const expanded = (index: number) =>
      document
        .querySelector(`[data-ticket-round="${index}"] [data-ticket-round-toggle]`)
        ?.getAttribute("aria-expanded");
    expect(expanded(2)).toBe("true");
    expect(expanded(1)).toBe("false");
  });

  it("換 change 時在途的舊工單回應不落地：新 change 進入 inReview 先見骨架", async () => {
    let settleX: (t: typeof REVIEW_TICKET) => void = () => {};
    const loadStationTicket = vi.fn((change: string) =>
      change === "x"
        ? new Promise<typeof REVIEW_TICKET>((resolve) => {
            settleX = resolve;
          })
        : new Promise<never>(() => {}),
    );
    const props = makeProps({ change: x({ reviewStatus: "inReview", verifyStatus: "none" }), loadStationTicket });
    const { rerender } = render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await waitFor(() => expect(loadStationTicket).toHaveBeenCalledWith("x", "review"));

    // 切到沒有工單的 y，x 的回應才到——必須被丟棄。
    rerender(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} change={x({ name: "y", reviewStatus: "none" })} />);
    await waitFor(() => expect(screen.queryByRole("tab", { name: /審查/ })).toBeNull());
    settleX(REVIEW_TICKET);
    await screen.findByText(/doc for proposal.md/);

    // y 進入 inReview：載入在途，分頁該是骨架，不得先閃出 x 的工單。
    rerender(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} change={x({ name: "y", reviewStatus: "inReview" })} />);
    await openTab(/審查/);
    await waitFor(() => expect(document.querySelector('[aria-busy="true"]')).toBeTruthy());
    expect(document.querySelector("[data-ticket-view]")).toBeNull();
  });

  it("載入中 → 骨架；載入器 reject 或回 null → 「工單尚未抵達或已被刪除」，非錯誤", async () => {
    const pending = () => new Promise<never>(() => {});
    const first = render(
      <ChangeDetailPage
        {...(makeProps({ change: x({ reviewStatus: "inReview" }), loadStationTicket: vi.fn(pending) }) as Parameters<typeof ChangeDetailPage>[0])}
      />,
    );
    await openTab(/審查/);
    await waitFor(() => expect(document.querySelector('[aria-busy="true"]')).toBeTruthy());
    expect(screen.queryByText("工單尚未抵達或已被刪除")).toBeNull();
    first.unmount();

    const second = render(
      <ChangeDetailPage
        {...(makeProps({
          change: x({ reviewStatus: "inReview" }),
          loadStationTicket: vi.fn(async () => Promise.reject(new Error("offline"))),
        }) as Parameters<typeof ChangeDetailPage>[0])}
      />,
    );
    await openTab(/審查/);
    await screen.findByText("工單尚未抵達或已被刪除");
    expect(document.querySelector('[aria-busy="true"]')).toBeNull();
    second.unmount();

    render(
      <ChangeDetailPage
        {...(makeProps({ change: x({ reviewStatus: "inReview" }), loadStationTicket: vi.fn(async () => null) }) as Parameters<typeof ChangeDetailPage>[0])}
      />,
    );
    await openTab(/審查/);
    await screen.findByText("工單尚未抵達或已被刪除");
  });
});

// spec 需求「詳情頁的文件動作」：內容卡標頭右端由宿主渲染「開啟 ▾」，頁依作用中分頁
// 回報作用中分頁對應的文件（change＋artifact）與已載入原文；排程分頁沒有檔案（null）；
// 工單分頁只載解析後的回合、不帶原文。磁碟位置由宿主解析，頁不拼路徑。
describe("詳情頁的文件動作槽", () => {
  async function openTab(name: RegExp) {
    await waitFor(() => screen.getByRole("tab", { name }));
    fireEvent.mouseDown(screen.getByRole("tab", { name }));
  }
  const lastCall = (fn: Mock) => (fn.mock.calls.at(-1) as [DocumentTarget | null])[0];
  const NAME = "desktop-shell-and-browser";
  const doc = (artifact: string) => ({ change: NAME, artifact });

  it("各文件回報對應檔案；排程回報 null；回傳節點渲染在閱讀卡標題列", async () => {
    const renderDocumentActions = vi.fn((target: DocumentTarget | null) =>
      target?.doc && "change" in target.doc ? <button type="button">開啟 {target.doc.artifact}</button> : null,
    );
    render(
      <ChangeDetailPage
        {...(makeProps({
          change: { ...change, reviewStatus: "inReview", verifyStatus: "inVerify" },
          renderDocumentActions,
          loadStationTicket: vi.fn(async () => null),
        }) as Parameters<typeof ChangeDetailPage>[0])}
      />,
    );
    await waitFor(() =>
      expect(lastCall(renderDocumentActions)).toEqual({ doc: doc("proposal.md"), content: "# doc for proposal.md" }),
    );
    const docHeader = () => document.querySelector("[data-doc-header]") as HTMLElement;
    expect(within(docHeader()).getByRole("button", { name: "開啟 proposal.md" })).toBeTruthy();
    expect(within(document.querySelector("[data-detail-tree]") as HTMLElement).queryByRole("button", { name: /^開啟 / })).toBeNull();

    await openTab(/設計/);
    expect(lastCall(renderDocumentActions)).toEqual({ doc: doc("design.md"), content: "# doc for design.md" });
    await openTab(/任務/);
    expect(lastCall(renderDocumentActions)?.doc).toEqual(doc("tasks.md"));
    await openTab(/規格/);
    expect(lastCall(renderDocumentActions)).toEqual({ doc: doc("specs/desktop-app/spec.md"), content: SPEC_MD });
    await openTab(/排程/);
    expect(lastCall(renderDocumentActions)).toBeNull();
    expect(within(docHeader()).queryByRole("button", { name: /^開啟 / })).toBeNull();
    await openTab(/審查/);
    expect(lastCall(renderDocumentActions)).toEqual({ doc: doc("review.md") });
    await openTab(/驗證/);
    expect(lastCall(renderDocumentActions)).toEqual({ doc: doc("verify.md") });
  });

  it("文件載入中回報的原文為 null", async () => {
    const renderDocumentActions = vi.fn(() => null);
    render(
      <ChangeDetailPage
        {...(makeProps({
          loadDocument: vi.fn(() => new Promise<never>(() => {})),
          renderDocumentActions,
        }) as Parameters<typeof ChangeDetailPage>[0])}
      />,
    );
    await waitFor(() => expect(renderDocumentActions).toHaveBeenCalled());
    expect(lastCall(renderDocumentActions as unknown as Mock)).toEqual({ doc: doc("proposal.md"), content: null });
  });

  it("頁內無全螢幕切換鈕", async () => {
    render(<ChangeDetailPage {...(makeProps() as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    expect(screen.queryByRole("button", { name: /全螢幕|還原大小/ })).toBeNull();
  });
});

// spec「詳情頁的文件樹、閱讀卡與本頁目錄」：規格列能力籤切到規格文件並選定該 capability；
// 切換文件時閱讀區自頂端呈現、閱讀卡標題換成該文件。
describe("文件樹與閱讀卡", () => {
  it("點規格列的能力籤切到規格文件，「開啟 ▾」指向該 capability", async () => {
    const renderDocumentActions = vi.fn((_target: DocumentTarget | null) => null);
    const props = makeProps({
      loadCapabilities: vi.fn(async () => ["auth", "desktop-app"]),
      renderDocumentActions,
    });
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    const specs = await waitFor(() => {
      const el = document.querySelector("[data-relation-specs]") as HTMLElement | null;
      expect(el).toBeTruthy();
      return el!;
    });
    fireEvent.click(within(specs).getByRole("button", { name: "desktop-app" }));
    expect(screen.getByRole("tab", { name: /規格/ }).getAttribute("aria-selected")).toBe("true");
    expect(within(document.querySelector("[data-doc-header]") as HTMLElement).getByRole("heading").textContent).toBe("規格");
    const last = renderDocumentActions.mock.calls.at(-1)![0];
    expect(last?.doc).toEqual({ change: "desktop-shell-and-browser", artifact: "specs/desktop-app/spec.md" });
  });

  it("能力籤每點一次捲到該 capability 一次；自文件樹切走再切回規格文件時自頂端呈現", async () => {
    const scrolled: string[] = [];
    HTMLElement.prototype.scrollIntoView = function (this: HTMLElement) {
      scrolled.push(this.dataset.specCap ?? "");
    };
    try {
      const props = makeProps({ loadCapabilities: vi.fn(async () => ["auth", "desktop-app"]) });
      render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
      const specs = await waitFor(() => {
        const el = document.querySelector("[data-relation-specs]") as HTMLElement | null;
        expect(el).toBeTruthy();
        return el!;
      });
      fireEvent.click(within(specs).getByRole("button", { name: "desktop-app" }));
      await waitFor(() => expect(scrolled).toEqual(["desktop-app"]));
      fireEvent.mouseDown(screen.getByRole("tab", { name: /提案/ }));
      fireEvent.mouseDown(screen.getByRole("tab", { name: /規格/ }));
      expect((document.querySelector("[data-detail-reading]") as HTMLElement).scrollTop).toBe(0);
      expect(scrolled).toEqual(["desktop-app"]);
      fireEvent.click(within(specs).getByRole("button", { name: "desktop-app" }));
      await waitFor(() => expect(scrolled).toEqual(["desktop-app", "desktop-app"]));
    } finally {
      delete (HTMLElement.prototype as { scrollIntoView?: unknown }).scrollIntoView;
    }
  });

  it("閱讀卡出處行：提案為相對變更目錄的路徑、排程為推算來源", async () => {
    render(<ChangeDetailPage {...(makeProps() as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    const provenance = () => document.querySelector("[data-doc-header] [data-doc-provenance]")?.textContent;
    expect(provenance()).toBe("proposal.md");
    fireEvent.mouseDown(screen.getByRole("tab", { name: /排程/ }));
    expect(provenance()).toBe("由 speclink plan 推算");
  });

  it("換變更（‹ ›）時即使同為提案文件，閱讀區也捲回頂端", async () => {
    const props = makeProps();
    const { rerender } = render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    const reading = document.querySelector("[data-detail-reading]") as HTMLElement;
    reading.scrollTop = 300;
    rerender(<ChangeDetailPage {...(makeProps({ change: { ...change, name: "another-change" } }) as Parameters<typeof ChangeDetailPage>[0])} />);
    expect((document.querySelector("[data-detail-reading]") as HTMLElement).scrollTop).toBe(0);
    await screen.findByText("MomoChen");
  });

  it("切到內文未到的文件：節點立即標示、內文為 skeleton；左欄捲動位置不變", async () => {
    const props = makeProps({
      loadDocument: vi.fn((_c: string, artifact: string) =>
        artifact === "design.md"
          ? new Promise<string>(() => {})
          : Promise.resolve(artifact.startsWith("specs/") ? SPEC_MD : `# doc for ${artifact}`),
      ),
    });
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    const tree = document.querySelector("[data-detail-tree]") as HTMLElement;
    tree.scrollTop = 120;
    fireEvent.mouseDown(screen.getByRole("tab", { name: /設計/ }));
    expect(screen.getByRole("tab", { name: /設計/ }).getAttribute("data-state")).toBe("active");
    const body = document.querySelector("[data-doc-body]") as HTMLElement;
    expect(body.querySelector('[aria-busy="true"]')).toBeTruthy();
    expect(tree.scrollTop).toBe(120);
  });

  it("切換文件後閱讀區捲回頂端、閱讀卡標題立即換成該文件", async () => {
    render(<ChangeDetailPage {...(makeProps() as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("MomoChen");
    const reading = document.querySelector("[data-detail-reading]") as HTMLElement;
    const heading = () => within(document.querySelector("[data-doc-header]") as HTMLElement).getByRole("heading");
    expect(heading().textContent).toBe("提案");
    reading.scrollTop = 240;
    fireEvent.mouseDown(screen.getByRole("tab", { name: /設計/ }));
    expect(screen.getByRole("tab", { name: /設計/ }).getAttribute("data-state")).toBe("active");
    expect(heading().textContent).toBe("設計");
    expect(reading.scrollTop).toBe(0);
  });
});

// design D5：各渲染器的區段標題帶 data-doc-heading，右欄本頁目錄以它們為錨點。
describe("本頁目錄錨點", () => {
  it("提案含 Why／What Changes／Impact 時內文錨點依序為三個章節標籤，本頁目錄列出同三筆", async () => {
    const props = makeProps({
      loadDocument: vi.fn(async (_c: string, artifact: string) =>
        artifact === "proposal.md"
          ? "## Why\n\n因為。\n\n## What Changes\n\n改這個。\n\n## Impact\n\n影響面。\n"
          : artifact.startsWith("specs/")
            ? SPEC_MD
            : `# doc for ${artifact}`,
      ),
    });
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    await screen.findByText("因為。");
    const body = document.querySelector("[data-doc-body]") as HTMLElement;
    const heads = Array.from(body.querySelectorAll("[data-doc-heading]")).map((el) => el.textContent);
    expect(heads).toEqual(["為什麼", "變更內容", "影響"]);
    expect(body.querySelector("[data-doc-heading]")!.tagName).toBe("H2");
    const toc = document.querySelector("[data-detail-toc]") as HTMLElement;
    await waitFor(() =>
      expect(Array.from(toc.querySelectorAll("[data-manual-anchor]")).map((el) => el.textContent)).toEqual([
        "為什麼",
        "變更內容",
        "影響",
      ]),
    );
  });

  // 分頁換文件時新內容晚一次畫面更新才掛上：本頁目錄要跟著重掃，不得停在前一份文件的錨點。
  it("提案切到設計後，本頁目錄換成設計的章節錨點", async () => {
    const props = makeProps({
      loadDocument: vi.fn(async (_c: string, artifact: string) =>
        artifact === "proposal.md"
          ? "## Why\n\n因為。\n\n## Impact\n\n影響面。\n"
          : artifact === "design.md"
            ? "## Context\n\n脈絡。\n\n## Decisions\n\n決定。\n"
            : `# doc for ${artifact}`,
      ),
    });
    render(<ChangeDetailPage {...(props as Parameters<typeof ChangeDetailPage>[0])} />);
    const anchors = () =>
      Array.from(document.querySelectorAll("[data-detail-toc] [data-manual-anchor]")).map((el) => el.textContent);
    await waitFor(() => expect(anchors()).toEqual(["為什麼", "影響"]));
    fireEvent.mouseDown(screen.getByRole("tab", { name: /設計/ }));
    // 前提：設計文件在切換前已載入，切換當下內文即在畫面上（不等待）。若改成切到分頁才載入，載入完成時重掃鍵
    // 變動會強制重掃，拿掉內文監看也會綠——此斷言先紅，提醒本測試已失去守門作用。
    expect(screen.getByText("脈絡。")).toBeTruthy();
    await waitFor(() => expect(anchors()).toEqual(["背景", "決策"]));
  });

  it("規格 delta 的色標區段標頭與排程四卡標題為錨點", async () => {
    render(
      <ChangeDetailPage
        {...(makeProps({ change: { ...change, wave: 1, blockedBy: [], dependsOn: [] } }) as Parameters<typeof ChangeDetailPage>[0])}
      />,
    );
    await screen.findByText("MomoChen");
    fireEvent.mouseDown(screen.getByRole("tab", { name: /規格/ }));
    await waitFor(() => expect(document.querySelector('[data-delta-section="added"]')).toBeTruthy());
    expect(document.querySelector('[data-delta-section="added"]')!.hasAttribute("data-doc-heading")).toBe(true);
    fireEvent.mouseDown(screen.getByRole("tab", { name: /排程/ }));
    const titles = Array.from(document.querySelectorAll("[data-plan-card-header] [data-doc-heading]"));
    expect(titles).toHaveLength(4);
  });
});

// spec「提案與設計章節以中文標籤呈現」「任務群組標題與章節標籤同款式」「規格分頁 delta 區段以色標呈現」
// 的閱讀卡籤槽、數字磚、提示框與跨文件跳轉（desktop-change-doc-views design D9）。
describe("變更文件的籤、數字磚、提示框與跳轉", () => {
  const flat = (el: Element | null) => (el?.textContent ?? "").replace(/\s+/g, " ").trim();
  const badges = () => flat(document.querySelector("[data-doc-badges]"));
  const tiles = () =>
    Array.from(document.querySelectorAll("[data-doc-tile]")).map((t) => `${flat(t.lastElementChild)} ${flat(t.firstElementChild)}`);
  const REFACTOR = [
    "## Summary",
    "",
    "一句話。",
    "",
    "## Motivation",
    "",
    "- 動機",
    "",
    "## Proposed Solution",
    "",
    ...[1, 2, 3, 4, 5, 6].map((n) => `${n}. 步驟 ${n}`),
    "",
    "## Non-Goals",
    "",
    ...[1, 2, 3, 4, 5].map((n) => `- 非目標 ${n}`),
    "",
    "## Alternatives Considered",
    "",
    ...[1, 2, 3, 4].map((n) => `- 方案 ${n}——理由 ${n}`),
    "",
    "## Capabilities",
    "",
    "### New Capabilities",
    "",
    "(none)",
    "",
    "### Modified Capabilities",
    "",
    "- `desktop-app`: 改版",
    "",
    "## Impact",
    "",
    "- Affected specs: desktop-app",
    "- Affected code:",
    "  - New: a.ts、b.ts",
    "  - Modified: c.ts、d.ts、e.ts、f.ts",
    "  - Removed: (none)",
  ].join("\n");
  const DESKTOP_DELTA = [
    "## MODIFIED Requirements",
    "",
    ...["甲", "乙", "丙"].flatMap((n) => [`### Requirement: ${n}`, "", `${n}需求文。`, ""]),
    "<!-- REMOVED-SCENARIO: 舊情境 -->",
  ].join("\n");
  const DESIGN = "## Decisions\n\n### D1 一\n\n- a\n\n### D3 三\n\n- c\n";
  const TASKS = [
    "## 1. 甲",
    "",
    "- [x] 1.1 a",
    "- [x] 1.2 b",
    "- [x] 1.3 c",
    "",
    "## 2. 乙",
    "",
    "- [x] 2.1 d",
    "- [x] 2.2 e",
    "- [x] 2.3 f",
    "",
    "## 3. 丙",
    "",
    "- [x] 3.1 g",
    "- [ ] [M] 3.2 手動驗收 `dev:desktop` 畫面",
    "- [ ] 3.3 h",
  ].join("\n");
  const docs: Record<string, string> = {
    "proposal.md": REFACTOR,
    "design.md": DESIGN,
    "tasks.md": TASKS,
    "specs/desktop-app/spec.md": DESKTOP_DELTA,
  };
  const props = (over: Record<string, unknown> = {}) =>
    makeProps({
      loadDocument: vi.fn(async (_c: string, artifact: string) => docs[artifact] ?? null),
      ...over,
    });

  it("提案文件：籤槽為模板形狀、四格磚依序、能力段之下的改寫提示框", async () => {
    render(<ChangeDetailPage {...(props() as Parameters<typeof ChangeDetailPage>[0])} />);
    await waitFor(() => expect(badges()).toBe("Refactor"));
    expect(tiles()).toEqual(["解法步驟 6", "非目標 5", "能力 1", "影響的檔案 6"]);
    await waitFor(() =>
      expect(flat(document.querySelector('[data-section="sections.capabilities"] [data-doc-callout]'))).toContain(
        "desktop-app 有 3 條需求被改寫，其中 1 個 scenario 以 REMOVED-SCENARIO 宣告",
      ),
    );
  });

  it("點提案能力籤後規格節點作用中，「開啟 ▾」指向該能力的 delta 規格", async () => {
    const renderDocumentActions = vi.fn((_target: DocumentTarget | null) => null);
    render(<ChangeDetailPage {...(props({ renderDocumentActions }) as Parameters<typeof ChangeDetailPage>[0])} />);
    const chip = await waitFor(() => {
      const el = document.querySelector('[data-section="sections.capabilities"] button[data-capability-chip]');
      expect(el).toBeTruthy();
      return el as HTMLElement;
    });
    fireEvent.click(chip);
    await waitFor(() => expect(screen.getByRole("tab", { name: /規格/ }).getAttribute("data-state")).toBe("active"));
    const target = renderDocumentActions.mock.calls.at(-1)![0];
    expect(target?.doc).toEqual({ change: change.name, artifact: "specs/desktop-app/spec.md" });
  });

  it("點任務的 D3 籤後設計節點作用中且 D3 決策卡展開", async () => {
    render(<ChangeDetailPage {...(props() as Parameters<typeof ChangeDetailPage>[0])} />);
    fireEvent.mouseDown(await screen.findByRole("tab", { name: /任務/ }));
    await waitFor(() => expect(taskListProps.at(-1)?.onJumpDecision).toBeTypeOf("function"));
    act(() => (taskListProps.at(-1)!.onJumpDecision as (id: string) => void)("D3"));
    await waitFor(() => expect(screen.getByRole("tab", { name: /設計/ }).getAttribute("data-state")).toBe("active"));
    await waitFor(() =>
      expect(document.querySelector('[data-decision="D3"] button')?.getAttribute("aria-expanded")).toBe("true"),
    );
    expect(document.querySelector('[data-decision="D1"] button')?.getAttribute("aria-expanded")).toBe("false");
  });

  it("任務文件：數字磚 9／7／1／3 與手動提示框列出 3.2；全部勾選後籤槽為「全部完成」", async () => {
    const first = render(<ChangeDetailPage {...(props() as Parameters<typeof ChangeDetailPage>[0])} />);
    fireEvent.mouseDown(await screen.findByRole("tab", { name: /任務/ }));
    await waitFor(() => expect(tiles()).toEqual(["任務 9", "已完成 7", "手動 1", "群組 3"]));
    expect(badges()).toBe("7/9");
    const callout = document.querySelector("[data-doc-callout]")!;
    expect(flat(callout)).toContain("1 個手動任務");
    // spec「介面狀態語意色分層」：手動任務提示框取暖黃手動色（design D14）。
    expect(callout.firstElementChild!.className).toContain("bg-status-manual/10");
    const manualRow = callout.querySelector("[data-manual-task]") as HTMLElement;
    expect(flat(manualRow)).toContain("3.2手動驗收 dev:desktop 畫面");
    // 標題的行內程式碼為等寬籤（不直出反引號）。
    expect(manualRow.querySelector("code")?.textContent).toBe("dev:desktop");
    expect(flat(manualRow)).not.toContain("`");
    // 「前往」：TaskList 收到第 8 個任務（3.2）的定位請求。
    fireEvent.click(within(manualRow).getByRole("button", { name: "前往" }));
    await waitFor(() => expect((taskListProps.at(-1)?.locate as { ordinal: number } | null)?.ordinal).toBe(8));
    first.unmount();

    render(<ChangeDetailPage {...(props({ loadDocument: withTasks(TASKS.replace(/- \[ \]/g, "- [x]")) }) as Parameters<typeof ChangeDetailPage>[0])} />);
    fireEvent.mouseDown(await screen.findByRole("tab", { name: /任務/ }));
    await waitFor(() => expect(badges()).toBe("全部完成"));
    // 手動任務全部勾完：「手動」數字磚仍計總數，提示框缺席。
    expect(tiles()).toContain("手動 1");
    expect(manualCallout()).toBeNull();
  });

  /** tasks.md 換成指定內容、其餘文件照 docs 的載入器。 */
  const withTasks = (tasksMd: string) =>
    vi.fn(async (_c: string, artifact: string) => (artifact === "tasks.md" ? tasksMd : (docs[artifact] ?? null)));
  // 手動任務 4.3、5.8、6.9、7.6 分屬四個群組、各夾一條一般任務（序數 2、4、6、8）；checked 為已勾的手動任務。
  const manualTasks = (checked: string[]) =>
    ["4.3", "5.8", "6.9", "7.6"]
      .map((n) =>
        [`## ${n[0]}. 群組`, "", `- [ ] ${n[0]}.1 一般`, `- [${checked.includes(n) ? "x" : " "}] [M] ${n} 手動`].join("\n"),
      )
      .join("\n\n");
  const manualProps = (checked: string[], over: Record<string, unknown> = {}) =>
    props({ loadDocument: withTasks(manualTasks(checked)), ...over });
  const toggle = (ordinal: number, done: boolean) =>
    act(() => (taskListProps.at(-1)!.onToggle as (o: number, d: boolean) => void)(ordinal, done));
  const manualCallout = () => document.querySelector("[data-doc-callout]");
  const manualRows = () =>
    Array.from(document.querySelectorAll("[data-doc-callout] [data-manual-task]")).map((row) => flat(row.firstElementChild));

  // spec Example「勾選狀態與提示框」：一列一題。
  it.each([
    { checked: [], title: "4 個手動任務", rows: ["4.3", "5.8", "6.9", "7.6"] },
    { checked: ["4.3"], title: "3 個手動任務", rows: ["5.8", "6.9", "7.6"] },
    { checked: ["4.3", "5.8", "6.9", "7.6"], title: null, rows: [] },
  ])("勾選狀態與提示框：已勾 $checked → $title", async ({ checked, title, rows }) => {
    render(<ChangeDetailPage {...(manualProps(checked) as Parameters<typeof ChangeDetailPage>[0])} />);
    fireEvent.mouseDown(await screen.findByRole("tab", { name: /任務/ }));
    await waitFor(() => expect(tiles()).toContain("手動 4"));
    if (title) expect(flat(manualCallout())).toContain(title);
    else expect(manualCallout()).toBeNull();
    expect(manualRows()).toEqual(rows);
  });

  it("部分勾選時提示框只列未勾選的手動任務", async () => {
    render(<ChangeDetailPage {...(manualProps(["4.3"], { onToggleTask: vi.fn().mockResolvedValue(undefined) }) as Parameters<typeof ChangeDetailPage>[0])} />);
    fireEvent.mouseDown(await screen.findByRole("tab", { name: /任務/ }));
    await waitFor(() => expect(tiles()).toContain("手動 4"));
    expect(flat(manualCallout())).toContain("3 個手動任務");
    expect(manualRows()).toEqual(["5.8", "6.9", "7.6"]);

    toggle(4, true);
    toggle(6, true);
    toggle(8, true);
    await waitFor(() => expect(manualCallout()).toBeNull());
    expect(tiles()).toContain("手動 4");

    toggle(6, false);
    await waitFor(() => expect(manualRows()).toEqual(["6.9"]));
    expect(flat(manualCallout())).toContain("1 個手動任務");
    expect(tiles()).toContain("手動 4");
  });

  it("勾選寫回失敗時提示框跟著回滾", async () => {
    render(
      <ChangeDetailPage
        {...(manualProps(["4.3"], { onToggleTask: vi.fn().mockRejectedValue(new Error("disk full")) }) as Parameters<typeof ChangeDetailPage>[0])}
      />,
    );
    fireEvent.mouseDown(await screen.findByRole("tab", { name: /任務/ }));
    await waitFor(() => expect(tiles()).toContain("手動 4"));
    toggle(4, true);
    // 樂觀勾選先生效（寫回的拒絕要到下一個 microtask 才回滾）。
    expect(manualRows()).toEqual(["6.9", "7.6"]);
    await waitFor(() => expect(screen.getByText(/寫回失敗/)).toBeTruthy());
    expect(flat(manualCallout())).toContain("3 個手動任務");
    expect(manualRows()).toEqual(["5.8", "6.9", "7.6"]);
  });

  it("「全部已完成」「重置任務」與外部變更：世代重載後提示框跟著 tasks.md", async () => {
    const { rerender } = render(<ChangeDetailPage {...(manualProps(["4.3"]) as Parameters<typeof ChangeDetailPage>[0])} refreshGen={0} />);
    fireEvent.mouseDown(await screen.findByRole("tab", { name: /任務/ }));
    await waitFor(() => expect(manualRows()).toEqual(["5.8", "6.9", "7.6"]));
    // 三者都由宿主 refresh 遞增世代、重讀 tasks.md 收斂。
    rerender(<ChangeDetailPage {...(manualProps(["4.3", "5.8", "6.9", "7.6"]) as Parameters<typeof ChangeDetailPage>[0])} refreshGen={1} />);
    await waitFor(() => expect(manualCallout()).toBeNull());
    rerender(<ChangeDetailPage {...(manualProps([]) as Parameters<typeof ChangeDetailPage>[0])} refreshGen={2} />);
    await waitFor(() => expect(manualRows()).toEqual(["4.3", "5.8", "6.9", "7.6"]));
    expect(tiles()).toContain("手動 4");
  });

  it("規格文件：籤槽為各操作計數、數字磚 2／5／2／「1 ／ 0」", async () => {
    const twoCaps = [
      "## MODIFIED Requirements",
      "",
      ...[1, 2, 3, 4, 5].flatMap((n) => [`### Requirement: 改 ${n}`, "", "內文。", ""]),
      "<!-- REMOVED-SCENARIO: 舊一 -->",
      "<!-- REMOVED-SCENARIO: 舊二 -->",
    ].join("\n");
    const added = "## ADDED Requirements\n\n### Requirement: 新\n\n內文。\n";
    render(
      <ChangeDetailPage
        {...(props({
          loadCapabilities: vi.fn(async () => ["desktop-app", "client-protocol"]),
          loadDocument: vi.fn(async (_c: string, artifact: string) =>
            artifact === "specs/desktop-app/spec.md" ? twoCaps : artifact === "specs/client-protocol/spec.md" ? added : "# doc",
          ),
        }) as Parameters<typeof ChangeDetailPage>[0])}
      />,
    );
    fireEvent.mouseDown(await screen.findByRole("tab", { name: /規格/ }));
    await waitFor(() => expect(tiles()).toEqual(["觸及的能力 2", "修改的需求 5", "移除的 scenario 2", "新增／更名 1 ／ 0"]));
    expect(Array.from(document.querySelectorAll("[data-doc-badges] [data-delta-count]")).map((b) => flat(b))).toEqual([
      "修改 5",
      "新增 1",
    ]);
  });

  it("設計文件：數字磚依序為決策、目標／非目標、風險與取捨、開放問題（「無」算 0）", async () => {
    const design = [
      "## Goals / Non-Goals",
      "",
      "**Goals:**",
      "",
      ...["一", "二", "三", "四", "五"].map((n) => `- 目標${n}`),
      "",
      "**Non-Goals:**",
      "",
      ...["一", "二", "三"].map((n) => `- 非目標${n}`),
      "",
      "## Decisions",
      "",
      ...["面板底與卡片層", "二", "三", "四", "五", "六"].flatMap((title, i) => [`### D${i + 1} ${title}`, "", "- 點", ""]),
      "## Risks / Trade-offs",
      "",
      ...["一", "二", "三", "四"].map((n) => `- [風險${n}] → 對策`),
      "",
      "## Open Questions",
      "",
      "無",
    ].join("\n");
    const loadDocument = vi.fn(async (_c: string, artifact: string) => (artifact === "design.md" ? design : (docs[artifact] ?? null)));
    render(<ChangeDetailPage {...(props({ loadDocument }) as Parameters<typeof ChangeDetailPage>[0])} />);
    fireEvent.mouseDown(await screen.findByRole("tab", { name: /設計/ }));
    await waitFor(() => expect(tiles()).toEqual(["決策 6", "目標／非目標 5 ／ 3", "風險與取捨 4", "開放問題 0"]));
    expect(document.querySelectorAll("[data-risk]")).toHaveLength(4);
  });

  it("提案列為新能力的規格段不出「看正式規格」（封存前沒有正式規格），修改的能力照常", async () => {
    const proposal = "## Capabilities\n\n### New Capabilities\n\n- `doc-views`: 新能力\n\n### Modified Capabilities\n\n- `desktop-app`: 改版\n";
    const loadDocument = vi.fn(async (_c: string, artifact: string) =>
      artifact === "proposal.md"
        ? proposal
        : artifact === "specs/doc-views/spec.md"
          ? "## ADDED Requirements\n\n### Requirement: 新\n\n內文。\n"
          : (docs[artifact] ?? null),
    );
    const loadCapabilities = vi.fn().mockResolvedValue(["desktop-app", "doc-views"]);
    render(<ChangeDetailPage {...(props({ loadDocument, loadCapabilities, onOpenSpec: vi.fn() }) as Parameters<typeof ChangeDetailPage>[0])} />);
    fireEvent.mouseDown(await screen.findByRole("tab", { name: /規格/ }));
    await waitFor(() => expect(document.querySelector('[data-spec-cap="doc-views"]')).toBeTruthy());
    expect(document.querySelector('[data-spec-cap="doc-views"]')!.textContent).not.toContain("看正式規格");
    expect(within(document.querySelector('[data-spec-cap="desktop-app"]') as HTMLElement).getByRole("button", { name: "看正式規格" })).toBeTruthy();
  });

  it("規格文件能力頭的「看正式規格」交宿主開規格詳情頁", async () => {
    const onOpenSpec = vi.fn();
    render(<ChangeDetailPage {...(props({ onOpenSpec }) as Parameters<typeof ChangeDetailPage>[0])} />);
    fireEvent.mouseDown(await screen.findByRole("tab", { name: /規格/ }));
    fireEvent.click(await screen.findByRole("button", { name: "看正式規格" }));
    expect(onOpenSpec).toHaveBeenCalledWith("desktop-app");
  });
});


describe('提案文件區不列歷史檢查紀錄',()=>{
 const record=(id:string,date:string,kind='review')=>({recordId:id,kind,createdAt:date,outcome:'passed',relativePath:`checks/${kind}/${id}.md`,contentDigest:'a'.repeat(64)});
 it('歷史紀錄不出現在文件樹，僅從已審查品質狀態展開',async()=>{
  const records=[record('r1','2026-10-19'),record('r2','2026-10-20'),record('v1','2026-10-20','verify'),record('s1','2026-10-20','security')];
  const checks={list:vi.fn().mockResolvedValue(records),read:vi.fn().mockResolvedValue('## 已保存紀錄\n\n[REDACTED] <script>alert(1)</script>')};
  render(<ChangeDetailPage {...makeProps({change:{...change,reviewStatus:'reviewed',reviewedAt:'2026-10-23'},checkRecords:checks})}/>);
  expect(checks.list).not.toHaveBeenCalled();
  expect(screen.queryByRole('tablist',{name:'檢查紀錄'})).toBeNull();
  expect(screen.queryByRole('tab',{name:/安全審查.*2026-/})).toBeNull();
  expect(checks.read).not.toHaveBeenCalled();
  fireEvent.click(await screen.findByRole('button',{name:'已審查'}));
  await waitFor(()=>expect(checks.list).toHaveBeenCalledWith(change.name,['review','verify']));
  const history=await screen.findByRole('region',{name:'審查紀錄'});
  fireEvent.click(within(history).getByRole('button',{name:/2026-10-20/}));
  expect(await screen.findByText('已保存紀錄')).toBeTruthy();expect(checks.read).toHaveBeenCalledWith(change.name,records[1]);expect(document.querySelector('script')).toBeNull();
  expect(screen.queryByText('2026-10-20 · s1')).toBeNull();
 });
 it('切換專案時遲到的歷史文件不覆蓋新提案，並可查新提案錯誤',async()=>{
  let settle!:(value:string)=>void;const old={list:vi.fn().mockResolvedValue([record('old','2026-10-19')]),read:vi.fn().mockImplementation(()=>new Promise<string>(resolve=>{settle=resolve}))};
  const view=render(<ChangeDetailPage {...makeProps({change:{...change,reviewStatus:'reviewed'},checkRecords:old})}/>);
  fireEvent.click(await screen.findByRole('button',{name:'已審查'}));
  fireEvent.click(await screen.findByRole('button',{name:/2026-10-19/}));await waitFor(()=>expect(old.read).toHaveBeenCalled());
  const next={list:vi.fn().mockRejectedValue(new Error('offline')),read:vi.fn()};view.rerender(<ChangeDetailPage {...makeProps({change:{...change,name:'other-change',reviewStatus:'reviewed'},checkRecords:next})}/>);await act(async()=>{settle('舊專案秘密內容')});
  expect(screen.queryByText('舊專案秘密內容')).toBeNull();
  expect(screen.getByRole('tab',{name:'提案'}).getAttribute('aria-selected')).toBe('true');
  fireEvent.click(await screen.findByRole('button',{name:'已審查'}));
  expect((await screen.findByRole('alert')).textContent).toContain('offline');expect(next.read).not.toHaveBeenCalled();
 });
})

describe('品質關卡的審查與驗證紀錄',()=>{
 const record=(id:string,started:string,createdAt:string,kind:'review'|'verify')=>({recordId:id,kind,createdAt,outcome:'passed',relativePath:`checks/${kind}/${started}${id}.md`,contentDigest:'a'.repeat(64)});
 it('狀態章分別展開開始時間倒序清單，選取後開啟對應紀錄',async()=>{
  const records=[
   record('01K86W6F000000000000000001','20261021153733','2026-10-23T01:00:00Z','review'),
   record('01K86W6F000000000000000002','20261022153733','2026-10-22T01:00:00Z','review'),
   record('01K86W6F000000000000000003','20261023153733','2026-10-21T01:00:00Z','verify'),
  ];
  const checks={list:vi.fn().mockResolvedValue(records),read:vi.fn(async(_change:string,entry:typeof records[number])=>`## ${entry.recordId}\n`)};
  const stamped:ChangeItem={...change,reviewStatus:'reviewed',reviewedAt:'2026-10-23',verifyStatus:'verified',verifiedAt:'2026-10-23'};
  render(<ChangeDetailPage {...makeProps({change:stamped,checkRecords:checks})}/>);
  const review=await screen.findByRole('button',{name:'已審查'});
  fireEvent.click(review);
  expect(review.getAttribute('aria-expanded')).toBe('true');
  const list=await screen.findByRole('region',{name:'審查紀錄'});
  const reviewButtons=await within(list).findAllByRole('button');
  expect(reviewButtons.map(button=>button.textContent?.split(' · ')[0])).toEqual(['2026-10-22 15:37:33 UTC','2026-10-21 15:37:33 UTC']);
  expect(within(list).queryByText('01K86W6F000000000000000003')).toBeNull();
  fireEvent.click(within(list).getByRole('button',{name:/2026-10-22 15:37:33 UTC/}));
  expect(await screen.findByText('01K86W6F000000000000000002')).toBeTruthy();
  const verify=await screen.findByRole('button',{name:'已驗證'});verify.focus();await userEvent.keyboard('{Enter}');
  expect((await screen.findByRole('region',{name:'驗證紀錄'})).textContent).toContain('2026-10-23 15:37:33 UTC');
  expect(screen.getByRole('button',{name:'已審查'}).getAttribute('aria-expanded')).toBe('true');
 });
 it('查詢失敗時可重試；切換提案後只顯示新提案紀錄',async()=>{
  const entry=record('01K86W6F000000000000000004','20261023153733','2026-10-23T01:00:00Z','review');
  const checks={list:vi.fn().mockRejectedValueOnce(new Error('offline')).mockResolvedValue([entry]),read:vi.fn().mockResolvedValue('## 新提案紀錄')};
  const props=makeProps({change:{...change,reviewStatus:'reviewed',reviewedAt:'2026-10-23'},checkRecords:checks});
  const view=render(<ChangeDetailPage {...props}/>);
  fireEvent.click(await screen.findByRole('button',{name:'已審查'}));
  const retry=await screen.findByRole('button',{name:'重試'});await userEvent.click(retry);
  expect(await screen.findByRole('button',{name:/2026-10-23 15:37:33 UTC/})).toBeTruthy();
  const next=record('01K86W6F000000000000000005','20261022153733','2026-10-22T01:00:00Z','review');
  const nextChecks={list:vi.fn().mockResolvedValue([next]),read:vi.fn()};
  view.rerender(<ChangeDetailPage {...makeProps({change:{...change,name:'other-change',reviewStatus:'reviewed',reviewedAt:'2026-10-23'},checkRecords:nextChecks})}/>);
  expect(screen.queryByRole('button',{name:/2026-10-23 15:37:33 UTC/})).toBeNull();
  fireEvent.click(await screen.findByRole('button',{name:'已審查'}));
  expect(await screen.findByRole('button',{name:/2026-10-22 15:37:33 UTC/})).toBeTruthy();
  expect(screen.queryByRole('button',{name:/2026-10-23 15:37:33 UTC/})).toBeNull();
 });
 it('保留異常與省略列，顯示逐筆狀態且選取讀取失敗後可重試',async()=>{
  const records=[
   {...record('01K86W6F000000000000000006','20261023153733','2026-10-23T01:00:00Z','review'),readError:'missing_check_record'},
   {...record('01K86W6F000000000000000007','20261022153733','2026-10-22T01:00:00Z','review'),readError:'integrity_error'},
   {...record('01K86W6F000000000000000008','20261021153733','2026-10-21T01:00:00Z','review'),readError:'check_record_read_error'},
   {...record('01K86W6F000000000000000009','20261020153733','2026-10-20T01:00:00Z','review'),outcome:'omitted'},
   record('01K86W6F000000000000000010','20261019153733','2026-10-19T01:00:00Z','review'),
  ];
  const repaired=records.map(entry=>entry.recordId==='01K86W6F000000000000000008'?{...entry,readError:undefined}:entry);
  const checks={list:vi.fn().mockResolvedValueOnce(records).mockResolvedValue(repaired),read:vi.fn().mockResolvedValueOnce('## 修復後成功').mockRejectedValueOnce(new Error('disk unavailable')).mockResolvedValue('## 重試成功')};
  render(<ChangeDetailPage {...makeProps({change:{...change,reviewStatus:'reviewed'},checkRecords:checks})}/>);
  fireEvent.click(await screen.findByRole('button',{name:'已審查'}));
  const list=await screen.findByRole('region',{name:'審查紀錄'});
  expect(list.textContent).toContain('文件缺失');
  expect(list.textContent).toContain('內容完整性錯誤');
  expect(list.textContent).toContain('讀取失敗');
  expect(list.textContent).toContain('依封存政策省略');
  expect(within(list).getByRole('button',{name:/通過.*文件缺失/})).toBeTruthy();
  fireEvent.click(within(list).getByRole('button',{name:/2026-10-20/}));
  expect(checks.read).not.toHaveBeenCalled();
  fireEvent.click(within(list).getByRole('button',{name:/2026-10-21/}));
  expect((await screen.findByRole('alert')).textContent).toContain('讀取失敗');
  expect(within(screen.getByRole('tabpanel')).getByRole('button',{name:'重試'})).toBeTruthy();
  await userEvent.click(within(screen.getByRole('tabpanel')).getByRole('button',{name:'重試'}));
  await waitFor(()=>expect(checks.list).toHaveBeenCalledTimes(2));
  await waitFor(()=>expect(checks.read).toHaveBeenCalledTimes(1));
  expect(await within(screen.getByRole('tabpanel')).findByText('修復後成功')).toBeTruthy();
  fireEvent.click(within(list).getByRole('button',{name:/2026-10-19/}));
  expect((await screen.findByRole('alert')).textContent).toContain('disk unavailable');
  expect(screen.queryByText('載入紀錄中…')).toBeNull();
  expect(within(screen.getByRole('tabpanel')).getByRole('button',{name:'重試'})).toBeTruthy();
  await userEvent.click(within(screen.getByRole('tabpanel')).getByRole('button',{name:'重試'}));
  await waitFor(()=>expect(checks.read).toHaveBeenCalledTimes(3));
  await waitFor(()=>expect(screen.queryByRole('alert')).toBeNull());
  expect(within(screen.getByRole('tabpanel')).queryByText('載入紀錄中…')).toBeNull();
 });
 it('收合歷史清單後仍可重新查詢失敗的正文',async()=>{
  const entry=record('01K86W6F000000000000000015','20261023153733','2026-10-23T01:00:00Z','review');
  const checks={list:vi.fn().mockResolvedValue([entry]),read:vi.fn().mockRejectedValueOnce(new Error('disk unavailable')).mockResolvedValue('## 重試成功')};
  render(<ChangeDetailPage {...makeProps({change:{...change,reviewStatus:'reviewed'},checkRecords:checks})}/>);
  const toggle=await screen.findByRole('button',{name:'已審查'});
  fireEvent.click(toggle);
  const list=await screen.findByRole('region',{name:'審查紀錄'});
  fireEvent.click(within(list).getByRole('button',{name:/2026-10-23/}));
  expect((await screen.findByRole('alert')).textContent).toContain('disk unavailable');
  fireEvent.click(toggle);
  expect(toggle.getAttribute('aria-expanded')).toBe('false');
  await userEvent.click(within(screen.getByRole('tabpanel')).getByRole('button',{name:'重試'}));
  await waitFor(()=>expect(checks.list).toHaveBeenCalledTimes(2));
  expect(await within(screen.getByRole('tabpanel')).findByText('重試成功')).toBeTruthy();
 });
 it('以檔名開始時間排序，並讓同秒紀錄及舊檔名排序穩定',async()=>{
  const records=[
   record('01K86W6F000000000000000013','20261023153733','2026-10-23T07:00:00Z','review'),
   record('01K86W6F000000000000000012','20261023153733','2026-10-23T06:00:00Z','review'),
   record('01K86W6F000000000000000014','20261023153732','2026-10-23T05:00:00Z','review'),
   {...record('legacy-b','20261023153731','2026-10-23T08:00:00Z','review'),relativePath:'checks/review/legacy-b.md'},
   {...record('legacy-a','20261023153730','2026-10-23T08:00:00Z','review'),relativePath:'checks/review/legacy-a.md'},
  ];
  const checks={list:vi.fn().mockResolvedValue(records),read:vi.fn()};
  render(<ChangeDetailPage {...makeProps({change:{...change,reviewStatus:'reviewed'},checkRecords:checks})}/>);
  fireEvent.click(await screen.findByRole('button',{name:'已審查'}));
  const rows=await within(await screen.findByRole('region',{name:'審查紀錄'})).findAllByRole('button');
  expect(rows.map(row=>row.textContent?.match(/(?:01K86W6F0000000000000000\d{2}|legacy-[ab])/)?.[0])).toEqual([
   '01K86W6F000000000000000012','01K86W6F000000000000000013','01K86W6F000000000000000014','legacy-a','legacy-b',
  ]);
 });
 it('依實際時間解析時區 offset，並將無效時間排在有效紀錄之後',async()=>{
  const records=[
   {...record('offset','20261023140000','2026-10-23T14:00:00+08:00','review'),relativePath:'checks/review/legacy-offset.md'},
   {...record('latest','20261023070000','2026-10-23T07:00:00Z','review'),relativePath:'checks/review/legacy-latest.md'},
   {...record('oldest','20261023050000','2026-10-23T05:00:00Z','review'),relativePath:'checks/review/legacy-oldest.md'},
   {...record('invalid','20261023090000','not-a-date','review'),relativePath:'checks/review/legacy-invalid.md'},
  ];
  const checks={list:vi.fn().mockResolvedValue(records),read:vi.fn()};
  render(<ChangeDetailPage {...makeProps({change:{...change,reviewStatus:'reviewed'},checkRecords:checks})}/>);
  fireEvent.click(await screen.findByRole('button',{name:'已審查'}));
  const rows=await within(await screen.findByRole('region',{name:'審查紀錄'})).findAllByRole('button');
  expect(rows.map(row=>row.textContent?.match(/(?:latest|offset|oldest|invalid)/)?.[0])).toEqual([
   'latest','offset','oldest','invalid',
  ]);
 });
});

it('同一提案的遠端重新讀取失敗時清除舊正文並顯示錯誤',async()=>{
 const entry={recordId:'r1',kind:'review',createdAt:'2026-10-20',outcome:'passed',relativePath:'checks/review/r1.md',contentDigest:'a'.repeat(64)};
 const checks={list:vi.fn().mockResolvedValue([entry]),read:vi.fn().mockResolvedValue('## 已載入的遠端紀錄\n\n唯讀內容。')};const props=makeProps({change:{...change,reviewStatus:'reviewed'},checkRecords:checks});const view=render(<ChangeDetailPage {...props}/>);
 fireEvent.click(await screen.findByRole('button',{name:'已審查'}));
 fireEvent.click(await screen.findByRole('button',{name:/2026-10-20/}));await screen.findByText('已載入的遠端紀錄');
 checks.list.mockRejectedValue(new Error('offline'));checks.read.mockRejectedValue(new Error('offline'));view.rerender(<ChangeDetailPage {...props} refreshGen={1}/>);
 await waitFor(()=>expect(screen.getAllByRole('alert').some(e=>e.textContent?.includes('offline'))).toBe(true));expect(within(screen.getByRole('tabpanel')).queryByText('已載入的遠端紀錄')).toBeNull();
})

import { describe, it, expect, vi, type Mock } from "vitest";
import { render, screen, waitFor, fireEvent, act } from "@testing-library/react";

import { RichDetailDrawer } from "../components/RichDetailDrawer";
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
    await latest.onReorder!(2, 5);
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
    await latest.onReorder!(1, 3, true);
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
    latest.onToggle!(1, true); // 互動進行中（onToggleTask 未 resolve）
    rerender(<RichDetailDrawer {...(props as never)} refreshGen={1} />);
    await new Promise((r) => setTimeout(r, 25));
    // 讓路：互動未結束前外部世代不觸發重載，不打斷進行中操作。
    expect(tasksReads()).toBe(before);
    release(); // 互動結束 → 補載一次
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
    latest.onToggle!(1, true);
    await waitFor(() => expect((props as { onToggleTask: Mock }).onToggleTask).toHaveBeenCalled());
    await new Promise((r) => setTimeout(r, 25));
    // 獨立局部重讀路徑已移除（design D2 單一資料流）。
    expect(tasksReads()).toBe(t0);
    // 宿主 onToggleTask 內 refresh → 世代遞增（此處由測試模擬）→ tasks 與 meta 一併重載。
    rerender(<RichDetailDrawer {...(props as never)} refreshGen={1} />);
    await waitFor(() => expect(tasksReads()).toBeGreaterThan(t0));
    await waitFor(() => expect(metaCalls()).toBeGreaterThan(m0));
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
    fireEvent.click(screen.getByRole("button", { name: /刪除/ }));
    expect(props.onDelete).toHaveBeenCalledWith("desktop-shell-and-browser");
  });

  it("fires onRunVerb for analyze / archive actions", async () => {
    const props = makeProps();
    render(<RichDetailDrawer {...(props as never)} />);
    fireEvent.click(screen.getByRole("button", { name: /分析/ }));
    expect(props.onRunVerb).toHaveBeenCalledWith("analyze", "desktop-shell-and-browser");
    fireEvent.click(screen.getByRole("button", { name: /封存/ }));
    expect(props.onRunVerb).toHaveBeenCalledWith("archive", "desktop-shell-and-browser");
  });
});

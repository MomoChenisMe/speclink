import { describe, it, expect, vi, type Mock } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";

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

  it("forwards a drag-reorder drop to onMoveTask and reloads tasks.md", async () => {
    const props = makeProps({ onMoveTask: vi.fn().mockResolvedValue(undefined) });
    render(<RichDetailDrawer {...(props as never)} />);
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
    // 一次到位轉發 from/to（側別未指定）；resolve 後以檔案真相重讀。
    expect((props as { onMoveTask: Mock }).onMoveTask).toHaveBeenCalledWith(
      "desktop-shell-and-browser",
      2,
      5,
      undefined,
    );
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

import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, fireEvent, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";

// 既有中文斷言包 I18nProvider locale zh-TW 後照舊斷言（i18n 抽 key 的回歸保護）。
const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

import { KanbanBoard } from "../components/KanbanBoard";
import { DetailDrawer } from "../components/DetailDrawer";
import { parseTasks } from "../tasks";
import type { ChangeItem, ArtifactStatus } from "../adapter";

const changes: ChangeItem[] = [
  // 欄位由生命週期標記驅動——全完成＝已就緒 ＞ started_at 或任務完成數>0＝進行中
  // ＞ 其餘＝提案中（剛 propose 完、全未勾、未開工）。
  { name: "proposing-x", status: "in-progress", totalTasks: 28, completedTasks: 0 },
  { name: "working-y", status: "in-progress", totalTasks: 10, completedTasks: 4, startedAt: "2026-07-06" },
  // 無章有進度（手改 tasks.md / agent 直改 / git pull 等繞道）——派生管顯示。
  { name: "progressed-w", status: "in-progress", totalTasks: 14, completedTasks: 2 },
  { name: "ready-z", status: "done", totalTasks: 5, completedTasks: 5 },
];

function column(id: string): HTMLElement {
  return document.querySelector(`[data-column="${id}"]`) as HTMLElement;
}

describe("KanbanBoard", () => {
  it("places each change in its lifecycle column (Chinese labels, no archived column)", () => {
    render(<KanbanBoard changes={changes} />);
    expect(screen.getByText("提案中")).toBeTruthy();
    expect(screen.getByText("進行中")).toBeTruthy();
    expect(screen.getByText("已就緒")).toBeTruthy();
    expect(within(column("proposed")).getByText("proposing-x")).toBeTruthy();
    expect(within(column("in-progress")).getByText("working-y")).toBeTruthy();
    // 無 started_at 而有任務進度的卡片列於進行中欄（spec Scenario「無章而有任務進度列於進行中」）。
    expect(within(column("in-progress")).getByText("progressed-w")).toBeTruthy();
    expect(within(column("ready")).getByText("ready-z")).toBeTruthy();
    // 封存欄不在看板（拖曳時才浮現落點）
    expect(column("archived")).toBeNull();
  });

  it("shows the archive button only on ready cards", () => {
    render(<KanbanBoard changes={changes} onArchive={vi.fn()} />);
    const readyCard = screen.getByText("ready-z").closest("[data-change]") as HTMLElement;
    expect(within(readyCard).getByRole("button", { name: /封存/ })).toBeTruthy();
    const workingCard = screen.getByText("working-y").closest("[data-change]") as HTMLElement;
    expect(within(workingCard).queryByRole("button", { name: /封存/ })).toBeNull();
  });

  it("fires onArchive from a ready card's archive button", () => {
    const onArchive = vi.fn();
    render(<KanbanBoard changes={changes} onArchive={onArchive} />);
    const readyCard = screen.getByText("ready-z").closest("[data-change]") as HTMLElement;
    fireEvent.click(within(readyCard).getByRole("button", { name: /封存/ }));
    expect(onArchive).toHaveBeenCalledWith("ready-z");
  });

  it("opens a change when its card body is clicked", () => {
    const onOpenChange = vi.fn();
    render(<KanbanBoard changes={changes} onOpenChange={onOpenChange} />);
    fireEvent.click(screen.getByText("working-y"));
    expect(onOpenChange).toHaveBeenCalledWith("working-y");
  });

  it("renders English strings under the en locale", () => {
    // 5.3：至少一個元件於 en 下渲染英文字串（i18n 生效的正向證明）。
    rtlRender(
      <I18nProvider locale="en">
        <KanbanBoard changes={changes} />
      </I18nProvider>,
    );
    expect(screen.getByText("Proposed")).toBeTruthy();
    expect(screen.getByText("In progress")).toBeTruthy();
    expect(screen.getByText("Ready")).toBeTruthy();
    expect(screen.queryByText("提案中")).toBeNull();
  });

  it("card copy button copies the change name without opening the card", () => {
    const onOpenChange = vi.fn();
    const writeText = vi.fn();
    Object.assign(navigator, { clipboard: { writeText } });
    render(<KanbanBoard changes={changes} onOpenChange={onOpenChange} />);
    const card = screen.getByText("working-y").closest("[data-change]") as HTMLElement;
    fireEvent.click(within(card).getByLabelText("複製名稱"));
    expect(writeText).toHaveBeenCalledWith("working-y");
    expect(onOpenChange).not.toHaveBeenCalled();
  });
});

describe("parseTasks", () => {
  it("parses checked and unchecked checkbox lines, ignoring other lines", () => {
    const md = "## Group\n\n- [x] done one\n- [ ] todo two\nsome prose\n- [X] done three\n";
    const tasks = parseTasks(md);
    expect(tasks).toHaveLength(3);
    expect(tasks.filter((t) => t.done)).toHaveLength(2);
    expect(tasks[1]).toEqual({ done: false, text: "todo two" });
  });
});

describe("DetailDrawer", () => {
  const artifacts: ArtifactStatus[] = [
    { id: "proposal", outputPath: "proposal.md", status: "done" },
    { id: "tasks", outputPath: "tasks.md", status: "ready" },
  ];
  it("renders artifacts and a task checklist when open", () => {
    render(
      <DetailDrawer
        open
        onOpenChange={() => {}}
        changeName="working-y"
        artifacts={artifacts}
        tasksMarkdown={"- [x] a\n- [ ] b\n"}
        doc={"## Why\nbody"}
      />,
    );
    expect(screen.getByText("working-y")).toBeTruthy();
    expect(screen.getByText("proposal")).toBeTruthy();
    expect(screen.getByText("a")).toBeTruthy();
    expect(screen.getByText("b")).toBeTruthy();
    expect(screen.getByText(/1\/2/)).toBeTruthy();
  });
});

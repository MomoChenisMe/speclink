import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent, within } from "@testing-library/react";

import { KanbanBoard } from "../components/KanbanBoard";
import { DetailDrawer } from "../components/DetailDrawer";
import { parseTasks } from "../tasks";
import type { ChangeItem, ArtifactStatus } from "../adapter";

const changes: ChangeItem[] = [
  { name: "proposing-x", status: "in-progress", totalTasks: 0, completedTasks: 0 },
  { name: "working-y", status: "in-progress", totalTasks: 10, completedTasks: 4 },
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

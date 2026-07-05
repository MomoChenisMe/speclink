import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";

import { TaskList } from "../components/TaskList";
import { parseTaskDoc } from "../tasks";

const MD = "## 1. Group A\n\n- [ ] 1.1 first\n- [x] 1.2 second\n\n## 2. Group B\n\n- [ ] 2.1 third\n";

describe("parseTaskDoc", () => {
  it("yields groups and ordinal-numbered tasks", () => {
    const items = parseTaskDoc(MD);
    expect(items).toEqual([
      { kind: "group", text: "1. Group A" },
      { kind: "task", ordinal: 1, done: false, text: "1.1 first" },
      { kind: "task", ordinal: 2, done: true, text: "1.2 second" },
      { kind: "group", text: "2. Group B" },
      { kind: "task", ordinal: 3, done: false, text: "2.1 third" },
    ]);
  });
});

describe("TaskList", () => {
  it("renders groups, checked state and fires onToggle with ordinal", () => {
    const onToggle = vi.fn();
    render(<TaskList markdown={MD} onToggle={onToggle} />);
    expect(screen.getByText("1. Group A")).toBeTruthy();
    const first = screen.getByLabelText("任務 1") as HTMLInputElement;
    const second = screen.getByLabelText("任務 2") as HTMLInputElement;
    expect(first.checked).toBe(false);
    expect(second.checked).toBe(true);
    fireEvent.click(first);
    expect(onToggle).toHaveBeenCalledWith(1, true);
    fireEvent.click(second);
    expect(onToggle).toHaveBeenCalledWith(2, false);
  });

  it("fires onMove up/down and disables edges", () => {
    const onMove = vi.fn();
    render(<TaskList markdown={MD} onMove={onMove} />);
    expect((screen.getByLabelText("上移任務 1") as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByLabelText("下移任務 3") as HTMLButtonElement).disabled).toBe(true);
    fireEvent.click(screen.getByLabelText("下移任務 1"));
    expect(onMove).toHaveBeenCalledWith(1, "down");
    fireEvent.click(screen.getByLabelText("上移任務 3"));
    expect(onMove).toHaveBeenCalledWith(3, "up");
  });
});

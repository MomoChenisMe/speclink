import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";

import { TaskList } from "../components/TaskList";
import { parseTaskDoc, resolveDropTarget } from "../tasks";

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

  it("renders a drag handle per task and no step buttons", () => {
    // spec 需求「任務清單拖放排序與自動重編號」：把手取代上下箭頭。
    render(<TaskList markdown={MD} onReorder={vi.fn()} />);
    expect(screen.getByLabelText("拖曳任務 1")).toBeTruthy();
    expect(screen.getByLabelText("拖曳任務 2")).toBeTruthy();
    expect(screen.getByLabelText("拖曳任務 3")).toBeTruthy();
    expect(screen.queryByLabelText("上移任務 1")).toBeNull();
    expect(screen.queryByLabelText("下移任務 1")).toBeNull();
  });

  it("readOnly renders neither handles nor interactive checkboxes", () => {
    render(<TaskList markdown={MD} readOnly />);
    expect(screen.queryByLabelText("拖曳任務 1")).toBeNull();
    expect((screen.getByLabelText("任務 1") as HTMLInputElement).disabled).toBe(true);
  });
});

describe("resolveDropTarget（拖放落點解析，design D6）", () => {
  // items: [g-0(群組1), task1, task2, g-3(群組2), task3, g-5(空群組)]
  const items = parseTaskDoc(
    "## 1. A\n\n- [ ] 1.1 a\n- [ ] 1.2 b\n\n## 2. B\n\n- [ ] 2.1 c\n\n## 3. 空的\n",
  );

  it("落點為任務：轉發 ordinal、側別留給方向推斷", () => {
    expect(resolveDropTarget(items, 1, 3)).toEqual({ to: 3 });
  });

  it("落點為群組標題：解析為組首任務 ordinal＋before=true", () => {
    expect(resolveDropTarget(items, 1, "g-3")).toEqual({ to: 3, before: true });
  });

  it("落點為空群組標題：無效落點回 null（不觸發 onReorder）", () => {
    expect(resolveDropTarget(items, 1, "g-5")).toBeNull();
  });

  it("落點為自己：回 null", () => {
    expect(resolveDropTarget(items, 2, 2)).toBeNull();
  });

  // spec Example「標題落點雙向」——標題是組界槽，依 active 相對位置解析。
  describe("雙向標題落點", () => {
    // ## 1. 前段: 1.1 甲(1)、1.2 乙(2)；## 2. 後段: 2.1 丙(3)、2.2 丁(4)
    const twoGroups = parseTaskDoc(
      "## 1. 前段\n\n- [ ] 1.1 甲\n- [ ] 1.2 乙\n\n## 2. 後段\n\n- [ ] 2.1 丙\n- [ ] 2.2 丁\n",
    );

    it("上方來→成為組首（before=true）", () => {
      expect(resolveDropTarget(twoGroups, 2, "g-3")).toEqual({ to: 3, before: true });
    });

    it("下方來（含組首自己）→移到標題之前、錨定上一群組末任務（before=false）", () => {
      // 乙已成 2.1 後拖回「## 2」標題：## 1: 甲(1)；## 2: 乙(2)、丙(3)、丁(4)
      const afterPromote = parseTaskDoc(
        "## 1. 前段\n\n- [ ] 1.1 甲\n\n## 2. 後段\n\n- [ ] 2.1 乙\n- [ ] 2.2 丙\n- [ ] 2.3 丁\n",
      );
      expect(resolveDropTarget(afterPromote, 2, "g-2")).toEqual({ to: 1, before: false });
      // 更深處的任務拖到標題也走同一規則（丙 → 上一組末＝甲）
      expect(resolveDropTarget(afterPromote, 3, "g-2")).toEqual({ to: 1, before: false });
    });

    it("標題之前無任務可錨定（檔首標題）→ null", () => {
      const headFirst = parseTaskDoc("## 1. X\n\n- [ ] 1.1 a\n- [ ] 1.2 b\n");
      expect(resolveDropTarget(headFirst, 2, "g-0")).toBeNull();
    });
  });
});

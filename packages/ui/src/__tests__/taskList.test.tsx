import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, fireEvent } from "@testing-library/react";
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

import { TaskList } from "../components/TaskList";
import { SUB_LABEL_CLS } from "../components/SectionedDoc";
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
    const first = screen.getByRole("checkbox", { name: "任務 1" });
    const second = screen.getByRole("checkbox", { name: "任務 2" });
    expect(first.getAttribute("aria-checked")).toBe("false");
    expect(second.getAttribute("aria-checked")).toBe("true");
    fireEvent.click(first);
    expect(onToggle).toHaveBeenCalledWith(1, true, undefined);
    fireEvent.click(second);
    expect(onToggle).toHaveBeenCalledWith(2, false, undefined);
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

  it("uses 16px type for task text", () => {
    // spec「markdown 內容保留文件結構呈現」：任務文字 16px 與內文對齊。
    render(<TaskList markdown={MD} />);
    expect(screen.getByText("1.1 first").className).toContain("text-base");
  });

  it("群組標題與章節標籤同款式（任務群組標題與章節標籤同款式，design D6 次級款＝原尺寸）", () => {
    render(<TaskList markdown={MD} />);
    const heading = screen.getByText("1. Group A");
    // 標籤家族次級款（共用常數 SUB_LABEL_CLS）：粗體、字級同內文基準 16px、小於章節主標題。
    for (const cls of SUB_LABEL_CLS.split(" ")) expect(heading.className).toContain(cls);
    for (const stale of ["text-xl", "text-xs", "uppercase", "tracking-wider", "text-muted-foreground"]) {
      expect(heading.className).not.toContain(stale);
    }
    // 標題文字照來源，不翻譯不改寫。
    expect(heading.textContent).toBe("1. Group A");
  });

  it("readOnly renders neither handles nor interactive checkboxes", () => {
    render(<TaskList markdown={MD} readOnly />);
    expect(screen.queryByLabelText("拖曳任務 1")).toBeNull();
    expect((screen.getByRole("checkbox", { name: "任務 1" }) as HTMLButtonElement).disabled).toBe(true);
  });

  it("無 onReorder 時不渲染把手（remote capability 停用），勾選與工具列照常", () => {
    // remote-workspace-data spec「capability 驅動停用且不偽造缺口」：拖排寫回
    // 缺席即整段不掛——絕不渲染點了沒事的假把手。
    const onToggle = vi.fn();
    const onSetAll = vi.fn();
    render(<TaskList markdown={MD} onToggle={onToggle} onSetAll={onSetAll} />);
    expect(screen.queryByLabelText("拖曳任務 1")).toBeNull();
    // 勾選照常互動。
    fireEvent.click(screen.getByRole("checkbox", { name: "任務 1" }));
    expect(onToggle).toHaveBeenCalledWith(1, true, undefined);
    // 批次工具列照常（setAllTasks 在 remote 是組合實作、仍支援）。
    fireEvent.click(screen.getByRole("button", { name: /全部已完成/ }));
    expect(onSetAll).toHaveBeenCalledWith(true);
  });
});

// spec「表單控制項與按鈕以主題化元件呈現」（desktop-shadcn-controls）
describe("TaskList 勾選框主題化", () => {
  it("勾選框為 checkbox 角色的非 input 元素，aria-checked 反映完成態", () => {
    render(<TaskList markdown={MD} onToggle={vi.fn()} />);
    const first = screen.getByRole("checkbox", { name: "任務 1" });
    const second = screen.getByRole("checkbox", { name: "任務 2" });
    expect(first.tagName).not.toBe("INPUT");
    expect(first.getAttribute("aria-checked")).toBe("false");
    expect(second.getAttribute("aria-checked")).toBe("true");
  });

  it("空白鍵切換觸發與點擊相同的 onToggle", async () => {
    const onToggle = vi.fn();
    render(<TaskList markdown={MD} onToggle={onToggle} />);
    const first = screen.getByRole("checkbox", { name: "任務 1" });
    // 主題化原語（button 元素）——原生 input 在此守衛即紅。
    expect(first.tagName).not.toBe("INPUT");
    first.focus();
    await userEvent.keyboard(" ");
    expect(onToggle).toHaveBeenCalledWith(1, true, undefined);
    fireEvent.click(first);
    // 點擊與空白鍵走同一回呼、同一參數。
    expect(onToggle).toHaveBeenNthCalledWith(2, 1, true, undefined);
  });

  it("readOnly 時勾選框不可互動", () => {
    const onToggle = vi.fn();
    render(<TaskList markdown={MD} readOnly onToggle={onToggle} />);
    const first = screen.getByRole("checkbox", { name: "任務 1" });
    expect(first.tagName).not.toBe("INPUT");
    expect((first as HTMLButtonElement).disabled).toBe(true);
    fireEvent.click(first);
    expect(onToggle).not.toHaveBeenCalled();
  });
});

// spec「任務分頁提供批次操作工具列」（desktop-task-interactions）
describe("TaskList 工具列", () => {
  it("renders the three batch actions and fires onSetAll", () => {
    const onSetAll = vi.fn();
    render(<TaskList markdown={MD} onSetAll={onSetAll} />);
    fireEvent.click(screen.getByRole("button", { name: "全部已完成" }));
    expect(onSetAll).toHaveBeenCalledWith(true);
    fireEvent.click(screen.getByRole("button", { name: "重置任務" }));
    expect(onSetAll).toHaveBeenCalledWith(false);
  });

  it("disables 全部已完成 and 下一個未完成 when everything is done", () => {
    const allDone = "## 1. G\n\n- [x] 1.1 a\n- [x] 1.2 b\n";
    render(<TaskList markdown={allDone} onSetAll={vi.fn()} />);
    expect((screen.getByRole("button", { name: "全部已完成" }) as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByRole("button", { name: /下一個未完成/ }) as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByRole("button", { name: "重置任務" }) as HTMLButtonElement).disabled).toBe(false);
  });

  it("readOnly hides the toolbar", () => {
    render(<TaskList markdown={MD} readOnly />);
    expect(screen.queryByRole("button", { name: "全部已完成" })).toBeNull();
    expect(screen.queryByRole("button", { name: "重置任務" })).toBeNull();
  });

  it("下一個未完成 scrolls the first unchecked task into view and highlights it", () => {
    const scrollSpy = vi.fn();
    Element.prototype.scrollIntoView = scrollSpy;
    render(<TaskList markdown={MD} onSetAll={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: /下一個未完成/ }));
    expect(scrollSpy).toHaveBeenCalled();
    // 第一個未完成＝任務 1（1.1 first），其列帶短暫高亮標記。
    const row = screen.getByText("1.1 first").closest("[data-highlight]");
    expect(row?.getAttribute("data-highlight")).toBe("true");
  });

  it("n key triggers the same next-undone locate, but not while typing", () => {
    const scrollSpy = vi.fn();
    Element.prototype.scrollIntoView = scrollSpy;
    const { container } = render(
      <div>
        <input aria-label="其他輸入框" />
        <TaskList markdown={MD} onSetAll={vi.fn()} />
      </div>,
    );
    fireEvent.keyDown(window, { key: "n" });
    expect(scrollSpy).toHaveBeenCalledTimes(1);
    // 輸入框打字不觸發定位。
    fireEvent.keyDown(container.querySelector("input")!, { key: "n" });
    expect(scrollSpy).toHaveBeenCalledTimes(1);
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

// --- stable task ID（spec task-identity「UI 剝離 ID 註解並以 stable ID 操作」）---

import { setTaskMark, taskKey } from "../tasks";

const TID_A = "tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV";
const TID_B = "tsk_01BX5ZZKBKACTAV9WEVGEMMVRZ";
const MD_IDS = `## 1. Group A\n\n- [ ] 1.1 first <!-- speclink-task:${TID_A} -->\n- [x] 1.2 second <!-- speclink-task:${TID_B} -->\n`;

describe("stable task IDs", () => {
  it("parseTaskDoc 剝離標記入 stableId，顯示文字與無註解時相同", () => {
    const withIds = parseTaskDoc(MD_IDS).filter((i) => i.kind === "task");
    const without = parseTaskDoc("## 1. Group A\n\n- [ ] 1.1 first\n- [x] 1.2 second\n").filter(
      (i) => i.kind === "task",
    );
    expect(withIds.map((t) => (t.kind === "task" ? t.text : ""))).toEqual(
      without.map((t) => (t.kind === "task" ? t.text : "")),
    );
    expect(withIds[0]).toMatchObject({ ordinal: 1, stableId: TID_A });
    expect(withIds[1]).toMatchObject({ ordinal: 2, stableId: TID_B });
    expect(without.every((t) => t.kind === "task" && t.stableId === undefined)).toBe(true);
  });

  it("taskKey 以 stable ID 為 key、無 ID 退回 ordinal", () => {
    expect(taskKey({ kind: "task", ordinal: 3, done: false, text: "x", stableId: TID_A })).toBe(
      TID_A,
    );
    expect(taskKey({ kind: "task", ordinal: 3, done: false, text: "x" })).toBe("3");
  });

  it("清單顯示不含標記、勾選請求攜 stable ID（無 ID 走 ordinal 相容）", () => {
    const onToggle = vi.fn();
    const stamped = render(<TaskList markdown={MD_IDS} onToggle={onToggle} />);
    expect(screen.getByText("1.1 first")).toBeTruthy();
    expect(document.body.textContent).not.toContain("speclink-task");
    fireEvent.click(screen.getByRole("checkbox", { name: "任務 1" }));
    expect(onToggle).toHaveBeenCalledWith(1, true, TID_A);
    stamped.unmount();
    // 無 ID 任務：stableId 缺席 → ordinal 相容路徑。
    const bare = vi.fn();
    render(<TaskList markdown={"- [ ] 1.1 solo\n"} onToggle={bare} />);
    fireEvent.click(screen.getByRole("checkbox", { name: "任務 1" }));
    expect(bare).toHaveBeenCalledWith(1, true, undefined);
  });

  it("setTaskMark 樂觀就地改寫保留行尾註解原文", () => {
    const next = setTaskMark(`- [ ] 1.1 first <!-- speclink-task:${TID_A} -->\n`, 1, true);
    expect(next).toBe(`- [x] 1.1 first <!-- speclink-task:${TID_A} -->\n`);
  });
});

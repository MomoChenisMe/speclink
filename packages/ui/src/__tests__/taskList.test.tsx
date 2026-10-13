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
import { SECTION_HEADING_CLS } from "../components/DocParts";
import { parseTaskDoc, parseTasks, resolveDropTarget } from "../tasks";

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

  // spec task-identity「UI 剝離 ID 註解並以 stable ID 操作」：行首標記前綴剝離,
  // `[M]` 落 manual 旗標、歷史遺留的 `[P]` 只剝不承載。
  it("strips leading markers and flags only [M] as manual", () => {
    const items = parseTaskDoc(
      "- [ ] [M] 手動驗證匯入結果\n- [ ] [M] 至外部服務放置金鑰\n- [x] [P] 舊任務\n- [x] [P] [M] 混用\n- [ ] 無標記\n",
    );
    expect(items).toEqual([
      { kind: "task", ordinal: 1, done: false, text: "手動驗證匯入結果", manual: true },
      // 非測試類的手動任務同樣解析——標記語意不判讀描述內容。
      { kind: "task", ordinal: 2, done: false, text: "至外部服務放置金鑰", manual: true },
      { kind: "task", ordinal: 3, done: true, text: "舊任務" },
      { kind: "task", ordinal: 4, done: true, text: "混用", manual: true },
      { kind: "task", ordinal: 5, done: false, text: "無標記" },
    ]);
  });

  // 與引擎同構(crates/engine/speclink-core/src/lifecycle/tasks.rs 的 parse/strip_markers 案例表
  // 對齊):checkbox 後恰一個空格才進標記槽,多餘空白不被吃掉;剝離後修剪與
  // display.trim() 一致。
  it("checkbox 後多一個空格不進標記槽——與引擎同構", () => {
    // `- [ ]  [M] x`(兩空格):引擎判非 manual、描述帶字面 [M],UI 必須一致,
    // 否則徽章與計數/閘門互相矛盾。
    expect(parseTaskDoc("- [ ]  [M] 手測\n")[0]).toEqual({
      kind: "task",
      ordinal: 1,
      done: false,
      text: "[M] 手測",
    });
    expect(parseTasks("- [ ]  [M] 手測\n")[0]).toEqual({ done: false, text: "[M] 手測" });
  });

  it("標記剝離後修剪前後空白——與引擎 display.trim() 同構", () => {
    expect(parseTaskDoc("- [ ] [M]  雙空格描述\n")[0]).toEqual({
      kind: "task",
      ordinal: 1,
      done: false,
      text: "雙空格描述",
      manual: true,
    });
  });

  it("strips each marker at most once and keeps ID comments out of the text", () => {
    // 至多一次：第二個 [P] 是描述的一部分,不被吃掉。
    expect(parseTaskDoc("- [ ] [P] [P] 兩個\n")[0]).toEqual({
      kind: "task",
      ordinal: 1,
      done: false,
      text: "[P] 兩個",
    });
    // 前綴剝離與行尾 ID 註解剝離兩者並存。
    expect(parseTaskDoc("- [ ] [M] 手測 <!-- speclink-task:tsk_A -->\n")[0]).toEqual({
      kind: "task",
      ordinal: 1,
      done: false,
      text: "手測",
      stableId: "tsk_A",
      manual: true,
    });
  });
});

describe("TaskList", () => {
  it("renders groups, checked state and fires onToggle with ordinal", () => {
    const onToggle = vi.fn();
    render(<TaskList markdown={MD} onToggle={onToggle} />);
    expect(screen.getByText("Group A")).toBeTruthy();
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

  it("uses 15px type for task text", () => {
    // spec「markdown 內容保留文件結構呈現」：閱讀卡內任務列文字 15px 與內文對齊。
    render(<TaskList markdown={MD} />);
    expect(screen.getByText("first").className).toContain("text-[15px]");
  });

  it("群組卡頭與章節標題同款式（spec「任務群組標題與章節標籤同款式」，design D5）", () => {
    const { container } = render(<TaskList markdown={MD} />);
    const heading = screen.getByText("Group A");
    // 16px 粗體章節標題（共用常數 SECTION_HEADING_CLS）。
    for (const cls of SECTION_HEADING_CLS.split(" ")) expect(heading.className).toContain(cls);
    for (const stale of ["text-xl", "text-xs", "uppercase", "tracking-wider", "text-muted-foreground"]) {
      expect(heading.className).not.toContain(stale);
    }
    // 標題文字照來源（序號拆成左端序號籤，不含序號後的空白），不翻譯不改寫。
    expect(heading.tagName).toBe("H3");
    expect(heading.textContent).toBe("Group A");
    expect(heading.previousElementSibling!.textContent).toBe("1");
    // 本頁目錄錨點（desktop-detail-doc-tree design D5）。
    expect(heading.hasAttribute("data-doc-heading")).toBe(true);
    expect(container.querySelectorAll("[data-task-group]")).toHaveLength(2);
  });

  // spec desktop-app「任務列的手動任務徽章」（desktop-change-doc-views design D14：整列淡底＋列尾徽章）
  describe("手動任務徽章", () => {
    const MARKED = "- [ ] [M] 3.2 手動驗證匯入結果\n- [x] 3.1 跑全套測試\n";
    const rowOf = (text: string) => screen.getByText(text).closest("label") as HTMLElement;

    it("`[M]` 列整列淡底、徽章在標題列尾；勾選框與序號起始欄與無標記列同位", () => {
      // Scenario「手動任務列的徽章呈現」。
      render(<TaskList markdown={MARKED} />);
      const badge = screen.getByLabelText("手動");
      // 精確比對：文案退回「手動測試」等任何形變都會被抓到，不需模糊反向網。
      expect(badge.textContent).toBe("手動");
      expect(screen.getAllByLabelText("手動")).toHaveLength(1);
      const marked = rowOf("手動驗證匯入結果");
      const plain = rowOf("跑全套測試");
      expect(marked.hasAttribute("data-task-manual")).toBe(true);
      expect(marked.className).toContain("bg-status-manual/12");
      expect(plain.hasAttribute("data-task-manual")).toBe(false);
      expect(plain.className).not.toContain("bg-status-manual");
      // 徽章在標題那一行的列尾（展開鈕之前），不在文字流裡、不獨佔一行。
      const titleRow = badge.parentElement as HTMLElement;
      expect(titleRow.hasAttribute("data-task-title-row")).toBe(true);
      expect(titleRow.firstElementChild!.textContent).toBe("3.2");
      expect(titleRow.children[1].textContent).toBe("手動驗證匯入結果");
      expect(Array.from(titleRow.children).indexOf(badge)).toBeGreaterThan(1);
      // 兩列結構相同：描述欄只有標題那一行（沒有徽章行），且緊接 checkbox。
      for (const row of [marked, plain]) {
        const col = row.querySelector("[data-task-title-row]")!.parentElement as HTMLElement;
        expect(col.firstElementChild!.hasAttribute("data-task-title-row")).toBe(true);
        expect(col.previousElementSibling?.getAttribute("role")).toBe("checkbox");
      }
    });

    it("徽章配色取手動語意色（暖黃），不是 muted 灰階、也不是進行中藍", () => {
      render(<TaskList markdown={MARKED} />);
      const badge = screen.getByLabelText("手動");
      expect(badge.className).toContain("text-status-manual");
      expect(badge.className).not.toContain("bg-muted");
      expect(badge.className).not.toContain("status-progress");
    });

    it("長描述換行時徽章仍在標題列尾、不佔描述的行", () => {
      // Scenario「長描述換行不動徽章」：jsdom 量不到實際換行，以結構為準。
      const long = `- [ ] [M] ${"很長的描述".repeat(40)}\n`;
      render(<TaskList markdown={long} />);
      const badge = screen.getByLabelText("手動");
      expect((badge.parentElement as HTMLElement).hasAttribute("data-task-title-row")).toBe(true);
      expect(badge.className).toContain("shrink-0");
    });

    it("勾完文字劃線、整列淡底退掉，徽章保留且不劃線", () => {
      // Scenario「勾完徽章保留」。
      render(<TaskList markdown={"- [x] [M] 3.2 手動驗證匯入結果\n"} />);
      expect(screen.getByText("手動驗證匯入結果").className).toContain("line-through");
      expect(rowOf("手動驗證匯入結果").className).not.toContain("bg-status-manual");
      const badge = screen.getByLabelText("手動");
      expect(badge.className).not.toContain("line-through");
    });

    it("舊 `[P]` 行剝離後無徽章、無淡底", () => {
      // Scenario「舊 [P] 行無徽章」。
      render(<TaskList markdown={"- [x] [P] 舊任務\n"} />);
      expect(screen.queryByLabelText("手動")).toBeNull();
      const row = rowOf("舊任務");
      expect(row.hasAttribute("data-task-manual")).toBe(false);
      expect(row.className).not.toContain("bg-status-manual");
    });

    it("無 `[M]` 任務的清單不長徽章與淡底", () => {
      const { container } = render(<TaskList markdown={MD} />);
      expect(screen.queryByLabelText("手動")).toBeNull();
      expect(container.querySelector("[data-task-manual]")).toBeNull();
    });

    it("可拖排時手動列同樣整列淡底", () => {
      render(<TaskList markdown={MARKED} onToggle={vi.fn()} onReorder={vi.fn()} />);
      expect(rowOf("手動驗證匯入結果").className).toContain("bg-status-manual/12");
    });

    it("en 徽章文案為 Manual（放寬後不提 test）", () => {
      // 語意自「手動測試」放寬為「需使用者親手操作的任務」——兩語系的文案
      // 都不得再把標記說成只限測試。
      rtlRender(
        <I18nProvider locale="en">
          <TaskList markdown={MARKED} />
        </I18nProvider>,
      );
      const badge = screen.getByLabelText("Manual");
      // 精確比對：退回 "Manual test"／"Manual Test" 任何大小寫形變都會被抓到。
      expect(badge.textContent).toBe("Manual");
    });
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

// spec「任務群組標題與章節標籤同款式」（desktop-change-doc-views design D5）：群組卡與三層任務列。
describe("TaskList 群組卡與三層任務列", () => {
  const LAYERED =
    "## 1. 解析器\n\n" +
    "- [ ] 1.1 新增 `a.ts`（design D3）：匯出 x；驗證：npm test\n" +
    "- [ ] 1.2 對齊匯出\n";

  it("三層切分：預設只有序號、標題與 D 籤；點展開鈕後顯示內文與驗證且 aria-expanded 為 true", () => {
    const { container } = render(<TaskList markdown={LAYERED} onToggle={vi.fn()} />);
    const titleRow = container.querySelector("[data-task-title-row]")!;
    expect(titleRow.querySelector("[data-task-title]")!.textContent).toBe("新增 a.ts");
    expect(titleRow.querySelector("[data-task-title] code")!.textContent).toBe("a.ts");
    expect(titleRow.querySelector("[data-task-decision]")!.textContent).toBe("D3");
    expect(screen.queryByText("匯出 x；")).toBeNull();
    expect(screen.queryByText("npm test")).toBeNull();
    const expand = screen.getByRole("button", { name: "展開任務 1" });
    expect(expand.getAttribute("aria-expanded")).toBe("false");
    fireEvent.click(expand);
    expect(screen.getByRole("button", { name: "收合任務 1" }).getAttribute("aria-expanded")).toBe("true");
    expect(screen.getByText("匯出 x；")).toBeTruthy();
    expect(screen.getByText("驗證")).toBeTruthy();
    expect(screen.getByText("npm test")).toBeTruthy();
    // 沒有內文與驗證的任務不長展開鈕。
    expect(screen.queryByRole("button", { name: "展開任務 2" })).toBeNull();
  });

  it.each([
    ["可拖排", true],
    ["無拖排", false],
  ])("%s：點展開鈕與 D 籤不切換勾選，D 籤回報決策編號", (_label, sortable) => {
    const onToggle = vi.fn();
    const onJumpDecision = vi.fn();
    const { container } = render(
      <TaskList
        markdown={LAYERED}
        onToggle={onToggle}
        onReorder={sortable ? vi.fn() : undefined}
        onJumpDecision={onJumpDecision}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "展開任務 1" }));
    fireEvent.click(container.querySelector("[data-task-decision]")!);
    fireEvent.click(screen.getByText("匯出 x；"));
    expect(onJumpDecision).toHaveBeenCalledWith("D3");
    expect(onToggle).not.toHaveBeenCalled();
  });

  it("兩個群組成兩張卡，卡頭右端為各自的小進度", () => {
    const md =
      "## 1. 甲\n\n- [x] 1.1 a\n- [x] 1.2 b\n\n" +
      "## 2. 乙\n\n- [x] 2.1 c\n- [x] 2.2 d\n- [x] 2.3 e\n- [ ] 2.4 f\n- [ ] 2.5 g\n";
    const { container } = render(<TaskList markdown={md} />);
    const cards = Array.from(container.querySelectorAll("[data-task-group]"));
    expect(cards.map((c) => c.querySelector("[data-task-group-progress]")!.textContent)).toEqual(["2/2", "3/5"]);
    expect(cards.map((c) => c.querySelector("h3")!.textContent)).toEqual(["甲", "乙"]);
  });

  it("第一個群組之前的任務以無卡頭清單呈現", () => {
    const { container } = render(<TaskList markdown={"- [ ] 前導任務\n\n## 1. 甲\n\n- [ ] 1.1 a\n"} />);
    expect(container.querySelectorAll("[data-task-group]")).toHaveLength(1);
    expect(screen.getByText("前導任務").closest("[data-task-group]")).toBeNull();
  });

  it("唯讀時展開鈕可用、勾選框 disabled", () => {
    render(<TaskList markdown={LAYERED} readOnly />);
    expect((screen.getByRole("checkbox", { name: "任務 1" }) as HTMLButtonElement).disabled).toBe(true);
    const expand = screen.getByRole("button", { name: "展開任務 1" }) as HTMLButtonElement;
    expect(expand.disabled).toBe(false);
    fireEvent.click(expand);
    expect(screen.getByText("npm test")).toBeTruthy();
  });
});

// spec「任務群組標題與章節標籤同款式」Scenario「手動提示框前往任務列」（design D14）：頁的「前往」經 locate 定位。
describe("TaskList 定位（locate）", () => {
  it.each([
    ["可寫", false],
    ["唯讀", true],
  ])("%s：locate 命中時捲到該列、短暫高亮並回報完成，勾選不變", (_label, readOnly) => {
    const scrollSpy = vi.fn();
    Element.prototype.scrollIntoView = scrollSpy;
    const onLocated = vi.fn();
    const onToggle = vi.fn();
    render(
      <TaskList markdown={MD} readOnly={readOnly} onToggle={onToggle} locate={{ ordinal: 3, seq: 1 }} onLocated={onLocated} />,
    );
    expect(scrollSpy).toHaveBeenCalled();
    expect(screen.getByText("third").closest("[data-highlight]")?.getAttribute("data-highlight")).toBe("true");
    expect(screen.getByText("first").closest("[data-highlight]")?.getAttribute("data-highlight")).toBe("false");
    expect(onLocated).toHaveBeenCalledTimes(1);
    expect(onToggle).not.toHaveBeenCalled();
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

// spec「任務列整列可點」（design D8）：整列為勾選框的 label；把手不切換；唯讀與
// 批次寫回中（busy）不回應；拖曳仍只由把手啟動。
describe("TaskList 整列可點", () => {
  const rowOf = (text: string) => screen.getByText(text).closest("label") as HTMLLabelElement;

  it.each([
    ["可拖排", true],
    ["無拖排", false],
  ])("%s：點任務文字呼叫 onToggle 一次，效果與點勾選框相同", (_label, sortable) => {
    const onToggle = vi.fn();
    render(<TaskList markdown={MD} onToggle={onToggle} onReorder={sortable ? vi.fn() : undefined} />);
    const row = rowOf("first");
    expect(row).toBeTruthy();
    const checkbox = screen.getByRole("checkbox", { name: "任務 1" });
    expect(row.htmlFor).toBe(checkbox.id);
    expect(row.className).toContain("cursor-(--cursor-interaction)");
    expect(row.className).toContain("hover:bg-foreground/5");
    fireEvent.click(screen.getByText("first"));
    expect(onToggle).toHaveBeenCalledTimes(1);
    expect(onToggle).toHaveBeenCalledWith(1, true, undefined);
  });

  it("點拖曳把手不切換", () => {
    const onToggle = vi.fn();
    render(<TaskList markdown={MD} onToggle={onToggle} onReorder={vi.fn()} />);
    const handle = screen.getByLabelText("拖曳任務 1");
    fireEvent.pointerDown(handle);
    fireEvent.click(handle);
    expect(onToggle).not.toHaveBeenCalled();
  });

  it("唯讀列點擊不切換，label 無 htmlFor、游標預設且沒有 hover 底色", () => {
    const onToggle = vi.fn();
    render(<TaskList markdown={MD} readOnly onToggle={onToggle} />);
    const row = rowOf("first");
    expect(row.htmlFor).toBe("");
    expect(row.className).toContain("cursor-default");
    expect(row.className).not.toContain("hover:");
    fireEvent.click(screen.getByText("first"));
    expect(onToggle).not.toHaveBeenCalled();
  });

  it.each([
    ["可拖排", true],
    ["無拖排", false],
  ])("%s：批次寫回中（busy）點列不切換", (_label, sortable) => {
    const onToggle = vi.fn();
    render(
      <TaskList markdown={MD} busy onToggle={onToggle} onReorder={sortable ? vi.fn() : undefined} />,
    );
    const row = rowOf("first");
    expect(row.htmlFor).toBe("");
    expect(row.className).toContain("cursor-default");
    fireEvent.click(screen.getByText("first"));
    fireEvent.click(screen.getByRole("checkbox", { name: "任務 1" }));
    expect(onToggle).not.toHaveBeenCalled();
  });

  it("拖曳仍只由把手啟動：sortable 屬性只在把手上，列本身沒有", () => {
    render(<TaskList markdown={MD} onToggle={vi.fn()} onReorder={vi.fn()} />);
    const row = rowOf("first");
    expect(row.getAttribute("aria-roledescription")).toBeNull();
    expect(screen.getByLabelText("拖曳任務 1").getAttribute("aria-roledescription")).toBe("sortable");
  });

  it("有寫入能力時工具列附灰字提示；沒有把手時不提把手；唯讀時沒有", () => {
    const { unmount } = render(
      <TaskList markdown={MD} onToggle={vi.fn()} onReorder={vi.fn()} onSetAll={vi.fn()} />,
    );
    const hint = screen.getByText("點整列即可勾選；拖曳把手改順序");
    expect(hint.className).toContain("text-muted-foreground");
    expect(hint.className).toContain("text-xs");
    unmount();
    const plain = render(<TaskList markdown={MD} onToggle={vi.fn()} onSetAll={vi.fn()} />);
    expect(screen.getByText("點整列即可勾選")).toBeTruthy();
    expect(screen.queryByText(/拖曳把手/)).toBeNull();
    plain.unmount();
    render(<TaskList markdown={MD} readOnly />);
    expect(screen.queryByText(/點整列即可勾選/)).toBeNull();
  });

  it.each([
    ["可拖排", true],
    ["無拖排", false],
  ])("%s：拖選文字後的 click 不切換；直接點勾選框不受影響", (_label, sortable) => {
    const onToggle = vi.fn();
    render(<TaskList markdown={MD} onToggle={onToggle} onReorder={sortable ? vi.fn() : undefined} />);
    const selection = vi.spyOn(window, "getSelection").mockReturnValue({ toString: () => "first" } as Selection);
    fireEvent.click(screen.getByText("first"));
    expect(onToggle).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("checkbox", { name: "任務 1" }));
    expect(onToggle).toHaveBeenCalledTimes(1);
    selection.mockRestore();
  });

  it("雙擊任務文字選字：兩下點擊互相抵銷，勾選狀態不變", () => {
    const onToggle = vi.fn();
    const reorder = vi.fn();
    const { rerender } = render(<TaskList markdown={MD} onToggle={onToggle} onReorder={reorder} />);
    fireEvent.click(screen.getByText("first"), { detail: 1 });
    // 宿主樂觀改寫 tasks.md；第二下按下時瀏覽器已選取該字。
    rerender(<TaskList markdown={MD.replace("- [ ] 1.1", "- [x] 1.1")} onToggle={onToggle} onReorder={reorder} />);
    const text = screen.getByText("first");
    const selection = vi.spyOn(window, "getSelection").mockReturnValue({ toString: () => "first" } as Selection);
    fireEvent.click(text, { detail: 2 });
    selection.mockRestore();
    expect(onToggle.mock.calls).toEqual([
      [1, true, undefined],
      [1, false, undefined],
    ]);
  });

  it("拖曳進行中按 Esc 標為已處理：只取消拖曳，不讓詳情頁的 Esc 關頁", () => {
    const onDragActiveChange = vi.fn();
    render(
      <TaskList markdown={MD} onToggle={vi.fn()} onReorder={vi.fn()} onDragActiveChange={onDragActiveChange} />,
    );
    expect(fireEvent.keyDown(window, { key: "Escape" })).toBe(true);
    const handle = screen.getByLabelText("拖曳任務 1");
    handle.focus();
    fireEvent.keyDown(handle, { code: "Space", key: " " });
    expect(onDragActiveChange).toHaveBeenLastCalledWith(true);
    expect(fireEvent.keyDown(window, { key: "Escape" })).toBe(false);
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
    const row = screen.getByText("first").closest("[data-highlight]");
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
    expect(screen.getByText("first")).toBeTruthy();
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

  // spec task-identity Scenario「標記前綴剝離且寫回保留」：顯示剝離只影響呈現,
  // 寫回是行級改寫——`[M] ` 前綴在檔案裡原樣留著。
  it("setTaskMark 保留行首標記前綴原文", () => {
    expect(setTaskMark("- [ ] [M] 手動驗證匯入結果\n", 1, true)).toBe(
      "- [x] [M] 手動驗證匯入結果\n",
    );
    expect(setTaskMark("- [x] [P] 舊任務\n", 1, false)).toBe("- [ ] [P] 舊任務\n");
  });
});

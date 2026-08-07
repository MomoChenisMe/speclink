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

// 捕捉 DndContext 的接線 props（onDragStart/onDragEnd）——jsdom 無法模擬真實
// pointer 拖曳手勢，改直接驅動接線回呼驗證落點渲染與 archived 分支守門；
// 元件本身仍以原 DndContext 渲染，既有測試不受影響。
const captured = vi.hoisted(() => ({ dnd: null as any }));
vi.mock("@dnd-kit/core", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@dnd-kit/core")>();
  const Wrapped = (props: any) => {
    captured.dnd = props;
    return <actual.DndContext {...props} />;
  };
  return { ...actual, DndContext: Wrapped };
});

import { act } from "@testing-library/react";

import { KanbanBoard, DRAG_ACTIVATION_DISTANCE } from "../components/KanbanBoard";
import { archiveZoneVisible, cardDndId, resolveCardDrop, type ColumnCards } from "../boardDnd";
import { parseTasks } from "../tasks";
import { SEMANTIC_TONE } from "../tone";
import type { ChangeItem, DiscussionLists } from "../adapter";

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

  it("shows the revert action only on in-progress cards (spec: 退回動作僅於進行中變更出現)", () => {
    render(<KanbanBoard changes={changes} onRevert={vi.fn()} />);
    // 蓋章的進行中卡與無章有進度的派生進行中卡皆顯示。
    const workingCard = screen.getByText("working-y").closest("[data-change]") as HTMLElement;
    expect(within(workingCard).getByRole("button", { name: /退回提案中/ })).toBeTruthy();
    const progressedCard = screen.getByText("progressed-w").closest("[data-change]") as HTMLElement;
    expect(within(progressedCard).getByRole("button", { name: /退回提案中/ })).toBeTruthy();
    // 提案中與已就緒不顯示。
    const proposedCard = screen.getByText("proposing-x").closest("[data-change]") as HTMLElement;
    expect(within(proposedCard).queryByRole("button", { name: /退回提案中/ })).toBeNull();
    const readyCard = screen.getByText("ready-z").closest("[data-change]") as HTMLElement;
    expect(within(readyCard).queryByRole("button", { name: /退回提案中/ })).toBeNull();
  });

  it("fires onRevert from an in-progress card's revert button without opening the card", () => {
    const onRevert = vi.fn();
    const onOpenChange = vi.fn();
    render(<KanbanBoard changes={changes} onRevert={onRevert} onOpenChange={onOpenChange} />);
    const workingCard = screen.getByText("working-y").closest("[data-change]") as HTMLElement;
    fireEvent.click(within(workingCard).getByRole("button", { name: /退回提案中/ }));
    expect(onRevert).toHaveBeenCalledWith("working-y");
    expect(onOpenChange).not.toHaveBeenCalled();
  });

  it("clicking the action row's blank area opens the card instead of being swallowed", () => {
    const onRevert = vi.fn();
    const onArchive = vi.fn();
    const onOpenChange = vi.fn();
    render(
      <KanbanBoard changes={changes} onRevert={onRevert} onArchive={onArchive} onOpenChange={onOpenChange} />,
    );
    // 進行中卡退回鈕列：點按鈕旁空白 → 開卡、不觸發退回。
    const workingCard = screen.getByText("working-y").closest("[data-change]") as HTMLElement;
    fireEvent.click(
      within(workingCard).getByRole("button", { name: /退回提案中/ }).parentElement as HTMLElement,
    );
    expect(onOpenChange).toHaveBeenCalledWith("working-y");
    expect(onRevert).not.toHaveBeenCalled();
    // 已就緒卡封存鈕列同理。
    const readyCard = screen.getByText("ready-z").closest("[data-change]") as HTMLElement;
    fireEvent.click(
      within(readyCard).getByRole("button", { name: /封存/ }).parentElement as HTMLElement,
    );
    expect(onOpenChange).toHaveBeenCalledWith("ready-z");
    expect(onArchive).not.toHaveBeenCalled();
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

  it("shows the restale badge only on cards whose change carries a restaleFrom flag", () => {
    // 「待重新反映」徽章：restaleFrom 非空的卡片顯示、為空/缺席不顯示；不影響欄位派生。
    const withFlag: ChangeItem[] = [
      { name: "stale-a", status: "in-progress", totalTasks: 4, completedTasks: 1, restaleFrom: ["alpha"] },
      { name: "fresh-b", status: "in-progress", totalTasks: 4, completedTasks: 1 },
    ];
    render(<KanbanBoard changes={withFlag} />);
    const staleCard = screen.getByText("stale-a").closest("[data-change]") as HTMLElement;
    const restale = within(staleCard).getByLabelText("待重新反映");
    expect(restale).toBeTruthy();
    // 警示語意走語意色表（含深色變體）——舊實作只有單一 amber-500，深色底下偏暗。
    expect(restale.className).toContain(SEMANTIC_TONE.warning);
    const freshCard = screen.getByText("fresh-b").closest("[data-change]") as HTMLElement;
    expect(within(freshCard).queryByLabelText("待重新反映")).toBeNull();
    // 兩張皆在進行中欄——徽章與欄位歸屬正交。
    expect(within(column("in-progress")).getByText("stale-a")).toBeTruthy();
    expect(within(column("in-progress")).getByText("fresh-b")).toBeTruthy();
  });

  it("shows the worktree marker only on cards whose change is mapped to a worktree", () => {
    // spec worktree-overlay「desktop 看板的 worktree 呈現」：有映射的卡片帶標示，
    // 無映射不帶——標示與欄位歸屬正交。
    const withWorktree: ChangeItem[] = [
      {
        name: "wt-a",
        status: "in-progress",
        totalTasks: 4,
        completedTasks: 1,
        worktree: { branch: "speclink/wt-a", path: "/work/speclink.worktrees/wt-a" },
      },
      { name: "plain-b", status: "in-progress", totalTasks: 4, completedTasks: 1 },
    ];
    render(<KanbanBoard changes={withWorktree} />);
    const wtCard = screen.getByText("wt-a").closest("[data-change]") as HTMLElement;
    const marker = within(wtCard).getByLabelText("worktree");
    expect(marker).toBeTruthy();
    // spec「worktree 標示以藍呈現」：worktree 掛著＝工作正於副本進行中，屬狀態
    // 而非靜態 metadata，走「進行中」語意色，不用主色淡化。
    expect(marker.className).toContain(SEMANTIC_TONE.inProgress);
    expect(marker.className).not.toContain("text-primary/60");
    const plainCard = screen.getByText("plain-b").closest("[data-change]") as HTMLElement;
    expect(within(plainCard).queryByLabelText("worktree")).toBeNull();
    expect(within(column("in-progress")).getByText("wt-a")).toBeTruthy();
  });

  it("shows the invalid-metadata marker only on cards whose change carries metaError", () => {
    // spec「看板照常開啟並標記損壞卡」：metaError 卡帶最小 invalid 標記
    // （tooltip 載解析原因），有效卡不帶；兩張卡照常列於看板。
    const withInvalid: ChangeItem[] = [
      { name: "broken-a", status: "in-progress", totalTasks: 4, completedTasks: 1, metaError: "invalid type: sequence" },
      { name: "valid-b", status: "in-progress", totalTasks: 4, completedTasks: 1 },
    ];
    render(<KanbanBoard changes={withInvalid} />);
    const brokenCard = screen.getByText("broken-a").closest("[data-change]") as HTMLElement;
    const marker = within(brokenCard).getByLabelText("metadata 損壞");
    expect(marker).toBeTruthy();
    expect(marker.getAttribute("title")).toBeNull();
    const validCard = screen.getByText("valid-b").closest("[data-change]") as HTMLElement;
    expect(within(validCard).queryByLabelText("metadata 損壞")).toBeNull();
    expect(within(column("in-progress")).getByText("broken-a")).toBeTruthy();
  });

  it("變更卡顯示建立者首字母圓標（有 createdBy）、無則省略（D3）", () => {
    const withAuthor: ChangeItem[] = [
      { name: "auth-a", status: "in-progress", totalTasks: 4, completedTasks: 1, createdBy: "Momo Chen <momo@example.com>" },
      { name: "anon-b", status: "in-progress", totalTasks: 4, completedTasks: 1 },
    ];
    render(<KanbanBoard changes={withAuthor} />);
    const authCard = screen.getByText("auth-a").closest("[data-change]") as HTMLElement;
    const avatar = within(authCard).getByLabelText("Momo Chen <momo@example.com>");
    expect(avatar.textContent).toBe("M");
    // 建立者是靜態 metadata，走中性；主色留給連結／互動／進度。
    expect(avatar.className).toContain("bg-muted");
    expect(avatar.className).not.toContain("bg-primary");
    const anonCard = screen.getByText("anon-b").closest("[data-change]") as HTMLElement;
    expect(within(anonCard).queryByLabelText(/@/)).toBeNull();
  });

  it("關係指示（來自討論／待重新反映）改用主題化提示，不再帶原生 title（D3）", () => {
    const rel: ChangeItem[] = [
      { name: "rel-a", status: "in-progress", totalTasks: 4, completedTasks: 1, fromDiscussions: ["alpha"], restaleFrom: ["beta"] },
    ];
    render(<KanbanBoard changes={rel} />);
    const card = screen.getByText("rel-a").closest("[data-change]") as HTMLElement;
    const from = within(card).getByLabelText("來自討論");
    const restale = within(card).getByLabelText("待重新反映");
    // shadcn Tooltip 取代原生 title——指示元件不再帶 title 屬性。
    expect(from.getAttribute("title")).toBeNull();
    expect(restale.getAttribute("title")).toBeNull();
  });
});

describe("KanbanBoard search（看板搜尋過濾卡片）", () => {
  // GIVEN 對齊 delta spec 的 Example 表：提案中欄兩張變更卡＋討論欄一張卡。
  const searchChanges: ChangeItem[] = [
    { name: "desktop-acp-agent", status: "in-progress", totalTasks: 8, completedTasks: 0, summary: "桌面版 ACP 代理" },
    { name: "web-role-views", status: "in-progress", totalTasks: 6, completedTasks: 0, summary: "情境 1 的角色檢視" },
  ];
  const searchDiscussions: DiscussionLists = {
    active: [
      { slug: "gui-auto-stamp", topic: "GUI 勾任務自動蓋開工章", status: "open", rounds: 2, created: "2026-07-06", promotedTo: [] },
    ],
    archived: [],
  };

  function renderBoard(query: string, onQuery = vi.fn()) {
    render(
      <KanbanBoard
        changes={searchChanges}
        discussions={searchDiscussions}
        query={query}
        onQuery={onQuery}
      />,
    );
    return onQuery;
  }

  function count(id: string): string {
    return within(column(id)).getByTestId("column-count").textContent ?? "";
  }

  it("renders a search input above the columns when query/onQuery are provided", () => {
    const onQuery = renderBoard("");
    const input = screen.getByPlaceholderText("搜尋看板卡片…");
    fireEvent.change(input, { target: { value: "desk" } });
    expect(onQuery).toHaveBeenCalledWith("desk");
  });

  it("renders the English search placeholder under the en locale", () => {
    // 同類搜尋輸入（已封存頁、清單）皆已 i18n 化——看板不落單（verify SUGGESTION）。
    rtlRender(
      <I18nProvider locale="en">
        <KanbanBoard changes={searchChanges} discussions={searchDiscussions} query="" onQuery={vi.fn()} />
      </I18nProvider>,
    );
    expect(screen.getByPlaceholderText("Search board cards…")).toBeTruthy();
  });

  it("does not render a search input when query is not provided", () => {
    render(<KanbanBoard changes={searchChanges} discussions={searchDiscussions} />);
    expect(screen.queryByPlaceholderText("搜尋看板卡片…")).toBeNull();
  });

  // spec Example 表逐行參數化：輸入 → 提案中欄顯示（計數）／討論欄顯示（計數）。
  // 卡片以 data-change／data-discussion 查詢——命中字段經高亮 mark 拆分（design D7），
  // 完整文字節點斷言不再適用。
  it.each([
    ["desktop", ["desktop-acp-agent"], "1", [], "0"], // 名稱子字串命中
    ["桌面", ["desktop-acp-agent"], "1", [], "0"], // 摘要命中
    [" GUI ", [], "0", ["gui-auto-stamp"], "1"], // 去頭尾空白、不分大小寫
    ["", ["desktop-acp-agent", "web-role-views"], "2", ["gui-auto-stamp"], "1"], // 清空還原全量
  ])(
    "query %j filters proposed column and discussion column per the spec example table",
    (query, proposedShown, proposedCount, discussionShown, discussionCount) => {
      renderBoard(query);
      for (const name of proposedShown) {
        expect(column("proposed").querySelector(`[data-change="${name}"]`)).toBeTruthy();
      }
      expect(column("proposed").querySelectorAll("[data-change]")).toHaveLength(proposedShown.length);
      expect(count("proposed")).toBe(proposedCount);
      for (const slug of discussionShown) {
        expect(column("discussions").querySelector(`[data-discussion="${slug}"]`)).toBeTruthy();
      }
      expect(column("discussions").querySelectorAll("[data-discussion]")).toHaveLength(discussionShown.length);
      expect(count("discussions")).toBe(discussionCount);
    },
  );

  it("matches case-insensitively and against the discussion slug", () => {
    // 需求文字：不分大小寫；討論卡以主題「與 slug」比對。
    renderBoard("gui");
    expect(column("discussions").querySelector('[data-discussion="gui-auto-stamp"]')).toBeTruthy();
    expect(count("discussions")).toBe("1");
  });

  it("matches against the discussion slug", () => {
    renderBoard("auto-stamp");
    expect(within(column("discussions")).getByText("GUI 勾任務自動蓋開工章")).toBeTruthy();
    expect(count("discussions")).toBe("1");
    expect(count("proposed")).toBe("0");
  });

  it("whitespace-only query shows everything (treated as empty)", () => {
    renderBoard("   ");
    expect(count("proposed")).toBe("2");
    expect(count("discussions")).toBe("1");
  });

  it("no match leaves empty columns with zero counts but keeps the column structure", () => {
    renderBoard("zzz-no-match");
    for (const id of ["discussions", "proposed", "in-progress", "ready"]) {
      expect(column(id)).toBeTruthy();
      expect(count(id)).toBe("0");
      expect(column(id).querySelectorAll("[data-change], [data-discussion]")).toHaveLength(0);
    }
  });
});

describe("KanbanBoard 拖排（design D6）", () => {
  // 每欄可見卡的識別碼（視覺序）——resolveCardDrop 的輸入形狀。
  const cols: ColumnCards[] = [
    { kind: "discussion", ids: ["d-one", "d-two"] },
    { kind: "change", ids: ["a", "b", "c"] },
    { kind: "change", ids: ["x"] },
  ];

  // dragEnd 落點解析：同欄放開 → 以 arrayMove 後的相鄰卡為 prevId/nextId。
  it.each([
    ["c", "a", null, "a"], // 拖到欄頂：prev=null
    ["a", "b", "b", "c"], // 向下一格：插於 b、c 之間
    ["a", "c", "c", null], // 拖到欄底：next=null
  ])(
    "same-column drop of %s over %s resolves prev=%s next=%s",
    (active, over, prev, next) => {
      const r = resolveCardDrop(cols, cardDndId("change", active), cardDndId("change", over));
      expect(r).toEqual({ kind: "change", id: active, prevId: prev, nextId: next });
    },
  );

  it("resolves discussion drops within the discussion column", () => {
    const r = resolveCardDrop(cols, cardDndId("discussion", "d-two"), cardDndId("discussion", "d-one"));
    expect(r).toEqual({ kind: "discussion", id: "d-two", prevId: null, nextId: "d-one" });
  });

  it("cross-column drops resolve to null (snap back, zero writes)", () => {
    // spec「跨欄拖曳不改變變更階段」：跨欄放開不得產生 reorder 呼叫。
    expect(resolveCardDrop(cols, cardDndId("change", "a"), cardDndId("change", "x"))).toBeNull();
    expect(resolveCardDrop(cols, cardDndId("change", "a"), cardDndId("discussion", "d-one"))).toBeNull();
    expect(resolveCardDrop(cols, cardDndId("discussion", "d-one"), cardDndId("change", "b"))).toBeNull();
  });

  it("column containers and the archive drop zone resolve to null", () => {
    // 封存落點走既有 onArchive 路徑、欄容器不成落點——皆不產生 reorder。
    expect(resolveCardDrop(cols, cardDndId("change", "a"), "archived")).toBeNull();
    expect(resolveCardDrop(cols, cardDndId("change", "a"), "proposed")).toBeNull();
    expect(resolveCardDrop(cols, cardDndId("change", "a"), cardDndId("change", "a"))).toBeNull();
  });

  it("change cards mount as sortables with a localized drag label", () => {
    render(<KanbanBoard changes={changes} onReorder={vi.fn()} />);
    const card = screen.getByText("working-y").closest('[aria-roledescription="sortable"]') as HTMLElement;
    expect(card).toBeTruthy();
    expect(card.getAttribute("aria-label")).toContain("working-y");
  });

  it("does not mount change cards as sortables when reorder is unavailable", () => {
    render(<KanbanBoard changes={changes} />);
    expect(document.querySelector('[aria-roledescription="sortable"]')).toBeNull();
  });

  it("shows the provided reorder-unavailable reason without adding a drag handle", () => {
    render(
      <KanbanBoard
        changes={changes}
        reorderUnavailableReason="此 Workspace 目前為唯讀狀態——無法拖曳調整卡片順序"
      />,
    );
    expect(screen.getByText(/唯讀狀態.*無法拖曳調整卡片順序/)).toBeTruthy();
    expect(document.querySelector('[aria-roledescription="sortable"]')).toBeNull();
  });

  it("pins the pointer activation distance at 8 (click-through lesson)", () => {
    // dnd-kit 可拖曳元素必須設 distance 8，否則單擊被拖曳監聽吃掉（CLAUDE.md）。
    expect(DRAG_ACTIVATION_DISTANCE).toBe(8);
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

// spec 需求「看板搜尋過濾卡片」的命中呈現（design D7）：子字串命中高亮、
// 僅模糊命中不高亮、全文命中卡片呈 snippet 行。
describe("命中高亮與 snippet（design D7）", () => {
  const one: ChangeItem[] = [
    { name: "engine-typed-core", status: "in-progress", totalTasks: 2, completedTasks: 0 },
  ];
  const cardEl = () =>
    document.querySelector('[data-change="engine-typed-core"]') as HTMLElement;

  it("子字串命中於卡名以 mark 高亮命中原文", () => {
    render(<KanbanBoard changes={one} query="engine" onQuery={() => {}} />);
    const mark = cardEl().querySelector("mark");
    expect(mark).toBeTruthy();
    expect(mark!.textContent).toBe("engine");
    // 搜尋高亮是「注意這裡」的警示層級，不是互動——改琥珀 mark，不佔用主色。
    expect(mark!.className).toContain("amber");
    expect(mark!.className).not.toContain("primary");
  });

  it("僅模糊命中（無連續子字串）顯示卡片但不高亮", () => {
    render(<KanbanBoard changes={one} query="etc" onQuery={() => {}} />);
    expect(cardEl()).toBeTruthy();
    expect(cardEl().querySelector("mark")).toBeNull();
  });

  it("全文命中卡片呈 snippet 行：artifact 名＋裁切前後文＋命中高亮", () => {
    render(
      <KanbanBoard
        changes={one}
        query="dispatch"
        onQuery={() => {}}
        fulltextHits={[
          {
            kind: "change",
            id: "engine-typed-core",
            artifact: "design.md",
            snippet: "…唯一 dispatch 相容層…",
          },
        ]}
      />,
    );
    const card = cardEl();
    const snippet = card.querySelector("[data-snippet]") as HTMLElement;
    expect(snippet).toBeTruthy();
    expect(within(snippet).getByText(/design\.md/)).toBeTruthy();
    expect(snippet.textContent).toContain("相容層");
    const mark = snippet.querySelector("mark");
    expect(mark?.textContent).toBe("dispatch");
  });
});

// spec 需求「拖曳封存落點以浮層呈現」的 jsdom 可驗部分（design D8＋
// archive-readiness-gating D3）：浮現條件純函式＋接線經捕捉的 DndContext
// props 驅動；真實拖曳的欄寬零變動與放開行為屬真視窗驗證。
describe("封存落點浮層（design D8 + archive-readiness-gating D3）", () => {
  // 就緒名單：落點僅於拖曳已就緒變更卡時浮現（fixture 中僅 ready-z 就緒）。
  const readyIds: ReadonlySet<string> = new Set(["ready-z"]);

  it("archiveZoneVisible：已就緒變更卡拖曳才浮現、非就緒與討論卡不浮現", () => {
    expect(archiveZoneVisible(cardDndId("change", "ready-z"), readyIds)).toBe(true);
    expect(archiveZoneVisible(cardDndId("change", "working-y"), readyIds)).toBe(false);
    expect(archiveZoneVisible(cardDndId("discussion", "collab"), readyIds)).toBe(false);
    expect(archiveZoneVisible("archived", readyIds)).toBe(false);
    expect(archiveZoneVisible(null, readyIds)).toBe(false);
  });

  it("未拖曳時看板不渲染封存落點", () => {
    render(<KanbanBoard changes={changes} />);
    expect(document.querySelector('[data-column="archived"]')).toBeNull();
  });

  const startDrag = (id: string) => act(() => captured.dnd.onDragStart({ active: { id } }));
  const endDrag = (id: string, overId: string | null) =>
    act(() =>
      captured.dnd.onDragEnd({ active: { id }, over: overId ? { id: overId } : null }),
    );

  it("拖曳進行中變更卡不渲染封存落點；已就緒卡渲染", () => {
    render(<KanbanBoard changes={changes} onArchive={vi.fn()} />);
    startDrag(cardDndId("change", "working-y"));
    expect(document.querySelector('[data-column="archived"]')).toBeNull();
    endDrag(cardDndId("change", "working-y"), null);
    startDrag(cardDndId("change", "ready-z"));
    expect(document.querySelector('[data-column="archived"]')).toBeTruthy();
  });

  it("dragEnd 於 over=archived 且卡非就緒時不觸發 onArchive；就緒卡照常", () => {
    const onArchive = vi.fn();
    render(<KanbanBoard changes={changes} onArchive={onArchive} />);
    // 落點不可見時理論上不會 over，此守門防快速手勢與測試路徑（design D3）。
    startDrag(cardDndId("change", "working-y"));
    endDrag(cardDndId("change", "working-y"), "archived");
    expect(onArchive).not.toHaveBeenCalled();
    startDrag(cardDndId("change", "ready-z"));
    endDrag(cardDndId("change", "ready-z"), "archived");
    expect(onArchive).toHaveBeenCalledWith("ready-z");
  });
});

// spec「看板卡片統一解剖學」的變更卡（board-card-anatomy design D1/D2）：
// 等寬標題單行截斷、省略號收尾、複製鈕行內尾隨（釘住 desktop-ux-polish 落地的位置
// 不回退）、whyExcerpt 描述列、變更卡無狀態 chip（所在欄即階段）。
describe("看板卡片統一解剖學（變更卡）", () => {
  const anatomyChanges: ChangeItem[] = [
    {
      name: "with-desc",
      status: "in-progress",
      totalTasks: 21,
      completedTasks: 5,
      createdBy: "Momo <m@example.com>",
      whyExcerpt: "看板卡片各自演化、無共用骨架。",
    },
    { name: "no-desc", status: "in-progress", totalTasks: 4, completedTasks: 1, whyExcerpt: null },
  ];

  it("標題等寬字型、單行不折行、複製鈕與標題同列尾隨", () => {
    render(<KanbanBoard changes={anatomyChanges} />);
    const title = screen.getByText("with-desc");
    expect(title.className).toContain("font-mono");
    // 名稱恆單行：過長就地截斷、以省略號收尾（truncate 一併帶入 nowrap），
    // 不折到次行把複製鈕帶下去。
    expect(title.className).toContain("truncate");
    // 行內尾隨：複製鈕與標題同在名稱列容器內，不是被推到卡片右緣的兄弟元素。
    const nameRow = title.parentElement as HTMLElement;
    expect(within(nameRow).getByLabelText("複製名稱")).toBeTruthy();
  });

  it("whyExcerpt 渲染為一行截斷描述列，null 時整列缺席", () => {
    render(<KanbanBoard changes={anatomyChanges} />);
    const withDesc = screen.getByText("with-desc").closest("[data-change]") as HTMLElement;
    const desc = within(withDesc).getByText("看板卡片各自演化、無共用骨架。");
    expect(desc.className).toContain("truncate");
    const noDesc = screen.getByText("no-desc").closest("[data-change]") as HTMLElement;
    expect(noDesc.querySelector("[data-desc]")).toBeNull();
  });

  it("變更卡無狀態 chip（所在欄即階段）", () => {
    render(<KanbanBoard changes={anatomyChanges} />);
    const card = screen.getByText("with-desc").closest("[data-change]") as HTMLElement;
    expect(within(card).queryByText(/提案中|進行中|已就緒/)).toBeNull();
  });
});

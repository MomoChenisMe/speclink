// spec desktop-roadmap「路線圖依波次排列變更」「路線圖列出尚未轉出的刀」「路線圖的成環與空狀態」
// 「路線圖畫出依賴線並亮起路線」：版面與互動；線的座標由手動在 dev 視窗驗收（jsdom 沒有版面）。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, fireEvent, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { BoardViewSwitch, RoadmapView } from "../components/RoadmapView";
import type { ChangeItem, DiscussionLists, UnpromotedCutItem } from "../adapter";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

/** 正規化空白後的文字內容。 */
function text(el: Element | null | undefined): string {
  return (el?.textContent ?? "").replace(/\s+/g, " ").trim();
}

function station(name: string): HTMLElement {
  return document.querySelector(`[data-station="${name}"]`) as HTMLElement;
}

function item(
  name: string,
  wave: number | undefined,
  tasks: [done: number, total: number],
  dependsOn: string[] = [],
  blockedBy: string[] = [],
): ChangeItem {
  const [completedTasks, totalTasks] = tasks;
  const base: ChangeItem = { name, status: "x", totalTasks, completedTasks };
  return wave === undefined ? base : { ...base, wave, dependsOn, blockedBy };
}

// Scenario「兩波與下一個」：清單順序即 plan 的配置序。
const TWO_WAVES: ChangeItem[] = [
  item("add-a", 1, [0, 4]),
  item("add-c", 1, [2, 5]),
  item("add-b", 2, [0, 3], ["add-a"], ["add-a"]),
];

const PANEL_CUT: UnpromotedCutItem = {
  discussion: "herdr-speclink-panel",
  cut: "cut 4",
  title: "討論詳情頁改用引擎的刀",
  changes: ["discussion-cuts-from-engine"],
  unpromoted: ["discussion-cuts-from-engine"],
};

const PANEL_DISCUSSIONS: DiscussionLists = {
  active: [
    {
      slug: "herdr-speclink-panel",
      topic: "panel",
      status: "concluded",
      rounds: 3,
      created: "2026-11-01",
      promotedTo: ["desktop-deep-link"],
    },
  ],
  archived: [],
};

interface ViewProps {
  changes?: ChangeItem[];
  next?: string | null;
  unpromotedCuts?: UnpromotedCutItem[];
  planError?: string | null;
  discussions?: DiscussionLists;
  onOpenChange?: (name: string) => void;
  onOpenDiscussion?: (slug: string) => void;
  loading?: boolean;
}

function view({
  changes = TWO_WAVES,
  next = "add-a",
  unpromotedCuts = [],
  planError = null,
  discussions = { active: [], archived: [] },
  onOpenChange = vi.fn(),
  onOpenDiscussion = vi.fn(),
  loading = false,
}: ViewProps = {}) {
  return render(
    <RoadmapView
      title="變更"
      changes={changes}
      next={next}
      unpromotedCuts={unpromotedCuts}
      planError={planError}
      discussions={discussions}
      onOpenChange={onOpenChange}
      onOpenDiscussion={onOpenDiscussion}
      viewSwitch={<span data-testid="view-switch" />}
      loading={loading}
    />,
  );
}

describe("RoadmapView 波次欄與站（需求「路線圖依波次排列變更」）", () => {
  it("每一波一欄，欄頭寫第 N 波、變更數與並行／等待提示（Scenario「兩波與下一個」）", () => {
    view();
    const waves = [...document.querySelectorAll("[data-roadmap-wave]")];
    expect(waves.map((w) => w.getAttribute("data-roadmap-wave"))).toEqual(["1", "2"]);
    expect(text(waves[0].querySelector("[data-wave-head]"))).toBe("第 1 波 · 2 · 可以並行");
    expect(text(waves[1].querySelector("[data-wave-head]"))).toBe("第 2 波 · 1 · 等第 1 波");
    const names = (w: Element) => [...w.querySelectorAll("[data-station]")].map((s) => s.getAttribute("data-station"));
    expect(names(waves[0])).toEqual(["add-a", "add-c"]);
    expect(names(waves[1])).toEqual(["add-b"]);
  });

  it("頁首帶頁標題與宿主給的檢視切換", () => {
    view();
    expect(screen.getByRole("heading", { name: "變更" })).toBeTruthy();
    expect(screen.getByTestId("view-switch")).toBeTruthy();
  });

  // Example「站的階段文字」。
  it("站的次行依階段：外框 n/m、進度圓圈 n/m、已就緒、被擋的站變淡並寫出等誰", () => {
    view({
      changes: [
        item("add-a", 1, [0, 4]),
        item("add-c", 1, [2, 5]),
        item("add-d", 1, [6, 6]),
        item("add-b", 2, [0, 3], ["add-a"], ["add-a"]),
      ],
      next: null,
    });
    const status = (name: string) => text(station(name).querySelector("[data-station-status]"));
    expect(station("add-a").getAttribute("data-stage")).toBe("proposed");
    expect(status("add-a")).toBe("0/4");
    expect(station("add-c").getAttribute("data-stage")).toBe("in-progress");
    expect(station("add-c").querySelector("[data-progress-ring]")).not.toBeNull();
    expect(status("add-c")).toBe("2/5");
    expect(station("add-d").getAttribute("data-stage")).toBe("ready");
    expect(status("add-d")).toBe("已就緒");
    expect(status("add-b")).toBe("0/3");
    expect(station("add-b").hasAttribute("data-blocked")).toBe(true);
    expect(text(station("add-b").querySelector("[data-station-blocked]"))).toBe("等待：add-a");
    for (const name of ["add-a", "add-c", "add-d"]) {
      expect(station(name).hasAttribute("data-blocked"), name).toBe(false);
      expect(station(name).querySelector("[data-progress-ring]") !== null, name).toBe(name === "add-c");
    }
  });

  it("名稱等於 next 的站帶「下一個」籤；next 為 null 時沒有站帶籤", () => {
    const { unmount } = view();
    expect(text(station("add-a").querySelector("[data-next-badge]"))).toBe("下一個");
    expect(document.querySelectorAll("[data-next-badge]")).toHaveLength(1);
    unmount();
    view({ next: null });
    expect(document.querySelectorAll("[data-next-badge]")).toHaveLength(0);
  });

  it("點站與聚焦後按 Enter 都以變更名呼叫 onOpenChange", async () => {
    const onOpenChange = vi.fn();
    view({ onOpenChange });
    fireEvent.click(station("add-b"));
    expect(onOpenChange).toHaveBeenLastCalledWith("add-b");
    station("add-c").focus();
    await userEvent.keyboard("{Enter}");
    expect(onOpenChange).toHaveBeenLastCalledWith("add-c");
    expect(onOpenChange).toHaveBeenCalledTimes(2);
  });

  it("沒有波次的變更不成為站，頁首下方一行說明（Scenario「無法排程的變更不成為站」）", () => {
    view({
      changes: [item("good-a", 1, [0, 1]), item("broken-meta", undefined, [0, 1]), item("good-b", 1, [0, 1])],
      next: null,
    });
    expect(document.querySelectorAll("[data-station]")).toHaveLength(2);
    expect(station("broken-meta")).toBeNull();
    expect(text(document.querySelector("[data-roadmap-unplaced]"))).toBe("1 個變更無法排程，只在看板顯示");
  });

  it("每個變更都排得到時沒有無法排程的說明", () => {
    view();
    expect(document.querySelector("[data-roadmap-unplaced]")).toBeNull();
  });
});

describe("RoadmapView 尚未轉出欄（需求「路線圖列出尚未轉出的刀」）", () => {
  it("最右一欄列出尚未轉出的刀（Scenario「列出一把尚未轉出的刀」）", () => {
    view({ unpromotedCuts: [PANEL_CUT], discussions: PANEL_DISCUSSIONS });
    const columns = document.querySelector("[data-roadmap-columns]")!.children;
    const cuts = columns[columns.length - 1] as HTMLElement;
    expect(cuts.hasAttribute("data-roadmap-cuts")).toBe(true);
    expect(text(cuts.querySelector("[data-cuts-head]"))).toBe("尚未轉出 · 1");
    const card = within(cuts).getAllByText("herdr-speclink-panel")[0].closest("[data-roadmap-cut]") as HTMLElement;
    expect(card.className).toContain("border-dashed");
    const head = text(card.querySelector("[data-cut-head]"));
    expect(head).toContain("herdr-speclink-panel · cut 4");
    expect(head).toContain("尚未轉出");
    expect(text(card.querySelector("[data-cut-unpromoted]"))).toBe("discussion-cuts-from-engine");
    expect(text(card)).toContain("討論詳情頁改用引擎的刀");
    expect(text(card.querySelector("[data-command-chip]"))).toBe(
      "/speclink-propose --from-discussion herdr-speclink-panel",
    );
  });

  it("卡的順序與 unpromotedCuts 相同；範圍句為空字串時不寫", () => {
    const second: UnpromotedCutItem = { ...PANEL_CUT, discussion: "other-talk", cut: "cut 2", title: "" };
    view({ unpromotedCuts: [PANEL_CUT, second] });
    const heads = [...document.querySelectorAll("[data-roadmap-cut] [data-cut-head]")].map(text);
    expect(heads[0]).toContain("herdr-speclink-panel · cut 4");
    expect(heads[1]).toContain("other-talk · cut 2");
    expect(document.querySelectorAll("[data-roadmap-cut] [data-cut-title]")).toHaveLength(1);
  });

  it("討論 slug 在討論清單內時可點並開該討論（Scenario「點討論名稱開討論詳情」）", () => {
    const onOpenDiscussion = vi.fn();
    view({ unpromotedCuts: [PANEL_CUT], discussions: PANEL_DISCUSSIONS, onOpenDiscussion });
    fireEvent.click(screen.getByRole("button", { name: "herdr-speclink-panel" }));
    expect(onOpenDiscussion).toHaveBeenCalledWith("herdr-speclink-panel");
  });

  it("討論 slug 不在討論清單內時是不可點的文字", () => {
    view({ unpromotedCuts: [PANEL_CUT] });
    expect(screen.queryByRole("button", { name: "herdr-speclink-panel" })).toBeNull();
    expect(text(document.querySelector("[data-cut-head]"))).toContain("herdr-speclink-panel");
  });

  it("unpromotedCuts 為空時沒有這一欄（Scenario「全部刀都已轉出時沒有這一欄」）", () => {
    view();
    expect(document.querySelector("[data-roadmap-cuts]")).toBeNull();
  });
});

describe("RoadmapView 成環、空狀態與載入中（需求「路線圖的成環與空狀態」）", () => {
  it("planError 非空時沒有波次欄與線，顯示成環提示與引擎訊息（Scenario「依賴成環」）", () => {
    view({
      changes: [item("add-a", undefined, [0, 1], ["add-b"]), item("add-b", undefined, [0, 1], ["add-a"])],
      next: null,
      planError: "dependency cycle: add-a -> add-b -> add-a",
    });
    expect(document.querySelector("[data-roadmap-wave]")).toBeNull();
    expect(document.querySelector("[data-roadmap-edge]")).toBeNull();
    expect(text(screen.getByRole("note"))).toBe("依賴成環：dependency cycle: add-a -> add-b -> add-a");
    expect(document.querySelector("[data-roadmap-unplaced]")).toBeNull();
    expect(screen.queryByText("沒有進行中的變更")).toBeNull();
  });

  it("沒有站也沒有刀時顯示「沒有進行中的變更」（Scenario「沒有進行中的變更」）", () => {
    view({ changes: [], next: null });
    expect(screen.getByText("沒有進行中的變更")).toBeTruthy();
    expect(document.querySelector("[data-roadmap-wave]")).toBeNull();
  });

  it("只有尚未轉出的刀時不顯示空狀態", () => {
    view({ changes: [], next: null, unpromotedCuts: [PANEL_CUT] });
    expect(screen.queryByText("沒有進行中的變更")).toBeNull();
    expect(document.querySelector("[data-roadmap-cuts]")).not.toBeNull();
  });

  it("載入中呈現佔位卡、不顯示空狀態", () => {
    view({ changes: [], next: null, loading: true });
    expect(document.querySelector('[aria-busy="true"]')).not.toBeNull();
    expect(screen.queryByText("沒有進行中的變更")).toBeNull();
  });
});

describe("RoadmapView 依賴線與路線高亮（需求「路線圖畫出依賴線並亮起路線」）", () => {
  // Scenario「停在中間的站亮起整條路線」：add-b depends_on add-a、add-c depends_on add-b、
  // add-x 與三者無依賴；另加 add-y→add-z 一條路線外的線。
  const ROUTE: ChangeItem[] = [
    item("add-a", 1, [0, 1]),
    item("add-x", 1, [0, 1]),
    item("add-y", 1, [0, 1]),
    item("add-b", 2, [0, 1], ["add-a"], ["add-a"]),
    item("add-z", 2, [0, 1], ["add-y"], ["add-y"]),
    item("add-c", 3, [0, 1], ["add-b"], ["add-b"]),
  ];

  function edge(key: string): Element | null {
    return document.querySelector(`[data-roadmap-edge="${key}"]`);
  }

  function dimmedStations(): string[] {
    return [...document.querySelectorAll("[data-station][data-dimmed]")].map((s) => s.getAttribute("data-station")!);
  }

  function expectRouteOfAddB() {
    expect(dimmedStations().sort()).toEqual(["add-x", "add-y", "add-z"]);
    for (const key of ["add-a→add-b", "add-b→add-c"]) {
      expect(edge(key)!.hasAttribute("data-active"), key).toBe(true);
      expect(edge(key)!.hasAttribute("data-dimmed"), key).toBe(false);
    }
    expect(edge("add-y→add-z")!.hasAttribute("data-dimmed")).toBe(true);
    expect(edge("add-y→add-z")!.hasAttribute("data-active")).toBe(false);
  }

  function expectNothingHighlighted() {
    expect(document.querySelectorAll("[data-dimmed]")).toHaveLength(0);
    expect(document.querySelectorAll("[data-active]")).toHaveLength(0);
  }

  it("每條兩端都在路線圖上的依賴畫一條線", () => {
    view({ changes: ROUTE, next: null });
    const keys = [...document.querySelectorAll("[data-roadmap-edge]")].map((p) => p.getAttribute("data-roadmap-edge"));
    expect(keys.sort()).toEqual(["add-a→add-b", "add-b→add-c", "add-y→add-z"]);
    for (const path of document.querySelectorAll("[data-roadmap-edge]")) {
      expect(path.tagName.toLowerCase()).toBe("path");
      expect(path.closest("svg")!.getAttribute("class")).toContain("pointer-events-none");
    }
  });

  it("滑鼠停在 add-b：路線外的站與線變淡、路線上的線強調；移出後恢復", () => {
    view({ changes: ROUTE, next: null });
    expectNothingHighlighted();
    fireEvent.mouseEnter(station("add-b"));
    expectRouteOfAddB();
    fireEvent.mouseLeave(station("add-b"));
    expectNothingHighlighted();
  });

  it("焦點落在 add-a 時同樣亮起整條路線，失焦後恢復", () => {
    view({ changes: ROUTE, next: null });
    fireEvent.focus(station("add-a"));
    expectRouteOfAddB();
    fireEvent.blur(station("add-a"));
    expectNothingHighlighted();
  });

  it("滑鼠移出但焦點仍在站上時維持焦點的路線", () => {
    view({ changes: ROUTE, next: null });
    fireEvent.focus(station("add-a"));
    fireEvent.mouseEnter(station("add-x"));
    expect(dimmedStations().sort()).toEqual(["add-a", "add-b", "add-c", "add-y", "add-z"]);
    fireEvent.mouseLeave(station("add-x"));
    expectRouteOfAddB();
  });

  it("高亮不呼叫任何回呼", () => {
    const onOpenChange = vi.fn();
    view({ changes: ROUTE, next: null, onOpenChange });
    fireEvent.mouseEnter(station("add-b"));
    fireEvent.focus(station("add-c"));
    expect(onOpenChange).not.toHaveBeenCalled();
  });
});

describe("BoardViewSwitch（看板頁頁首的「看板｜路線圖」切換）", () => {
  it("兩個分頁標出目前的檢視，點另一個以其值呼叫 onChange", async () => {
    const onChange = vi.fn();
    render(<BoardViewSwitch value="cards" onChange={onChange} />);
    expect(screen.getByRole("tablist", { name: "看板檢視" })).toBeTruthy();
    expect(screen.getByRole("tab", { name: "看板" }).getAttribute("aria-selected")).toBe("true");
    expect(screen.getByRole("tab", { name: "路線圖" }).getAttribute("aria-selected")).toBe("false");
    await userEvent.click(screen.getByRole("tab", { name: "路線圖" }));
    expect(onChange).toHaveBeenCalledWith("roadmap");
  });
});

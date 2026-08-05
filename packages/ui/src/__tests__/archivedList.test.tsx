// spec 需求「已封存頁含討論節」（兩節卡片清單、點卡開抽屜、搜尋同時過濾）＋
// 「規格與封存卡片收合資訊」（封存變更卡與封存討論卡）：行內展開全數移除
//（正典「已封存變更可展開檢視」依 delta 移除，檢視由抽屜承接）。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, fireEvent, waitFor, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { ArchivedList } from "../components/ArchivedList";
import type { ArchivedItem, DiscussionItem } from "../adapter";

// 既有中文斷言包 I18nProvider locale zh-TW 後照舊斷言（i18n 抽 key 的回歸保護）。
const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

// spec Example「封存變更卡任務徽章配色分級」：20/21 未全完成（警示）＋48/48 全完成。
const WARN: ArchivedItem = {
  datedName: "2026-07-03-incomplete-change",
  date: "2026-07-03",
  name: "incomplete-change",
  tasksTotal: 21,
  tasksDone: 20,
  specCount: 3,
  createdBy: "MomoChen <momo@example.com>",
  fromDiscussions: ["alpha-ux", "beta-flow"],
};

const FULL: ArchivedItem = {
  datedName: "2026-07-05-desktop-shell-and-browser",
  date: "2026-07-05",
  name: "desktop-shell-and-browser",
  tasksTotal: 48,
  tasksDone: 48,
  specCount: 1,
  createdBy: null,
  fromDiscussions: [],
};

const BARE: ArchivedItem = {
  datedName: "2026-07-01-no-tasks",
  date: "2026-07-01",
  name: "no-tasks",
  specCount: 0,
  createdBy: null,
  fromDiscussions: [],
};

const DISCUSSION: DiscussionItem = {
  slug: "old-topic",
  topic: "Old settled topic",
  status: "promoted",
  rounds: 2,
  created: "2026-06-30",
  promotedTo: ["first-cut"],
};

function renderList(
  items: ArchivedItem[] = [WARN, FULL, BARE],
  over: Record<string, unknown> = {},
) {
  const onOpen = vi.fn();
  render(
    <ArchivedList
      archived={items}
      query=""
      onQuery={() => {}}
      archivedDiscussions={[DISCUSSION]}
      onOpen={onOpen}
      {...(over as object)}
    />,
  );
  return onOpen;
}

const card = (sel: string) => document.querySelector(sel) as HTMLElement;

describe("ArchivedList（封存變更卡）", () => {
  it("點卡觸發 onOpen（change target）；無 chevron 與行內展開", () => {
    const onOpen = renderList();
    fireEvent.click(screen.getByText("desktop-shell-and-browser"));
    expect(onOpen).toHaveBeenCalledWith({
      kind: "change",
      datedName: "2026-07-05-desktop-shell-and-browser",
    });
    // 行內展開移除：點擊後卡片內不出現分頁（頁面級「變更／討論」子頁籤除外）。
    const clicked = card('[data-archived="2026-07-05-desktop-shell-and-browser"]');
    expect(within(clicked).queryByRole("tab")).toBeNull();
    expect(document.querySelector(".lucide-chevron-right")).toBeNull();
    expect(document.querySelector(".lucide-chevron-down")).toBeNull();
    expect(document.querySelector("[aria-expanded]")).toBeNull();
  });

  it("任務徽章配色分級：未全完成琥珀警示、全完成一般樣式、無 tasks.md 不顯示", () => {
    renderList();
    // spec Example：前者警示樣式顯示 20/21，後者一般樣式顯示 48/48。
    const warnBadge = screen.getByText("20/21");
    expect(warnBadge.className).toContain("amber");
    const fullBadge = screen.getByText("48/48");
    expect(fullBadge.className).not.toContain("amber");
    // 任務數徽章維持 pill（狀態徽章例外，design D7 增補）。
    expect(warnBadge.className).toContain("rounded-full");
    expect(fullBadge.className).toContain("rounded-full");
    // 無 tasks.md 的封存項不顯示徽章。
    const bare = card('[data-archived="2026-07-01-no-tasks"]');
    expect(bare).toBeTruthy();
    expect(within(bare).queryByText(/\d+\/\d+/)).toBeNull();
  });

  it("卡片收合資訊：觸及規格數、createdBy 頭像圓點 tooltip、來源討論標記缺席不顯示", () => {
    renderList();
    const warn = card('[data-archived="2026-07-03-incomplete-change"]');
    // 觸及規格數。
    expect(within(warn).getByLabelText("觸及 3 份規格")).toBeTruthy();
    // createdBy 頭像圓點（與 ChangeCard 同款：首字母圓標、aria-label 全名）。
    const avatar = within(warn).getByLabelText("MomoChen <momo@example.com>");
    expect(avatar.textContent).toBe("M");
    // 靜態 metadata 走中性（與變更卡、討論卡同款）。
    expect(avatar.className).toContain("bg-muted");
    expect(avatar.className).not.toContain("bg-primary");
    // 來源討論 icon（tooltip 列 slug）。
    expect(within(warn).getByLabelText("來自討論")).toBeTruthy();
    // 缺席語意：無 createdBy／無來源討論的卡不顯示對應標記。
    const full = card('[data-archived="2026-07-05-desktop-shell-and-browser"]');
    expect(within(full).queryByLabelText("來自討論")).toBeNull();
    // specCount 0 → 觸及規格標記缺席。
    const bare = card('[data-archived="2026-07-01-no-tasks"]');
    expect(within(bare).queryByLabelText(/觸及/)).toBeNull();
  });

  it("複製鈕位於標題群組內，點擊複製封存名稱且不開抽屜", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", { value: { writeText }, configurable: true });
    const onOpen = renderList();
    const full = card('[data-archived="2026-07-05-desktop-shell-and-browser"]');
    const group = full.querySelector("[data-title-group]") as HTMLElement;
    expect(group).toBeTruthy();
    expect(within(group).getByText("desktop-shell-and-browser")).toBeTruthy();
    fireEvent.click(within(group).getByLabelText("複製封存名稱"));
    expect(writeText).toHaveBeenCalledWith("2026-07-05-desktop-shell-and-browser");
    expect(onOpen).not.toHaveBeenCalled();
  });
});

// 討論卡活在「討論」子頁籤下（design D3）——先切子頁籤（Radix TabsTrigger 以
// mousedown 觸發）再斷言卡片。
const toDiscussionsTab = () =>
  fireEvent.mouseDown(screen.getByRole("tab", { name: /已封存的討論/ }));

describe("ArchivedList（封存討論卡）", () => {
  it("點卡觸發 onOpen（discussion target）；卡顯示日期＋topic＋輪數＋衍生變更數", () => {
    const onOpen = renderList();
    toDiscussionsTab();
    const disc = card('[data-archived-discussion="old-topic"]');
    expect(disc).toBeTruthy();
    expect(within(disc).getByText("2026-06-30")).toBeTruthy();
    expect(within(disc).getByText("Old settled topic")).toBeTruthy();
    expect(within(disc).getByText("2 輪")).toBeTruthy();
    // 衍生變更數自既有 promotedTo 長度派生；計數 meta 統一「裸 icon＋數字」
    //（design D7 增補）：非 Badge 圓圈、帶 icon。
    const promoted = within(disc).getByLabelText("衍生 1 個變更");
    expect(promoted.className).not.toContain("rounded-full");
    expect(promoted.querySelector("svg")).toBeTruthy();
    fireEvent.click(screen.getByText("Old settled topic"));
    expect(onOpen).toHaveBeenCalledWith({ kind: "discussion", slug: "old-topic" });
    // 行內展開移除：區段標題不出現在清單。
    expect(screen.queryByText("討論過程")).toBeNull();
  });

  it("複製 slug 鈕寫入剪貼簿且不開抽屜", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", { value: { writeText }, configurable: true });
    const onOpen = renderList();
    toDiscussionsTab();
    const disc = card('[data-archived-discussion="old-topic"]');
    fireEvent.click(within(disc).getByLabelText("複製 slug"));
    expect(writeText).toHaveBeenCalledWith("old-topic");
    expect(onOpen).not.toHaveBeenCalled();
    await waitFor(() => expect(within(disc).queryByLabelText("已複製")).toBeTruthy());
  });
});

// spec 需求「已封存頁含討論節」（子頁籤＋筆數徽章）與「清單最新在前與換頁瀏覽」
//（datedName／created 降冪、每頁 20 筆、頁碼獨立、搜尋回第 1 頁）。
describe("ArchivedList（子頁籤、排序與換頁）", () => {
  const mkChange = (datedName: string): ArchivedItem => ({
    datedName,
    date: datedName.slice(0, 10),
    name: datedName.slice(11),
    specCount: 0,
    createdBy: null,
    fromDiscussions: [],
  });
  const mkDisc = (slug: string, created: string, topic = `topic ${slug}`): DiscussionItem => ({
    slug,
    topic,
    status: "promoted",
    rounds: 1,
    created,
    promotedTo: [],
  });
  const changeOrder = () =>
    Array.from(document.querySelectorAll("[data-archived]")).map((el) => el.getAttribute("data-archived"));
  const discOrder = () =>
    Array.from(document.querySelectorAll("[data-archived-discussion]")).map((el) =>
      el.getAttribute("data-archived-discussion"),
    );

  it("呈現「變更」「討論」兩子頁籤且預設顯示變更；討論卡不在預設頁籤出現", () => {
    renderList();
    const tabs = screen.getAllByRole("tab");
    expect(tabs).toHaveLength(2);
    expect(screen.getByRole("tab", { name: /已封存的變更/ }).getAttribute("data-state")).toBe("active");
    expect(screen.getByText("desktop-shell-and-browser")).toBeTruthy();
    expect(screen.queryByText("Old settled topic")).toBeNull();
  });

  it("archivedDiscussions 未提供時子頁籤列缺席、僅顯示變更清單", () => {
    renderList([WARN, FULL, BARE], { archivedDiscussions: undefined });
    expect(screen.queryByRole("tab")).toBeNull();
    expect(screen.getByText("desktop-shell-and-browser")).toBeTruthy();
    expect(screen.queryByText("Old settled topic")).toBeNull();
  });

  it("封存變更依 datedName 字典序降冪（封存日期新→舊）", () => {
    // spec Scenario「封存變更最新在前」Example 值；傳入順序刻意打亂。
    renderList([
      mkChange("2026-07-04-oldest"),
      mkChange("2026-07-11-newest"),
      mkChange("2026-07-08-middle"),
    ]);
    expect(changeOrder()).toEqual(["2026-07-11-newest", "2026-07-08-middle", "2026-07-04-oldest"]);
  });

  it("封存討論依 created 降冪、同日以 slug 字母升冪決勝", () => {
    renderList([WARN], {
      archivedDiscussions: [
        mkDisc("beta-flow", "2026-07-10"),
        mkDisc("zulu-late", "2026-07-12"),
        mkDisc("alpha-ux", "2026-07-10"),
      ],
    });
    toDiscussionsTab();
    expect(discOrder()).toEqual(["zulu-late", "alpha-ux", "beta-flow"]);
  });

  it("搜尋僅命中討論時：「變更」徽章 0＋無結果空狀態、「討論」徽章 3，切換即見命中", () => {
    renderList([WARN, FULL, BARE], {
      query: "settled",
      archivedDiscussions: [
        mkDisc("t1", "2026-06-30", "First settled topic"),
        mkDisc("t2", "2026-06-29", "Second settled topic"),
        mkDisc("t3", "2026-06-28", "Third settled topic"),
      ],
    });
    const changesTab = screen.getByRole("tab", { name: /已封存的變更/ });
    const discussionsTab = screen.getByRole("tab", { name: /已封存的討論/ });
    expect(within(changesTab).getByText("0")).toBeTruthy();
    expect(within(discussionsTab).getByText("3")).toBeTruthy();
    // 變更子頁籤（預設）顯示無結果空狀態。
    expect(screen.getByText("沒有已封存的變更")).toBeTruthy();
    toDiscussionsTab();
    expect(discOrder()).toHaveLength(3);
  });

  // c-01 最新 … c-21 最舊；t-01 最新 … t-21 最舊（兩側各兩頁）。
  const MANY_CHANGES = Array.from({ length: 21 }, (_, i) =>
    mkChange(`2026-06-${String(22 - (i + 1)).padStart(2, "0")}-c-${String(i + 1).padStart(2, "0")}`),
  );
  const MANY_DISCS = Array.from({ length: 21 }, (_, i) =>
    mkDisc(`t-${String(i + 1).padStart(2, "0")}`, `2026-05-${String(22 - (i + 1)).padStart(2, "0")}`),
  );

  it("兩子頁籤頁碼互相獨立：變更翻到第 2 頁不影響討論頁碼", () => {
    renderList(MANY_CHANGES, { archivedDiscussions: MANY_DISCS });
    // 變更子頁籤第 1 頁 20 筆，翻到第 2 頁。
    expect(changeOrder()).toHaveLength(20);
    fireEvent.click(screen.getByRole("button", { name: "下一頁" }));
    expect(screen.getByText("第 2／2 頁")).toBeTruthy();
    expect(screen.getByText("c-21")).toBeTruthy();
    // 討論子頁籤仍在第 1 頁。
    toDiscussionsTab();
    expect(screen.getByText("第 1／2 頁")).toBeTruthy();
    expect(discOrder()).toHaveLength(20);
    expect(screen.queryByText(/t-21/)).toBeNull();
    // 切回變更：其頁碼保持第 2 頁。
    fireEvent.mouseDown(screen.getByRole("tab", { name: /已封存的變更/ }));
    expect(screen.getByText("第 2／2 頁")).toBeTruthy();
  });

  it("搜尋字串變更後兩側頁碼皆回第 1 頁", () => {
    const onOpen = vi.fn();
    const props = {
      archived: MANY_CHANGES,
      onQuery: () => {},
      archivedDiscussions: MANY_DISCS,
      onOpen,
    };
    const { rerender } = render(<ArchivedList {...props} query="" />);
    // 變更翻到第 2 頁 → 切討論翻到第 2 頁。
    fireEvent.click(screen.getByRole("button", { name: "下一頁" }));
    toDiscussionsTab();
    fireEvent.click(screen.getByRole("button", { name: "下一頁" }));
    expect(screen.getByText("第 2／2 頁")).toBeTruthy();
    // 查詢 "-" 兩側皆命中全部 21 筆（仍兩頁）——頁碼必須重設回第 1 頁。
    rerender(<ArchivedList {...props} query="-" />);
    expect(screen.getByText("第 1／2 頁")).toBeTruthy();
    fireEvent.mouseDown(screen.getByRole("tab", { name: /已封存的變更/ }));
    expect(screen.getByText("第 1／2 頁")).toBeTruthy();
    expect(screen.getByText("c-01")).toBeTruthy();
  });
});

// spec 需求「清單最新在前與換頁瀏覽」（填滿高度增補）：版面填滿視窗高度、
// 卡片清單於內部容器捲動、換頁控制列沉底常駐（不捲動即可換頁）、換頁後
// 內部捲動容器捲回頂部。
describe("ArchivedList（填滿高度版面與換頁控制列沉底）", () => {
  const mkChange = (datedName: string): ArchivedItem => ({
    datedName,
    date: datedName.slice(0, 10),
    name: datedName.slice(11),
    specCount: 0,
    createdBy: null,
    fromDiscussions: [],
  });
  const mkDisc = (slug: string, created: string): DiscussionItem => ({
    slug,
    topic: `topic ${slug}`,
    status: "promoted",
    rounds: 1,
    created,
    promotedTo: [],
  });
  // 兩側各 21 筆（兩頁）使換頁控制列出現。
  const MANY_CHANGES = Array.from({ length: 21 }, (_, i) =>
    mkChange(`2026-06-${String(22 - (i + 1)).padStart(2, "0")}-c-${String(i + 1).padStart(2, "0")}`),
  );
  const MANY_DISCS = Array.from({ length: 21 }, (_, i) =>
    mkDisc(`t-${String(i + 1).padStart(2, "0")}`, `2026-05-${String(22 - (i + 1)).padStart(2, "0")}`),
  );
  const renderMany = (over: Record<string, unknown> = {}) =>
    render(
      <ArchivedList
        archived={MANY_CHANGES}
        query=""
        onQuery={() => {}}
        archivedDiscussions={MANY_DISCS}
        onOpen={vi.fn()}
        {...(over as object)}
      />,
    );
  const scrollEl = () => document.querySelector("[data-list-scroll]") as HTMLElement;

  it("根容器為填滿高度 flex 直欄；清單容器內部捲動；換頁控制列在捲動容器外沉底", () => {
    const { container } = renderMany();
    const root = container.firstElementChild as HTMLElement;
    expect(root.className).toContain("h-full");
    expect(root.className).toContain("flex-col");
    const scroll = scrollEl();
    expect(scroll).toBeTruthy();
    expect(scroll.className).toContain("overflow-y-auto");
    expect(scroll.className).toContain("flex-1");
    expect(scroll.className).toContain("min-h-0");
    // 換頁控制列是捲動容器的手足（直欄末端），不被清單內容捲走。
    const nextBtn = screen.getByRole("button", { name: "下一頁" });
    expect(scroll.contains(nextBtn)).toBe(false);
    expect(scroll.parentElement!.contains(nextBtn)).toBe(true);
  });

  it("換頁後內部捲動容器捲回頂部（兩子頁籤各自歸位）", () => {
    renderMany();
    // 變更子頁籤：模擬捲到中段後換頁。
    scrollEl().scrollTop = 150;
    fireEvent.click(screen.getByRole("button", { name: "下一頁" }));
    expect(scrollEl().scrollTop).toBe(0);
    // 討論子頁籤：自有捲動容器，同樣換頁歸位。
    fireEvent.mouseDown(screen.getByRole("tab", { name: /已封存的討論/ }));
    scrollEl().scrollTop = 150;
    fireEvent.click(screen.getByRole("button", { name: "下一頁" }));
    expect(scrollEl().scrollTop).toBe(0);
  });

  it("無子頁籤相容路徑（archivedDiscussions 未提供）同樣填滿高度且清單內部捲動", () => {
    const { container } = renderMany({ archivedDiscussions: undefined });
    const root = container.firstElementChild as HTMLElement;
    expect(root.className).toContain("h-full");
    const scroll = scrollEl();
    expect(scroll).toBeTruthy();
    expect(scroll.className).toContain("overflow-y-auto");
    const nextBtn = screen.getByRole("button", { name: "下一頁" });
    expect(scroll.contains(nextBtn)).toBe(false);
  });
});

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
    // 行內展開移除：點擊後清單內不出現分頁。
    expect(screen.queryByRole("tab")).toBeNull();
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

describe("ArchivedList（封存討論卡）", () => {
  it("點卡觸發 onOpen（discussion target）；卡顯示日期＋topic＋輪數＋衍生變更數", () => {
    const onOpen = renderList();
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
    const disc = card('[data-archived-discussion="old-topic"]');
    fireEvent.click(within(disc).getByLabelText("複製 slug"));
    expect(writeText).toHaveBeenCalledWith("old-topic");
    expect(onOpen).not.toHaveBeenCalled();
    await waitFor(() => expect(within(disc).queryByLabelText("已複製")).toBeTruthy());
  });
});

describe("ArchivedList（兩節與搜尋）", () => {
  it("變更與討論兩節分列，各有標題與計數", () => {
    renderList();
    expect(screen.getByText("已封存的變更")).toBeTruthy();
    expect(screen.getByText("已封存的討論")).toBeTruthy();
  });

  it("搜尋同時過濾兩節：命中討論 topic 時變更節無項目、反之亦然", () => {
    renderList([WARN, FULL, BARE], { query: "settled" });
    expect(screen.getByText("Old settled topic")).toBeTruthy();
    expect(screen.queryByText("desktop-shell-and-browser")).toBeNull();
  });

  it("搜尋命中變更名時討論節無項目", () => {
    renderList([WARN, FULL, BARE], { query: "desktop-shell" });
    expect(screen.getByText("desktop-shell-and-browser")).toBeTruthy();
    expect(screen.queryByText("Old settled topic")).toBeNull();
  });
});

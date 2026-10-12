// spec「討論詳情頁檢視與轉出變更」衍生變更段的渲染器與列資料推導（desktop-discussion-doc-views design D5）。
import { describe, expect, it, vi } from "vitest";
import { fireEvent, render as rtlRender } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import type { ArchivedItem, ChangeItem } from "../adapter";
import type { CutCard } from "../discussionModel";
import { I18nProvider, MESSAGES } from "../i18n";
import { PromotedDoc, promotedRows, type PromotedRow } from "../components/PromotedDoc";

const wrapper = ({ children }: { children: ReactNode }) => <I18nProvider locale="zh-TW">{children}</I18nProvider>;
const render = (ui: ReactElement) => rtlRender(ui, { wrapper });
const t = (key: string) => MESSAGES["zh-TW"][key] ?? key;

// spec Example「列的欄位來源」：一列一案。
const changes: ChangeItem[] = [
  {
    name: "ready-stamped",
    status: "ready",
    totalTasks: 9,
    completedTasks: 9,
    reviewStatus: "reviewed",
    reviewedAt: "2026-10-09T10:00:00+08:00",
    verifyStatus: "verified",
    verifiedAt: "2026-10-10T09:00:00+08:00",
  },
  { name: "in-review", status: "in-progress", totalTasks: 8, completedTasks: 3, reviewStatus: "inReview" },
];
const archived: ArchivedItem[] = [
  {
    datedName: "2026-10-05-archived-ok",
    date: "2026-10-05",
    name: "archived-ok",
    tasksDone: 17,
    tasksTotal: 17,
    reviewStatus: "reviewed",
    verifyStatus: "verified",
  },
  {
    datedName: "2026-10-06-archived-not-passed",
    date: "2026-10-06",
    name: "archived-not-passed",
    tasksDone: 4,
    tasksTotal: 5,
    reviewStatus: "reviewedNotPassed",
    verifyStatus: "verified",
  },
];
const NAMES = ["ready-stamped", "in-review", "archived-ok", "archived-not-passed", "gone"];

describe("promotedRows", () => {
  it("作用中、已封存、已刪除三態的階段籤、任務、兩章、日期與開啟", () => {
    const openActive = vi.fn();
    const openArchived = vi.fn();
    const rows = promotedRows(NAMES, changes, archived, t, openActive, openArchived);
    expect(rows.map((r) => [r.name, r.stage.label, r.stage.tone, r.state, r.tasks, r.review, r.verify, r.date])).toEqual([
      ["ready-stamped", "已就緒", "success", "active", { done: 9, total: 9 }, true, true, "2026-10-10"],
      ["in-review", "進行中", "progress", "active", { done: 3, total: 8 }, false, false, null],
      ["archived-ok", "已封存", "neutral", "archived", { done: 17, total: 17 }, true, true, "2026-10-05"],
      ["archived-not-passed", "已封存", "neutral", "archived", { done: 4, total: 5 }, false, true, "2026-10-06"],
      ["gone", "已刪除", "destructive", "deleted", null, false, false, null],
    ]);
    rows[0].onOpen!();
    expect(openActive).toHaveBeenCalledWith("ready-stamped");
    rows[2].onOpen!();
    expect(openArchived).toHaveBeenCalledWith("2026-10-05-archived-ok");
    expect(rows[4].onOpen).toBeUndefined();
  });
});

describe("PromotedDoc", () => {
  const rows = (names: string[]) => promotedRows(names, changes, archived, t, vi.fn(), vi.fn());

  it("列的欄位：階段籤、等寬名、任務 n/m、審／驗章、日期、開啟鈕", () => {
    const { container } = render(<PromotedDoc rows={rows(NAMES)} cuts={[]} />);
    const row = (name: string) => container.querySelector(`[data-promoted-row="${name}"]`) as HTMLElement;
    const stamps = (name: string) => Array.from(row(name).querySelectorAll("[data-stamp]")).map((s) => s.textContent);
    expect(row("ready-stamped").textContent).toContain("已就緒");
    expect(row("ready-stamped").textContent).toContain("9/9");
    expect(stamps("ready-stamped")).toEqual(["審", "驗"]);
    expect(row("ready-stamped").textContent).toContain("2026-10-10");
    expect(row("in-review").textContent).toContain("3/8");
    expect(stamps("in-review")).toEqual([]);
    expect(stamps("archived-not-passed")).toEqual(["驗"]);
    expect(row("gone").textContent).not.toMatch(/\d+\/\d+/);
    expect(row("gone").querySelector(".line-through")!.textContent).toBe("已刪除");
    expect(row("gone").querySelector("button")).toBeNull();
    expect(row("in-review").querySelector("button")).toBeTruthy();
  });

  it("開啟鈕呼叫該列的開啟", () => {
    const onOpen = vi.fn();
    const row: PromotedRow = {
      name: "x",
      stage: { label: "進行中", tone: "progress" },
      state: "active",
      tasks: null,
      review: false,
      verify: false,
      date: null,
      onOpen,
    };
    const { container } = render(<PromotedDoc rows={[row]} cuts={[]} />);
    fireEvent.click(container.querySelector('[data-promoted-row="x"] button')!);
    expect(onOpen).toHaveBeenCalled();
  });

  it("7 刀分組：組頭數 7、cut 2 組內 3 列、組頭為 data-doc-heading、不在任何刀的列接在最後無組頭", () => {
    const cutNames = [["a0"], ["a1"], ["b1", "b2", "b3"], ["c1", "c2"], ["d1", "d2"], ["e1", "e2", "e3"], ["f1"]];
    const ids = ["cut 0", "cut 1", "cut 2", "cut 3a", "cut 3b", "cut 4", "cut 5"];
    const cuts: CutCard[] = cutNames.map((names, i) => ({ id: ids[i], title: `範圍 ${i}`, names, items: [] }));
    const all = [...cutNames.flat(), "stray"];
    const { container } = render(<PromotedDoc rows={rows(all)} cuts={cuts} />);
    const groups = Array.from(container.querySelectorAll<HTMLElement>("[data-cut-group]"));
    expect(groups.map((g) => g.getAttribute("data-cut-group"))).toEqual(ids);
    const headings = Array.from(container.querySelectorAll("h3[data-doc-heading]")).map((h) => h.textContent);
    expect(headings).toEqual(ids.map((id, i) => `${id} 範圍 ${i}`));
    const cut2 = groups[2];
    expect(Array.from(cut2.querySelectorAll("[data-promoted-row]")).map((r) => r.getAttribute("data-promoted-row"))).toEqual([
      "b1",
      "b2",
      "b3",
    ]);
    expect(cut2.textContent).toContain("3 個變更");
    const allRows = Array.from(container.querySelectorAll("[data-promoted-row]"));
    expect(allRows[allRows.length - 1].getAttribute("data-promoted-row")).toBe("stray");
    expect(allRows[allRows.length - 1].closest("[data-cut-group]")).toBeNull();
  });

  it("cuts 空時無組頭、依傳入序", () => {
    const { container } = render(<PromotedDoc rows={rows(["in-review", "archived-ok"])} cuts={[]} />);
    expect(container.querySelector("[data-cut-group]")).toBeNull();
    expect(container.querySelector("[data-doc-heading]")).toBeNull();
    expect(Array.from(container.querySelectorAll("[data-promoted-row]")).map((r) => r.getAttribute("data-promoted-row"))).toEqual([
      "in-review",
      "archived-ok",
    ]);
  });

  it("rows 空時空態文案", () => {
    const { container } = render(<PromotedDoc rows={[]} cuts={[]} />);
    expect(container.textContent).toBe("尚未轉出任何變更。");
  });
});

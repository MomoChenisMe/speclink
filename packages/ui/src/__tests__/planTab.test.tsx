// 詳情抽屜的排程分頁（spec desktop-app「詳情抽屜的排程分頁」；plan-requirement-overlap-desktop
// design「排程分頁卡片化，阻擋併入前置卡的狀態徽章」）：四張邊框卡片（波次、前置、重疊、
// 封存順序）、前置卡的狀態徽章、前置的新增與移除經 onSetDepends、無 onSetDepends
// （capability 假）時不長編輯控制項、缺 wave 時只剩一句說明。
import { describe, it, expect, vi, afterEach } from "vitest";
import { render as rtlRender, screen, waitFor, fireEvent, within, cleanup, act } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { RichDetailDrawer, type RichDetailDrawerProps } from "../components/RichDetailDrawer";
import type { ChangeItem, DeltaOperation } from "../adapter";
import { DELTA_COLORS } from "../components/DeltaBadges";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

afterEach(() => cleanup());

const base = { status: "proposed", totalTasks: 3, completedTasks: 0 };
/** add-c 與 add-b 同動 desktop-app 的「看板與任務」（雙方 MODIFIED，不衝突）。 */
const boardRow = {
  change: "add-b",
  capability: "desktop-app",
  requirement: "看板與任務",
  ownOperation: "MODIFIED",
  otherOperation: "MODIFIED",
  conflict: false,
} as const;
/** Scenario「可開工徽章與同名衝突標籤」的資料：add-c 與 add-x 都 ADDED auth 的「登入」。 */
const clash: ChangeItem = {
  name: "add-c",
  ...base,
  wave: 1,
  blockedBy: [],
  dependsOn: [],
  overlaps: [],
  requirementOverlap: [
    { change: "add-x", capability: "auth", requirement: "登入", ownOperation: "ADDED", otherOperation: "ADDED", conflict: true },
  ],
  archiveAfter: [],
};
/** 四個作用中變更：add-c 是抽屜主角（Scenario「排程分頁四段」的資料：第 2 波、前置
 * add-a、與 add-b 同動一個 requirement、排在 add-b 之後封存）。 */
const changes: ChangeItem[] = [
  { name: "add-a", ...base, wave: 1, blockedBy: [], dependsOn: [], overlaps: [], requirementOverlap: [], archiveAfter: [] },
  {
    name: "add-b",
    ...base,
    wave: 2,
    blockedBy: ["add-a"],
    dependsOn: ["add-a"],
    overlaps: [{ change: "add-c", capabilities: ["desktop-app"] }],
    requirementOverlap: [{ ...boardRow, change: "add-c" }],
    archiveAfter: [],
  },
  {
    name: "add-c",
    ...base,
    wave: 2,
    dependsOn: ["add-a"],
    blockedBy: ["add-a"],
    overlaps: [{ change: "add-b", capabilities: ["desktop-app"] }],
    requirementOverlap: [boardRow],
    archiveAfter: ["add-b"],
  },
  { name: "add-d", ...base, wave: 3, blockedBy: ["add-c"], dependsOn: ["add-c"], overlaps: [], requirementOverlap: [], archiveAfter: [] },
];

function makeProps(over: Partial<RichDetailDrawerProps> = {}): RichDetailDrawerProps {
  return {
    open: true,
    onOpenChange: vi.fn(),
    change: changes[2],
    changes,
    loadDocument: vi.fn(async () => "# doc"),
    loadCapabilities: vi.fn(async () => ["desktop-app"]),
    loadMeta: vi.fn(async () => null),
    onSetDepends: vi.fn(),
    ...over,
  };
}

async function openPlanTab(props: RichDetailDrawerProps) {
  render(<RichDetailDrawer {...props} />);
  await waitFor(() => screen.getByRole("tab", { name: /排程/ }));
  fireEvent.mouseDown(screen.getByRole("tab", { name: /排程/ }));
  await waitFor(() => screen.getByRole("tabpanel"));
}

const section = (name: string) =>
  document.querySelector(`[data-plan-section="${name}"]`) as HTMLElement;
/** 卡片的標題列（PlanCard 的 data 錨點，不靠 DOM 位置）。 */
const header = (name: string) => section(name).querySelector("[data-plan-card-header]") as HTMLElement;

describe("排程分頁", () => {
  it("分頁在規格分頁之後，四張邊框卡片依序：波次、前置（附移除鈕與新增下拉）、重疊、封存順序", async () => {
    // Scenario「排程分頁四段」的卡片骨架；重疊卡與封存順序卡的內容另有專測。
    const props = makeProps();
    await openPlanTab(props);
    const tabs = screen.getAllByRole("tab").map((t) => t.textContent ?? "");
    expect(tabs.findIndex((t) => t.includes("排程"))).toBe(tabs.findIndex((t) => t.includes("規格")) + 1);

    const cards = Array.from(document.querySelectorAll<HTMLElement>("[data-plan-section]"));
    expect(cards.map((c) => c.dataset.planSection)).toEqual(["wave", "depends", "overlaps", "archive"]);
    // 與審查／驗證分頁同一種邊框卡片：標題列帶卡片標題、內容區以分隔線分格。
    const titles = ["波次", "前置", "重疊", "封存順序"];
    cards.forEach((card, i) => {
      expect(card.className).toContain("rounded-md border border-border/60 bg-muted/20");
      expect(within(header(card.dataset.planSection!)).getByText(titles[i])).toBeTruthy();
      expect(card.querySelector("[data-plan-card-body]")?.className).toContain("border-t border-border/60");
    });

    const wave = section("wave");
    expect(within(wave).getByText("第 2 波")).toBeTruthy();
    expect(within(wave).getByText("add-b")).toBeTruthy(); // 同波夥伴
    expect(within(wave).queryByText("add-c")).toBeNull(); // 不列自己

    const depends = section("depends");
    expect(within(depends).getByText("add-a")).toBeTruthy();
    expect(within(depends).getByRole("button", { name: "移除前置 add-a" })).toBeTruthy();
    expect(within(depends).getByRole("combobox", { name: "新增前置" })).toBeTruthy();
    expect(within(section("overlaps")).queryByRole("button")).toBeNull(); // 唯讀
  });

  it("depends_card_badge_ready_and_waiting：前置卡標題列的狀態徽章，blockedBy 空為「可以開工」、非空為「等 N 項」且 title 列出前置名", async () => {
    const waitingOn: ChangeItem = { ...changes[2], dependsOn: ["add-a", "add-c"], blockedBy: ["add-a", "add-c"] };
    await openPlanTab(makeProps({ change: waitingOn }));
    const waiting = within(header("depends")).getByText("等 2 項");
    expect(waiting.getAttribute("title")).toBe("等待：add-a、add-c");
    expect(waiting.className).not.toContain("bg-secondary"); // outline
    expect(within(section("depends")).queryByText("可以開工")).toBeNull();
    cleanup();

    await openPlanTab(makeProps({ change: changes[0] }));
    const ready = within(header("depends")).getByText("可以開工");
    expect(ready.className).toContain("bg-secondary"); // secondary
    expect(within(section("depends")).queryByText(/等 \d+ 項/)).toBeNull();
  });

  it("overlaps_card_lists_requirement_level_rows：重疊卡每列對方名、`capability › requirement` 與雙方操作兩枚標籤，不顯示目錄級 overlaps", async () => {
    // Scenario「排程分頁四段」的重疊卡；目錄級 overlaps 另有 add-d（tray-status-menu），不得出現。
    const item: ChangeItem = {
      ...changes[2],
      overlaps: [...(changes[2].overlaps ?? []), { change: "add-d", capabilities: ["tray-status-menu"] }],
    };
    await openPlanTab(makeProps({ change: item }));
    const overlaps = section("overlaps");
    const rows = within(overlaps).getAllByRole("listitem");
    expect(rows).toHaveLength(1);
    expect(within(rows[0]).getByText("add-b")).toBeTruthy();
    expect(within(rows[0]).getByText("desktop-app › 看板與任務")).toBeTruthy();
    // 操作標籤與規格分頁的 delta 區段同一套用詞與配色（MODIFIED → 「修改」）。
    const ops = within(rows[0]).getAllByText("修改");
    expect(ops.map((op) => op.getAttribute("title"))).toEqual(["add-c", "add-b"]); // 本變更在前、對方在後
    for (const op of ops) {
      expect(op.className).toContain("rounded border border-border/60 px-1 py-0.5 text-[10px]");
      expect(op.className).toContain(DELTA_COLORS.modified);
    }
    expect(within(rows[0]).queryByText("同名衝突")).toBeNull();
    expect(within(overlaps).queryByText("add-d")).toBeNull();
    expect(within(overlaps).queryByText("tray-status-menu")).toBeNull();
    expect(within(overlaps).queryByRole("button")).toBeNull(); // 唯讀
    cleanup();

    await openPlanTab(makeProps({ change: changes[0] }));
    expect(within(section("overlaps")).getByText("無重疊")).toBeTruthy();
  });

  it("conflict_row_shows_name_conflict_tag：conflict 為 true 的重疊列多一枚 destructive 色「同名衝突」標籤", async () => {
    // Scenario「可開工徽章與同名衝突標籤」：兩者都 ADDED auth 的「登入」。
    await openPlanTab(makeProps({ change: clash, changes: [clash] }));
    expect(within(header("depends")).getByText("可以開工")).toBeTruthy();
    const row = within(section("overlaps")).getByRole("listitem");
    expect(within(row).getByText("add-x")).toBeTruthy();
    expect(within(row).getByText("auth › 登入")).toBeTruthy();
    expect(within(row).getAllByText("新增")).toHaveLength(2);
    expect(within(row).getByText("同名衝突").className).toContain("text-destructive");
  });

  it("archive_card_lists_names_and_hint：封存順序卡列 archiveAfter 的變更名，下方一句提示", async () => {
    // Scenario「排程分頁四段」的封存順序卡：add-b 與提示句。
    await openPlanTab(makeProps());
    const archive = section("archive");
    expect(within(archive).getByText("add-b")).toBeTruthy();
    expect(within(archive).getByText("先封存它們，再對照正式規格重寫同名 requirement 後封存本變更")).toBeTruthy();
    expect(within(archive).queryByText("可直接封存")).toBeNull();
  });

  it("archive_card_empty_state：archiveAfter 為空時顯示「可直接封存」、不出提示句", async () => {
    // 需求本文：archiveAfter 為空且無同名衝突時「可直接封存」（有衝突時改顯示警示，見
    // archive_card_warns_name_conflict）。
    await openPlanTab(makeProps({ change: changes[0] }));
    const archive = section("archive");
    expect(within(archive).getByText("可直接封存")).toBeTruthy();
    expect(within(archive).queryByText(/先封存它們/)).toBeNull();
  });

  it("archive_card_warns_name_conflict：有同名衝突列時封存卡顯示衝突警示，不顯示「可直接封存」", async () => {
    // Scenario「可開工徽章與同名衝突標籤」的封存順序卡：兩者都 ADDED「登入」，引擎不把對方
    // 列進 archiveAfter，但較晚封存的一方一定被拒。
    await openPlanTab(makeProps({ change: clash, changes: [clash] }));
    const archive = section("archive");
    const warning = within(archive).getByText("與 add-x 同名衝突：先改掉其中一邊，較晚封存的一方才不會被拒");
    expect(warning.className).toContain("text-destructive");
    expect(within(archive).queryByText("可直接封存")).toBeNull();
  });

  it("archive_card_warns_mutual_wait：與對方互相列入 archiveAfter 時封存卡顯示互相等待警示", async () => {
    // Scenario「互相等待時封存卡警示」：add-a 要等 add-b 帶進的「R1」，add-b 要等 add-a
    // 修改完才能拿走「R2」——兩邊都得先等對方封存。
    const row = (change: string, requirement: string, own: DeltaOperation, other: DeltaOperation) => ({
      change,
      capability: "desktop-app",
      requirement,
      ownOperation: own,
      otherOperation: other,
      conflict: false,
    });
    const a: ChangeItem = {
      name: "add-a",
      ...base,
      wave: 1,
      blockedBy: [],
      dependsOn: [],
      overlaps: [{ change: "add-b", capabilities: ["desktop-app"] }],
      requirementOverlap: [row("add-b", "R1", "MODIFIED", "ADDED"), row("add-b", "R2", "MODIFIED", "REMOVED")],
      archiveAfter: ["add-b"],
    };
    const b: ChangeItem = {
      ...a,
      name: "add-b",
      overlaps: [{ change: "add-a", capabilities: ["desktop-app"] }],
      requirementOverlap: [row("add-a", "R1", "ADDED", "MODIFIED"), row("add-a", "R2", "REMOVED", "MODIFIED")],
      archiveAfter: ["add-a"],
    };
    await openPlanTab(makeProps({ change: a, changes: [a, b] }));
    const archive = section("archive");
    expect(within(archive).getByText("add-b")).toBeTruthy();
    const warning = within(archive).getByText("與 add-b 互相等待：兩邊都要先等對方封存，須調整拆分");
    expect(warning.className).toContain("text-destructive");
    cleanup();

    // 單向的封存順序不出警示。
    await openPlanTab(makeProps());
    expect(within(section("archive")).queryByText(/互相等待/)).toBeNull();
  });

  it("old_server_missing_fields_show_empty_states：帶 wave 但缺 requirementOverlap 與 archiveAfter（舊 server）時兩卡顯示空態", async () => {
    // Scenario「舊 server 缺新欄位的空態」：其餘兩卡照常。
    const old: ChangeItem = { name: "add-c", ...base, wave: 2, blockedBy: ["add-a"], dependsOn: ["add-a"], overlaps: [] };
    await openPlanTab(makeProps({ change: old, changes: [changes[0], old] }));
    expect(within(section("overlaps")).getByText("無重疊")).toBeTruthy();
    expect(within(section("archive")).getByText("可直接封存")).toBeTruthy();
    expect(within(section("wave")).getByText("第 2 波")).toBeTruthy();
    expect(within(section("depends")).getByText("等 1 項")).toBeTruthy();
  });

  it("old_server_overlap_waits_listed_in_depends_card：blockedBy 含 dependsOn 以外的名稱（舊 server 依重疊判定阻擋）時，前置卡以唯讀列列出，徽章數字與內容一致", async () => {
    // Scenario「舊 server 的重疊阻擋列在前置卡」：0.7.0 server 的 blockedBy＝宣告前置 ∪
    // 先配置、共用 capability 的變更，且沒有 requirementOverlap 與 archiveAfter。
    const old: ChangeItem = {
      name: "add-c",
      ...base,
      wave: 2,
      blockedBy: ["add-a"],
      dependsOn: [],
      overlaps: [{ change: "add-a", capabilities: ["desktop-app"] }],
    };
    await openPlanTab(makeProps({ change: old, changes: [changes[0], old] }));
    const depends = section("depends");
    expect(within(header("depends")).getByText("等 1 項")).toBeTruthy();
    expect(within(depends).getByText("尚無前置")).toBeTruthy();
    const waits = depends.querySelector("[data-plan-undeclared-waits]") as HTMLElement;
    expect(within(waits).getByText("另需等待（server 依重疊判定）：")).toBeTruthy();
    expect(within(waits).getByText("add-a")).toBeTruthy();
    expect(within(depends).queryByRole("button", { name: "移除前置 add-a" })).toBeNull(); // 沒宣告，無從移除
    cleanup();

    // 新 server 與本機：blockedBy 只含宣告前置，不出現這一列。
    await openPlanTab(makeProps());
    expect(within(section("depends")).queryByText(/另需等待/)).toBeNull();
  });

  it("no_blocked_section_rendered：阻擋併入前置卡徽章，頁面不存在 data-plan-section=\"blocked\"", async () => {
    await openPlanTab(makeProps());
    expect(section("blocked")).toBeNull();
    expect(screen.queryByText("阻擋")).toBeNull();
    expect(within(section("depends")).getByText("等 1 項")).toBeTruthy();
  });

  it("下拉只列其他作用中變更（排除自己與既有前置），選擇即 onSetDepends(change, [name], false)", async () => {
    // Scenario「新增前置寫回」的觸發面。
    const props = makeProps();
    await openPlanTab(props);
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await user.click(screen.getByRole("combobox", { name: "新增前置" }));
    const options = (await screen.findAllByRole("option")).map((o) => o.textContent);
    expect(options).toEqual(["add-b", "add-d"]);
    await user.click(screen.getByRole("option", { name: "add-d" }));
    expect(props.onSetDepends).toHaveBeenCalledWith("add-c", ["add-d"], false);
  });

  it("資料源提供名冊查詢時，候選以查詢結果為準（worktree 映射時的副本名冊）", async () => {
    // Scenario「worktree 映射時候選來自副本名冊」的 UI 面：清單有 add-b、add-d，
    // 名冊只有 add-a、add-b——add-a 已是前置、add-d 不在名冊，下拉只剩 add-b。
    const loadDependsCandidates = vi.fn(async () => ["add-a", "add-b"]);
    const props = makeProps({ loadDependsCandidates });
    await openPlanTab(props);
    expect(loadDependsCandidates).toHaveBeenCalledWith("add-c");
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await user.click(await screen.findByRole("combobox", { name: "新增前置" }));
    const options = (await screen.findAllByRole("option")).map((o) => o.textContent);
    expect(options).toEqual(["add-b"]);
  });

  it("名冊查詢未完成時不長新增下拉，失敗時退回自清單派生", async () => {
    const pending = makeProps({ loadDependsCandidates: vi.fn(() => new Promise<string[]>(() => {})) });
    await openPlanTab(pending);
    expect(within(section("depends")).getByText("add-a")).toBeTruthy();
    expect(screen.queryByRole("combobox", { name: "新增前置" })).toBeNull();
    cleanup();

    const failing = makeProps({ loadDependsCandidates: vi.fn(async () => Promise.reject(new Error("boom"))) });
    await openPlanTab(failing);
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await user.click(await screen.findByRole("combobox", { name: "新增前置" }));
    const options = (await screen.findAllByRole("option")).map((o) => o.textContent);
    expect(options).toEqual(["add-b", "add-d"]);
  });

  it("重新載入時名冊查詢失敗，沿用上一次的名冊，不退回自清單派生", async () => {
    // 首載取得名冊 add-a、add-b；刷新世代前進後查詢失敗——候選仍只有 add-b（add-d 不在名冊）。
    const loadDependsCandidates = vi
      .fn<(change: string) => Promise<string[]>>()
      .mockResolvedValueOnce(["add-a", "add-b"])
      .mockRejectedValue(new Error("boom"));
    const props = makeProps({ loadDependsCandidates, refreshGen: 0 });
    const { rerender } = render(<RichDetailDrawer {...props} />);
    fireEvent.mouseDown(await screen.findByRole("tab", { name: /排程/ }));
    await screen.findByRole("combobox", { name: "新增前置" });
    rerender(<RichDetailDrawer {...props} refreshGen={1} />);
    await waitFor(() => expect(loadDependsCandidates).toHaveBeenCalledTimes(2));
    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 0));
    });
    const user = userEvent.setup({ pointerEventsCheck: 0 });
    await user.click(await screen.findByRole("combobox", { name: "新增前置" }));
    const options = (await screen.findAllByRole("option")).map((o) => o.textContent);
    expect(options).toEqual(["add-b"]);
  });

  it("移除鈕觸發 onSetDepends(change, [name], true)", async () => {
    const props = makeProps();
    await openPlanTab(props);
    fireEvent.click(screen.getByRole("button", { name: "移除前置 add-a" }));
    expect(props.onSetDepends).toHaveBeenCalledWith("add-c", ["add-a"], true);
  });

  it("無 onSetDepends（capability 為假）時不渲染移除鈕與下拉，四卡照常", async () => {
    // Scenario「remote 分頁唯讀」。
    const props = makeProps({ onSetDepends: undefined });
    await openPlanTab(props);
    expect(within(section("depends")).getByText("add-a")).toBeTruthy();
    expect(screen.queryByRole("button", { name: /移除前置/ })).toBeNull();
    expect(screen.queryByRole("combobox", { name: "新增前置" })).toBeNull();
    for (const name of ["wave", "depends", "overlaps", "archive"]) {
      expect(section(name)).toBeTruthy();
    }
  });

  it("同波無他人顯示「本波只有這個變更」", async () => {
    const solo: ChangeItem = { name: "add-a", ...base, wave: 1, blockedBy: [], dependsOn: [], overlaps: [] };
    const props = makeProps({ change: solo, changes: [solo, changes[3]] });
    await openPlanTab(props);
    expect(within(section("wave")).getByText("本波只有這個變更")).toBeTruthy();
  });

  it("cycle_renders_only_depends_card_without_badge：plan 成環（缺 wave 但帶 dependsOn）時一句成環說明＋只有前置卡，無徽章、無下拉", async () => {
    // Scenario「成環時可移除前置」：payload 成環時 dependsOn 仍為 meta 原文，分頁只給
    // 解環的出口（移除鈕），不長狀態徽章與新增下拉，也不長波次／重疊／封存順序卡。
    const cyc: ChangeItem = { name: "add-c", ...base, dependsOn: ["add-a"] };
    const props = makeProps({ change: cyc, changes: [cyc, { name: "add-a", ...base, dependsOn: ["add-c"] }] });
    await openPlanTab(props);
    expect(screen.getByText("依賴成環，先移除一條前置才有排程資訊")).toBeTruthy();
    expect(screen.queryByText("此模式尚未提供排程資訊")).toBeNull();
    const depends = section("depends");
    expect(within(depends).getByText("add-a")).toBeTruthy();
    fireEvent.click(within(depends).getByRole("button", { name: "移除前置 add-a" }));
    expect(props.onSetDepends).toHaveBeenCalledWith("add-c", ["add-a"], true);
    expect(screen.queryByRole("combobox", { name: "新增前置" })).toBeNull();
    expect(within(depends).queryByText(/可以開工|等 \d+ 項/)).toBeNull();
    expect(Array.from(document.querySelectorAll<HTMLElement>("[data-plan-section]")).map((c) => c.dataset.planSection)).toEqual([
      "depends",
    ]);
  });

  it("plan 成環時環外的變更（dependsOn 為空）前置卡顯示「尚無前置」，不是空框", async () => {
    // local 任一處成環，環外每個有效項都帶 dependsOn: []（query.rs 以 meta 宣告原文填）。
    const outside: ChangeItem = { name: "add-e", ...base, dependsOn: [] };
    await openPlanTab(makeProps({ change: outside, changes: [outside] }));
    expect(screen.getByText("依賴成環，先移除一條前置才有排程資訊")).toBeTruthy();
    expect(within(section("depends")).getByText("尚無前置")).toBeTruthy();
    expect(within(section("depends")).queryByRole("button", { name: /移除前置/ })).toBeNull();
  });

  it("plan 成環且 capability 為假：前置卡唯讀（無移除鈕）", async () => {
    const cyc: ChangeItem = { name: "add-c", ...base, dependsOn: ["add-a"] };
    const props = makeProps({ change: cyc, changes: [cyc], onSetDepends: undefined });
    await openPlanTab(props);
    expect(within(section("depends")).getByText("add-a")).toBeTruthy();
    expect(screen.queryByRole("button", { name: /移除前置/ })).toBeNull();
  });

  it("缺 wave 且缺 dependsOn（remote 摘要）時只顯示一句說明，不渲染四卡", async () => {
    const bare: ChangeItem = { name: "add-c", ...base };
    const props = makeProps({ change: bare, changes: [bare] });
    await openPlanTab(props);
    expect(screen.getByText("此模式尚未提供排程資訊")).toBeTruthy();
    expect(document.querySelector("[data-plan-section]")).toBeNull();
    expect(screen.queryByRole("combobox", { name: "新增前置" })).toBeNull();
  });
});

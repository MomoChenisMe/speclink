// 詳情抽屜的排程分頁（spec desktop-app「詳情抽屜的排程分頁」；add-change-plan-desktop
// design D6）：四段內容、前置的新增與移除經 onSetDepends、無 onSetDepends（capability
// 假）時不長編輯控制項、缺 wave 時只剩一句說明。
import { describe, it, expect, vi, afterEach } from "vitest";
import { render as rtlRender, screen, waitFor, fireEvent, within, cleanup, act } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { RichDetailDrawer, type RichDetailDrawerProps } from "../components/RichDetailDrawer";
import type { ChangeItem } from "../adapter";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

afterEach(() => cleanup());

const base = { status: "proposed", totalTasks: 3, completedTasks: 0 };
/** 四個作用中變更：add-c 是抽屜主角（第 2 波、前置 add-a、與 add-b 重疊、被兩項擋住）。 */
const changes: ChangeItem[] = [
  { name: "add-a", ...base, wave: 1, blockedBy: [], dependsOn: [], overlaps: [] },
  { name: "add-b", ...base, wave: 2, blockedBy: ["add-a"], dependsOn: ["add-a"], overlaps: [{ change: "add-c", capabilities: ["desktop-app"] }] },
  {
    name: "add-c",
    ...base,
    wave: 2,
    dependsOn: ["add-a"],
    overlaps: [{ change: "add-b", capabilities: ["desktop-app"] }],
    blockedBy: ["add-a", "add-b"],
  },
  { name: "add-d", ...base, wave: 3, blockedBy: ["add-c"], dependsOn: ["add-c"], overlaps: [] },
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

describe("排程分頁", () => {
  it("分頁在規格分頁之後，四段依序：波次、前置（附移除鈕與新增下拉）、重疊、阻擋", async () => {
    // Scenario「排程分頁四段」。
    const props = makeProps();
    await openPlanTab(props);
    const tabs = screen.getAllByRole("tab").map((t) => t.textContent ?? "");
    expect(tabs.findIndex((t) => t.includes("排程"))).toBe(tabs.findIndex((t) => t.includes("規格")) + 1);

    const wave = section("wave");
    expect(within(wave).getByText("第 2 波")).toBeTruthy();
    expect(within(wave).getByText("add-b")).toBeTruthy(); // 同波夥伴
    expect(within(wave).queryByText("add-c")).toBeNull(); // 不列自己

    const depends = section("depends");
    expect(within(depends).getByText("add-a")).toBeTruthy();
    expect(within(depends).getByRole("button", { name: "移除前置 add-a" })).toBeTruthy();
    expect(within(depends).getByRole("combobox", { name: "新增前置" })).toBeTruthy();

    const overlaps = section("overlaps");
    expect(within(overlaps).getByText("add-b")).toBeTruthy();
    expect(within(overlaps).getByText("desktop-app")).toBeTruthy();
    expect(within(overlaps).queryByRole("button")).toBeNull(); // 唯讀

    const blocked = section("blocked");
    expect(within(blocked).getByText("add-a")).toBeTruthy();
    expect(within(blocked).getByText("add-b")).toBeTruthy();

    const order = ["wave", "depends", "overlaps", "blocked"].map((n) =>
      Array.from(document.querySelectorAll("[data-plan-section]")).indexOf(section(n)),
    );
    expect(order).toEqual([0, 1, 2, 3]);
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

  it("無 onSetDepends（capability 為假）時不渲染移除鈕與下拉，四段照常", async () => {
    // Scenario「remote 分頁唯讀」。
    const props = makeProps({ onSetDepends: undefined });
    await openPlanTab(props);
    expect(section("depends")).toBeTruthy();
    expect(within(section("depends")).getByText("add-a")).toBeTruthy();
    expect(screen.queryByRole("button", { name: /移除前置/ })).toBeNull();
    expect(screen.queryByRole("combobox", { name: "新增前置" })).toBeNull();
    expect(section("blocked")).toBeTruthy();
  });

  it("同波無他人顯示「本波只有這個變更」、無阻擋顯示「可以開工」", async () => {
    const solo: ChangeItem = { name: "add-a", ...base, wave: 1, blockedBy: [], dependsOn: [], overlaps: [] };
    const props = makeProps({ change: solo, changes: [solo, changes[3]] });
    await openPlanTab(props);
    expect(within(section("wave")).getByText("本波只有這個變更")).toBeTruthy();
    expect(within(section("blocked")).getByText("可以開工")).toBeTruthy();
  });

  it("plan 成環（缺 wave 但帶 dependsOn）：一句成環說明＋前置段可移除，無下拉、無其他三段", async () => {
    // Scenario「成環時可移除前置」：payload 成環時 dependsOn 仍為 meta 原文，分頁只給
    // 解環的出口（移除鈕），不長新增下拉，也不長波次／重疊／阻擋段。
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
    expect(section("wave")).toBeNull();
    expect(section("overlaps")).toBeNull();
    expect(section("blocked")).toBeNull();
  });

  it("plan 成環且 capability 為假：前置段唯讀（無移除鈕）", async () => {
    const cyc: ChangeItem = { name: "add-c", ...base, dependsOn: ["add-a"] };
    const props = makeProps({ change: cyc, changes: [cyc], onSetDepends: undefined });
    await openPlanTab(props);
    expect(within(section("depends")).getByText("add-a")).toBeTruthy();
    expect(screen.queryByRole("button", { name: /移除前置/ })).toBeNull();
  });

  it("缺 wave 且缺 dependsOn（remote 摘要）時只顯示一句說明，不渲染四段", async () => {
    const bare: ChangeItem = { name: "add-c", ...base };
    const props = makeProps({ change: bare, changes: [bare] });
    await openPlanTab(props);
    expect(screen.getByText("此模式尚未提供排程資訊")).toBeTruthy();
    expect(document.querySelector("[data-plan-section]")).toBeNull();
    expect(screen.queryByRole("combobox", { name: "新增前置" })).toBeNull();
  });
});

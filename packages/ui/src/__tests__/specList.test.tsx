// spec 需求「桌面 app 呈現 change 與 spec 的清單與內容」（抽屜語意：點整列開
// 抽屜、無行內展開）＋「規格與封存卡片收合資訊」（規格卡）：收合資訊欄位、
// 名稱搜尋（維持現狀）、複製名稱回饋、空狀態。
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render as rtlRender, screen, fireEvent, waitFor, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { SpecList } from "../components/SpecList";
import type { SpecItem } from "../adapter";

// 既有中文斷言包 I18nProvider locale zh-TW（與 archivedList.test 同型）。
const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

// spec Scenario「規格卡收合資訊」的主角：7 條 Requirement、Purpose 已填寫、溯源自
// 3 個變更；desktop-config 為缺 Purpose／零溯源的容錯樣本；node-sdk 為佔位樣本。
const SPECS: SpecItem[] = [
  {
    id: "desktop-app",
    modifiedAt: "2026-07-08",
    requirementCount: 7,
    purposeExcerpt: "桌面 app 的行為契約。",
    purposeTbd: false,
    traceCount: 3,
  },
  {
    id: "desktop-config",
    modifiedAt: "2026-07-07",
    requirementCount: 2,
    purposeExcerpt: null,
    purposeTbd: false,
    traceCount: 0,
  },
  {
    id: "node-sdk",
    modifiedAt: "2026-07-05",
    requirementCount: 1,
    purposeExcerpt: "TBD - created by archiving change 'old'. Update Purpose after archive.",
    purposeTbd: true,
    traceCount: 1,
  },
];

function renderList(specs: SpecItem[] = SPECS, onOpen = vi.fn()) {
  render(<SpecList specs={specs} onOpen={onOpen} />);
  return onOpen;
}

const card = (id: string) => document.querySelector(`[data-spec="${id}"]`) as HTMLElement;

describe("SpecList（規格頁清單）", () => {
  beforeEach(() => {
    // 僅假造 Date 固定相對時間基準；timer 保持真實讓 waitFor 照常運作。
    vi.useFakeTimers({ toFake: ["Date"] });
    vi.setSystemTime(new Date("2026-07-08T04:00:00Z"));
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  it("卡片於收合狀態顯示需求數、溯源變更數、相對時間與 Purpose 摘要一行截斷", () => {
    renderList();
    const app = card("desktop-app");
    // spec Example 值：需求數 7、溯源變更數 3、今天修改。
    const reqCount = within(app).getByLabelText("7 條需求");
    expect(reqCount).toBeTruthy();
    // 計數 meta 統一「裸 icon＋數字」（design D7 增補）：無 pill 底色、帶 icon。
    expect(reqCount.className).not.toContain("rounded-full");
    expect(reqCount.className).not.toContain("bg-muted");
    expect(reqCount.querySelector("svg")).toBeTruthy();
    const traceCount = within(app).getByLabelText("溯源自 3 個變更");
    expect(traceCount.className).not.toContain("rounded-full");
    expect(traceCount.querySelector("svg")).toBeTruthy();
    expect(within(app).getByText("今天")).toBeTruthy();
    // Purpose 摘要獨立成描述列、一行截斷。
    const excerpt = within(app).getByText("桌面 app 的行為契約。");
    expect(excerpt.className).toContain("truncate");
    // 容錯樣本：無 Purpose → 描述列缺席；零溯源 → 溯源標記缺席；需求數照常。
    const config = card("desktop-config");
    expect(within(config).getByLabelText("2 條需求")).toBeTruthy();
    expect(within(config).queryByLabelText(/溯源自/)).toBeNull();
    expect(within(config).getByText("昨天")).toBeTruthy();
  });

  it("modifiedAt 缺席時相對時間該行不渲染", () => {
    renderList([{ id: "bare-cap", requirementCount: 0, purposeExcerpt: null, purposeTbd: false, traceCount: 0 }]);
    const bare = card("bare-cap");
    expect(bare).toBeTruthy();
    expect(within(bare).queryByText(/今天|昨天|天前/)).toBeNull();
  });

  it("Purpose 佔位時以琥珀警示顯示「Purpose 待補」，不顯示佔位原文", () => {
    renderList();
    const sdk = card("node-sdk");
    const hint = within(sdk).getByText("Purpose 待補");
    expect(hint.className).toContain("amber");
    expect(within(sdk).queryByText(/TBD - created by archiving/)).toBeNull();
  });

  it("點整列觸發 onOpen 開抽屜；無 chevron 與行內展開", () => {
    const onOpen = renderList();
    fireEvent.click(screen.getByText("desktop-app"));
    expect(onOpen).toHaveBeenCalledWith("desktop-app");
    // 卡片本身不展開內容（載入語意已搬進抽屜）。
    expect(screen.queryByText("載入中…")).toBeNull();
    // chevron 與 aria-expanded 全數移除。
    expect(document.querySelector(".lucide-chevron-right")).toBeNull();
    expect(document.querySelector(".lucide-chevron-down")).toBeNull();
    expect(document.querySelector("[aria-expanded]")).toBeNull();
  });

  it("複製鈕位於標題群組內（標題後緊跟、hover 顯現），點擊寫入剪貼簿且不開抽屜", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", { value: { writeText }, configurable: true });
    const onOpen = renderList();
    const app = card("desktop-app");
    // 標題與複製鈕同屬一個標題群組（design D7：標題＋複製鈕成群組、meta 靠右）。
    const group = app.querySelector("[data-title-group]") as HTMLElement;
    expect(group).toBeTruthy();
    expect(within(group).getByText("desktop-app")).toBeTruthy();
    const copyBtn = within(group).getByLabelText("複製名稱");
    // hover 顯現：預設透明、group-hover 顯示。
    expect(copyBtn.className).toContain("opacity-0");
    expect(copyBtn.className).toContain("group-hover:opacity-100");
    fireEvent.click(copyBtn);
    expect(writeText).toHaveBeenCalledWith("desktop-app");
    await waitFor(() => expect(within(app).getByLabelText("已複製")).toBeTruthy());
    expect(onOpen).not.toHaveBeenCalled();
  });

  it("搜尋以名稱子字串過濾（大小寫不敏感）、無結果顯示空狀態、清空還原", () => {
    renderList();
    const input = screen.getByPlaceholderText("搜尋規格…");
    // Example 列 1：desktop → desktop-app、desktop-config
    fireEvent.change(input, { target: { value: "desktop" } });
    expect(screen.getByText("desktop-app")).toBeTruthy();
    expect(screen.getByText("desktop-config")).toBeTruthy();
    expect(screen.queryByText("node-sdk")).toBeNull();
    // Example 列 2：SDK → node-sdk（大小寫不敏感）
    fireEvent.change(input, { target: { value: "SDK" } });
    expect(screen.getByText("node-sdk")).toBeTruthy();
    expect(screen.queryByText("desktop-app")).toBeNull();
    // Example 列 3：zzz → 無結果空狀態
    fireEvent.change(input, { target: { value: "zzz" } });
    expect(screen.queryByText("node-sdk")).toBeNull();
    expect(screen.getByText("沒有符合的規格")).toBeTruthy();
    // 清空輸入 → 清單還原
    fireEvent.change(input, { target: { value: "" } });
    expect(screen.getByText("desktop-app")).toBeTruthy();
    expect(screen.getByText("desktop-config")).toBeTruthy();
    expect(screen.getByText("node-sdk")).toBeTruthy();
    expect(screen.queryByText("沒有符合的規格")).toBeNull();
  });

  it("無 spec 專案顯示空狀態文案", () => {
    renderList([]);
    expect(screen.getByText("此專案尚無正典規格")).toBeTruthy();
  });
});

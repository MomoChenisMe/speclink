// spec 需求「規格頁提供清單、搜尋與展開檢視」（列式容器卡、頁標題區搜尋、點列開
// 檢視、無行內展開）＋「規格與封存卡片收合資訊」（規格列）：收合資訊欄位、
// 名稱搜尋（維持現狀）、複製名稱回饋、空狀態。
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render as rtlRender, screen, fireEvent, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
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
    expect(hint.className).toContain("status-warning");
    expect(within(sdk).queryByText(/TBD - created by archiving/)).toBeNull();
  });

  it("點整列觸發 onOpen 開檢視；列為共用 ListRow（等寬標題、› 收尾）、無行內展開", () => {
    const onOpen = renderList();
    fireEvent.click(screen.getByText("desktop-app"));
    expect(onOpen).toHaveBeenCalledWith("desktop-app");
    // 列本身不展開內容（載入語意已搬進抽屜）。
    expect(screen.queryByText("載入中…")).toBeNull();
    const app = card("desktop-app");
    expect(app.getAttribute("role")).toBe("button");
    expect(within(app).getByText("desktop-app").className).toContain("font-mono");
    expect(app.querySelector(".lucide-chevron-right")).toBeTruthy();
    // 行內展開的 chevron-down 與 aria-expanded 全數移除（列內斷言：工具列的每頁下拉自帶 chevron-down）。
    expect(app.querySelector(".lucide-chevron-down")).toBeNull();
    expect(app.querySelector("[aria-expanded]")).toBeNull();
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
    // 共用 CopyButton：成功以 status 宣告「已複製」，複製鈕名稱維持不變。
    await waitFor(() => expect(within(app).getByRole("status").textContent).toBe("已複製"));
    expect(within(group).getByLabelText("複製名稱")).toBe(copyBtn);
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
    expect(screen.getByText("此專案尚無正式規格")).toBeTruthy();
  });
});

// spec 需求「清單最新在前與換頁瀏覽」：規格依 modifiedAt 新→舊、缺席者殿後
// 且依名稱字母升冪；每頁 20 筆換頁、搜尋字串變更回第 1 頁。
describe("SpecList（最新在前與換頁）", () => {
  beforeEach(() => {
    vi.useFakeTimers({ toFake: ["Date"] });
    vi.setSystemTime(new Date("2026-07-08T04:00:00Z"));
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  const bare = (id: string, modifiedAt?: string): SpecItem => ({
    id,
    modifiedAt: modifiedAt ?? null,
    requirementCount: 0,
    purposeExcerpt: null,
    purposeTbd: false,
    traceCount: 0,
  });

  const cardOrder = () =>
    Array.from(document.querySelectorAll("[data-spec]")).map((el) => el.getAttribute("data-spec"));

  it("modifiedAt 新→舊排序；缺席者排最後且彼此依名稱字母升冪", () => {
    // spec Scenario「規格修改時間缺席排最後」Example 值：beta 今天、alpha 3 天前、
    // zeta 與 delta 無 modifiedAt → 順序 beta、alpha、delta、zeta。傳入順序刻意打亂。
    renderList([bare("zeta"), bare("alpha", "2026-07-05"), bare("delta"), bare("beta", "2026-07-08")]);
    expect(cardOrder()).toEqual(["beta", "alpha", "delta", "zeta"]);
  });

  // s01 最新 … s21 最舊：排序後 s01–s20 落第 1 頁、s21 落第 2 頁。
  const MANY: SpecItem[] = Array.from({ length: 21 }, (_, i) => {
    const n = i + 1;
    const day = String(22 - n).padStart(2, "0");
    return bare(`s${String(n).padStart(2, "0")}`, `2026-06-${day}`);
  });

  it("21 筆時第 1 頁僅 20 筆且工具列有頁碼，點下一頁顯示第 21 筆", () => {
    renderList(MANY);
    expect(cardOrder()).toHaveLength(20);
    expect(screen.getByText("s01")).toBeTruthy();
    expect(screen.queryByText("s21")).toBeNull();
    expect(screen.getByText("第 1–20 筆，共 21 筆")).toBeTruthy();
    expect(screen.getByRole("button", { name: "第 2 頁" })).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "下一頁" }));
    expect(screen.getByText("s21")).toBeTruthy();
    expect(screen.queryByText("s01")).toBeNull();
    expect(screen.getByText("第 21–21 筆，共 21 筆")).toBeTruthy();
  });

  it("13 筆時工具列只有筆數範圍與每頁筆數：無頁碼鈕、無 ‹ ›、無跳頁輸入", () => {
    renderList(MANY.slice(0, 13));
    expect(cardOrder()).toHaveLength(13);
    expect(screen.getByText("第 1–13 筆，共 13 筆")).toBeTruthy();
    expect(screen.getByRole("combobox", { name: "每頁 20 個" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "下一頁" })).toBeNull();
    expect(screen.queryByRole("button", { name: /^第 \d+ 頁$/ })).toBeNull();
    expect(screen.queryByRole("spinbutton")).toBeNull();
  });

  it("於第 2 頁修改搜尋字串後回到第 1 頁", () => {
    renderList(MANY);
    fireEvent.click(screen.getByRole("button", { name: "下一頁" }));
    expect(screen.getByText("第 21–21 筆，共 21 筆")).toBeTruthy();
    // 查詢 "s" 命中全部 21 筆（仍兩頁）——頁碼必須重設回第 1 頁。
    fireEvent.change(screen.getByPlaceholderText("搜尋規格…"), { target: { value: "s" } });
    expect(screen.getByText("第 1–20 筆，共 21 筆")).toBeTruthy();
    expect(screen.getByText("s01")).toBeTruthy();
    expect(screen.queryByText("s21")).toBeNull();
  });

  it("每頁改 50 後 21 筆單頁顯示、頁碼鈕消失；改回 20 回到第 1 頁", async () => {
    const user = userEvent.setup();
    renderList(MANY);
    fireEvent.click(screen.getByRole("button", { name: "下一頁" }));
    expect(screen.getByText("s21")).toBeTruthy();
    await user.click(screen.getByRole("combobox", { name: "每頁 20 個" }));
    await user.click(await screen.findByRole("option", { name: "每頁 50 個" }));
    expect(cardOrder()).toHaveLength(21);
    expect(screen.getByText("第 1–21 筆，共 21 筆")).toBeTruthy();
    expect(screen.queryByRole("button", { name: /^第 \d+ 頁$/ })).toBeNull();
    // 改回 20：頁碼已被鉗制成第 1 頁，不回到先前的第 2 頁。
    await user.click(screen.getByRole("combobox", { name: "每頁 50 個" }));
    await user.click(await screen.findByRole("option", { name: "每頁 20 個" }));
    expect(screen.getByText("第 1–20 筆，共 21 筆")).toBeTruthy();
    expect(screen.getByText("s01")).toBeTruthy();
  });
});

// spec 需求「清單最新在前與換頁瀏覽」（填滿高度增補）：版面填滿視窗高度、
// 卡片清單於內部容器捲動、換頁控制列沉底常駐、換頁後內部捲動容器捲回頂部。
describe("SpecList（填滿高度版面與換頁控制列沉底）", () => {
  const bare = (id: string, modifiedAt: string): SpecItem => ({
    id,
    modifiedAt,
    requirementCount: 0,
    purposeExcerpt: null,
    purposeTbd: false,
    traceCount: 0,
  });
  // 21 筆（兩頁）使換頁控制列出現。
  const MANY: SpecItem[] = Array.from({ length: 21 }, (_, i) =>
    bare(`s${String(i + 1).padStart(2, "0")}`, `2026-06-${String(22 - (i + 1)).padStart(2, "0")}`),
  );
  const scrollEl = () => document.querySelector("[data-list-scroll]") as HTMLElement;
  const renderMany = () => render(<SpecList specs={MANY} onOpen={vi.fn()} />);

  it("根容器為填滿高度 flex 直欄；列於列表卡內捲動；工具列在捲動容器外的卡底", () => {
    const { container } = renderMany();
    const root = container.firstElementChild as HTMLElement;
    expect(root.className).toContain("h-full");
    expect(root.className).toContain("flex-col");
    const scroll = scrollEl();
    expect(scroll).toBeTruthy();
    expect(scroll.className).toContain("overflow-y-auto");
    expect(scroll.className).toContain("flex-1");
    expect(scroll.className).toContain("min-h-0");
    // 列在列表卡內；工具列是捲動容器的手足（卡底），不被清單內容捲走。
    const listCard = document.querySelector("[data-list-card]") as HTMLElement;
    expect(listCard.contains(scroll)).toBe(true);
    expect(listCard.className).toContain("rounded-2xl");
    const nextBtn = screen.getByRole("button", { name: "下一頁" });
    expect(scroll.contains(nextBtn)).toBe(false);
    expect(scroll.parentElement!.contains(nextBtn)).toBe(true);
  });

  it("換頁後內部捲動容器捲回頂部", () => {
    renderMany();
    scrollEl().scrollTop = 150;
    fireEvent.click(screen.getByRole("button", { name: "下一頁" }));
    expect(scrollEl().scrollTop).toBe(0);
  });
});

// spec 需求「規格頁提供清單、搜尋與展開檢視」（頁標題區）：標題、灰字說明與全圓搜尋框
// 在列表卡之上；未傳 title 時只有搜尋框。
describe("SpecList（頁標題區）", () => {
  it("傳 title 與 description 時出現 h2、說明與全圓搜尋框", () => {
    render(<SpecList specs={SPECS} onOpen={vi.fn()} title="規格" description="正式規格一覽。" />);
    const header = document.querySelector("[data-page-header]") as HTMLElement;
    expect(within(header).getByRole("heading", { level: 2 }).textContent).toBe("規格");
    expect(header.textContent).toContain("正式規格一覽。");
    const input = within(header).getByPlaceholderText("搜尋規格…");
    expect(input.className).toContain("rounded-full");
    expect(document.querySelector("[data-list-card]")!.contains(header)).toBe(false);
  });

  it("未傳 title 時無頁標題區，搜尋框仍在", () => {
    render(<SpecList specs={SPECS} onOpen={vi.fn()} />);
    expect(document.querySelector("[data-page-header]")).toBeNull();
    expect(screen.queryByRole("heading", { level: 2 })).toBeNull();
    expect(screen.getByPlaceholderText("搜尋規格…").className).toContain("rounded-full");
  });
});

// desktop-list-pages-reskin design D7：每頁筆數可受控（桌面 store 記住並傳入）；值與回呼
// 都給時以 prop 為準，改下拉只呼叫回呼。
describe("SpecList（受控每頁筆數）", () => {
  const MANY21: SpecItem[] = Array.from({ length: 21 }, (_, i) => ({
    id: `c${String(i + 1).padStart(2, "0")}`,
    modifiedAt: `2026-06-${String(22 - (i + 1)).padStart(2, "0")}`,
    requirementCount: 0,
    purposeExcerpt: null,
    purposeTbd: false,
    traceCount: 0,
  }));

  it("傳 pageSize={50} 時 21 筆單頁顯示；選每頁 100 呼叫 onPageSizeChange(100)，值仍以 prop 為準", async () => {
    const user = userEvent.setup();
    const onPageSizeChange = vi.fn();
    render(<SpecList specs={MANY21} onOpen={vi.fn()} pageSize={50} onPageSizeChange={onPageSizeChange} />);
    expect(document.querySelectorAll("[data-spec]")).toHaveLength(21);
    expect(screen.getByText("第 1–21 筆，共 21 筆")).toBeTruthy();
    await user.click(screen.getByRole("combobox", { name: "每頁 50 個" }));
    await user.click(await screen.findByRole("option", { name: "每頁 100 個" }));
    expect(onPageSizeChange).toHaveBeenCalledWith(100);
    expect(screen.getByRole("combobox", { name: "每頁 50 個" })).toBeTruthy();
  });
});

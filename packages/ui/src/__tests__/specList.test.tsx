// spec 需求「規格頁提供清單、搜尋與展開檢視」：卡片清單（名稱＋相對修改時間）、
// 名稱子字串搜尋（Example 表為準）、展開懶載入全文、縮合、複製名稱回饋、空狀態。
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render as rtlRender, screen, waitFor, fireEvent, within } from "@testing-library/react";
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

// Example 表的三個 spec；modifiedAt 對固定「現在」分別為今天／昨天／3 天前。
const SPECS: SpecItem[] = [
  { id: "desktop-app", modifiedAt: "2026-07-08" },
  { id: "desktop-config", modifiedAt: "2026-07-07" },
  { id: "node-sdk", modifiedAt: "2026-07-05" },
];

function makeLoadDocument() {
  return vi.fn(async (cap: string) => `# ${cap} Specification\n\n${cap} 的全文內容。`);
}

function renderList(specs: SpecItem[] = SPECS, loadDocument = makeLoadDocument()) {
  const view = render(<SpecList specs={specs} loadDocument={loadDocument} refreshGen={0} />);
  return { loadDocument, view };
}

describe("SpecList（規格頁清單）", () => {
  beforeEach(() => {
    // 僅假造 Date 固定相對時間基準；timer 保持真實讓 waitFor 照常運作。
    vi.useFakeTimers({ toFake: ["Date"] });
    vi.setSystemTime(new Date("2026-07-08T04:00:00Z"));
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  it("清單渲染各卡名稱與相對修改時間；modifiedAt 缺席時該行不渲染", () => {
    renderList([...SPECS, { id: "bare-cap" }]);
    expect(screen.getByText("desktop-app")).toBeTruthy();
    expect(screen.getByText("desktop-config")).toBeTruthy();
    expect(screen.getByText("node-sdk")).toBeTruthy();
    expect(screen.getByText("今天")).toBeTruthy();
    expect(screen.getByText("昨天")).toBeTruthy();
    expect(screen.getByText("3 天前")).toBeTruthy();
    // mtime 不可得的卡片存在、但無任何相對時間字樣（spec：該資訊缺席）。
    const bare = document.querySelector('[data-spec="bare-cap"]') as HTMLElement;
    expect(bare).toBeTruthy();
    expect(within(bare).queryByText(/今天|昨天|天前/)).toBeNull();
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

  it("點標題才載入內容：首次展開呈載入態、再點縮合、同 session 重展不重載", async () => {
    // 可控 promise：先斷言載入態，再放行內容。
    const resolvers: Array<(s: string | null) => void> = [];
    const loadDocument = vi.fn(
      (_cap: string) => new Promise<string | null>((resolve) => resolvers.push(resolve)),
    );
    renderList(SPECS, loadDocument);
    expect(loadDocument).not.toHaveBeenCalled();

    fireEvent.click(screen.getByText("desktop-app"));
    expect(loadDocument).toHaveBeenCalledTimes(1);
    expect(loadDocument).toHaveBeenCalledWith("desktop-app");
    expect(screen.getByText("載入中…")).toBeTruthy();

    resolvers[0]("# desktop-app Specification\n\n全文內容段落。");
    await waitFor(() => expect(screen.getByText("全文內容段落。")).toBeTruthy());

    // 再點標題縮合；重展用元件內快取、不重呼叫 loadDocument（design D4）。
    fireEvent.click(screen.getByText("desktop-app"));
    expect(screen.queryByText("全文內容段落。")).toBeNull();
    fireEvent.click(screen.getByText("desktop-app"));
    expect(screen.getByText("全文內容段落。")).toBeTruthy();
    expect(loadDocument).toHaveBeenCalledTimes(1);
  });

  it("展開另一張卡不影響已展開者", async () => {
    renderList();
    fireEvent.click(screen.getByText("desktop-app"));
    await waitFor(() => expect(screen.getByText("desktop-app 的全文內容。")).toBeTruthy());
    fireEvent.click(screen.getByText("node-sdk"));
    await waitFor(() => expect(screen.getByText("node-sdk 的全文內容。")).toBeTruthy());
    expect(screen.getByText("desktop-app 的全文內容。")).toBeTruthy();
  });

  it("refreshGen 遞增清空快取：已展開卡片重載新內容", async () => {
    // design D4／契約 5：外部變更後（workspace-changed → refresh）世代遞增，
    // 已展開內容重載至磁碟現況。
    let body = "第一版內容。";
    const loadDocument = vi.fn(async (_cap: string) => `# spec\n\n${body}`);
    const { view } = renderList(SPECS, loadDocument);
    fireEvent.click(screen.getByText("desktop-app"));
    await waitFor(() => expect(screen.getByText("第一版內容。")).toBeTruthy());
    body = "第二版內容。";
    view.rerender(<SpecList specs={SPECS} loadDocument={loadDocument} refreshGen={1} />);
    await waitFor(() => expect(screen.getByText("第二版內容。")).toBeTruthy());
    expect(loadDocument).toHaveBeenCalledTimes(2);
  });

  it("複製名稱鈕寫入剪貼簿並顯示回饋，且不觸發展開", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", { value: { writeText }, configurable: true });
    const { loadDocument } = renderList();
    const card = document.querySelector('[data-spec="desktop-app"]') as HTMLElement;
    fireEvent.click(within(card).getByLabelText("複製名稱"));
    expect(writeText).toHaveBeenCalledWith("desktop-app");
    // 回饋：控制項切為「已複製」狀態（無障礙標籤可見）。
    await waitFor(() => expect(within(card).getByLabelText("已複製")).toBeTruthy());
    expect(loadDocument).not.toHaveBeenCalled();
  });

  // spec.md 帶 @trace（archive.rs trace_block 格式）的載入器；sources 為各區塊 source
  // （null＝畸形、略去 source 行）。
  function traceDoc(sources: Array<string | null>): string {
    const blocks = sources.map((s) => {
      const head = s === null ? "" : `source: ${s}\n`;
      return `<!-- @trace\n${head}updated: 2026-07-09\ncode:\n  - a.rs\n-->`;
    });
    return `# spec\n\n全文段落。\n\n${blocks.join("\n\n")}`;
  }

  it("展開含 source 的 spec：全文下方顯示來源變更 footer（去重保序＋在地標籤）", async () => {
    const loadDocument = vi.fn(async (_cap: string) =>
      traceDoc(["alpha-change", "alpha-change", "beta-change"]),
    );
    renderList(SPECS, loadDocument);
    fireEvent.click(screen.getByText("desktop-app"));
    await waitFor(() => expect(screen.getByText("全文段落。")).toBeTruthy());
    // footer：在地標籤前置，source 去重且依首次出現保序。
    expect(screen.getByText("來源變更：alpha-change、beta-change")).toBeTruthy();
  });

  it("展開 @trace 缺 source 的 spec：footer 缺席", async () => {
    const loadDocument = vi.fn(async (_cap: string) => traceDoc([null]));
    renderList(SPECS, loadDocument);
    fireEvent.click(screen.getByText("desktop-app"));
    await waitFor(() => expect(screen.getByText("全文段落。")).toBeTruthy());
    expect(screen.queryByText(/來源變更/)).toBeNull();
  });
});

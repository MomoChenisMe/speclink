// 唯讀規格抽屜（spec「桌面 app 呈現 change 與 spec 的清單與內容」；drawer-provenance-links
// design D1/D2/D3）：標頭為標題列（capability＋複製名稱鈕）與出身列（「來自」＋溯源變更籤，
// 首籤為最早封存的變更、其餘收 +N 浮層、無封存記錄者不可點），內文底部無溯源文字；
// 缺件空狀態、世代重載不清空、latest-wins 防交錯、寬度樣式與全螢幕切換與變更詳情抽屜一致。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, waitFor, fireEvent, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { SpecDrawer } from "../components/SpecDrawer";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

const SPEC_MD = `# desktop-app Specification

## Purpose

桌面 app 的行為。

## Requirements

### Requirement: 顯示清單

清單內文。

<!-- @trace
source: change-one
updated: 2026-07-01
code:
  - a.rs
-->

### Requirement: 其他

其他內文。

<!-- @trace
source: change-two
updated: 2026-07-02
code:
  - b.rs
-->

<!-- @trace
source: change-one
updated: 2026-07-03
code:
  - c.rs
-->
`;

const NO_TRACE_MD = `# plain Specification

## Purpose

無溯源內文。
`;

/** 封存清單 fixture：change-two 較晚封存、change-one 較早——與文件出現序相反，驗證排序取日期。 */
const ARCHIVED = [
  { datedName: "2026-07-02-change-two", date: "2026-07-02", name: "change-two" },
  { datedName: "2026-07-01-change-one", date: "2026-07-01", name: "change-one" },
];

function makeProps(over: Record<string, unknown> = {}) {
  return {
    open: true,
    onOpenChange: vi.fn(),
    capability: "desktop-app",
    loadDocument: vi.fn(async () => SPEC_MD),
    archivedChanges: ARCHIVED,
    onOpenArchivedChange: vi.fn(),
    ...over,
  };
}

const drawerEl = () => document.querySelector("[data-spec-drawer]") as HTMLElement | null;
const provenanceRow = () => document.querySelector("[data-provenance-row]") as HTMLElement | null;
const overflowList = () =>
  waitFor(() => {
    const el = document.querySelector("[data-source-overflow-list]") as HTMLElement | null;
    expect(el).toBeTruthy();
    return el!;
  });
const clipboard = () => {
  const writeText = vi.fn().mockResolvedValue(undefined);
  Object.defineProperty(navigator, "clipboard", { value: { writeText }, configurable: true });
  return writeText;
};

describe("SpecDrawer（標頭：標題列與出身列）", () => {
  it("開啟載入正典全文；出身列「來自」＋首籤為最早封存的變更、其餘收 +N；內文底部無溯源文字", async () => {
    const props = makeProps();
    render(<SpecDrawer {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("清單內文。")).toBeTruthy());
    expect(props.loadDocument).toHaveBeenCalledWith("desktop-app");
    expect(screen.getByText("desktop-app")).toBeTruthy();
    expect(screen.getByText("其他內文。")).toBeTruthy();
    // 出身列：「來自」＋首籤（封存最早的 change-one，雖然文件裡 change-two 之前就出現過它）。
    const row = provenanceRow();
    expect(row).toBeTruthy();
    expect(row!.textContent).toContain("來自");
    expect(within(row!).getByRole("button", { name: /change-one/ })).toBeTruthy();
    expect(within(row!).queryByRole("button", { name: /change-two/ })).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: /其餘 1 份/ }));
    const popover = await overflowList();
    expect(within(popover).getByRole("button", { name: /change-two/ })).toBeTruthy();
    // 浮層項副標為封存日期。
    expect(popover.textContent).toContain("2026-07-02");
    // 內文底部不再有溯源文字行。
    expect(screen.queryByText(/來源變更：/)).toBeNull();
  });

  it("溯源籤依封存日期升冪、首籤為出身（spec Example「三個來源變更的排序」）", async () => {
    const doc = [
      "# x\n\n## Requirements\n\n### Requirement: a\n\n內文 a。\n",
      "<!-- @trace\nsource: drawer-polish\nupdated: 2026-08-04\n-->\n",
      "### Requirement: b\n\n內文 b。\n",
      "<!-- @trace\nsource: spec-archive-drawer\nupdated: 2026-07-11\n-->\n",
      "### Requirement: c\n\n內文 c。\n",
      "<!-- @trace\nsource: desktop-archived-parity\nupdated: 2026-08-11\n-->\n",
    ].join("\n");
    const props = makeProps({
      loadDocument: vi.fn(async () => doc),
      archivedChanges: [
        { datedName: "2026-08-04-drawer-polish", date: "2026-08-04", name: "drawer-polish" },
        { datedName: "2026-07-11-spec-archive-drawer", date: "2026-07-11", name: "spec-archive-drawer" },
        { datedName: "2026-08-11-desktop-archived-parity", date: "2026-08-11", name: "desktop-archived-parity" },
      ],
    });
    render(<SpecDrawer {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("內文 a。")).toBeTruthy());
    const row = provenanceRow()!;
    expect(within(row).getByRole("button", { name: /spec-archive-drawer/ })).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: /其餘 2 份/ }));
    const popover = await overflowList();
    const names = within(popover)
      .getAllByRole("button")
      .map((b) => b.textContent ?? "");
    expect(names[0]).toContain("drawer-polish");
    expect(names[0]).toContain("2026-08-04");
    expect(names[1]).toContain("desktop-archived-parity");
    expect(names[1]).toContain("2026-08-11");
  });

  it("點擊可點籤以該變更的 datedName 呼叫 onOpenArchivedChange（含浮層項）", async () => {
    const props = makeProps();
    render(<SpecDrawer {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("清單內文。")).toBeTruthy());
    fireEvent.click(within(provenanceRow()!).getByRole("button", { name: /change-one/ }));
    expect(props.onOpenArchivedChange).toHaveBeenCalledWith("2026-07-01-change-one");
    fireEvent.click(screen.getByRole("button", { name: /其餘 1 份/ }));
    const popover = await overflowList();
    fireEvent.click(within(popover).getByRole("button", { name: /change-two/ }));
    expect(props.onOpenArchivedChange).toHaveBeenCalledWith("2026-07-02-change-two");
  });

  it("無封存記錄的來源變更不可點：灰籤排最後、aria-disabled、副標「無封存記錄」、點擊不呼叫", async () => {
    // spec Scenario「無封存記錄的來源變更不可點」（design D3）。
    const props = makeProps({ archivedChanges: [ARCHIVED[1]] }); // 只有 change-one 有封存記錄
    render(<SpecDrawer {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("清單內文。")).toBeTruthy());
    const row = provenanceRow()!;
    // 可點者在前：首籤仍是 change-one。
    expect(within(row).getByRole("button", { name: /change-one/ })).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: /其餘 1 份/ }));
    const popover = await overflowList();
    const ghost = within(popover).getByRole("button", { name: /change-two/ });
    expect(ghost.getAttribute("aria-disabled")).toBe("true");
    expect(ghost.textContent).toContain("無封存記錄");
    fireEvent.click(ghost);
    expect(props.onOpenArchivedChange).not.toHaveBeenCalled();
    // 不可點項不關閉浮層。
    expect(document.querySelector("[data-source-overflow-list]")).toBeTruthy();
  });

  it("封存清單為空時首籤亦不可點且不呼叫回呼", async () => {
    const props = makeProps({ archivedChanges: [] });
    render(<SpecDrawer {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("清單內文。")).toBeTruthy());
    const chip = within(provenanceRow()!).getByRole("button", { name: /change-one/ });
    expect(chip.getAttribute("aria-disabled")).toBe("true");
    fireEvent.click(chip);
    expect(props.onOpenArchivedChange).not.toHaveBeenCalled();
  });

  it("正典全文無 @trace 時出身列缺席", async () => {
    render(<SpecDrawer {...(makeProps({ loadDocument: vi.fn(async () => NO_TRACE_MD) }) as never)} />);
    await waitFor(() => expect(screen.getByText("無溯源內文。")).toBeTruthy());
    expect(provenanceRow()).toBeNull();
    expect(screen.queryByText("來自")).toBeNull();
    expect(screen.queryByText(/來源變更：/)).toBeNull();
  });

  it("複製名稱鈕寫入 capability 名並顯示已複製回饋", async () => {
    const writeText = clipboard();
    render(<SpecDrawer {...(makeProps() as never)} />);
    await waitFor(() => expect(screen.getByText("清單內文。")).toBeTruthy());
    fireEvent.click(screen.getByLabelText("複製名稱"));
    expect(writeText).toHaveBeenCalledWith("desktop-app");
    await waitFor(() => expect(screen.queryByLabelText("已複製")).toBeTruthy());
  });

  it("文件缺席顯示空狀態而非錯誤，且出身列缺席", async () => {
    const props = makeProps({ loadDocument: vi.fn(async () => null) });
    render(<SpecDrawer {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("（無內容）")).toBeTruthy());
    expect(provenanceRow()).toBeNull();
  });

  it("寬度樣式與變更詳情抽屜一致，含全螢幕切換與還原（design D1）", async () => {
    render(<SpecDrawer {...(makeProps() as never)} />);
    await waitFor(() => expect(screen.getByText("清單內文。")).toBeTruthy());
    const content = drawerEl();
    expect(content).toBeTruthy();
    expect(content!.className).toContain("w-[max(720px,42vw)]");
    expect(content!.className).toContain("max-w-[95vw]");
    fireEvent.click(screen.getByRole("button", { name: "全螢幕" }));
    expect(drawerEl()!.className).toContain("w-[96vw]");
    fireEvent.click(screen.getByRole("button", { name: "還原大小" }));
    expect(drawerEl()!.className).toContain("w-[max(720px,42vw)]");
  });

  it("refreshGen 世代重載不清空，回應到達後單次替換（design D3）", async () => {
    const pending: Array<(v: string | null) => void> = [];
    let hang = false;
    const loadDocument = vi.fn((_cap: string) =>
      hang
        ? new Promise<string | null>((r) => pending.push(r))
        : Promise.resolve<string | null>(SPEC_MD),
    );
    const props = makeProps({ loadDocument });
    const { rerender } = render(<SpecDrawer {...(props as never)} refreshGen={0} />);
    await waitFor(() => expect(screen.getByText("清單內文。")).toBeTruthy());
    hang = true;
    rerender(<SpecDrawer {...(props as never)} refreshGen={1} />);
    await waitFor(() => expect(pending.length).toBe(1));
    // 不清空：舊內容持續呈現，無載入中閃爍。
    expect(screen.getByText("清單內文。")).toBeTruthy();
    expect(screen.queryByText("載入中…")).toBeNull();
    pending[0]("# 新版\n\n更新後內文。\n");
    await waitFor(() => expect(screen.getByText("更新後內文。")).toBeTruthy());
    expect(screen.queryByText("清單內文。")).toBeNull();
  });

  it("latest-wins：舊世代回應晚到不覆蓋新世代內容", async () => {
    const pending: Array<(v: string | null) => void> = [];
    const loadDocument = vi.fn(() => new Promise<string | null>((r) => pending.push(r)));
    const props = makeProps({ loadDocument });
    const { rerender } = render(<SpecDrawer {...(props as never)} refreshGen={0} />);
    await waitFor(() => expect(pending.length).toBe(1)); // 初載請求（gen 0）
    rerender(<SpecDrawer {...(props as never)} refreshGen={1} />);
    await waitFor(() => expect(pending.length).toBe(2)); // 世代重載請求（gen 1）
    pending[1]("# v2\n\nv2 內文。\n"); // 新世代回應先到
    await waitFor(() => expect(screen.getByText("v2 內文。")).toBeTruthy());
    pending[0]("# v1\n\nv1-stale 內文。\n"); // 舊世代回應後到——必須被丟棄
    await new Promise((r) => setTimeout(r, 25));
    expect(screen.queryByText("v1-stale 內文。")).toBeNull();
    expect(screen.getByText("v2 內文。")).toBeTruthy();
  });

  it("換目標時清空並全量重載（design D3：載入中屬新內容的正確呈現）", async () => {
    const loadDocument = vi.fn(async (cap: string) =>
      cap === "desktop-app" ? SPEC_MD : "# 另一份\n\n另一份內文。\n",
    );
    const props = makeProps({ loadDocument });
    const { rerender } = render(<SpecDrawer {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("清單內文。")).toBeTruthy());
    rerender(<SpecDrawer {...(props as never)} capability="desktop-config" />);
    await waitFor(() => expect(screen.getByText("另一份內文。")).toBeTruthy());
    expect(loadDocument).toHaveBeenCalledWith("desktop-config");
    expect(screen.queryByText("清單內文。")).toBeNull();
  });
});

// spec 需求「markdown 文件內容行寬有上限」（design D4）：規格抽屜捲動容器內存在
// 共用置中容器，正典內文在欄內；溯源已搬進標頭，欄內不再有溯源文字。
describe("SpecDrawer（閱讀欄置中）", () => {
  it("捲動容器內有置中容器（w-full＋max-w-[96ch]＋mx-auto）且內文在欄內、無溯源文字", async () => {
    render(<SpecDrawer {...(makeProps() as never)} />);
    await waitFor(() => expect(screen.getByText("清單內文。")).toBeTruthy());
    const col = document.querySelector("[data-reading-column]") as HTMLElement;
    expect(col).toBeTruthy();
    expect(col.className).toContain("w-full");
    expect(col.className).toContain("max-w-[96ch]");
    expect(col.className).toContain("mx-auto");
    expect(col.parentElement?.className).toContain("overflow-y-auto");
    expect(col.textContent).toContain("清單內文。");
    expect(col.textContent).not.toContain("來源變更：");
    expect(col.querySelector("[data-provenance-row]")).toBeNull();
  });
});

// spec「抽屜文件載入以 skeleton 呈現」（design D3）：規格抽屜三態同款分流。
describe("規格抽屜文件三態", () => {
  it("載入中 → 文件骨架，不出空態文案", async () => {
    render(<SpecDrawer {...(makeProps({ loadDocument: vi.fn(() => new Promise<never>(() => {})) }) as never)} />);
    await waitFor(() => expect(document.querySelector('[aria-busy="true"]')).toBeTruthy());
    expect(screen.queryByText("（無內容）")).toBeNull();
  });

  it("載入完成且不存在 → 空態文案，無骨架", async () => {
    render(<SpecDrawer {...(makeProps({ loadDocument: vi.fn(async () => null) }) as never)} />);
    await waitFor(() => expect(screen.getByText("（無內容）")).toBeTruthy());
    expect(document.querySelector('[aria-busy="true"]')).toBeNull();
  });

  it("載入完成且有內容 → 內容照常，無骨架", async () => {
    render(<SpecDrawer {...(makeProps() as never)} />);
    await waitFor(() => expect(screen.getByText("清單內文。")).toBeTruthy());
    expect(document.querySelector('[aria-busy="true"]')).toBeNull();
  });

  it("首載失敗 → 落到空態文案，不停在骨架", async () => {
    const props = makeProps({ loadDocument: vi.fn().mockRejectedValue(new Error("offline")) });
    render(<SpecDrawer {...(props as never)} />);
    // 失敗停在 undefined 就是永久骨架——必須收斂到終態。
    await waitFor(() => expect(screen.getByText("（無內容）")).toBeTruthy());
    expect(document.querySelector('[aria-busy="true"]')).toBeNull();
  });

  it("關閉時 capability 同步變 null：面板留在 DOM 跑滑出動畫，animationend 後才卸載", async () => {
    // desktop-app「抽屜與浮層的開關動畫」：宿主關抽屜時 detailSpec 同步歸 null，元件不得
    // 因此整棵卸載。jsdom 無動畫，以 getComputedStyle 假回 closed 態的 animationName
    // 讓 Radix Presence 走 ANIMATION_OUT 分支。
    const styleSpy = vi.spyOn(window, "getComputedStyle").mockImplementation(
      (el) =>
        ({
          get animationName() {
            return (el as HTMLElement).getAttribute("data-state") === "closed" ? "exit" : "none";
          },
          display: "block",
        }) as unknown as CSSStyleDeclaration,
    );
    try {
      const load = vi.fn(async () => SPEC_MD);
      const { rerender } = render(
        <SpecDrawer open capability="desktop-app" loadDocument={load} onOpenChange={() => {}} />,
      );
      await screen.findByText("清單內文。");
      rerender(<SpecDrawer open={false} capability={null} loadDocument={load} onOpenChange={() => {}} />);
      const content = document.querySelector('[role="dialog"]') as HTMLElement;
      expect(content).toBeTruthy();
      expect(content.getAttribute("data-state")).toBe("closed");
      expect(screen.getByText("清單內文。")).toBeTruthy();
      // jsdom 無 AnimationEvent，fireEvent 的 init 不會落到 event.animationName；Presence
      // 以它比對目前動畫名，故手刻事件並明給。
      const end = new Event("animationend", { bubbles: true });
      Object.defineProperty(end, "animationName", { value: "exit" });
      content.dispatchEvent(end);
      await waitFor(() => expect(document.querySelector('[role="dialog"]')).toBeNull());
    } finally {
      styleSpy.mockRestore();
    }
  });
});

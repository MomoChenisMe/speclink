// 唯讀規格抽屜（spec-archive-drawer design D1/D3；spec「選定 spec 以抽屜顯示其
// 正典內容」）：開啟載入正典全文＋溯源 footer、缺件空狀態、世代重載不清空、
// latest-wins 防交錯、寬度樣式與全螢幕切換與變更詳情抽屜一致。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, waitFor, fireEvent } from "@testing-library/react";
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

function makeProps(over: Record<string, unknown> = {}) {
  return {
    open: true,
    onOpenChange: vi.fn(),
    capability: "desktop-app",
    loadDocument: vi.fn(async () => SPEC_MD),
    ...over,
  };
}

const drawerEl = () => document.querySelector("[data-spec-drawer]") as HTMLElement | null;

describe("SpecDrawer", () => {
  it("開啟載入正典全文與溯源 footer（source 去重保序）", async () => {
    const props = makeProps();
    render(<SpecDrawer {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("清單內文。")).toBeTruthy());
    expect(props.loadDocument).toHaveBeenCalledWith("desktop-app");
    // 標題＝capability id；全文照 markdown 呈現。
    expect(screen.getByText("desktop-app")).toBeTruthy();
    expect(screen.getByText("其他內文。")).toBeTruthy();
    // 溯源 footer：@trace source 去重、依出現順序。
    expect(screen.getByText(/來源變更：change-one、change-two/)).toBeTruthy();
  });

  it("文件缺席顯示空狀態而非錯誤，且不渲染溯源 footer", async () => {
    const props = makeProps({ loadDocument: vi.fn(async () => null) });
    render(<SpecDrawer {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("（無內容）")).toBeTruthy());
    expect(screen.queryByText(/來源變更：/)).toBeNull();
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
// 共用置中容器，正典內文與溯源 footer 同欄對齊置中。
describe("SpecDrawer（閱讀欄置中）", () => {
  it("捲動容器內有置中容器（w-full＋max-w-[96ch]＋mx-auto）且內文與溯源 footer 在欄內", async () => {
    render(<SpecDrawer {...(makeProps() as never)} />);
    await waitFor(() => expect(screen.getByText(/來源變更：/)).toBeTruthy());
    const col = document.querySelector("[data-reading-column]") as HTMLElement;
    expect(col).toBeTruthy();
    expect(col.className).toContain("w-full");
    expect(col.className).toContain("max-w-[96ch]");
    expect(col.className).toContain("mx-auto");
    expect(col.parentElement?.className).toContain("overflow-y-auto");
    expect(col.textContent).toContain("清單內文。");
    expect(col.textContent).toContain("來源變更：");
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
});

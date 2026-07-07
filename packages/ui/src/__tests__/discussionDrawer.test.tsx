import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, waitFor, fireEvent, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";

// 既有中文斷言包 I18nProvider locale zh-TW 後照舊斷言（i18n 抽 key 的回歸保護）。
const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

import { DiscussionDrawer, splitDiscussionSections } from "../components/DiscussionDrawer";
import { ChangeCard } from "../components/ChangeCard";
import { RichDetailDrawer } from "../components/RichDetailDrawer";
import type { ChangeItem, ArchivedItem, DiscussionItem } from "../adapter";

// spec 需求「討論抽屜檢視與 GUI 促轉」的 jsdom 可驗部分。

const DOC = `---
topic: Alpha search
slug: alpha-search
status: concluded
created: 2026-07-01
---

# Discussion: Alpha search

## Context

框架脈絡內容。

## Rounds

### Round 1 — assumptions (2026-07-01)

**Focus**: 範圍界定

## Conclusion

**Decision**: 建置 alpha 搜尋
`;

const concludedD: DiscussionItem = {
  slug: "alpha-search",
  topic: "Alpha search",
  status: "concluded",
  rounds: 1,
  created: "2026-07-01",
  promotedTo: [],
};
const promotedD: DiscussionItem = {
  ...concludedD,
  status: "promoted",
  promotedTo: ["cut-a", "cut-gone"],
};
const openD: DiscussionItem = { ...concludedD, status: "open", promotedTo: [] };

const changes: ChangeItem[] = [
  { name: "cut-a", status: "in-progress", totalTasks: 24, completedTasks: 0 },
];
const archivedChanges: ArchivedItem[] = [];

function makeProps(over: Record<string, unknown> = {}) {
  return {
    open: true,
    onOpenChange: vi.fn(),
    discussion: concludedD,
    loadDocument: vi.fn(async () => DOC),
    changes,
    archivedChanges,
    onPromote: vi.fn(),
    onOpenChangeCard: vi.fn(),
    ...over,
  };
}

describe("splitDiscussionSections（區段切分）", () => {
  it("切出脈絡/回合/結論三區段", () => {
    const s = splitDiscussionSections(DOC);
    expect(s).not.toBeNull();
    expect(s!.context).toContain("框架脈絡內容");
    expect(s!.rounds).toContain("範圍界定");
    expect(s!.conclusion).toContain("建置 alpha 搜尋");
  });

  it("非預期格式（缺區段）回 null → 整篇退回", () => {
    expect(splitDiscussionSections("手寫的自由格式記錄，沒有標準區段。")).toBeNull();
    expect(splitDiscussionSections("## Context\n\n只有脈絡。\n")).toBeNull();
  });
});

const OPEN_DOC = `---
topic: Alpha search
slug: alpha-search
status: open
created: 2026-07-01
---

# Discussion: Alpha search

## Context

開放討論的背景內容。

## Rounds

### Round 1 — assumptions (2026-07-01)

**Focus**: 範圍界定

## Conclusion

<!-- Written by speclink discuss conclude -->
`;

describe("DiscussionDrawer", () => {
  it("分頁依序為 結論/討論過程 N/背景/衍生變更，結論非空時預設呈現結論", async () => {
    render(<DiscussionDrawer {...(makeProps() as never)} />);
    // 預設分頁＝結論（讀者第一想看的），無需切換即可見。
    await waitFor(() => expect(screen.getByText(/建置 alpha 搜尋/)).toBeTruthy());
    const tabs = screen.getAllByRole("tab").map((t) => t.textContent ?? "");
    expect(tabs[0]).toContain("結論");
    expect(tabs[1]).toContain("討論過程");
    expect(tabs[1]).toContain("1");
    expect(tabs[2]).toContain("背景");
    expect(tabs[3]).toContain("衍生變更");
    fireEvent.mouseDown(screen.getByRole("tab", { name: /討論過程/ }));
    expect(screen.getByText(/範圍界定/)).toBeTruthy();
    fireEvent.mouseDown(screen.getByRole("tab", { name: /背景/ }));
    expect(screen.getByText(/框架脈絡內容/)).toBeTruthy();
    expect(screen.queryByRole("tab", { name: /脈絡/ })).toBeNull();
    expect(screen.queryByRole("tab", { name: /^促轉$/ })).toBeNull();
  });

  it("結論為空（僅鷹架註解）時預設呈現背景", async () => {
    const props = makeProps({ discussion: openD, loadDocument: vi.fn(async () => OPEN_DOC) });
    render(<DiscussionDrawer {...(props as never)} />);
    await waitFor(() => expect(screen.getByText(/開放討論的背景內容/)).toBeTruthy());
  });

  it("生命週期階梯：三站可見且現站可辨", async () => {
    // concluded → 現站「已結論」。
    const { unmount } = render(<DiscussionDrawer {...(makeProps() as never)} />);
    await waitFor(() => screen.getByText("討論中"));
    expect(screen.getByText("已結論")).toBeTruthy();
    expect(screen.getByText("轉出變更")).toBeTruthy();
    expect(screen.getByText("已結論").closest("[aria-current]")).toBeTruthy();
    expect(screen.getByText("討論中").closest("[aria-current]")).toBeNull();
    unmount();
    // promoted → 現站「轉出變更」。
    render(<DiscussionDrawer {...(makeProps({ discussion: promotedD }) as never)} />);
    await waitFor(() => screen.getByText("轉出變更"));
    expect(screen.getByText("轉出變更").closest("[aria-current]")).toBeTruthy();
  });

  it("非預期格式整篇以單一檢視退回（無背景分頁、全文可見）", async () => {
    const props = makeProps({
      loadDocument: vi.fn(async () => "手寫的自由格式記錄，沒有標準區段。"),
    });
    render(<DiscussionDrawer {...(props as never)} />);
    await waitFor(() => expect(screen.getByText(/自由格式記錄/)).toBeTruthy());
    expect(screen.queryByRole("tab", { name: /背景/ })).toBeNull();
  });

  it("衍生變更分頁列出子變更現況；存活者可跳轉、已刪除者不可", async () => {
    const props = makeProps({ discussion: promotedD });
    render(<DiscussionDrawer {...(props as never)} />);
    await waitFor(() => screen.getByRole("tab", { name: /衍生變更/ }));
    fireEvent.mouseDown(screen.getByRole("tab", { name: /衍生變更/ }));
    const rowA = screen.getByText("cut-a").closest("[data-promoted-row]") as HTMLElement;
    expect(within(rowA).getByText("提案中")).toBeTruthy();
    fireEvent.click(within(rowA).getByRole("button", { name: /開啟卡片/ }));
    expect(props.onOpenChangeCard).toHaveBeenCalledWith("cut-a");
    const rowGone = screen.getByText("cut-gone").closest("[data-promoted-row]") as HTMLElement;
    expect(within(rowGone).getByText("已刪除")).toBeTruthy();
    expect(within(rowGone).queryByRole("button", { name: /開啟卡片/ })).toBeNull();
  });

  it("concluded 為「轉為變更」、promoted 為「再轉出一個變更」；open 無轉出按鈕", async () => {
    const props = makeProps();
    const { unmount } = render(<DiscussionDrawer {...(props as never)} />);
    fireEvent.mouseDown(screen.getByRole("tab", { name: /衍生變更/ }));
    fireEvent.click(screen.getByRole("button", { name: /轉為變更/ }));
    expect(props.onPromote).toHaveBeenCalledWith("alpha-search");
    unmount();

    const props2 = makeProps({ discussion: promotedD });
    const { unmount: unmount2 } = render(<DiscussionDrawer {...(props2 as never)} />);
    fireEvent.mouseDown(screen.getByRole("tab", { name: /衍生變更/ }));
    fireEvent.click(screen.getByRole("button", { name: /再轉出一個變更/ }));
    expect(props2.onPromote).toHaveBeenCalledWith("alpha-search");
    unmount2();

    const props3 = makeProps({ discussion: openD, loadDocument: vi.fn(async () => OPEN_DOC) });
    render(<DiscussionDrawer {...(props3 as never)} />);
    fireEvent.mouseDown(screen.getByRole("tab", { name: /衍生變更/ }));
    expect(screen.queryByRole("button", { name: /轉為變更/ })).toBeNull();
    expect(screen.queryByRole("button", { name: /促轉/ })).toBeNull();
  });

  it("促轉失敗以單行錯誤呈現", async () => {
    const props = makeProps({ error: "Change 'alpha-search' already exists." });
    render(<DiscussionDrawer {...(props as never)} />);
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain("already exists");
  });
});

describe("change 側同源連結", () => {
  it("來自討論的 change 卡帶討論徽章；無來源者不帶", () => {
    const withSource: ChangeItem = {
      name: "cut-a",
      status: "in-progress",
      totalTasks: 24,
      completedTasks: 0,
      fromDiscussion: "alpha-search",
    };
    const { unmount } = render(<ChangeCard change={withSource} />);
    expect(screen.getByLabelText("來自討論")).toBeTruthy();
    unmount();
    render(<ChangeCard change={{ ...withSource, fromDiscussion: undefined }} />);
    expect(screen.queryByLabelText("來自討論")).toBeNull();
  });

  it("change 抽屜顯示「來自討論」與同源清單並可互跳", async () => {
    const onOpenDiscussion = vi.fn();
    const onOpenSibling = vi.fn();
    const change: ChangeItem = {
      name: "cut-a",
      status: "in-progress",
      totalTasks: 24,
      completedTasks: 0,
      fromDiscussion: "alpha-search",
    };
    render(
      <RichDetailDrawer
        open
        onOpenChange={vi.fn()}
        change={change}
        loadDocument={vi.fn(async () => "# doc")}
        loadCapabilities={vi.fn(async () => [])}
        loadMeta={vi.fn(async () => ({ created: "2026-07-05" }))}
        sourceDiscussion={{ slug: "alpha-search", topic: "Alpha search" }}
        siblingChanges={["cut-b"]}
        onOpenDiscussion={onOpenDiscussion}
        onOpenSibling={onOpenSibling}
      />,
    );
    await waitFor(() => expect(screen.getByText(/來自討論/)).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: /Alpha search/ }));
    expect(onOpenDiscussion).toHaveBeenCalledWith("alpha-search");
    fireEvent.click(screen.getByRole("button", { name: /cut-b/ }));
    expect(onOpenSibling).toHaveBeenCalledWith("cut-b");
  });
});

describe("DiscussionDrawer 世代重載（spec：外部推進討論後抽屜內容更新）", () => {
  it("refreshGen 遞增時重載記錄，回合分頁呈現新回合且分頁選擇不重置", async () => {
    const props = makeProps();
    const { rerender } = render(<DiscussionDrawer {...(props as never)} refreshGen={0} />);
    await waitFor(() => screen.getByRole("tab", { name: /討論過程/ }));
    fireEvent.mouseDown(screen.getByRole("tab", { name: /討論過程/ }));
    await waitFor(() => expect(screen.getByText(/範圍界定/)).toBeTruthy());
    const calls = () => (props.loadDocument as ReturnType<typeof vi.fn>).mock.calls.length;
    const c0 = calls();
    // 外部 speclink discuss add-round 後 watcher 觸發 refresh → 世代遞增。
    const DOC2 = DOC.replace(
      "## Conclusion",
      "### Round 2 — assumptions (2026-07-02)\n\n**Focus**: 第二輪新內容\n\n## Conclusion",
    );
    (props.loadDocument as ReturnType<typeof vi.fn>).mockResolvedValue(DOC2);
    rerender(<DiscussionDrawer {...(props as never)} refreshGen={1} />);
    await waitFor(() => expect(calls()).toBeGreaterThan(c0));
    await waitFor(() => expect(screen.getByText(/第二輪新內容/)).toBeTruthy());
    // 舊回合仍在、分頁停留在討論過程——重載為就地替換，非重開抽屜。
    expect(screen.getByText(/範圍界定/)).toBeTruthy();
  });
});

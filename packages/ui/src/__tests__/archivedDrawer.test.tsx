// 已封存項目以抽屜檢視（spec「已封存項目以抽屜檢視」；design D1：discriminated
// target 兩型同檔）：封存變更四分頁唯讀（任務核取方塊 disabled、無批次工具列）、
// 封存討論「背景」「討論過程」「結論」區段、缺件文件空狀態、無任何寫入動詞、
// 寬度與全螢幕切換與變更詳情抽屜一致、開啟期間外部變更反映。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, waitFor, fireEvent } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { ArchivedDrawer } from "../components/ArchivedDrawer";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

const PROPOSAL = "## Why\n\n封存提案內文。\n";
const TASKS_MD = "## 1. G\n\n- [x] 1.1 done-task\n- [ ] 1.2 open-task\n";
const SPEC_DELTA = "## ADDED Requirements\n\n### Requirement: 新需求\n\n內文。\n";
const DISCUSSION_DOC = `---
topic: Alpha topic
slug: alpha-search
status: promoted
created: 2026-01-02
---

# Discussion: Alpha topic

## Context

討論背景內文。

## Rounds

### Round 1 — assumptions (2026-01-02)

**Focus**: scope

## Conclusion

**Decision**: 就這麼辦
`;

function makeProps(over: Record<string, unknown> = {}) {
  return {
    open: true,
    onOpenChange: vi.fn(),
    target: { kind: "change", datedName: "2026-07-04-old-change" },
    loadDocument: vi.fn(async (_d: string, artifact: string) => {
      if (artifact === "proposal.md") return PROPOSAL;
      if (artifact === "design.md") return null; // 缺件樣本
      if (artifact === "tasks.md") return TASKS_MD;
      return artifact.startsWith("specs/") ? SPEC_DELTA : null;
    }),
    loadCapabilities: vi.fn(async () => ["cap-x"]),
    loadDiscussionDocument: vi.fn(async () => DISCUSSION_DOC),
    ...over,
  };
}

const drawerEl = () => document.querySelector("[data-archived-drawer]") as HTMLElement | null;

describe("ArchivedDrawer（封存變更 target）", () => {
  it("開啟呈現提案／設計／任務／規格四分頁與提案內容，標題為封存名稱", async () => {
    const props = makeProps();
    render(<ArchivedDrawer {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    expect(screen.getByText("2026-07-04-old-change")).toBeTruthy();
    for (const name of [/提案/, /設計/, /任務/, /規格/]) {
      expect(screen.getByRole("tab", { name })).toBeTruthy();
    }
    expect(props.loadDocument).toHaveBeenCalledWith("2026-07-04-old-change", "proposal.md");
  });

  it("任務分頁唯讀：核取方塊 disabled、無批次工具列", async () => {
    render(<ArchivedDrawer {...(makeProps() as never)} />);
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    fireEvent.mouseDown(screen.getByRole("tab", { name: /任務/ }));
    await waitFor(() => expect(screen.getByText("1.1 done-task")).toBeTruthy());
    const boxes = screen.getAllByRole("checkbox");
    expect(boxes.length).toBe(2);
    for (const box of boxes) {
      expect((box as HTMLButtonElement).disabled).toBe(true);
    }
    // 批次工具列缺席（readOnly：無「全部已完成」／「重置任務」）。
    expect(screen.queryByText("全部已完成")).toBeNull();
    expect(screen.queryByText("重置任務")).toBeNull();
  });

  it("規格分頁呈現 delta 規格；缺件設計分頁顯示空狀態而非錯誤", async () => {
    render(<ArchivedDrawer {...(makeProps() as never)} />);
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    fireEvent.mouseDown(screen.getByRole("tab", { name: /規格/ }));
    await waitFor(() => expect(screen.getByText(/新需求/)).toBeTruthy());
    // 缺件文件顯示空狀態（spec Scenario「缺件文件顯示空狀態」），其餘分頁照常可用。
    fireEvent.mouseDown(screen.getByRole("tab", { name: /設計/ }));
    await waitFor(() => expect(screen.getByText("（此變更無設計文件）")).toBeTruthy());
  });

  it("無任何寫入動詞（封存／刪除／分析／轉為變更皆缺席）", async () => {
    render(<ArchivedDrawer {...(makeProps() as never)} />);
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    for (const name of [/封存/, /刪除/, /分析/, /轉為變更/]) {
      expect(screen.queryByRole("button", { name })).toBeNull();
    }
  });

  it("寬度樣式與變更詳情抽屜一致，含全螢幕切換與還原", async () => {
    render(<ArchivedDrawer {...(makeProps() as never)} />);
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    const content = drawerEl();
    expect(content).toBeTruthy();
    expect(content!.className).toContain("w-[max(720px,42vw)]");
    expect(content!.className).toContain("max-w-[95vw]");
    fireEvent.click(screen.getByRole("button", { name: "全螢幕" }));
    expect(drawerEl()!.className).toContain("w-[96vw]");
    fireEvent.click(screen.getByRole("button", { name: "還原大小" }));
    expect(drawerEl()!.className).toContain("w-[max(720px,42vw)]");
  });

  it("sourceDiscussions 顯示可點 topic chips，點擊以 slug 呼叫 onOpenDiscussion（design D1 增補）", async () => {
    // spec Scenario「自封存變更抽屜跳轉來源討論」的元件面。
    const props = makeProps({
      sourceDiscussions: [
        { slug: "alpha-ux", topic: "Alpha UX 討論" },
        { slug: "beta-flow", topic: "beta-flow" },
      ],
      onOpenDiscussion: vi.fn(),
    });
    render(<ArchivedDrawer {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    expect(screen.getByText("來自討論：")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Alpha UX 討論" }));
    expect(props.onOpenDiscussion).toHaveBeenCalledWith("alpha-ux");
    // 多來源全列（topic 解析缺席時退回 slug 由宿主處理，元件原樣顯示）。
    expect(screen.getByRole("button", { name: "beta-flow" })).toBeTruthy();
  });

  it("sourceDiscussions 缺席或空陣列時來源討論區塊不渲染", async () => {
    const { unmount } = render(<ArchivedDrawer {...(makeProps() as never)} />);
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    expect(screen.queryByText("來自討論：")).toBeNull();
    unmount();
    render(<ArchivedDrawer {...(makeProps({ sourceDiscussions: [] }) as never)} />);
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    expect(screen.queryByText("來自討論：")).toBeNull();
  });

  it("refreshGen 遞增時就地重載至磁碟現況（開啟期間外部變更反映）", async () => {
    let body = "第一版提案。";
    const loadDocument = vi.fn(async (_d: string, artifact: string) =>
      artifact === "proposal.md" ? `## Why\n\n${body}\n` : null,
    );
    const props = makeProps({ loadDocument, loadCapabilities: vi.fn(async () => []) });
    const { rerender } = render(<ArchivedDrawer {...(props as never)} refreshGen={0} />);
    await waitFor(() => expect(screen.getByText("第一版提案。")).toBeTruthy());
    body = "第二版提案。";
    rerender(<ArchivedDrawer {...(props as never)} refreshGen={1} />);
    await waitFor(() => expect(screen.getByText("第二版提案。")).toBeTruthy());
    expect(screen.queryByText("第一版提案。")).toBeNull();
  });
});

describe("ArchivedDrawer（封存討論 target）", () => {
  const discussionProps = (over: Record<string, unknown> = {}) =>
    makeProps({ target: { kind: "discussion", slug: "alpha-search" }, ...over });

  it("呈現「背景」「討論過程」「結論」區段與記錄內容，無寫入動詞", async () => {
    const props = discussionProps();
    render(<ArchivedDrawer {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("討論背景內文。")).toBeTruthy());
    expect(props.loadDiscussionDocument).toHaveBeenCalledWith("alpha-search");
    expect(screen.getByText("背景")).toBeTruthy();
    expect(screen.getByText("討論過程")).toBeTruthy();
    expect(screen.getByText("結論")).toBeTruthy();
    expect(screen.getByText(/就這麼辦/)).toBeTruthy();
    // 無轉為變更、封存或任何寫入按鈕（spec Scenario「點擊封存討論卡開啟唯讀抽屜」）。
    for (const name of [/轉為變更/, /封存/, /刪除/] as RegExp[]) {
      expect(screen.queryByRole("button", { name })).toBeNull();
    }
  });

  it("記錄缺席顯示空狀態而非錯誤", async () => {
    const props = discussionProps({ loadDiscussionDocument: vi.fn(async () => null) });
    render(<ArchivedDrawer {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("（無內容）")).toBeTruthy());
  });

  it("封存討論 target 不顯示來源討論 chips（即使 props 帶值）", async () => {
    const props = discussionProps({
      sourceDiscussions: [{ slug: "alpha-ux", topic: "Alpha UX 討論" }],
      onOpenDiscussion: vi.fn(),
    });
    render(<ArchivedDrawer {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("討論背景內文。")).toBeTruthy());
    expect(screen.queryByText("來自討論：")).toBeNull();
  });
});

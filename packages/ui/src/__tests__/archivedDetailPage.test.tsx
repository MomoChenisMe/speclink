// 已封存詳情頁（spec「已封存項目以詳情頁檢視」「已封存詳情頁的工單分頁」「已封存側的審查標示」
// 「已封存側的驗證標示」；desktop-readonly-detail-pages design D3）：封存變更——標題列（變更名＋
// 複製封存目錄名＋結局籤）、出身列、「狀態」卡三列、提案／設計／任務／規格＋條件式工單分頁、
// 任務唯讀；封存討論——slug＋「已封存」籤、階段列現站「已封存」、衍生變更三態、結論／討論過程／
// 背景／衍生變更分頁、「開啟 ▾」只能複製內容。兩者皆無寫入動詞。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, waitFor, fireEvent, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { ArchivedDetailPage } from "../components/ArchivedDetailPage";

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
const discussionProps = (over: Record<string, unknown> = {}) =>
  makeProps({ target: { kind: "discussion", slug: "alpha-search" }, discussionTopic: "Alpha topic", ...over });

const provenanceRow = () => document.querySelector("[data-identity-provenance]") as HTMLElement | null;
const identity = () => document.querySelector("[data-detail-identity]") as HTMLElement;
const statusRow = (mark: string) => document.querySelector(`[data-status-${mark}]`) as HTMLElement | null;
const clipboard = () => {
  const writeText = vi.fn().mockResolvedValue(undefined);
  Object.defineProperty(navigator, "clipboard", { value: { writeText }, configurable: true });
  return writeText;
};
const tabNames = () => screen.getAllByRole("tab").map((el) => (el.textContent ?? "").replace(/\s+/g, " ").trim());

describe("ArchivedDetailPage（封存變更）", () => {
  it("標題為變更名、分頁提案／設計／任務／規格，預設呈現提案", async () => {
    const props = makeProps();
    render(<ArchivedDetailPage {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("old-change");
    expect(tabNames().map((n) => n.replace(/\s*\d.*$/, ""))).toEqual(["提案", "設計", "任務", "規格"]);
    expect(props.loadDocument).toHaveBeenCalledWith("2026-07-04-old-change", "proposal.md");
  });

  it("複製鈕寫入含日期前綴的封存目錄名並顯示已複製回饋", async () => {
    const writeText = clipboard();
    render(<ArchivedDetailPage {...(makeProps() as never)} />);
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    fireEvent.click(screen.getByLabelText("複製封存名稱"));
    expect(writeText).toHaveBeenCalledWith("2026-07-04-old-change");
    await waitFor(() => expect(screen.getByRole("status").textContent).toBe("已複製"));
  });

  it("身分區帶已封存籤與結局籤；狀態列三列為任務 n/m、品質關卡結局、封存日期；無進度條與動作區", async () => {
    // spec Scenario「封存變更詳情頁的標題列、出身列與狀態卡」。
    render(
      <ArchivedDetailPage
        {...(makeProps({
          reviewStatus: "reviewed",
          verifyStatus: "verifiedNotPassed",
          archivedDate: "2026-07-04",
        }) as never)}
      />,
    );
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    const title = identity();
    expect(title.querySelector("[data-archived-status]")!.textContent).toBe("已封存");
    expect(title.querySelector("[data-review-outcome]")!.textContent).toContain("已審查");
    expect(title.querySelector("[data-verify-outcome]")!.textContent).toContain("曾驗證未通過");
    await waitFor(() => expect(statusRow("tasks")!.textContent).toContain("1/2"));
    expect(statusRow("quality")!.textContent).toContain("已審查");
    expect(statusRow("quality")!.textContent).toContain("曾驗證未通過");
    expect(statusRow("archived")!.textContent).toContain("2026-07-04");
    expect(document.querySelector("[data-progress-track]")).toBeNull();
    for (const name of ["封存", "刪除", "分析", "轉為變更", "退回提案中"]) {
      expect(screen.queryByRole("button", { name })).toBeNull();
    }
  });

  it("皆無結局時品質關卡列顯示「未經品質關卡」、身分區無結局籤；無 tasks.md 時任務列為「—」", async () => {
    const loadDocument = vi.fn(async (_d: string, artifact: string) => (artifact === "proposal.md" ? PROPOSAL : null));
    render(<ArchivedDetailPage {...(makeProps({ loadDocument }) as never)} />);
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    expect(statusRow("quality")!.textContent).toContain("未經品質關卡");
    expect(document.querySelector("[data-review-outcome]")).toBeNull();
    await waitFor(() => expect(statusRow("tasks")!.textContent).toContain("—"));
  });

  it("出身列：建立者首字母圓標＋名字、建立日期、「封存 日期」", async () => {
    render(
      <ArchivedDetailPage
        {...(makeProps({
          createdBy: "MomoChen <momo@example.com>",
          created: "2026-06-20",
          archivedDate: "2026-07-04",
        }) as never)}
      />,
    );
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    const row = provenanceRow()!;
    expect(row.className).toContain("whitespace-nowrap");
    expect(within(row).getByText("M").className).toContain("rounded-full");
    expect(within(row).getByText("MomoChen")).toBeTruthy();
    expect(row.textContent).toContain("2026-06-20");
    expect(row.textContent).toContain("封存 2026-07-04");
  });

  it("出身列欄位缺席時該欄缺席、其餘照常（建立日期不可得）", async () => {
    // spec Scenario「出身列欄位缺席時其餘照常」。
    render(
      <ArchivedDetailPage
        {...(makeProps({ createdBy: "MomoChen <momo@example.com>", archivedDate: "2026-07-04" }) as never)}
      />,
    );
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    const row = provenanceRow()!;
    expect(row.textContent).toBe("MMomoChen·封存 2026-07-04");
  });

  it("來源討論籤：首籤直出、點擊以 slug 呼叫 onOpenDiscussion；缺席時不顯示", async () => {
    const onOpenDiscussion = vi.fn();
    const props = makeProps({ sourceDiscussions: [{ slug: "alpha-ux", topic: "Alpha UX 討論" }], onOpenDiscussion });
    const { unmount } = render(<ArchivedDetailPage {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    const from = document.querySelector("[data-relation-from]") as HTMLElement;
    expect(from.textContent).toContain("來自");
    fireEvent.click(within(from).getByRole("button", { name: /alpha-ux/ }));
    expect(onOpenDiscussion).toHaveBeenCalledWith("alpha-ux");
    unmount();
    render(<ArchivedDetailPage {...(makeProps() as never)} />);
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    expect(document.querySelector("[data-source-discussion-label]")).toBeNull();
  });

  it("任務分頁唯讀：核取方塊 disabled、無把手、無批次工具列", async () => {
    render(<ArchivedDetailPage {...(makeProps() as never)} />);
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    fireEvent.mouseDown(screen.getByRole("tab", { name: /任務/ }));
    await waitFor(() => expect(screen.getByText("done-task")).toBeTruthy());
    const boxes = screen.getAllByRole("checkbox");
    expect(boxes).toHaveLength(2);
    for (const box of boxes) expect((box as HTMLButtonElement).disabled).toBe(true);
    expect(screen.queryByText("全部已完成")).toBeNull();
    expect(screen.queryByText("重置任務")).toBeNull();
    expect(document.querySelector(".lucide-grip-vertical")).toBeNull();
    fireEvent.click(screen.getByText("open-task"));
    expect((boxes[1] as HTMLButtonElement).getAttribute("data-state")).toBe("unchecked");
  });

  it("規格分頁呈現 delta 規格；缺件設計分頁顯示空狀態而非錯誤", async () => {
    render(<ArchivedDetailPage {...(makeProps() as never)} />);
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    fireEvent.mouseDown(screen.getByRole("tab", { name: /規格/ }));
    await waitFor(() => expect(screen.getByText(/新需求/)).toBeTruthy());
    fireEvent.mouseDown(screen.getByRole("tab", { name: /設計/ }));
    await waitFor(() => expect(screen.getByText("（此變更無設計文件）")).toBeTruthy());
  });

  it("「開啟 ▾」依作用中分頁收到封存目錄內的對應文件", async () => {
    const renderDocumentActions = vi.fn(() => null);
    render(<ArchivedDetailPage {...(makeProps({ renderDocumentActions }) as never)} />);
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    expect(renderDocumentActions).toHaveBeenLastCalledWith({
      doc: { archived: "2026-07-04-old-change", artifact: "proposal.md" },
      content: PROPOSAL,
    });
    fireEvent.mouseDown(screen.getByRole("tab", { name: /規格/ }));
    await waitFor(() =>
      expect(renderDocumentActions).toHaveBeenLastCalledWith({
        doc: { archived: "2026-07-04-old-change", artifact: "specs/cap-x/spec.md" },
        content: SPEC_DELTA,
      }),
    );
  });

  it("refreshGen 遞增時就地重載至磁碟現況", async () => {
    let body = "第一版提案。";
    const loadDocument = vi.fn(async (_d: string, artifact: string) =>
      artifact === "proposal.md" ? `## Why\n\n${body}\n` : null,
    );
    const props = makeProps({ loadDocument, loadCapabilities: vi.fn(async () => []) });
    const { rerender } = render(<ArchivedDetailPage {...(props as never)} refreshGen={0} />);
    await waitFor(() => expect(screen.getByText("第一版提案。")).toBeTruthy());
    body = "第二版提案。";
    rerender(<ArchivedDetailPage {...(props as never)} refreshGen={1} />);
    await waitFor(() => expect(screen.getByText("第二版提案。")).toBeTruthy());
    expect(screen.queryByText("第一版提案。")).toBeNull();
  });

  it("換目標時清空並重載、分頁回到提案", async () => {
    const loadDocument = vi.fn(async (d: string, artifact: string) =>
      artifact === "proposal.md" ? `## Why\n\n${d} 的提案。\n` : null,
    );
    const props = makeProps({ loadDocument });
    const { rerender } = render(<ArchivedDetailPage {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("2026-07-04-old-change 的提案。")).toBeTruthy());
    fireEvent.mouseDown(screen.getByRole("tab", { name: /設計/ }));
    rerender(<ArchivedDetailPage {...(props as never)} target={{ kind: "change", datedName: "2026-08-01-next" }} />);
    await waitFor(() => expect(screen.getByText("2026-08-01-next 的提案。")).toBeTruthy());
    expect(screen.getByRole("tab", { name: /提案/ }).getAttribute("data-state")).toBe("active");
  });
});

// spec「詳情頁的文件樹、閱讀卡與本頁目錄」Scenario「已封存詳情頁的唯讀變體」：同骨架、無標題列動作。
describe("已封存詳情頁的唯讀骨架", () => {
  it("已封存詳情頁無 data-detail-actions 子節點、身分區含已封存籤、文件節點在左欄", async () => {
    const { unmount } = render(<ArchivedDetailPage {...(makeProps({ archivedDate: "2026-07-04" }) as never)} />);
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    expect(document.querySelector("[data-detail-actions]")).toBeNull();
    expect(within(identity()).getByText("已封存")).toBeTruthy();
    expect(provenanceRow()!.textContent).toContain("封存 2026-07-04");
    const tree = document.querySelector("[data-detail-tree]") as HTMLElement;
    expect(within(tree).getByRole("tablist")).toBeTruthy();
    expect(document.querySelector("[data-doc-header]")!.textContent).toContain("提案");
    unmount();
    render(<ArchivedDetailPage {...(discussionProps() as never)} />);
    await waitFor(() => expect(screen.getByText(/就這麼辦/)).toBeTruthy());
    expect(document.querySelector("[data-detail-actions]")).toBeNull();
    expect(within(identity()).getByText("已封存")).toBeTruthy();
  });

  it("閱讀卡出處行：封存變更為相對變更目錄的路徑，封存討論沒有出處行", async () => {
    const { unmount } = render(<ArchivedDetailPage {...(makeProps() as never)} />);
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    expect(document.querySelector("[data-doc-header] [data-doc-provenance]")!.textContent).toBe("proposal.md");
    fireEvent.mouseDown(screen.getByRole("tab", { name: /規格/ }));
    expect(document.querySelector("[data-doc-header] [data-doc-provenance]")!.textContent).toBe("specs/cap-x/spec.md");
    unmount();
    render(<ArchivedDetailPage {...(discussionProps() as never)} />);
    await waitFor(() => expect(screen.getByText(/就這麼辦/)).toBeTruthy());
    expect(document.querySelector("[data-doc-header]")).toBeTruthy();
    expect(document.querySelector("[data-doc-provenance]")).toBeNull();
  });
});

describe("ArchivedDetailPage（封存討論）", () => {
  it("標題為 slug＋「已封存」籤、topic 為副標；階段列現站「已封存」；分頁結論／討論過程／背景／衍生變更；無寫入動詞", async () => {
    const props = discussionProps({ discussionKind: "improve" });
    render(<ArchivedDetailPage {...(props as never)} />);
    await waitFor(() => expect(screen.getByText(/就這麼辦/)).toBeTruthy());
    expect(props.loadDiscussionDocument).toHaveBeenCalledWith("alpha-search");
    const title = identity();
    expect(within(title).getByRole("heading", { level: 1 }).textContent).toBe("alpha-search");
    expect(title.querySelector("[data-archived-status]")!.textContent).toBe("已封存");
    expect(title.querySelector('[data-discussion-kind="improve"]')).toBeTruthy();
    expect(screen.getByText("Alpha topic")).toBeTruthy();
    expect(statusRow("stage")!.querySelector('[aria-current="step"]')!.textContent).toBe("已封存");
    expect(tabNames().map((n) => n.replace(/\s*\d+$/, ""))).toEqual(["結論", "討論過程", "背景", "衍生變更"]);
    for (const name of ["轉為變更", "封存", "刪除"]) {
      expect(screen.queryByRole("button", { name })).toBeNull();
    }
  });

  it("出身列：建立者與建立日期；清單資料不帶封存日期時「封存」欄缺席、其餘照常", async () => {
    render(
      <ArchivedDetailPage
        {...(discussionProps({ createdBy: "MomoChen <momo@example.com>", created: "2026-01-02" }) as never)}
      />,
    );
    await waitFor(() => expect(screen.getByText(/就這麼辦/)).toBeTruthy());
    expect(provenanceRow()!.textContent).toBe("MMomoChen·2026-01-02");
  });

  it("複製鈕寫入 slug", async () => {
    const writeText = clipboard();
    render(<ArchivedDetailPage {...(discussionProps() as never)} />);
    await waitFor(() => expect(screen.getByText(/就這麼辦/)).toBeTruthy());
    fireEvent.click(screen.getByLabelText("複製 slug"));
    expect(writeText).toHaveBeenCalledWith("alpha-search");
  });

  it("衍生變更列三態：依轉出順序、副標與可點性；點可點者以變更名呼叫，點已刪除者無動作", async () => {
    // spec Example「三態對照」。
    const onOpenPromotedChange = vi.fn();
    const props = discussionProps({
      promotedChanges: [
        { slug: "child-a", topic: "2026-09-02" },
        { slug: "child-b", topic: "進行中" },
        { slug: "child-c", topic: "無封存記錄", disabled: true },
      ],
      onOpenPromotedChange,
    });
    render(<ArchivedDetailPage {...(props as never)} />);
    await waitFor(() => expect(screen.getByText(/就這麼辦/)).toBeTruthy());
    const row = statusRow("promoted")!;
    const chips = Array.from(row.querySelectorAll<HTMLElement>("[data-promoted-chip]"));
    expect(chips.map((c) => c.textContent)).toEqual(["child-a2026-09-02", "child-b進行中", "child-c無封存記錄"]);
    fireEvent.click(within(chips[0]).getByRole("button"));
    expect(onOpenPromotedChange).toHaveBeenCalledWith("child-a");
    fireEvent.click(within(chips[1]).getByRole("button"));
    expect(onOpenPromotedChange).toHaveBeenCalledWith("child-b");
    const gone = within(chips[2]).getByRole("button") as HTMLButtonElement;
    expect(gone.disabled).toBe(true);
    fireEvent.click(gone);
    expect(onOpenPromotedChange).toHaveBeenCalledTimes(2);
  });

  it("無衍生變更時衍生列缺席；封存變更頁即使傳入也無該列", async () => {
    const { unmount } = render(<ArchivedDetailPage {...(discussionProps({ promotedChanges: [] }) as never)} />);
    await waitFor(() => expect(screen.getByText(/就這麼辦/)).toBeTruthy());
    expect(statusRow("promoted")).toBeNull();
    expect(statusRow("stage")).toBeTruthy();
    unmount();
    render(<ArchivedDetailPage {...(makeProps({ promotedChanges: [{ slug: "x", topic: "y" }] }) as never)} />);
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    expect(statusRow("promoted")).toBeNull();
  });

  it("「開啟 ▾」收到不帶檔案的目標（只能複製內容）", async () => {
    const renderDocumentActions = vi.fn(() => null);
    render(<ArchivedDetailPage {...(discussionProps({ renderDocumentActions }) as never)} />);
    await waitFor(() => expect(screen.getByText(/就這麼辦/)).toBeTruthy());
    expect(renderDocumentActions).toHaveBeenLastCalledWith({ doc: null, content: DISCUSSION_DOC });
  });

  it("記錄缺席顯示空狀態而非錯誤", async () => {
    render(<ArchivedDetailPage {...(discussionProps({ loadDiscussionDocument: vi.fn(async () => null) }) as never)} />);
    await waitFor(() => expect(screen.getByText("（無內容）")).toBeTruthy());
  });

  it("封存討論不顯示來源討論籤（即使 props 帶值）", async () => {
    render(
      <ArchivedDetailPage
        {...(discussionProps({ sourceDiscussions: [{ slug: "alpha-ux", topic: "x" }], onOpenDiscussion: vi.fn() }) as never)}
      />,
    );
    await waitFor(() => expect(screen.getByText(/就這麼辦/)).toBeTruthy());
    expect(document.querySelector("[data-source-discussion-label]")).toBeNull();
  });
});

// spec「詳情頁文件載入以 skeleton 呈現」：各分頁載入中畫骨架、失敗收斂到空態。
describe("已封存詳情頁文件三態", () => {
  const pending = () => new Promise<never>(() => {});

  it("封存變更載入中 → 骨架，且不出假空態", async () => {
    render(
      <ArchivedDetailPage {...(makeProps({ loadDocument: vi.fn(pending), loadCapabilities: vi.fn(pending) }) as never)} />,
    );
    await waitFor(() => expect(document.querySelector('[aria-busy="true"]')).toBeTruthy());
    expect(screen.queryByText("（此變更無設計文件）")).toBeNull();
  });

  it("封存討論載入中 → 骨架，不出空態文案", async () => {
    render(<ArchivedDetailPage {...(discussionProps({ loadDiscussionDocument: vi.fn(pending) }) as never)} />);
    await waitFor(() => expect(document.querySelector('[aria-busy="true"]')).toBeTruthy());
    expect(screen.queryByText("（無內容）")).toBeNull();
  });

  it("封存變更文件首載失敗 → 空態文案，不停在骨架", async () => {
    render(
      <ArchivedDetailPage {...(makeProps({ loadDocument: vi.fn().mockRejectedValue(new Error("offline")) }) as never)} />,
    );
    await waitFor(() => expect(screen.getByText("（無提案文件）")).toBeTruthy());
    expect(document.querySelector('[aria-busy="true"]')).toBeNull();
  });

  it("封存討論首載失敗 → 空態文案，不停在骨架", async () => {
    render(
      <ArchivedDetailPage
        {...(discussionProps({ loadDiscussionDocument: vi.fn().mockRejectedValue(new Error("offline")) }) as never)}
      />,
    );
    await waitFor(() => expect(screen.getByText("（無內容）")).toBeTruthy());
    expect(document.querySelector('[aria-busy="true"]')).toBeNull();
  });

  it("loadCapabilities 失敗 → 規格分頁收斂為空態文案", async () => {
    render(
      <ArchivedDetailPage {...(makeProps({ loadCapabilities: vi.fn(async () => Promise.reject("讀不到")) }) as never)} />,
    );
    await waitFor(() => screen.getByRole("tab", { name: /規格/ }));
    fireEvent.mouseDown(screen.getByRole("tab", { name: /規格/ }));
    await waitFor(() => expect(screen.getByText("（此變更無 delta 規格）")).toBeTruthy());
    expect(document.querySelector('[aria-busy="true"]')).toBeNull();
  });
});

// desktop-change-doc-views design D9：已封存變更共用同一批渲染器與推導（唯讀、無排程）。
describe("已封存變更的文件渲染器", () => {
  const flat = (el: Element | null) => (el?.textContent ?? "").replace(/\s+/g, " ").trim();
  const docs: Record<string, string> = {
    "proposal.md": "## Why\n\n- 動機\n\n## What Changes\n\n- 第一步\n- 第二步\n\n## Non-Goals\n\n- 不做\n",
    "tasks.md": "## 1. 甲\n\n- [x] 1.1 已做（design D2）：內文；驗證：npm test\n- [ ] 1.2 未做\n- [ ] [M] 1.3 手動看畫面\n",
    "specs/cap-x/spec.md": SPEC_DELTA,
  };
  const props = () => makeProps({ loadDocument: vi.fn(async (_d: string, artifact: string) => docs[artifact] ?? null) });

  it("提案文件有籤與數字磚；任務卡勾選框 disabled 而展開鈕可用", async () => {
    render(<ArchivedDetailPage {...(props() as never)} />);
    await waitFor(() => expect(flat(document.querySelector("[data-doc-badges]"))).toBe("Feature"));
    expect(Array.from(document.querySelectorAll("[data-doc-tile]")).map((t) => flat(t))).toEqual(["2解法步驟", "1非目標"]);
    fireEvent.mouseDown(screen.getByRole("tab", { name: /任務/ }));
    const expand = await screen.findByRole("button", { name: "展開任務 1" });
    for (const box of screen.getAllByRole("checkbox")) expect((box as HTMLButtonElement).disabled).toBe(true);
    expect((expand as HTMLButtonElement).disabled).toBe(false);
    fireEvent.click(expand);
    expect(screen.getByText("npm test")).toBeTruthy();
    expect(flat(document.querySelector("[data-doc-badges]"))).toBe("1/3");
  });

  it("手動提示框的「前往」使該任務列高亮（唯讀照常）", async () => {
    Element.prototype.scrollIntoView = vi.fn();
    render(<ArchivedDetailPage {...(props() as never)} />);
    await screen.findByText("動機");
    fireEvent.mouseDown(screen.getByRole("tab", { name: /任務/ }));
    const go = await screen.findByRole("button", { name: "前往" });
    fireEvent.click(go);
    await waitFor(() =>
      expect(document.querySelector("[data-task-manual]")?.getAttribute("data-highlight")).toBe("true"),
    );
  });

  it("任務的 D 籤切到設計文件；規格文件能力頭有「看正式規格」", async () => {
    const onOpenSpec = vi.fn();
    render(<ArchivedDetailPage {...(makeProps({ ...props(), onOpenSpec }) as never)} />);
    await screen.findByText("動機");
    fireEvent.mouseDown(screen.getByRole("tab", { name: /任務/ }));
    fireEvent.click((await waitFor(() => document.querySelector('[data-task-decision="D2"]'))) as HTMLElement);
    await waitFor(() => expect(screen.getByRole("tab", { name: /設計/ }).getAttribute("data-state")).toBe("active"));
    fireEvent.mouseDown(screen.getByRole("tab", { name: /規格/ }));
    fireEvent.click(await screen.findByRole("button", { name: "看正式規格" }));
    expect(onOpenSpec).toHaveBeenCalledWith("cap-x");
  });
});

// spec「已封存詳情頁的工單分頁」：reviewedNotPassed／verifiedNotPassed 於「規格」之後出現對應
// 分頁（審查在前），以 (datedName, station) 載入化石工單；reviewed／verified／none 不出分頁、不發請求。
describe("已封存詳情頁的工單分頁", () => {
  const HASH = `sha256:${"a".repeat(64)}`;
  const TICKET = {
    rounds: [
      {
        index: 1,
        phase: "discovery",
        patchHash: HASH,
        scope: ["src/a.rs", "src/b.rs"],
        findings: [{ severity: "CRITICAL", path: "src/a.rs", text: "Correctness: 未處理空清單" }],
      },
      {
        index: 2,
        phase: "validation",
        patchHash: HASH,
        scope: ["src/a.rs", "src/b.rs"],
        findings: [{ severity: "CRITICAL", path: "src/a.rs", text: "Correctness: 未處理空清單" }],
      },
    ],
  };
  // 段標題列已撤（desktop-change-doc-views design D8）：輪數與階段詞在籤槽、計數在數字磚。
  const flat = (el: Element | null) => (el?.textContent ?? "").replace(/\s+/g, " ").trim();
  const badges = () => flat(document.querySelector("[data-doc-badges]"));
  const tiles = () =>
    Array.from(document.querySelectorAll("[data-doc-tile]")).map((t) => `${flat(t.lastElementChild)} ${flat(t.firstElementChild)}`);

  it("reviewedNotPassed → 「審查」分頁排在規格之後，以 (datedName, review) 載入並呈現末輪", async () => {
    const loadStationTicket = vi.fn(async () => TICKET);
    render(
      <ArchivedDetailPage
        {...(makeProps({ reviewStatus: "reviewedNotPassed", verifyStatus: "verified", loadStationTicket }) as never)}
      />,
    );
    await waitFor(() => screen.getByRole("tab", { name: /審查/ }));
    const names = tabNames();
    expect(names).toHaveLength(5);
    expect(names[3]).toMatch(/規格/);
    expect(names[4]).toMatch(/審查/);
    expect(screen.queryByRole("tab", { name: /驗證/ })).toBeNull();
    await waitFor(() => expect(loadStationTicket).toHaveBeenCalledWith("2026-07-04-old-change", "review"));
    fireEvent.mouseDown(screen.getByRole("tab", { name: /審查/ }));
    await waitFor(() => expect(badges()).toBe("第 2 輪複驗"));
    expect(tiles()).toEqual(["輪 2", "CRITICAL（末輪） 1", "WARNING（末輪） 0", "SUGGESTION（末輪） 0"]);
    const r2 = document.querySelector('[data-ticket-round="2"]') as HTMLElement;
    expect(r2.querySelector("[data-ticket-round-toggle]")?.getAttribute("aria-expanded")).toBe("true");
  });

  it("verifiedNotPassed → 「驗證」分頁以 (datedName, verify) 載入", async () => {
    const loadStationTicket = vi.fn(async () => TICKET);
    render(
      <ArchivedDetailPage
        {...(makeProps({ reviewStatus: "reviewed", verifyStatus: "verifiedNotPassed", loadStationTicket }) as never)}
      />,
    );
    await waitFor(() => screen.getByRole("tab", { name: /驗證/ }));
    expect(screen.queryByRole("tab", { name: /審查/ })).toBeNull();
    await waitFor(() => expect(loadStationTicket).toHaveBeenCalledWith("2026-07-04-old-change", "verify"));
    fireEvent.mouseDown(screen.getByRole("tab", { name: /驗證/ }));
    await waitFor(() => expect(badges()).toBe("第 2 輪複驗"));
  });

  it("兩站皆帶走 → 「審查」「驗證」依序；reviewed／none → 皆缺席且未發請求", async () => {
    const both = vi.fn(async () => TICKET);
    const first = render(
      <ArchivedDetailPage
        {...(makeProps({ reviewStatus: "reviewedNotPassed", verifyStatus: "verifiedNotPassed", loadStationTicket: both }) as never)}
      />,
    );
    await waitFor(() => screen.getByRole("tab", { name: /驗證/ }));
    const names = tabNames();
    expect(names).toHaveLength(6);
    expect(names[4]).toMatch(/審查/);
    expect(names[5]).toMatch(/驗證/);
    first.unmount();

    const none = vi.fn(async () => TICKET);
    render(
      <ArchivedDetailPage
        {...(makeProps({ reviewStatus: "reviewed", verifyStatus: "none", loadStationTicket: none }) as never)}
      />,
    );
    await screen.findByText("封存提案內文。");
    expect(tabNames()).toHaveLength(4);
    expect(none).not.toHaveBeenCalled();
  });

  it("載入器回 null 或 reject → 分頁顯示「工單尚未抵達或已被刪除」", async () => {
    const first = render(
      <ArchivedDetailPage
        {...(makeProps({ reviewStatus: "reviewedNotPassed", loadStationTicket: vi.fn(async () => null) }) as never)}
      />,
    );
    await waitFor(() => screen.getByRole("tab", { name: /審查/ }));
    fireEvent.mouseDown(screen.getByRole("tab", { name: /審查/ }));
    await screen.findByText("工單尚未抵達或已被刪除");
    first.unmount();

    render(
      <ArchivedDetailPage
        {...(makeProps({
          verifyStatus: "verifiedNotPassed",
          loadStationTicket: vi.fn(async () => Promise.reject(new Error("offline"))),
        }) as never)}
      />,
    );
    await waitFor(() => screen.getByRole("tab", { name: /驗證/ }));
    fireEvent.mouseDown(screen.getByRole("tab", { name: /驗證/ }));
    await screen.findByText("工單尚未抵達或已被刪除");
  });
});

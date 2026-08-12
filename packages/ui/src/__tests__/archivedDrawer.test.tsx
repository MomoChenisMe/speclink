// 已封存項目以抽屜檢視（spec「已封存項目以抽屜檢視」；design D1：discriminated
// target 兩型同檔）：封存變更四分頁唯讀（任務核取方塊 disabled、無批次工具列）、
// 封存討論「背景」「討論過程」「結論」區段、缺件文件空狀態、無任何寫入動詞、
// 寬度與全螢幕切換與變更詳情抽屜一致、開啟期間外部變更反映。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, waitFor, fireEvent, within } from "@testing-library/react";
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
    // 動詞名精確比對——標頭的「複製封存名稱」鈕不是寫入動詞，不得被寬鬆正則誤判。
    for (const name of ["封存", "刪除", "分析", "轉為變更"]) {
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

  it("sourceDiscussions 以 slug 籤呈現，首籤直出、其餘收 +N 浮層，點擊以 slug 呼叫 onOpenDiscussion", async () => {
    // spec Scenario「自封存變更抽屜跳轉來源討論」的元件面（change-drawer-header-redesign 同構改寫）。
    const props = makeProps({
      sourceDiscussions: [
        { slug: "alpha-ux", topic: "Alpha UX 討論" },
        { slug: "beta-flow", topic: "beta-flow" },
      ],
      onOpenDiscussion: vi.fn(),
    });
    render(<ArchivedDrawer {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    // 首籤（出身）slug 直出；topic 全文不在可視文字。
    fireEvent.click(screen.getByRole("button", { name: /alpha-ux/ }));
    expect(props.onOpenDiscussion).toHaveBeenCalledWith("alpha-ux");
    expect(screen.queryByText("Alpha UX 討論")).toBeNull();
    // 第二筆收進 +1 浮層，不直接渲染。
    expect(screen.queryByRole("button", { name: /beta-flow/ })).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: /其餘 1 份/ }));
    const popover = await waitFor(() => {
      const el = document.querySelector("[data-source-overflow-list]") as HTMLElement | null;
      expect(el).toBeTruthy();
      return el!;
    });
    fireEvent.click(within(popover).getByRole("button", { name: /beta-flow/ }));
    expect(props.onOpenDiscussion).toHaveBeenCalledWith("beta-flow");
  });

  // spec 需求「抽屜標頭標記受寬度約束且抽屜不產生水平捲軸」：同一約束須於已封存抽屜成立
  // （change-drawer-header-redesign 改寫：籤面 slug、寬度上限、topic 降提示）。
  it("籤面為 slug（等寬、寬度上限、截斷），面板關閉水平捲動", async () => {
    const LONG_TOPIC =
      "目前前端專案的設計，是完全純文字的web服務，我希望可以設計一下，因為有好幾個端點都要自己打URL才能夠進去，完全不符合人性，所以請你幫我重新設計一下";
    const props = makeProps({
      sourceDiscussions: [{ slug: "web-service-navigation-redesign", topic: LONG_TOPIC }],
      onOpenDiscussion: vi.fn(),
    });
    render(<ArchivedDrawer {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    const chip = screen.getByRole("button", { name: /web-service-navigation-redesign/ });
    expect(chip.className).toContain("font-mono");
    expect(chip.className).toContain("max-w-[140px]");
    expect(screen.queryByText(LONG_TOPIC)).toBeNull();
    const label = chip.querySelector("[data-source-discussion-label]") as HTMLElement | null;
    expect(label).toBeTruthy();
    expect(label!.className).toContain("truncate");
    expect(label!.textContent).toBe("web-service-navigation-redesign");
    const panel = drawerEl();
    expect(panel!.className).toContain("overflow-y-auto");
    expect(panel!.className).toContain("overflow-x-hidden");
  });

  it("sourceDiscussions 缺席或空陣列時來源討論區塊不渲染", async () => {
    const { unmount } = render(<ArchivedDrawer {...(makeProps() as never)} />);
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    expect(document.querySelector("[data-source-discussion-label]")).toBeNull();
    unmount();
    render(<ArchivedDrawer {...(makeProps({ sourceDiscussions: [] }) as never)} />);
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    expect(document.querySelector("[data-source-discussion-label]")).toBeNull();
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
    for (const name of ["轉為變更", "封存", "刪除"]) {
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
    expect(document.querySelector("[data-source-discussion-label]")).toBeNull();
  });
});

// spec 需求「已封存項目以抽屜檢視」的標頭：標題後緊跟複製鈕（封存變更複製
// datedName、封存討論複製 slug）＋標題下方出身列（建立者／建立日期／封存日期），
// 無進度條與動詞動作列——封存是唯讀定格。
describe("ArchivedDrawer（標頭複製鈕與出身列）", () => {
  const clipboard = () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", { value: { writeText }, configurable: true });
    return writeText;
  };

  it("封存變更標頭的複製鈕寫入含日期前綴的封存目錄名並顯示已複製回饋", async () => {
    const writeText = clipboard();
    render(<ArchivedDrawer {...(makeProps() as never)} />);
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    fireEvent.click(screen.getByLabelText("複製封存名稱"));
    expect(writeText).toHaveBeenCalledWith("2026-07-04-old-change");
    await waitFor(() => expect(screen.queryByLabelText("已複製")).toBeTruthy());
  });

  it("封存討論標頭的複製鈕寫入 slug", async () => {
    const writeText = clipboard();
    render(
      <ArchivedDrawer
        {...(makeProps({ target: { kind: "discussion", slug: "alpha-search" } }) as never)}
      />,
    );
    await waitFor(() => expect(screen.getByText("討論背景內文。")).toBeTruthy());
    fireEvent.click(screen.getByLabelText("複製 slug"));
    expect(writeText).toHaveBeenCalledWith("alpha-search");
  });

  it("出身列顯示建立者首字母圓標＋名字、建立日期與封存日期；無進度條與動詞動作列", async () => {
    // spec Example「封存變更抽屜標頭的複製鈕與出身列」。
    render(
      <ArchivedDrawer
        {...(makeProps({
          createdBy: "MomoChen <momo@example.com>",
          created: "2026-06-20",
          archivedDate: "2026-07-04",
        }) as never)}
      />,
    );
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    const row = document.querySelector("[data-provenance-row]") as HTMLElement;
    expect(row).toBeTruthy();
    // 恆定單行溢出裁切（與變更詳情抽屜出身列同構）。
    expect(row.className).toContain("whitespace-nowrap");
    expect(row.className).toContain("overflow-hidden");
    // 建立者：首字母圓標＋名字，完整識別收 tooltip。
    const avatar = within(row).getByText("M");
    expect(avatar.className).toContain("rounded-full");
    expect(within(row).getByText("MomoChen")).toBeTruthy();
    expect(row.textContent).toContain("2026-06-20 建立");
    expect(row.textContent).toContain("2026-07-04 封存");
    // 封存是唯讀定格：無進度條、無動詞動作列。
    expect(document.querySelector("[data-progress-track]")).toBeNull();
    expect(document.querySelector("[data-status-row]")).toBeNull();
  });

  it("出身列欄位缺席時該欄缺席、其餘照常（建立日期不可得）", async () => {
    // spec Example「出身列欄位缺席時其餘照常」。
    render(
      <ArchivedDrawer
        {...(makeProps({
          createdBy: "MomoChen <momo@example.com>",
          archivedDate: "2026-07-04",
        }) as never)}
      />,
    );
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    const row = document.querySelector("[data-provenance-row]") as HTMLElement;
    expect(row).toBeTruthy();
    expect(within(row).getByText("MomoChen")).toBeTruthy();
    expect(row.textContent).toContain("2026-07-04 封存");
    expect(row.textContent).not.toContain("建立");
  });
});

// spec 需求「markdown 文件內容行寬有上限」（design D4）：封存抽屜兩型 target 的
// 捲動容器內皆存在共用置中容器，分頁內容／討論區段與內文同欄對齊置中。
describe("ArchivedDrawer（閱讀欄置中）", () => {
  it("封存變更 target：捲動容器內有置中容器且包住分頁內容（含任務清單）", async () => {
    render(<ArchivedDrawer {...(makeProps() as never)} />);
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    const col = document.querySelector("[data-reading-column]") as HTMLElement;
    expect(col).toBeTruthy();
    expect(col.className).toContain("w-full");
    expect(col.className).toContain("max-w-[96ch]");
    expect(col.className).toContain("mx-auto");
    expect(col.parentElement?.className).toContain("overflow-y-auto");
    expect(col.textContent).toContain("封存提案內文。");
    // 任務分頁：唯讀任務清單同欄。
    fireEvent.mouseDown(screen.getByRole("tab", { name: /任務/ }));
    expect(col.textContent).toContain("open-task");
  });

  it("封存討論 target：捲動容器內有置中容器且包住三區段", async () => {
    render(
      <ArchivedDrawer
        {...(makeProps({ target: { kind: "discussion", slug: "alpha-search" } }) as never)}
      />,
    );
    await waitFor(() => expect(screen.getByText("討論背景內文。")).toBeTruthy());
    const col = document.querySelector("[data-reading-column]") as HTMLElement;
    expect(col).toBeTruthy();
    expect(col.className).toContain("max-w-[96ch]");
    expect(col.className).toContain("mx-auto");
    expect(col.parentElement?.className).toContain("overflow-y-auto");
    expect(col.textContent).toContain("討論背景內文。");
    expect(col.textContent).toContain("就這麼辦");
  });
});

// spec「抽屜文件載入以 skeleton 呈現」（design D3）：已封存抽屜各分頁載入中畫骨架。
describe("已封存抽屜文件三態", () => {
  const pending = () => new Promise<never>(() => {});

  it("封存變更載入中 → 骨架，且不出「無設計文件」等假空態", async () => {
    render(
      <ArchivedDrawer
        {...(makeProps({ loadDocument: vi.fn(pending), loadCapabilities: vi.fn(pending) }) as never)}
      />,
    );
    await waitFor(() => expect(document.querySelector('[aria-busy="true"]')).toBeTruthy());
    expect(screen.queryByText("（此變更無設計文件）")).toBeNull();
  });

  it("封存討論載入中 → 骨架，不出空態文案", async () => {
    render(
      <ArchivedDrawer
        {...(makeProps({
          target: { kind: "discussion", slug: "some-topic" },
          loadDiscussionDocument: vi.fn(pending),
        }) as never)}
      />,
    );
    await waitFor(() => expect(document.querySelector('[aria-busy="true"]')).toBeTruthy());
    expect(screen.queryByText("（無內容）")).toBeNull();
  });

  it("載入完成 → 內容照常，無骨架", async () => {
    render(<ArchivedDrawer {...(makeProps() as never)} />);
    await waitFor(() => expect(screen.getByText("封存提案內文。")).toBeTruthy());
    expect(document.querySelector('[aria-busy="true"]')).toBeNull();
  });

  it("封存變更文件首載失敗 → 空態文案，不停在骨架", async () => {
    render(
      <ArchivedDrawer
        {...(makeProps({ loadDocument: vi.fn().mockRejectedValue(new Error("offline")) }) as never)}
      />,
    );
    await waitFor(() => expect(screen.getByText("（無提案文件）")).toBeTruthy());
    expect(document.querySelector('[aria-busy="true"]')).toBeNull();
  });

  it("封存討論首載失敗 → 空態文案，不停在骨架", async () => {
    render(
      <ArchivedDrawer
        {...(makeProps({
          target: { kind: "discussion", slug: "some-topic" },
          loadDiscussionDocument: vi.fn().mockRejectedValue(new Error("offline")),
        }) as never)}
      />,
    );
    await waitFor(() => expect(screen.getByText("（無內容）")).toBeTruthy());
    expect(document.querySelector('[aria-busy="true"]')).toBeNull();
  });
});

// 三態的 undefined 是「還在載」，不是「載不到」。capability 載入失敗時若停在
// undefined，規格分頁會永久掛骨架（與 RichDetailDrawer 同一收斂）。
describe("已封存抽屜規格分頁載入失敗的收斂", () => {
  it("loadCapabilities 失敗 → 收斂為空態文案，不永久掛骨架", async () => {
    render(
      <ArchivedDrawer
        {...(makeProps({ loadCapabilities: vi.fn(async () => Promise.reject("讀不到")) }) as never)}
      />,
    );
    await waitFor(() => screen.getByRole("tab", { name: /規格/ }));
    fireEvent.mouseDown(screen.getByRole("tab", { name: /規格/ }));
    await waitFor(() => expect(screen.getByText("（此變更無 delta 規格）")).toBeTruthy());
    expect(document.querySelector('[aria-busy="true"]')).toBeNull();
  });
});

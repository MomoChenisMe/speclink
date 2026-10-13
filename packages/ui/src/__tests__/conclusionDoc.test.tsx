// spec「討論結論以欄位標籤呈現」的結論渲染器（desktop-discussion-doc-views design D2）。
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render as rtlRender, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { ConclusionDoc } from "../components/ConclusionDoc";

const wrapper = ({ children }: { children: ReactNode }) => <I18nProvider locale="zh-TW">{children}</I18nProvider>;
const render = (ui: ReactElement) => rtlRender(ui, { wrapper });

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const section = (container: HTMLElement, key: string) =>
  container.querySelector(`[data-section="${key}"]`) as HTMLElement;
const countOf = (el: HTMLElement) => el.querySelector("h2")!.parentElement!.querySelector(".tabular-nums")?.textContent;

// spec Scenario「決定主句放大與細節收合」：一句定論＋27 條頂層列點（其中 7 條為分刀列）。
const MANY = [
  "**Decision**: 一句定論。",
  // 每 3 條細節後插一條分刀列，最後一刀接在尾端：20 條細節＋7 條分刀。
  ...Array.from({ length: 20 }, (_, i) => [
    `- 細節 ${i + 1}`,
    ...(i % 3 === 2 || i === 19 ? [`- **cut ${Math.floor(i / 3)} \`chg-${i}\`**: 範圍 ${i}`] : []),
  ]).flat(),
].join("\n");

// spec Scenario「分刀卡與變更籤」：detail-content-redesign 的三條分刀列，前兩刀已封存、第三刀未轉出。
const CUTS = [
  "**Decision**: 分三刀落地。",
  "- 範圍與驗收：只改 UI 渲染",
  "- **cut 1 `desktop-detail-doc-tree`**: 骨架——三種詳情頁改三欄",
  "  - 文件樹欄",
  "  - 分頁列退場",
  "- **cut 2 `desktop-change-doc-views`**: 變更文件渲染器——提案、設計的新畫法",
  "  - 章節白名單",
  "- **cut 3 `desktop-discussion-doc-views`**: 討論文件渲染器——結論分刀卡",
  "**Rationale**: 理由一段。",
  "**Rejected alternatives**:",
  ...Array.from({ length: 14 }, (_, i) => `- 方案 ${i} — 理由 ${i}`),
  "**Deferred**:",
  "- 擱置甲 — 等第二個消費者",
  "- 擱置乙——不在範圍",
  "- 擱置丙沒有理由",
  "- 擱置丁 — 理由丁",
  "- 擱置戊 — 理由戊",
  "**Capture to**: proposal、design、spec（desktop-app MODIFIED）、tasks",
  "**Next**: /speclink-propose --from-discussion detail-content-redesign（先建 cut 1）",
].join("\n");

describe("ConclusionDoc", () => {
  it("27 條細節含 7 條分刀：細線列 4 條、鈕文字「展開其餘 16 項定案細節」、點後 20 條、計數 20", () => {
    const { container } = render(<ConclusionDoc text={MANY} />);
    const decision = section(container, "conclusion.decision");
    expect(decision.querySelector("[data-verdict]")!.textContent).toBe("一句定論。");
    const rows = () => decision.querySelectorAll("[data-row-list] > li");
    expect(rows().length).toBe(4);
    expect(countOf(decision)).toBe("20");
    fireEvent.click(within(decision).getByRole("button", { name: "展開其餘 16 項定案細節" }));
    expect(rows().length).toBe(20);
    expect(within(decision).queryByRole("button", { name: /展開其餘/ })).toBeNull();
    expect(container.querySelectorAll("[data-cut]").length).toBe(7);
  });

  it("三張分刀卡的識別籤與範圍句、前兩張變更籤帶階段籤且點擊呼叫 stageOf 回的 onOpen、第三張不可點", () => {
    const opened = vi.fn();
    const stageOf = (name: string) =>
      name === "desktop-detail-doc-tree" || name === "desktop-change-doc-views"
        ? { label: "已封存", tone: "neutral" as const, onOpen: () => opened(name) }
        : null;
    const { container } = render(<ConclusionDoc text={CUTS} stageOf={stageOf} />);
    const cuts = section(container, "conclusion.cuts");
    expect(countOf(cuts)).toBe("3");
    const cards = Array.from(cuts.querySelectorAll<HTMLElement>("[data-cut]"));
    expect(cards.map((c) => c.getAttribute("data-cut"))).toEqual(["cut 1", "cut 2", "cut 3"]);
    const header = cards[0].querySelector("header")!;
    expect(header.textContent).toContain("cut 1");
    expect(header.textContent).toContain("骨架");
    expect(header.textContent).toContain("1 個變更");
    expect(header.textContent).not.toContain("三種詳情頁");
    // 「——」之後的文字成為卡內第一條細線列，其後是巢狀子項。
    expect(Array.from(cards[0].querySelectorAll("[data-row-list] > li")).map((li) => li.textContent)).toEqual([
      "三種詳情頁改三欄",
      "文件樹欄",
      "分頁列退場",
    ]);
    expect(within(cards[0]).getByText("已封存")).toBeTruthy();
    fireEvent.click(within(cards[1]).getByRole("button", { name: "desktop-change-doc-views" }));
    expect(opened).toHaveBeenCalledWith("desktop-change-doc-views");
    expect(within(cards[2]).queryByRole("button", { name: "desktop-discussion-doc-views" })).toBeNull();
    expect(within(cards[2]).getByText("desktop-discussion-doc-views")).toBeTruthy();
    expect(within(cards[2]).queryByText("已封存")).toBeNull();
    // 第三刀整刀未轉出：橄欖色虛線框、卡頭「尚未轉出」籤；前兩刀照舊。
    expect(cards[2].hasAttribute("data-cut-todo")).toBe(true);
    expect(cards[2].className).toContain("border-dashed");
    expect(cards[2].className).toContain("border-status-manual/40");
    expect(cards[2].querySelector("header")!.textContent).toContain("尚未轉出");
    for (const card of cards.slice(0, 2)) {
      expect(card.hasAttribute("data-cut-todo")).toBe(false);
      expect(card.textContent).not.toContain("尚未轉出");
    }
    // 分刀列不再出現在定案細節。
    expect(section(container, "conclusion.decision").querySelectorAll("[data-row-list] > li").length).toBe(1);
  });

  it("部分轉出：卡無 data-cut-todo，只有未轉出的變更名籤之後有「尚未轉出」", () => {
    const stageOf = (name: string) => (name === "a-one" ? { label: "進行中", tone: "progress" as const } : null);
    const { container } = render(
      <ConclusionDoc text={"**Decision**: 定。\n- **cut 2 `a-one` `b-two`**: 外殼"} stageOf={stageOf} />,
    );
    const card = container.querySelector<HTMLElement>("[data-cut]")!;
    expect(card.hasAttribute("data-cut-todo")).toBe(false);
    expect(card.className).not.toContain("border-dashed");
    expect(card.querySelector("header")!.textContent).not.toContain("尚未轉出");
    const promoted = card.querySelector<HTMLElement>('[data-cut-change="a-one"]')!;
    expect(promoted.textContent).toBe("a-one進行中");
    const todo = card.querySelector<HTMLElement>('[data-cut-change="b-two"]')!;
    expect(todo.textContent).toBe("b-two尚未轉出");
    expect(todo.lastElementChild!.className).toContain("text-status-manual");
  });

  it("刀列沒有變更名籤時卡無 data-cut-todo、無「尚未轉出」", () => {
    const { container } = render(<ConclusionDoc text={"**Decision**: 定。\n- **刀三**：系統匣面板"} stageOf={() => null} />);
    const card = container.querySelector<HTMLElement>("[data-cut]")!;
    expect(card.getAttribute("data-cut")).toBe("刀三");
    expect(card.hasAttribute("data-cut-todo")).toBe(false);
    expect(card.textContent).not.toContain("尚未轉出");
  });

  it("jump 請求把該刀卡捲到畫面中央並呼叫 onJumped", () => {
    const targets: Element[] = [];
    const scrollSpy = vi.spyOn(Element.prototype, "scrollIntoView").mockImplementation(function (this: Element) {
      targets.push(this);
    });
    try {
      const onJumped = vi.fn();
      const { container } = render(<ConclusionDoc text={CUTS} jump={{ id: "cut 2", seq: 1 }} onJumped={onJumped} />);
      expect(targets).toEqual([container.querySelector('[data-cut="cut 2"]')]);
      expect(scrollSpy).toHaveBeenCalledWith(expect.objectContaining({ block: "center" }));
      expect(onJumped).toHaveBeenCalledTimes(1);
    } finally {
      scrollSpy.mockRestore();
    }
  });

  it("jump 的目標刀卡不存在時不捲動、照樣呼叫 onJumped", () => {
    const scrollSpy = vi.spyOn(Element.prototype, "scrollIntoView").mockImplementation(() => {});
    try {
      const onJumped = vi.fn();
      render(<ConclusionDoc text={CUTS} jump={{ id: "cut 9", seq: 1 }} onJumped={onJumped} />);
      expect(scrollSpy).not.toHaveBeenCalled();
      expect(onJumped).toHaveBeenCalledTimes(1);
    } finally {
      scrollSpy.mockRestore();
    }
  });

  it("否決 14 列方案／理由、擱置其中一列無理由行且每列有時鐘圖示", () => {
    const { container } = render(<ConclusionDoc text={CUTS} />);
    const rejected = section(container, "conclusion.rejected");
    expect(countOf(rejected)).toBe("14");
    const rejectedRows = rejected.querySelectorAll("[data-alternative]");
    expect(rejectedRows.length).toBe(14);
    expect(rejectedRows[0].querySelector("[data-alternative-option]")!.textContent).toBe("方案 0");
    expect(rejectedRows[0].querySelector("[data-alternative-reason]")!.textContent).toBe("理由 0");
    const deferred = section(container, "conclusion.deferred");
    expect(countOf(deferred)).toBe("5");
    const deferredRows = Array.from(deferred.querySelectorAll("[data-alternative]"));
    expect(deferredRows.map((r) => r.querySelector("[data-alternative-reason]")?.textContent ?? null)).toEqual([
      "等第二個消費者",
      "不在範圍",
      null,
      "理由丁",
      "理由戊",
    ]);
    for (const row of deferredRows) expect(row.querySelector("svg.lucide-clock")).toBeTruthy();
    expect(rejectedRows[0].querySelector("svg.lucide-clock")).toBeNull();
  });

  it("記錄去向 desktop-app 籤呼叫 onOpenSpec、proposal 籤不可點；下一步指令籤複製後剪貼簿為指令原文", async () => {
    const writeText = vi.fn<(v: string) => Promise<void>>().mockResolvedValue();
    vi.stubGlobal("navigator", { clipboard: { writeText } });
    const onOpenSpec = vi.fn();
    const { container } = render(<ConclusionDoc text={CUTS} specNames={["desktop-app"]} onOpenSpec={onOpenSpec} />);
    const capture = section(container, "conclusion.captureTo");
    expect(Array.from(capture.querySelectorAll("[data-capture-chip]")).map((c) => c.textContent)).toEqual([
      "proposal",
      "design",
      "spec（desktop-app MODIFIED）",
      "tasks",
    ]);
    fireEvent.click(within(capture).getByRole("button", { name: "desktop-app" }));
    expect(onOpenSpec).toHaveBeenCalledWith("desktop-app");
    expect(within(capture).queryByRole("button", { name: "proposal" })).toBeNull();
    const next = section(container, "conclusion.next");
    const chips = next.querySelectorAll("[data-command-chip]");
    expect(chips.length).toBe(1);
    expect(chips[0].textContent).toContain("/speclink-propose --from-discussion detail-content-redesign");
    fireEvent.click(within(chips[0] as HTMLElement).getByRole("button", { name: "複製指令" }));
    await act(async () => {});
    expect(writeText).toHaveBeenCalledWith("/speclink-propose --from-discussion detail-content-redesign");
  });

  it("記錄去向的長句籤可折行、不撐出卡片；整顆規格籤維持不換行", () => {
    const long = "design 的 D3 補一段說明背景文件名稱籤的鍵盤操作與焦點順序";
    const { container } = render(
      <ConclusionDoc text={`**Decision**: 定\n**Capture to**: desktop-app、${long}`} specNames={["desktop-app"]} onOpenSpec={() => {}} />,
    );
    const chips = Array.from(section(container, "conclusion.captureTo").querySelectorAll<HTMLElement>("[data-capture-chip]"));
    expect(chips.map((c) => c.textContent)).toEqual(["desktop-app", long]);
    expect(chips[1].className).not.toContain("shrink-0");
    expect(chips[1].className).toContain("break-words");
    expect(chips[1].className).toContain("max-w-full");
  });

  it("[data-doc-heading] 依序為決定、分刀、理由、否決替代案、擱置、記錄去向、下一步", () => {
    const { container } = render(<ConclusionDoc text={CUTS} />);
    expect(Array.from(container.querySelectorAll("[data-doc-heading]")).map((h) => h.textContent)).toEqual([
      "決定",
      "分刀",
      "理由",
      "否決替代案",
      "擱置",
      "記錄去向",
      "下一步",
    ]);
  });

  it("缺席欄位不渲染空區段；記錄去向切不出籤時照 markdown", () => {
    const { container } = render(<ConclusionDoc text={"**Decision**: 只有決定\n**Capture to**: 寫進設計文件。"} />);
    expect(section(container, "conclusion.decision")).toBeTruthy();
    expect(section(container, "conclusion.deferred")).toBeNull();
    expect(section(container, "conclusion.cuts")).toBeNull();
    const capture = section(container, "conclusion.captureTo");
    expect(capture.querySelector("[data-capture-chip]")).toBeNull();
    expect(capture.textContent).toContain("寫進設計文件。");
  });

  it("自由格式整篇 Markdown、無 data-section", () => {
    const { container } = render(<ConclusionDoc text="就這樣定了，大家都同意這個方向。" />);
    expect(container.querySelector("[data-section]")).toBeNull();
    expect(container.textContent).toContain("就這樣定了");
  });
});

import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, waitFor, fireEvent, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider, MESSAGES, useI18n } from "../i18n";

// 既有中文斷言包 I18nProvider locale zh-TW 後照舊斷言（i18n 抽 key 的回歸保護）。
const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

import { DiscussionDetailPage } from "../components/DiscussionDetailPage";
import type { DocumentTarget } from "../components/DetailHeader";
import { SECTION_HEADING_CLS } from "../components/DocParts";
import { ChangeCard } from "../components/ChangeCard";
import { ChangeDetailPage } from "../components/ChangeDetailPage";
import type { ChangeItem, ArchivedItem, DiscussionItem } from "../adapter";
import { DocHeader, DocTiles } from "../components/DetailShell";
import {
  discussionDocData,
  discussionDocProvenance,
  discussionDocSummary,
  type DiscussionDocData,
  type DiscussionTab,
} from "../components/discussionDocSummary";
import { promotedRows, type PromotedRow } from "../components/PromotedDoc";

// spec 需求「討論詳情頁檢視與轉出變更」的 jsdom 可驗部分。

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
    discussion: concludedD,
    loadDocument: vi.fn(async () => DOC),
    changes,
    archivedChanges,
    onOpenChangeCard: vi.fn(),
    ...over,
  };
}

// spec 需求「討論輪以卡片呈現」的輪切分（design D1/D2）。
const ROUNDS_TEXT = `
<!-- \`### Round N — <mode> (<date>)\` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-08)

**Focus**: 第一輪焦點
**Position**: 總綰一句：
- 列點甲
- 列點乙
**Open**: 未解之一

### Round 2 — interview (2026-07-09)

**Focus**: 第二輪焦點
**Position**: 直答
**Note**: 這行不是欄位
**Ruled out**: 淘汰項
**Open**: 無
`;

// spec 需求「討論輪以卡片呈現」的渲染面（design D1/D2 輪卡片＋欄位標籤區塊）。
const CARDS_DOC = `---
topic: Alpha search
slug: alpha-search
status: concluded
created: 2026-07-01
---

# Discussion: Alpha search

## Context

框架脈絡內容。

## Rounds
${ROUNDS_TEXT}
## Conclusion

**Decision**: 建置 alpha 搜尋
`;

describe("輪卡片渲染（討論輪以卡片呈現）", () => {
  async function openRoundsTab(doc: string) {
    const props = makeProps({ loadDocument: vi.fn(async () => doc) });
    const result = render(<DiscussionDetailPage {...(props as never)} />);
    await waitFor(() => screen.getByRole("tab", { name: /討論過程/ }));
    fireEvent.mouseDown(screen.getByRole("tab", { name: /討論過程/ }));
    return result;
  }

  it("scaffold 記錄逐輪成卡：卡頭含 Round N、mode、日期", async () => {
    const { baseElement } = await openRoundsTab(CARDS_DOC);
    await waitFor(() =>
      expect(baseElement.querySelectorAll("[data-round]").length).toBe(2),
    );
    const card1 = baseElement.querySelector('[data-round="1"]') as HTMLElement;
    const roundChip = within(card1).getByText("Round 1");
    expect(roundChip).toBeTruthy();
    expect(within(card1).getByText("assumptions")).toBeTruthy();
    expect(within(card1).getByText("2026-07-08")).toBeTruthy();
    // 輪次籤是靜態編號，與同列 mode／日期同層級：走中性，不佔用主色。
    expect(roundChip.className).toContain("bg-muted");
    expect(roundChip.className).not.toContain("primary");
  });

  it("卡頭含焦點、欄位以標籤呈現，「**Focus**:」粗體前綴原文不出現", async () => {
    const { baseElement } = await openRoundsTab(CARDS_DOC);
    await waitFor(() => screen.getByText("第一輪焦點"));
    const card1 = baseElement.querySelector('[data-round="1"]') as HTMLElement;
    expect(card1.querySelector("button[aria-expanded]")!.textContent).toContain("第一輪焦點");
    expect(within(card1).getByText("立場")).toBeTruthy();
    expect(within(card1).getByText("未解")).toBeTruthy();
    // 舊行為把 **Focus**: 渲染成粗體 Focus 文字；卡片模式下英文前綴不再出現。
    expect(screen.queryByText("Focus")).toBeNull();
    expect(screen.queryByText("Position")).toBeNull();
  });

  it("缺席欄位不渲染空標籤", async () => {
    const { baseElement } = await openRoundsTab(CARDS_DOC);
    await waitFor(() => screen.getByText("第一輪焦點"));
    const card1 = baseElement.querySelector('[data-round="1"]') as HTMLElement;
    const card2 = baseElement.querySelector('[data-round="2"]') as HTMLElement;
    expect(within(card1).queryByText("淘汰")).toBeNull();
    expect(within(card2).getByText("淘汰")).toBeTruthy();
  });

  it("非標準輪標題整篇以單一 markdown 檢視退回", async () => {
    const bad = CARDS_DOC.replace("### Round 2 — interview (2026-07-09)", "### 插入的手寫標題");
    const { baseElement } = await openRoundsTab(bad);
    await waitFor(() => screen.getByText("插入的手寫標題"));
    expect(baseElement.querySelectorAll("[data-round]").length).toBe(0);
  });

  // spec「詳情頁的文件樹、閱讀卡與本頁目錄」的「討論文件的本頁目錄錨點」：本頁目錄自 DOM 推導，跟著輪卡由新到舊。
  it("本頁目錄的輪次錨點與輪卡同序：Round 2 在前、Round 1 在後", async () => {
    const { baseElement } = await openRoundsTab(CARDS_DOC);
    const toc = () => baseElement.querySelector("[data-detail-toc]") as HTMLElement;
    await waitFor(() =>
      expect(Array.from(toc().querySelectorAll("[data-manual-anchor]")).map((a) => a.textContent)).toEqual([
        "Round 2",
        "Round 1",
      ]),
    );
    expect(baseElement.querySelector("[data-round-timeline] [data-round]")!.getAttribute("data-round")).toBe("2");
  });
});

// spec 需求「討論結論以欄位標籤呈現」（design D7 六詞白名單共用欄位解析）。
function docWithConclusion(conclusion: string): string {
  return DOC.replace("**Decision**: 建置 alpha 搜尋", conclusion);
}

// spec Scenario「未轉出刀的提示框與前往」：cut 1 已轉出、cut 2 尚未轉出。
const WAD_DECISION = [
  "**Decision**: 分兩刀。",
  "- **cut 1 `add-team-plugin-library`**: 團隊外掛庫",
  "- **cut 2 `remove-builtin-wad-work`**: 移除內建 wad-work",
].join("\n");

const FULL_CONCLUSION = [
  "**Decision**: 拍板做 A",
  "**Rationale**: 因為證據充分",
  "**Rejected alternatives**: B 案——太貴",
  "**Deferred**: 無",
  "**Capture to**: proposal",
  "**Next**: /speclink-propose --from-discussion alpha-search",
].join("\n");

describe("結論欄位標籤化（討論結論以欄位標籤呈現）", () => {
  // 欄位標籤同時是本頁目錄的錨點：斷言限定在閱讀卡內文。
  const body = () => within(document.querySelector("[data-doc-body]") as HTMLElement);
  it("scaffold 結論六欄位成標籤區塊，粗體前綴原文不出現", async () => {
    const props = makeProps({ loadDocument: vi.fn(async () => docWithConclusion(FULL_CONCLUSION)) });
    render(<DiscussionDetailPage {...(props as never)} />);
    // 結論非空 → 預設分頁即結論。
    await waitFor(() => expect(screen.getByText("拍板做 A")).toBeTruthy());
    for (const label of ["決定", "理由", "否決替代案", "擱置", "記錄去向", "下一步"]) {
      expect(body().getByText(label)).toBeTruthy();
    }
    expect(screen.queryByText("Decision")).toBeNull();
    expect(screen.queryByText("Rationale")).toBeNull();
  });

  it("缺席欄位不渲染空標籤", async () => {
    const doc = docWithConclusion("**Decision**: 只有決定\n**Rationale**: 與理由");
    const props = makeProps({ loadDocument: vi.fn(async () => doc) });
    render(<DiscussionDetailPage {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("只有決定")).toBeTruthy());
    expect(body().getByText("決定")).toBeTruthy();
    expect(body().getByText("理由")).toBeTruthy();
    expect(screen.queryByText("擱置")).toBeNull();
    expect(screen.queryByText("下一步")).toBeNull();
  });

  // 段落標籤從 20px 粗體大標改為 16px 章節標題（detail-content-redesign 結論）。
  it("結論欄位的區段標題為 16px 粗體章節標題", async () => {
    const props = makeProps({ loadDocument: vi.fn(async () => docWithConclusion(FULL_CONCLUSION)) });
    render(<DiscussionDetailPage {...(props as never)} />);
    await waitFor(() => expect(body().getByText("決定")).toBeTruthy());
    const conclusionLabel = body().getByText("決定");
    for (const cls of SECTION_HEADING_CLS.split(" ")) expect(conclusionLabel.className).toContain(cls);
    expect(conclusionLabel.className).not.toContain("text-xl");
  });

  it("結論的分刀卡變更籤點擊開啟已封存詳情頁（onOpenArchivedChange 收到 datedName）", async () => {
    const onOpenArchivedChange = vi.fn();
    const props = makeProps({
      discussion: { ...concludedD, status: "promoted", promotedTo: ["cut-old"] },
      archivedChanges: [{ datedName: "2026-07-20-cut-old", date: "2026-07-20", name: "cut-old" }],
      onOpenArchivedChange,
      loadDocument: vi.fn(async () => docWithConclusion("**Decision**: 一刀\n- **cut 1 `cut-old`**: 骨架")),
    });
    render(<DiscussionDetailPage {...(props as never)} />);
    const card = await waitFor(() => {
      const el = document.querySelector('[data-cut="cut 1"]') as HTMLElement | null;
      expect(el).toBeTruthy();
      return el!;
    });
    expect(within(card).getByText("已封存")).toBeTruthy();
    fireEvent.click(within(card).getByRole("button", { name: "cut-old" }));
    expect(onOpenArchivedChange).toHaveBeenCalledWith("2026-07-20-cut-old");
  });

  it("點提示框的「前往」後 cut 2 的刀卡被捲動", async () => {
    const targets: Element[] = [];
    const scrollSpy = vi.spyOn(Element.prototype, "scrollIntoView").mockImplementation(function (this: Element) {
      targets.push(this);
    });
    try {
      const props = makeProps({
        discussion: {
          ...concludedD,
          slug: "wad-plugins-merge-or-split",
          status: "promoted",
          concluded: true,
          hold: true,
          promotedTo: ["add-team-plugin-library"],
        },
        changes: [{ name: "add-team-plugin-library", status: "in-progress", totalTasks: 3, completedTasks: 1 }],
        loadDocument: vi.fn(async () => docWithConclusion(WAD_DECISION)),
      });
      render(<DiscussionDetailPage {...(props as never)} />);
      const row = await waitFor(() => {
        const el = document.querySelector("[data-doc-callout] [data-unpromoted-cut]") as HTMLElement | null;
        expect(el).toBeTruthy();
        return el!;
      });
      targets.length = 0;
      scrollSpy.mockClear();
      fireEvent.click(within(row).getByRole("button", { name: /前往/ }));
      await waitFor(() => expect(targets).toEqual([document.querySelector('[data-cut="cut 2"]')]));
      expect(scrollSpy).toHaveBeenCalledWith(expect.objectContaining({ block: "center" }));
    } finally {
      scrollSpy.mockRestore();
    }
  });

  it("自由格式結論整篇以單一 markdown 檢視退回", async () => {
    const props = makeProps({
      loadDocument: vi.fn(async () => docWithConclusion("就這樣定了，大家都同意這個方向。")),
    });
    render(<DiscussionDetailPage {...(props as never)} />);
    await waitFor(() => expect(screen.getByText(/就這樣定了/)).toBeTruthy());
    expect(screen.queryByText("決定")).toBeNull();
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

describe("DiscussionDetailPage", () => {
  it("分頁依序為 結論/討論過程 N/背景/衍生變更，結論非空時預設呈現結論", async () => {
    render(<DiscussionDetailPage {...(makeProps() as never)} />);
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

  it("建立者圓標為中性，不佔用主色（五處頭像同款）", async () => {
    const withAuthor = { ...concludedD, createdBy: "Momo Chen" };
    render(<DiscussionDetailPage {...(makeProps({ discussion: withAuthor }) as never)} />);
    const avatar = await waitFor(() => {
      const el = Array.from(document.querySelectorAll("span")).find(
        (s) => s.textContent === "M" && s.className.includes("rounded-full"),
      );
      expect(el).toBeTruthy();
      return el as HTMLElement;
    });
    expect(avatar.className).toContain("bg-muted");
    expect(avatar.className).not.toContain("bg-primary");
  });

  it("結論為空（僅鷹架註解）時預設呈現背景", async () => {
    const props = makeProps({ discussion: openD, loadDocument: vi.fn(async () => OPEN_DOC) });
    render(<DiscussionDetailPage {...(props as never)} />);
    await waitFor(() => expect(screen.getByText(/開放討論的背景內容/)).toBeTruthy());
  });

  it("狀態卡階段列：四站可見且現站可辨（已封存站保留給已封存討論頁）", async () => {
    const stageRow = () => document.querySelector("[data-status-stage]") as HTMLElement;
    // concluded → 現站「已結論」。
    const { unmount } = render(<DiscussionDetailPage {...(makeProps() as never)} />);
    await waitFor(() => expect(stageRow()).toBeTruthy());
    const row = stageRow();
    for (const label of ["討論中", "已結論", "已轉出", "已封存"]) expect(within(row).getByText(label)).toBeTruthy();
    expect(within(row).getByText("已結論").closest("[aria-current]")).toBeTruthy();
    expect(within(row).getByText("討論中").closest("[aria-current]")).toBeNull();
    unmount();
    // promoted → 現站「已轉出」。
    render(<DiscussionDetailPage {...(makeProps({ discussion: promotedD }) as never)} />);
    await waitFor(() => expect(stageRow()).toBeTruthy());
    expect(within(stageRow()).getByText("已轉出").closest("[aria-current]")).toBeTruthy();
    expect(within(stageRow()).getByText("已封存").closest("[aria-current]")).toBeNull();
  });

  it("非預期格式整篇以單一檢視退回（無背景分頁、全文可見）", async () => {
    const props = makeProps({
      loadDocument: vi.fn(async () => "手寫的自由格式記錄，沒有標準區段。"),
    });
    render(<DiscussionDetailPage {...(props as never)} />);
    await waitFor(() => expect(screen.getByText(/自由格式記錄/)).toBeTruthy());
    expect(screen.queryByRole("tab", { name: /背景/ })).toBeNull();
  });

  it("衍生變更分頁列出子變更現況；存活者可跳轉、已刪除者不可", async () => {
    const props = makeProps({ discussion: promotedD });
    render(<DiscussionDetailPage {...(props as never)} />);
    // 先等文件載入落定（section 分頁組掛上、背景分頁出現）再切衍生變更：否則
    // loadDocument resolve 會把 fallback 分頁組換成 sections 分頁組（key 變、重新
    // mount），選中分頁重置回預設，衍生變更面板被隱藏、cut-a 從 DOM 消失。與本檔
    // 其他分頁測試一致的等待。
    await screen.findByRole("tab", { name: /背景/ });
    fireEvent.mouseDown(screen.getByRole("tab", { name: /衍生變更/ }));
    // 狀態卡的衍生變更籤也列同名——分頁內的列以分頁面板為範圍。
    const panel = screen.getByRole("tabpanel");
    const rowA = within(panel).getByText("cut-a").closest("[data-promoted-row]") as HTMLElement;
    expect(within(rowA).getByText("提案中")).toBeTruthy();
    fireEvent.click(within(rowA).getByRole("button", { name: /開啟卡片/ }));
    expect(props.onOpenChangeCard).toHaveBeenCalledWith("cut-a");
    const rowGone = within(panel).getByText("cut-gone").closest("[data-promoted-row]") as HTMLElement;
    expect(within(rowGone).getByText("已刪除")).toBeTruthy();
    expect(within(rowGone).queryByRole("button", { name: /開啟卡片/ })).toBeNull();
  });

  it("衍生變更分頁唯讀：concluded／promoted 皆無「轉為變更／再轉出一個變更」動作（D3）", async () => {
    // promote 已自 GUI 撤除——衍生變更分頁只列子變更與跳轉，無任何轉出鈕。
    // 各實例先等初始載入落地（標準格式解析出「背景」分頁），避免 act 警告。
    const { unmount } = render(<DiscussionDetailPage {...(makeProps() as never)} />);
    await screen.findByRole("tab", { name: /背景/ });
    fireEvent.mouseDown(screen.getByRole("tab", { name: /衍生變更/ }));
    expect(screen.queryByRole("button", { name: /轉為變更/ })).toBeNull();
    expect(screen.queryByRole("button", { name: /促轉/ })).toBeNull();
    unmount();

    const { unmount: unmount2 } = render(
      <DiscussionDetailPage {...(makeProps({ discussion: promotedD }) as never)} />,
    );
    await screen.findByRole("tab", { name: /背景/ });
    fireEvent.mouseDown(screen.getByRole("tab", { name: /衍生變更/ }));
    expect(screen.queryByRole("button", { name: /再轉出一個變更/ })).toBeNull();
    expect(screen.queryByRole("button", { name: /轉為變更/ })).toBeNull();
    unmount2();

    const props3 = makeProps({ discussion: openD, loadDocument: vi.fn(async () => OPEN_DOC) });
    render(<DiscussionDetailPage {...(props3 as never)} />);
    await screen.findByRole("tab", { name: /背景/ });
    fireEvent.mouseDown(screen.getByRole("tab", { name: /衍生變更/ }));
    expect(screen.queryByRole("button", { name: /轉為變更/ })).toBeNull();
  });
});

describe("change 側同源連結", () => {
  it("來自討論的 change 卡帶單一討論徽章；無來源者不帶", () => {
    const withSource: ChangeItem = {
      name: "cut-a",
      status: "in-progress",
      totalTasks: 24,
      completedTasks: 0,
      fromDiscussions: ["alpha-search"],
    };
    const { unmount } = render(<ChangeCard change={withSource} />);
    expect(screen.getByLabelText("來自討論")).toBeTruthy();
    unmount();
    render(<ChangeCard change={{ ...withSource, fromDiscussions: [] }} />);
    expect(screen.queryByLabelText("來自討論")).toBeNull();
  });

  it("多來源的 change 卡仍是單一徽章、以主題化提示取代原生 title（D3）", () => {
    const multi: ChangeItem = {
      name: "cut-a",
      status: "in-progress",
      totalTasks: 24,
      completedTasks: 0,
      fromDiscussions: ["alpha-search", "beta-cache"],
    };
    render(<ChangeCard change={multi} />);
    // 單一徽章（不因多來源而增生）。
    expect(screen.getAllByLabelText("來自討論")).toHaveLength(1);
    // D3：改用 shadcn Tooltip——徽章不再帶原生 title（提示列出全部來源於 hover 呈現，實窗驗證）。
    expect(screen.getByLabelText("來自討論").getAttribute("title")).toBeNull();
  });

  it("change 詳情頁列出全部來源討論與同源清單並可互跳", async () => {
    const onOpenDiscussion = vi.fn();
    const onOpenSibling = vi.fn();
    const change: ChangeItem = {
      name: "cut-a",
      status: "in-progress",
      totalTasks: 24,
      completedTasks: 0,
      fromDiscussions: ["alpha-search", "beta-cache"],
    };
    render(
      <ChangeDetailPage
        change={change}
        loadDocument={vi.fn(async () => "# doc")}
        loadCapabilities={vi.fn(async () => [])}
        loadMeta={vi.fn(async () => ({ created: "2026-07-05" }))}
        sourceDiscussions={[
          { slug: "alpha-search", topic: "Alpha search" },
          { slug: "beta-cache", topic: "Beta cache" },
        ]}
        siblingChanges={["cut-b"]}
        onOpenDiscussion={onOpenDiscussion}
        onOpenSibling={onOpenSibling}
      />,
    );
    await waitFor(() => expect(screen.getByText("來自")).toBeTruthy());
    // 全部來源討論可及並互跳（change-drawer-header-redesign：首籤 slug 直出、其餘經 +N 浮層）。
    fireEvent.click(screen.getByRole("button", { name: /alpha-search/ }));
    expect(onOpenDiscussion).toHaveBeenCalledWith("alpha-search");
    fireEvent.click(screen.getByRole("button", { name: /其餘 1 份/ }));
    const popover = await waitFor(() => {
      const el = document.querySelector("[data-source-overflow-list]") as HTMLElement | null;
      expect(el).toBeTruthy();
      return el!;
    });
    fireEvent.click(within(popover).getByRole("button", { name: /beta-cache/ }));
    expect(onOpenDiscussion).toHaveBeenCalledWith("beta-cache");
    fireEvent.click(screen.getByRole("button", { name: /cut-b/ }));
    expect(onOpenSibling).toHaveBeenCalledWith("cut-b");
  });
});

describe("DiscussionDetailPage 世代重載（spec：外部推進討論後頁內容更新）", () => {
  it("refreshGen 遞增時重載記錄，回合分頁呈現新回合且分頁選擇不重置", async () => {
    const props = makeProps();
    const { rerender } = render(<DiscussionDetailPage {...(props as never)} refreshGen={0} />);
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
    rerender(<DiscussionDetailPage {...(props as never)} refreshGen={1} />);
    await waitFor(() => expect(calls()).toBeGreaterThan(c0));
    await waitFor(() => expect(screen.getByText(/第二輪新內容/)).toBeTruthy());
    // 舊回合仍在、分頁停留在討論過程——重載為就地替換，非重開頁。
    expect(screen.getByText(/範圍界定/)).toBeTruthy();
  });
});

// spec 需求「討論詳情頁檢視與轉出變更」：頁標題以 slug 為題（等寬）＋複製鈕、
// topic 為標題下方一行（LANGUAGE.md 受控例外）。
describe("DiscussionDetailPage 標題 slug 化（design D4）", () => {
  it("標題呈 slug（等寬字型）、topic 降為副標、複製鈕寫入剪貼簿", async () => {
    const writeText = vi.fn();
    Object.assign(navigator, { clipboard: { writeText } });
    render(<DiscussionDetailPage {...(makeProps() as never)} />);
    await screen.findByRole("tab", { name: /背景/ });
    const title = screen.getByText("alpha-search");
    expect(title.className).toContain("font-mono");
    expect(screen.getByText("Alpha search")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "複製 slug" }));
    expect(writeText).toHaveBeenCalledWith("alpha-search");
    // 共用 CopyButton：成功以 status 宣告「已複製」。
    await waitFor(() => expect(screen.getByRole("status").textContent).toBe("已複製"));
  });
});

// spec 需求「markdown 文件內容行寬有上限」（design D4）與「詳情頁的文件樹、閱讀卡與本頁目錄」：
// 閱讀卡內文為共用置中容器，討論輪卡片與內文同欄對齊置中；閱讀區內容欄 768px 置中。
describe("討論詳情頁閱讀欄置中", () => {
  // spec「markdown 文件內容行寬有上限」（desktop-change-doc-views design D10）：閱讀卡內以 768px 卡欄為上限，不再另套 96ch。
  it("閱讀卡內文撐滿卡欄（w-full、無 96ch）且輪卡片在欄內", async () => {
    const { baseElement } = render(<DiscussionDetailPage {...(makeProps() as never)} />);
    await screen.findByRole("tab", { name: /討論過程/ });
    const col = baseElement.querySelector("[data-doc-body]") as HTMLElement;
    expect(col).toBeTruthy();
    expect(col.className).toContain("w-full");
    expect(col.className).not.toContain("max-w-[96ch]");
    const page = baseElement.querySelector("[data-discussion-detail]") as HTMLElement;
    const frame = page.querySelector("[data-detail-reading]")!.firstElementChild as HTMLElement;
    expect(frame.className).toContain("max-w-[768px]");
    expect(frame.contains(col)).toBe(true);
    // 結論分頁（預設）內文在欄內。
    expect(within(col).getByText("建置 alpha 搜尋")).toBeTruthy();
    // 討論過程分頁：輪卡片同欄。
    fireEvent.mouseDown(screen.getByRole("tab", { name: /討論過程/ }));
    expect(col.querySelector("[data-round]")).toBeTruthy();
  });
});

// spec「討論詳情頁檢視與轉出變更」頁內封存場景:concluded 且未封存(宿主提供
// handler)才出現;走與討論卡同一 onArchiveDiscussion 呼叫(確認流程在宿主)。
describe("頁內封存動詞", () => {
  it("concluded 討論顯示封存動作,點擊觸發 onArchiveDiscussion", async () => {
    const onArchiveDiscussion = vi.fn();
    render(<DiscussionDetailPage {...(makeProps({ onArchiveDiscussion }) as never)} />);
    const btn = await screen.findByRole("button", { name: /封存/ });
    fireEvent.click(btn);
    expect(onArchiveDiscussion).toHaveBeenCalledWith("alpha-search");
  });

  it("非 concluded(open/promoted)不顯示封存動作", async () => {
    render(
      <DiscussionDetailPage
        {...(makeProps({ discussion: openD, onArchiveDiscussion: vi.fn() }) as never)}
      />,
    );
    await screen.findByText("Alpha search");
    expect(screen.queryByRole("button", { name: /封存/ })).toBeNull();

    render(
      <DiscussionDetailPage
        {...(makeProps({ discussion: promotedD, onArchiveDiscussion: vi.fn() }) as never)}
      />,
    );
    expect(screen.queryByRole("button", { name: /封存/ })).toBeNull();
  });

  it("已封存(宿主不提供 handler)不顯示封存動作", async () => {
    render(<DiscussionDetailPage {...(makeProps() as never)} />);
    await screen.findByText("Alpha search");
    expect(screen.queryByRole("button", { name: /封存/ })).toBeNull();
  });
});

// spec「詳情頁文件載入以 skeleton 呈現」：討論詳情頁載入中畫骨架。
describe("討論詳情頁文件三態", () => {
  it("載入中 → 文件骨架，不出空態文案", async () => {
    render(
      <DiscussionDetailPage
        {...(makeProps({ loadDocument: vi.fn(() => new Promise<never>(() => {})) }) as never)}
      />,
    );
    await waitFor(() => expect(document.querySelector('[aria-busy="true"]')).toBeTruthy());
    expect(screen.queryByText("（無內容）")).toBeNull();
  });

  it("載入完成且記錄不存在 → 空態文案，無骨架", async () => {
    render(<DiscussionDetailPage {...(makeProps({ loadDocument: vi.fn(async () => null) }) as never)} />);
    await waitFor(() => expect(screen.getByText("（無內容）")).toBeTruthy());
    expect(document.querySelector('[aria-busy="true"]')).toBeNull();
  });

  it("首載失敗 → 落到空態文案，不停在骨架", async () => {
    render(
      <DiscussionDetailPage
        {...(makeProps({ loadDocument: vi.fn().mockRejectedValue(new Error("offline")) }) as never)}
      />,
    );
    await waitFor(() => expect(screen.getByText("（無內容）")).toBeTruthy());
    expect(document.querySelector('[aria-busy="true"]')).toBeNull();
  });
});

// spec「討論詳情頁檢視與轉出變更」Scenario「狀態卡的保留與衍生列」（design D7）：
// 標題列狀態籤、階段列現站、保留在途列與衍生變更籤列。
describe("討論詳情頁狀態卡的保留與衍生列", () => {
  const heldD: DiscussionItem = {
    ...concludedD,
    status: "promoted",
    concluded: true,
    hold: true,
    promotedTo: ["cut-old", "cut-b"],
  };
  const liveChanges: ChangeItem[] = [
    { name: "cut-b", status: "in-progress", totalTasks: 10, completedTasks: 3 },
  ];
  const archived: ArchivedItem[] = [{ datedName: "2026-07-20-cut-old", date: "2026-07-20", name: "cut-old" }];
  const row = (mark: string) => document.querySelector(`[data-status-${mark}]`) as HTMLElement | null;

  it("promoted＋hold＋兩項衍生：保留中狀態籤、現站已轉出、保留列、兩顆籤帶階段、點進行中籤跳轉、封存停用", async () => {
    const onOpenChangeCard = vi.fn();
    const onArchiveDiscussion = vi.fn();
    render(
      <DiscussionDetailPage
        {...(makeProps({
          discussion: heldD,
          changes: liveChanges,
          archivedChanges: archived,
          onOpenChangeCard,
          onArchiveDiscussion,
        }) as never)}
      />,
    );
    await screen.findByRole("tab", { name: /背景/ });
    const status = document.querySelector("[data-discussion-status]") as HTMLElement;
    expect(status.textContent).toBe("已轉出・保留中");
    expect(status.className).toContain("text-status-warning");
    expect(within(row("stage")!).getByText("已轉出").closest("[aria-current]")).toBeTruthy();
    expect(row("hold")!.textContent).toContain("結論保留在途；最後一刀轉出後自動封存");
    const old = document.querySelector('[data-promoted-chip="cut-old"]') as HTMLElement;
    const live = document.querySelector('[data-promoted-chip="cut-b"]') as HTMLElement;
    expect(row("promoted")!.contains(old) && row("promoted")!.contains(live)).toBe(true);
    expect(within(old).getByText("已封存")).toBeTruthy();
    expect(within(live).getByText("進行中")).toBeTruthy();
    fireEvent.click(within(live).getByRole("button", { name: "cut-b" }));
    expect(onOpenChangeCard).toHaveBeenCalledWith("cut-b");
    expect((within(old).getByRole("button", { name: "cut-old" }) as HTMLButtonElement).disabled).toBe(true);
    const archive = screen.getByRole("button", { name: /封存/ }) as HTMLButtonElement;
    expect(archive.disabled).toBe(true);
    fireEvent.click(archive);
    expect(onArchiveDiscussion).not.toHaveBeenCalled();
  });

  // spec Scenario「狀態卡的保留與衍生列」「尚無結論的已轉出討論的步驟列」「已轉出且未保留的討論現站為主色」：
  // 步驟列現站與身分區狀態籤同色；尚無結論時「已結論」站不打勾。
  const current = () => row("stage")!.querySelector('[aria-current="step"]') as HTMLElement;
  const station = (label: string) => within(row("stage")!).getByText(label);

  it("hold 為 true 的已轉出討論：現站為 warning 色、「已結論」站打勾", async () => {
    render(<DiscussionDetailPage {...(makeProps({ discussion: heldD, changes: liveChanges, archivedChanges: archived }) as never)} />);
    await screen.findByRole("tab", { name: /背景/ });
    expect(current().textContent).toBe("已轉出");
    expect(current().className).toContain("status-warning");
    expect(station("已結論").querySelector("svg.lucide-check")).toBeTruthy();
  });

  it("concluded 為 false 的已轉出討論：現站為 warning 色、「已結論」站為未到站灰圈", async () => {
    const unconcluded: DiscussionItem = { ...concludedD, status: "promoted", concluded: false, promotedTo: ["cut-b"] };
    render(<DiscussionDetailPage {...(makeProps({ discussion: unconcluded, changes: liveChanges }) as never)} />);
    await screen.findByRole("tab", { name: /背景/ });
    expect((document.querySelector("[data-discussion-status]") as HTMLElement).textContent).toBe("已轉出・尚無結論");
    expect(current().textContent).toBe("已轉出");
    expect(current().className).toContain("status-warning");
    const concluded = station("已結論");
    expect(concluded.querySelector("svg.lucide-check")).toBeNull();
    expect(concluded.querySelector("svg.lucide-circle")).toBeTruthy();
    expect(concluded.className).toContain("text-muted-foreground/50");
    expect(station("討論中").querySelector("svg.lucide-check")).toBeTruthy();
  });

  it("hold 為 false、已結論的已轉出討論：現站為主色", async () => {
    const collapsed: DiscussionItem = { ...heldD, hold: false };
    render(<DiscussionDetailPage {...(makeProps({ discussion: collapsed, changes: liveChanges, archivedChanges: archived }) as never)} />);
    await screen.findByRole("tab", { name: /背景/ });
    expect(current().className).toContain("primary");
    expect(current().className).not.toContain("status-warning");
    expect(station("已結論").querySelector("svg.lucide-check")).toBeTruthy();
  });

  it("無 hold 無 promoted_to 時保留列與衍生列缺席", async () => {
    render(<DiscussionDetailPage {...(makeProps() as never)} />);
    await screen.findByRole("tab", { name: /背景/ });
    expect(row("stage")).toBeTruthy();
    expect(row("hold")).toBeNull();
    expect(row("promoted")).toBeNull();
  });

  it("分頁標頭：討論過程 N 與衍生變更 N 帶計數", async () => {
    render(<DiscussionDetailPage {...(makeProps({ discussion: promotedD }) as never)} />);
    await screen.findByRole("tab", { name: /背景/ });
    expect(screen.getByRole("tab", { name: /討論過程/ }).textContent).toContain("1");
    expect(screen.getByRole("tab", { name: /衍生變更/ }).textContent).toContain("2");
  });
});

// spec「詳情頁的文件動作」：討論各分頁皆為討論記錄檔（磁碟位置——含已封存的日期檔名——由宿主解析）。
describe("討論詳情頁的文件動作槽", () => {
  it("回報討論 slug 與記錄原文，渲染在閱讀卡標題列", async () => {
    const renderDocumentActions = vi.fn((target: DocumentTarget | null) =>
      target ? <button type="button">開啟記錄</button> : null,
    );
    render(<DiscussionDetailPage {...(makeProps({ renderDocumentActions }) as never)} />);
    await screen.findByRole("tab", { name: /背景/ });
    expect(renderDocumentActions).toHaveBeenLastCalledWith({ doc: { discussion: "alpha-search" }, content: DOC });
    const docHeader = document.querySelector("[data-doc-header]") as HTMLElement;
    expect(within(docHeader).getByRole("button", { name: "開啟記錄" })).toBeTruthy();
  });
});

// spec「討論詳情頁檢視與轉出變更」：身分區「關聯」列（自背景的 Prior discussions 行計數、
// none 或缺行時缺席、點擊切到背景）；封存動作經 DetailActions 進標題列（無宿主時原地）。
describe("討論詳情頁的關聯列與標題列動作", () => {
  const withPrior = (line: string) => DOC.replace("框架脈絡內容。", `框架脈絡內容。\n\n${line}`);

  it("Prior discussions 三筆時關聯列顯示「先前討論 3」，點擊切到背景", async () => {
    const props = makeProps({ loadDocument: vi.fn(async () => withPrior("Prior discussions: a-one, b-two, c-three")) });
    render(<DiscussionDetailPage {...(props as never)} />);
    const related = await waitFor(() => {
      const el = document.querySelector("[data-relation-related]") as HTMLElement | null;
      expect(el).toBeTruthy();
      return el!;
    });
    expect(within(related).getByText("關聯")).toBeTruthy();
    // 預設文件為結論；點關聯籤切到背景。
    expect(screen.getByRole("tab", { name: /結論/ }).getAttribute("aria-selected")).toBe("true");
    fireEvent.click(within(related).getByRole("button", { name: "先前討論 3" }));
    expect(screen.getByRole("tab", { name: /背景/ }).getAttribute("aria-selected")).toBe("true");
    expect(screen.getByText(/框架脈絡內容/)).toBeTruthy();
  });

  it("頓號分隔且帶括號備註時依項目計數（括號內的逗號不算）", async () => {
    const line = "Prior discussions: alpha-one（備註，含逗號）、beta-two（另一則）";
    render(<DiscussionDetailPage {...(makeProps({ loadDocument: vi.fn(async () => withPrior(line)) }) as never)} />);
    const related = await waitFor(() => {
      const el = document.querySelector("[data-relation-related]") as HTMLElement | null;
      expect(el).toBeTruthy();
      return el!;
    });
    expect(within(related).getByRole("button").textContent).toBe("先前討論 2");
  });

  it("Prior discussions 為 none 或缺行時無關聯列", async () => {
    const r = render(
      <DiscussionDetailPage
        {...(makeProps({ loadDocument: vi.fn(async () => withPrior("Prior discussions: none")) }) as never)}
      />,
    );
    await screen.findByRole("tab", { name: /背景/ });
    expect(document.querySelector("[data-relation-related]")).toBeNull();
    r.unmount();
    render(<DiscussionDetailPage {...(makeProps() as never)} />);
    await screen.findByRole("tab", { name: /背景/ });
    expect(document.querySelector("[data-relation-related]")).toBeNull();
  });

  it("閱讀卡出處行依文件切換", async () => {
    render(<DiscussionDetailPage {...(makeProps({ discussion: promotedD }) as never)} />);
    await screen.findByRole("tab", { name: /背景/ });
    const provenance = () => document.querySelector("[data-doc-header] [data-doc-provenance]")?.textContent;
    expect(provenance()).toBe("openspec/discussions/alpha-search.md·1 輪之後寫入");
    fireEvent.mouseDown(screen.getByRole("tab", { name: /討論過程/ }));
    expect(provenance()).toBe("2026-07-01·assumptions 1 輪");
    fireEvent.mouseDown(screen.getByRole("tab", { name: /背景/ }));
    expect(provenance()).toBe("記錄建立時寫入一次·2026-07-01");
    fireEvent.mouseDown(screen.getByRole("tab", { name: /衍生變更/ }));
    expect(provenance()).toBe("依轉出順序");
  });

  it("背景的規格籤呼叫 onOpenSpec、討論籤呼叫 onOpenDiscussion", async () => {
    const onOpenSpec = vi.fn();
    const onOpenDiscussion = vi.fn();
    const doc = DOC.replace("框架脈絡內容。", "框架脈絡提到 desktop-app 與 beta-talk，也提到自己 alpha-search。");
    render(
      <DiscussionDetailPage
        {...(makeProps({
          loadDocument: vi.fn(async () => doc),
          specNames: ["desktop-app"],
          discussionSlugs: ["alpha-search", "beta-talk"],
          onOpenSpec,
          onOpenDiscussion,
        }) as never)}
      />,
    );
    fireEvent.mouseDown(await screen.findByRole("tab", { name: /背景/ }));
    const chip = await waitFor(() => {
      const el = document.querySelector('[data-name-chip="spec:desktop-app"]') as HTMLElement | null;
      expect(el).toBeTruthy();
      return el!;
    });
    fireEvent.click(chip);
    expect(onOpenSpec).toHaveBeenCalledWith("desktop-app");
    fireEvent.click(document.querySelector('[data-name-chip="discussion:beta-talk"]')!);
    expect(onOpenDiscussion).toHaveBeenCalledWith("beta-talk");
    expect(document.querySelector('[data-name-chip="discussion:alpha-search"]')).toBeNull();
  });

  it("換討論時即使同為結論文件，閱讀區也捲回頂端", async () => {
    const { rerender } = render(<DiscussionDetailPage {...(makeProps() as never)} />);
    await screen.findByRole("tab", { name: /背景/ });
    const reading = document.querySelector("[data-detail-reading]") as HTMLElement;
    reading.scrollTop = 300;
    rerender(<DiscussionDetailPage {...(makeProps({ discussion: { ...concludedD, slug: "beta-search" } }) as never)} />);
    expect((document.querySelector("[data-detail-reading]") as HTMLElement).scrollTop).toBe(0);
    await screen.findByRole("tab", { name: /背景/ });
  });

  it("封存鈕在 DetailActions 內，不在左欄與閱讀卡", async () => {
    render(<DiscussionDetailPage {...(makeProps({ onArchiveDiscussion: vi.fn() }) as never)} />);
    await screen.findByRole("tab", { name: /背景/ });
    const actions = document.querySelector("[data-detail-actions]") as HTMLElement;
    const archive = within(actions).getByRole("button", { name: /封存/ });
    expect(document.querySelector("[data-detail-tree]")!.contains(archive)).toBe(false);
    expect(document.querySelector("[data-doc-header]")!.contains(archive)).toBe(false);
  });
});


// spec「討論結論以欄位標籤呈現」「討論輪以卡片呈現」「討論詳情頁檢視與轉出變更」的數字磚、出處行與提示框
// （desktop-discussion-doc-views design D6）：推導入口的輸出直接畫進閱讀卡的標題區零件。
describe("數字磚、出處行與提示框", () => {
  const NAMES = { specs: [], active: [], archived: [], discussions: [] };
  const t = (key: string) => MESSAGES["zh-TW"][key] ?? key;
  const live = (name: string, done: number, total: number): ChangeItem => ({
    name,
    status: "ready",
    totalTasks: total,
    completedTasks: done,
  });
  const roundsOf = (modes: string[], dates: string[], lastOpen = "無：寫入結論") =>
    modes
      .map(
        (mode, i) =>
          `### Round ${i + 1} — ${mode} (${dates[i]})\n\n**Focus**: 第 ${i + 1} 輪\n**Position**: 立場${
            i === modes.length - 1 ? `\n**Open**: ${lastOpen}` : ""
          }`,
      )
      .join("\n\n");
  const record = (rounds: string, conclusion: string, context = "背景。") =>
    `# Discussion\n\n## Context\n\n${context}\n\n## Rounds\n\n${rounds}\n\n## Conclusion\n\n${conclusion}\n`;

  function Harness({ tab, data }: { tab: DiscussionTab; data: DiscussionDocData }) {
    const { t } = useI18n();
    const summary = discussionDocSummary(tab, data, t);
    return (
      <>
        <DocHeader title="文件" provenance={discussionDocProvenance(tab, data, t)} />
        <DocTiles tiles={summary.tiles} />
        {summary.callout && <div data-doc-callout>{summary.callout}</div>}
      </>
    );
  }
  const tiles = () =>
    Array.from(document.querySelectorAll("[data-doc-tile]")).map((tile) =>
      Array.from(tile.children).map((c) => c.textContent),
    );
  const provenance = () => document.querySelector("[data-doc-provenance]")?.textContent ?? null;
  const callout = () => document.querySelector("[data-doc-callout]") as HTMLElement | null;

  it("討論過程的數字磚、出處行與已收斂提示框", () => {
    const modes = Array.from({ length: 20 }, (_, i) => (i % 2 === 0 ? "assumptions" : "interview"));
    const data = discussionDocData({
      text: record(roundsOf(modes, Array(20).fill("2026-10-08")), "**Decision**: 定案"),
      status: "concluded",
      hold: false,
      created: "2026-10-08",
      recordPath: "openspec/discussions/demo.md",
      rows: [],
      names: NAMES,
    });
    render(<Harness tab="rounds" data={data} />);
    expect(tiles()).toEqual([
      ["20", "輪"],
      ["1", "天"],
      ["10", "assumptions"],
      ["10", "interview"],
    ]);
    expect(provenance()).toBe("2026-10-08·assumptions 10 輪·interview 10 輪");
    expect(callout()!.textContent).toContain("討論已收斂");
    expect(callout()!.textContent).toContain("第 20 輪之後寫入結論");
    expect(callout()!.textContent).toContain("最後一輪待決：無");
  });

  it("結論文件的數字磚與出處行", () => {
    const decision = [
      "**Decision**: 一句定論。",
      ...Array.from({ length: 20 }, (_, i) => `- 細節 ${i}`),
      ...[1, 2, 3].map((n) => `- **cut ${n} \`chg-${n}\`**: 第 ${n} 刀`),
      "**Rejected alternatives**:",
      ...Array.from({ length: 11 }, (_, i) => `- 方案 ${i} — 理由`),
      "**Deferred**:",
      ...Array.from({ length: 4 }, (_, i) => `- 擱置 ${i}`),
    ].join("\n");
    const data = discussionDocData({
      text: record(roundsOf(Array(11).fill("interview"), Array(11).fill("2026-10-12")), decision),
      status: "concluded",
      hold: false,
      created: "2026-10-12",
      recordPath: "openspec/discussions/demo.md",
      rows: promotedRows(["chg-1", "chg-2", "chg-3"], [1, 2, 3].map((n) => live(`chg-${n}`, n, n)), [], t),
      names: NAMES,
    });
    render(<Harness tab="conclusion" data={data} />);
    expect(provenance()).toBe("openspec/discussions/demo.md·11 輪之後寫入");
    expect(tiles()).toEqual([
      ["20", "定案細節"],
      ["3", "分刀 · 3 個變更"],
      ["11", "否決的替代方案"],
      ["4", "擱置"],
    ]);
    expect(callout()).toBeNull();
  });

  // spec Scenario「未轉出刀的提示框與前往」「沒有整刀未轉出的刀時無提示框」「已封存討論頁不出未轉出提示框」。
  const wadData = (decision: string, promoted: string[], status = "promoted") =>
    discussionDocData({
      text: record(roundsOf(["interview"], ["2026-10-12"]), decision),
      status,
      hold: true,
      created: "2026-10-12",
      recordPath: "openspec/discussions/wad-plugins-merge-or-split.md",
      rows: promotedRows(promoted, promoted.map((name) => live(name, 1, 3)), [], t),
      names: { ...NAMES, self: "wad-plugins-merge-or-split" },
    });

  it("結論有整刀未轉出的刀時有橄欖色提示框", () => {
    render(<Harness tab="conclusion" data={wadData(WAD_DECISION, ["add-team-plugin-library"])} />);
    const box = callout()!;
    expect(box).toBeTruthy();
    expect(box.querySelector(".markdown-alert-title")!.textContent).toBe("1 刀尚未轉出");
    expect(box.firstElementChild!.className).toContain("status-manual");
    const rows = Array.from(box.querySelectorAll("[data-unpromoted-cut]"));
    expect(rows.length).toBe(1);
    expect(rows[0].textContent).toContain("cut 2");
    expect(rows[0].textContent).toContain("移除內建 wad-work");
    expect(box.querySelector("[data-command-chip]")!.textContent).toBe(
      "/speclink-propose --from-discussion wad-plugins-merge-or-split",
    );
  });

  it("全部刀已轉出或只部分轉出時無提示框", () => {
    const decision = [
      "**Decision**: 三刀。",
      "- **cut 1 `a-one` `b-two`**: 部分轉出",
      "- **cut 2 `c-three`**: 已轉出",
      "- **刀三**：沒有變更名",
    ].join("\n");
    render(<Harness tab="conclusion" data={wadData(decision, ["a-one", "c-three"])} />);
    expect(tiles()[1]).toEqual(["3", "分刀 · 3 個變更"]);
    expect(callout()).toBeNull();
  });

  it("status 為 archived 時無提示框", () => {
    render(<Harness tab="conclusion" data={wadData(WAD_DECISION, ["add-team-plugin-library"], "archived")} />);
    expect(tiles()[1]).toEqual(["2", "分刀 · 2 個變更"]);
    expect(callout()).toBeNull();
  });

  it("背景的數字磚、出處行與「沒有 grill 階段」提示框", () => {
    const context = [
      "提到 desktop-app、tray-status-menu、add-copilot-tool 與 tray-panel-card-design。",
      "",
      "Related specs: desktop-app（79 條）、tray-status-menu",
      "Prior discussions: card-drawer-header-colors, admin-console-redesign",
      "Source doc: none",
    ].join("\n");
    const data = discussionDocData({
      text: record(roundsOf(["assumptions", "interview"], ["2026-10-08", "2026-10-08"]), "", context),
      status: "open",
      hold: false,
      created: "2026-10-08",
      recordPath: "openspec/discussions/demo.md",
      rows: [],
      names: {
        specs: ["desktop-app", "tray-status-menu"],
        active: ["add-copilot-tool"],
        archived: ["tray-panel-card-design"],
        discussions: ["card-drawer-header-colors", "admin-console-redesign"],
      },
    });
    render(<Harness tab="context" data={data} />);
    expect(tiles()).toEqual([
      ["2", "命中的正式規格"],
      ["1", "在途變更"],
      ["1", "相關已封存變更"],
      ["2", "先前討論"],
    ]);
    expect(provenance()).toBe("記錄建立時寫入一次·2026-10-08");
    expect(callout()!.textContent).toContain("沒有 grill 階段");
    expect(callout()!.textContent).toContain("首輪直接列假設");
  });

  // spec Scenario「非標準格式整篇退回」「自由格式結論整篇退回」「結論缺席欄位不渲染空標籤」講數字磚的半句。
  it("區段切得開、但輪標題不符 scaffold 時討論過程無數字磚、無提示框", () => {
    const rounds = roundsOf(["interview", "interview"], ["2026-10-08", "2026-10-09"]).replace(
      "### Round 2 — interview (2026-10-09)",
      "### 插入的手寫標題",
    );
    const data = discussionDocData({
      text: record(rounds, "**Decision**: 定案"),
      status: "concluded",
      hold: false,
      created: null,
      recordPath: null,
      rows: [],
      names: NAMES,
    });
    expect(data.sections).not.toBeNull();
    render(<Harness tab="rounds" data={data} />);
    expect(document.querySelector("[data-doc-tiles]")).toBeNull();
    expect(callout()).toBeNull();
  });

  it("自由格式結論整篇退回時結論無數字磚", () => {
    const data = discussionDocData({
      text: record(roundsOf(["interview"], ["2026-10-08"]), "就這樣定了，大家都同意這個方向。"),
      status: "concluded",
      hold: false,
      created: null,
      recordPath: null,
      rows: [],
      names: NAMES,
    });
    render(<Harness tab="conclusion" data={data} />);
    expect(document.querySelector("[data-doc-tiles]")).toBeNull();
  });

  it("結論缺 Deferred 時數字磚沒有「擱置」格", () => {
    const data = discussionDocData({
      text: record(
        roundsOf(["interview"], ["2026-10-08"]),
        "**Decision**: 定論。\n- 細節一\n**Rejected alternatives**:\n- 方案 — 理由",
      ),
      status: "concluded",
      hold: false,
      created: null,
      recordPath: null,
      rows: [],
      names: NAMES,
    });
    render(<Harness tab="conclusion" data={data} />);
    expect(tiles()).toEqual([
      ["1", "定案細節"],
      ["1", "否決的替代方案"],
    ]);
  });

  it("背景無關聯時數字磚整列缺席；首輪不是 assumptions 時無提示框", () => {
    const data = discussionDocData({
      text: record(roundsOf(["interview"], ["2026-10-08"]), "", "兩段純散文。\n\n第二段。"),
      status: "open",
      hold: false,
      created: null,
      recordPath: null,
      rows: [],
      names: NAMES,
    });
    render(<Harness tab="context" data={data} />);
    expect(document.querySelector("[data-doc-tiles]")).toBeNull();
    expect(callout()).toBeNull();
    expect(provenance()).toBe("記錄建立時寫入一次");
  });

  describe("衍生變更", () => {
    const gone = (name: string, n: number): ArchivedItem => ({
      datedName: `2026-10-0${n}-${name}`,
      date: `2026-10-0${n}`,
      name,
      tasksDone: n,
      tasksTotal: n,
    });
    const docData = (rows: PromotedRow[], over: Partial<{ status: string; hold: boolean; cuts: boolean }> = {}) =>
      discussionDocData({
        text: record(
          roundsOf(["interview"], ["2026-10-08"]),
          over.cuts ? "**Decision**: 定案\n- **cut 1 `a-one`**: 一" : "**Decision**: 定案",
        ),
        status: over.status ?? "promoted",
        hold: over.hold ?? false,
        created: "2026-10-08",
        recordPath: null,
        rows,
        names: NAMES,
      });

    it("只差最後一刀提示框：已轉出、hold 為 false、恰一個作用中子變更", () => {
      const rows = promotedRows(
        ["a-one", "b-two", "tray-panel-restyle"],
        [live("tray-panel-restyle", 9, 9)],
        [gone("a-one", 3), gone("b-two", 4)],
        t,
      );
      render(<Harness tab="promote" data={docData(rows, { cuts: true })} />);
      expect(tiles()).toEqual([
        ["3", "變更"],
        ["2", "已封存"],
        ["1", "作用中"],
        ["16 / 16", "任務"],
      ]);
      expect(provenance()).toBe("依結論的分刀分組");
      expect(callout()!.textContent).toContain("只差最後一刀");
      expect(callout()!.textContent).toContain("tray-panel-restyle（已就緒）");
    });

    it("hold 為 true、或作用中子變更有兩個時無此提示框；讀不到分刀時出處行為依轉出順序", () => {
      const one = promotedRows(["a-one", "tray-panel-restyle"], [live("tray-panel-restyle", 1, 2)], [gone("a-one", 3)], t);
      const { unmount } = render(<Harness tab="promote" data={docData(one, { hold: true })} />);
      expect(callout()).toBeNull();
      expect(provenance()).toBe("依轉出順序");
      unmount();
      const two = promotedRows(["x-one", "y-two"], [live("x-one", 1, 2), live("y-two", 0, 3)], [], t);
      render(<Harness tab="promote" data={docData(two)} />);
      expect(callout()).toBeNull();
    });

    it("結論未寫入時無「討論已收斂」提示框；衍生變更為空時無數字磚與出處行", () => {
      const { unmount } = render(<Harness tab="rounds" data={docData([], { status: "open" })} />);
      expect(callout()!.textContent).toContain("討論已收斂");
      unmount();
      const open = discussionDocData({
        text: record(roundsOf(["interview", "interview", "interview"], ["2026-10-08", "2026-10-09", "2026-10-10"]), "<!-- scaffold -->"),
        status: "open",
        hold: false,
        created: null,
        recordPath: null,
        rows: [],
        names: NAMES,
      });
      const r = render(<Harness tab="rounds" data={open} />);
      expect(callout()).toBeNull();
      expect(tiles()[0]).toEqual(["3", "輪"]);
      expect(provenance()).toBe("2026-10-08 – 2026-10-10·interview 3 輪");
      r.unmount();
      render(<Harness tab="promote" data={open} />);
      expect(document.querySelector("[data-doc-tiles]")).toBeNull();
      expect(provenance()).toBeNull();
    });
  });

  it("記錄切分失敗時三者皆空", () => {
    const data = discussionDocData({
      text: "手寫的自由格式記錄。",
      status: "open",
      hold: false,
      created: "2026-10-08",
      recordPath: "openspec/discussions/demo.md",
      rows: [],
      names: NAMES,
    });
    for (const tab of ["conclusion", "rounds", "context", "record"] as const) {
      const { unmount } = render(<Harness tab={tab} data={data} />);
      expect(document.querySelector("[data-doc-tiles]")).toBeNull();
      expect(provenance()).toBeNull();
      expect(callout()).toBeNull();
      unmount();
    }
  });
});

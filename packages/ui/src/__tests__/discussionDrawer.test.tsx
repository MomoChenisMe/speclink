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

import { DiscussionDrawer, splitDiscussionSections, splitRounds } from "../components/DiscussionDrawer";
import { LABEL_CLS } from "../components/SectionedDoc";
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

describe("splitRounds（輪切分，design D1 行掃描解析 scaffold）", () => {
  it("scaffold 記錄解析出輪陣列：輪次、mode、日期、欄位對應", () => {
    const rounds = splitRounds(ROUNDS_TEXT);
    expect(rounds).not.toBeNull();
    expect(rounds!.length).toBe(2);
    expect(rounds![0].round).toBe(1);
    expect(rounds![0].mode).toBe("assumptions");
    expect(rounds![0].date).toBe("2026-07-08");
    expect(rounds![1].round).toBe(2);
    expect(rounds![1].mode).toBe("interview");
    expect(rounds![1].date).toBe("2026-07-09");
    expect(rounds![0].fields.Focus).toBe("第一輪焦點");
  });

  it("Position 標籤行後的列點多行歸屬 Position 欄位", () => {
    const rounds = splitRounds(ROUNDS_TEXT)!;
    expect(rounds[0].fields.Position).toContain("總綰一句");
    expect(rounds[0].fields.Position).toContain("- 列點甲");
    expect(rounds[0].fields.Position).toContain("- 列點乙");
    expect(rounds[0].fields.Open).toBe("未解之一");
  });

  it("來源缺 Ruled out 時欄位對應無該鍵", () => {
    const rounds = splitRounds(ROUNDS_TEXT)!;
    expect(rounds[0].fields["Ruled out"]).toBeUndefined();
    expect(rounds[1].fields["Ruled out"]).toBe("淘汰項");
  });

  it("非四詞白名單的粗體前綴行按內文歸屬當前欄位（design D2）", () => {
    const rounds = splitRounds(ROUNDS_TEXT)!;
    expect(Object.keys(rounds[1].fields)).not.toContain("Note");
    expect(rounds[1].fields.Position).toContain("**Note**: 這行不是欄位");
  });

  it("任一輪標題不符 scaffold 格式時回 null（整篇退回）", () => {
    expect(splitRounds("### Round 1 — assumptions\n\n**Focus**: 缺日期括號\n")).toBeNull();
    expect(
      splitRounds("### Round 1 — assumptions (2026-07-08)\n\n**Focus**: 好輪\n\n### 附註\n\n手寫段落\n"),
    ).toBeNull();
    expect(splitRounds("手寫的自由格式輪記錄，沒有輪標題。")).toBeNull();
  });

  it("零輪（僅 scaffold 註解與空行）回空陣列", () => {
    expect(
      splitRounds("\n<!-- \\`### Round N — <mode> (<date>)\\` entries are appended here by the CLI. -->\n\n"),
    ).toEqual([]);
  });
});

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
    const result = render(<DiscussionDrawer {...(props as never)} />);
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

  it("欄位以標籤區塊呈現，「**Focus**:」粗體前綴原文不出現", async () => {
    const { baseElement } = await openRoundsTab(CARDS_DOC);
    await waitFor(() => screen.getByText("第一輪焦點"));
    const card1 = baseElement.querySelector('[data-round="1"]') as HTMLElement;
    expect(within(card1).getByText("焦點")).toBeTruthy();
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
});

// spec 需求「討論結論以欄位標籤呈現」（design D7 六詞白名單共用欄位解析）。
function docWithConclusion(conclusion: string): string {
  return DOC.replace("**Decision**: 建置 alpha 搜尋", conclusion);
}

const FULL_CONCLUSION = [
  "**Decision**: 拍板做 A",
  "**Rationale**: 因為證據充分",
  "**Rejected alternatives**: B 案——太貴",
  "**Deferred**: 無",
  "**Capture to**: proposal",
  "**Next**: /speclink-propose --from-discussion alpha-search",
].join("\n");

describe("結論欄位標籤化（討論結論以欄位標籤呈現）", () => {
  it("scaffold 結論六欄位成標籤區塊，粗體前綴原文不出現", async () => {
    const props = makeProps({ loadDocument: vi.fn(async () => docWithConclusion(FULL_CONCLUSION)) });
    render(<DiscussionDrawer {...(props as never)} />);
    // 結論非空 → 預設分頁即結論。
    await waitFor(() => expect(screen.getByText("拍板做 A")).toBeTruthy());
    for (const label of ["決定", "理由", "否決替代案", "擱置", "記錄去向", "下一步"]) {
      expect(screen.getByText(label)).toBeTruthy();
    }
    expect(screen.queryByText("Decision")).toBeNull();
    expect(screen.queryByText("Rationale")).toBeNull();
  });

  it("缺席欄位不渲染空標籤", async () => {
    const doc = docWithConclusion("**Decision**: 只有決定\n**Rationale**: 與理由");
    const props = makeProps({ loadDocument: vi.fn(async () => doc) });
    render(<DiscussionDrawer {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("只有決定")).toBeTruthy());
    expect(screen.getByText("決定")).toBeTruthy();
    expect(screen.getByText("理由")).toBeTruthy();
    expect(screen.queryByText("擱置")).toBeNull();
    expect(screen.queryByText("下一步")).toBeNull();
  });

  // spec「標籤為大標題且字級大於內文」：輪／結論欄位標籤與章節標籤同一款式常數（design D6）。
  it("輪與結論欄位標籤為粗體大標題款式（與章節標籤同源）", async () => {
    const props = makeProps({ loadDocument: vi.fn(async () => docWithConclusion(FULL_CONCLUSION)) });
    render(<DiscussionDrawer {...(props as never)} />);
    await waitFor(() => expect(screen.getByText("決定")).toBeTruthy());
    const conclusionLabel = screen.getByText("決定");
    for (const cls of LABEL_CLS.split(" ")) expect(conclusionLabel.className).toContain(cls);
    expect(conclusionLabel.className).not.toContain("text-xs");
    fireEvent.mouseDown(screen.getByRole("tab", { name: /討論過程/ }));
    await waitFor(() => expect(screen.getByText("焦點")).toBeTruthy());
    const roundLabel = screen.getByText("焦點");
    for (const cls of LABEL_CLS.split(" ")) expect(roundLabel.className).toContain(cls);
    expect(roundLabel.className).not.toContain("uppercase");
  });

  it("自由格式結論整篇以單一 markdown 檢視退回", async () => {
    const props = makeProps({
      loadDocument: vi.fn(async () => docWithConclusion("就這樣定了，大家都同意這個方向。")),
    });
    render(<DiscussionDrawer {...(props as never)} />);
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

  it("建立者圓標為中性，不佔用主色（五處頭像同款）", async () => {
    const withAuthor = { ...concludedD, createdBy: "Momo Chen" };
    render(<DiscussionDrawer {...(makeProps({ discussion: withAuthor }) as never)} />);
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
    // 先等文件載入落定（section 分頁組掛上、背景分頁出現）再切衍生變更：否則
    // loadDocument resolve 會把 fallback 分頁組換成 sections 分頁組（key 變、重新
    // mount），選中分頁重置回預設，衍生變更面板被隱藏、cut-a 從 DOM 消失。與本檔
    // 其他分頁測試一致的等待。
    await screen.findByRole("tab", { name: /背景/ });
    fireEvent.mouseDown(screen.getByRole("tab", { name: /衍生變更/ }));
    const rowA = screen.getByText("cut-a").closest("[data-promoted-row]") as HTMLElement;
    expect(within(rowA).getByText("提案中")).toBeTruthy();
    fireEvent.click(within(rowA).getByRole("button", { name: /開啟卡片/ }));
    expect(props.onOpenChangeCard).toHaveBeenCalledWith("cut-a");
    const rowGone = screen.getByText("cut-gone").closest("[data-promoted-row]") as HTMLElement;
    expect(within(rowGone).getByText("已刪除")).toBeTruthy();
    expect(within(rowGone).queryByRole("button", { name: /開啟卡片/ })).toBeNull();
  });

  it("衍生變更分頁唯讀：concluded／promoted 皆無「轉為變更／再轉出一個變更」動作（D3）", async () => {
    // promote 已自 GUI 撤除——衍生變更分頁只列子變更與跳轉，無任何轉出鈕。
    // 各實例先等初始載入落地（標準格式解析出「背景」分頁），避免 act 警告。
    const { unmount } = render(<DiscussionDrawer {...(makeProps() as never)} />);
    await screen.findByRole("tab", { name: /背景/ });
    fireEvent.mouseDown(screen.getByRole("tab", { name: /衍生變更/ }));
    expect(screen.queryByRole("button", { name: /轉為變更/ })).toBeNull();
    expect(screen.queryByRole("button", { name: /促轉/ })).toBeNull();
    unmount();

    const { unmount: unmount2 } = render(
      <DiscussionDrawer {...(makeProps({ discussion: promotedD }) as never)} />,
    );
    await screen.findByRole("tab", { name: /背景/ });
    fireEvent.mouseDown(screen.getByRole("tab", { name: /衍生變更/ }));
    expect(screen.queryByRole("button", { name: /再轉出一個變更/ })).toBeNull();
    expect(screen.queryByRole("button", { name: /轉為變更/ })).toBeNull();
    unmount2();

    const props3 = makeProps({ discussion: openD, loadDocument: vi.fn(async () => OPEN_DOC) });
    render(<DiscussionDrawer {...(props3 as never)} />);
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

  it("change 抽屜列出全部來源討論與同源清單並可互跳", async () => {
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
      <RichDetailDrawer
        open
        onOpenChange={vi.fn()}
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

// spec 需求「討論抽屜檢視與轉出變更」：抽屜標題以 slug 為題（等寬）＋複製鈕、
// topic 降為副標（design D4，LANGUAGE.md 受控例外擴充）。
describe("DiscussionDrawer 標題 slug 化（design D4）", () => {
  it("標題呈 slug（等寬字型）、topic 降為副標、複製鈕寫入剪貼簿", async () => {
    const writeText = vi.fn();
    Object.assign(navigator, { clipboard: { writeText } });
    render(<DiscussionDrawer {...(makeProps() as never)} />);
    await screen.findByRole("tab", { name: /背景/ });
    const title = screen.getByText("alpha-search");
    expect(title.className).toContain("font-mono");
    expect(screen.getByText("Alpha search")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "複製 slug" }));
    expect(writeText).toHaveBeenCalledWith("alpha-search");
  });
});

// spec 需求「markdown 文件內容行寬有上限」（design D4）：討論抽屜分頁內容的捲動
// 容器內存在共用置中容器，討論輪卡片與內文同欄對齊置中。
describe("討論抽屜閱讀欄置中", () => {
  it("捲動容器內有置中容器（w-full＋max-w-[96ch]＋mx-auto）且輪卡片在欄內", async () => {
    const { baseElement } = render(<DiscussionDrawer {...(makeProps() as never)} />);
    await screen.findByRole("tab", { name: /討論過程/ });
    const col = baseElement.querySelector("[data-reading-column]") as HTMLElement;
    expect(col).toBeTruthy();
    expect(col.className).toContain("w-full");
    expect(col.className).toContain("max-w-[96ch]");
    expect(col.className).toContain("mx-auto");
    expect(col.parentElement?.className).toContain("overflow-y-auto");
    // 結論分頁（預設）內文在欄內。
    expect(within(col).getByText("建置 alpha 搜尋")).toBeTruthy();
    // 討論過程分頁：輪卡片同欄。
    fireEvent.mouseDown(screen.getByRole("tab", { name: /討論過程/ }));
    expect(col.querySelector("[data-round]")).toBeTruthy();
  });
});

// spec「討論抽屜檢視與轉出變更」抽屜內封存場景:concluded 且未封存(宿主提供
// handler)才出現;走與討論卡同一 onArchiveDiscussion 呼叫(確認流程在宿主)。
describe("抽屜內封存動詞", () => {
  it("concluded 討論顯示封存動作,點擊觸發 onArchiveDiscussion", async () => {
    const onArchiveDiscussion = vi.fn();
    render(<DiscussionDrawer {...(makeProps({ onArchiveDiscussion }) as never)} />);
    const btn = await screen.findByRole("button", { name: /封存/ });
    fireEvent.click(btn);
    expect(onArchiveDiscussion).toHaveBeenCalledWith("alpha-search");
  });

  it("非 concluded(open/promoted)不顯示封存動作", async () => {
    render(
      <DiscussionDrawer
        {...(makeProps({ discussion: openD, onArchiveDiscussion: vi.fn() }) as never)}
      />,
    );
    await screen.findByText("Alpha search");
    expect(screen.queryByRole("button", { name: /封存/ })).toBeNull();

    render(
      <DiscussionDrawer
        {...(makeProps({ discussion: promotedD, onArchiveDiscussion: vi.fn() }) as never)}
      />,
    );
    expect(screen.queryByRole("button", { name: /封存/ })).toBeNull();
  });

  it("已封存(宿主不提供 handler)不顯示封存動作", async () => {
    render(<DiscussionDrawer {...(makeProps() as never)} />);
    await screen.findByText("Alpha search");
    expect(screen.queryByRole("button", { name: /封存/ })).toBeNull();
  });
});

// spec「抽屜文件載入以 skeleton 呈現」（design D3）：討論抽屜載入中畫骨架。
describe("討論抽屜文件三態", () => {
  it("載入中 → 文件骨架，不出空態文案", async () => {
    render(
      <DiscussionDrawer
        {...(makeProps({ loadDocument: vi.fn(() => new Promise<never>(() => {})) }) as never)}
      />,
    );
    await waitFor(() => expect(document.querySelector('[aria-busy="true"]')).toBeTruthy());
    expect(screen.queryByText("（無內容）")).toBeNull();
  });

  it("載入完成且記錄不存在 → 空態文案，無骨架", async () => {
    render(<DiscussionDrawer {...(makeProps({ loadDocument: vi.fn(async () => null) }) as never)} />);
    await waitFor(() => expect(screen.getByText("（無內容）")).toBeTruthy());
    expect(document.querySelector('[aria-busy="true"]')).toBeNull();
  });
});

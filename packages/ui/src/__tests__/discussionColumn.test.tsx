import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, fireEvent, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider, MESSAGES } from "../i18n";

// 既有中文斷言包 I18nProvider locale zh-TW 後照舊斷言（i18n 抽 key 的回歸保護）。
const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

import { KanbanBoard } from "../components/KanbanBoard";
import { DiscussionColumn, discussionChipStage } from "../components/DiscussionColumn";
import type { ChangeItem, ArchivedItem, DiscussionItem, DiscussionLists } from "../adapter";

// spec 需求「討論於看板第 0 欄兩級呈現」的 jsdom 可驗部分。

const openD: DiscussionItem = {
  slug: "open-topic",
  topic: "Open topic",
  status: "open",
  rounds: 3,
  created: "2026-07-01",
  promotedTo: [],
};
const concludedD: DiscussionItem = {
  slug: "settled",
  topic: "Settled topic",
  status: "concluded",
  rounds: 5,
  created: "2026-07-02",
  promotedTo: [],
};
const promotedD: DiscussionItem = {
  slug: "fanout",
  topic: "Fanout topic",
  status: "promoted",
  rounds: 4,
  created: "2026-07-03",
  promotedTo: ["cut-a", "cut-b", "cut-gone"],
};

// spec Example「chip 階段派生矩陣」的 active 清單值。
const chipChanges: ChangeItem[] = [
  { name: "cut-a", status: "in-progress", totalTasks: 24, completedTasks: 0 },
  { name: "cut-b", status: "in-progress", totalTasks: 24, completedTasks: 13, startedAt: "2026-07-05" },
  { name: "cut-ready", status: "done", totalTasks: 24, completedTasks: 24 },
];
const chipArchived: ArchivedItem[] = [
  { datedName: "2026-07-05-cut-arch", date: "2026-07-05", name: "cut-arch" },
];

function column(id: string): HTMLElement | null {
  return document.querySelector(`[data-column="${id}"]`);
}

describe("DiscussionColumn 拖排（design D6）", () => {
  it("sortable 開啟時全卡掛 sortable 與拖曳標籤；promoted 收合列不可拖", () => {
    render(
      <DiscussionColumn
        discussions={[openD, concludedD, promotedD]}
        changes={chipChanges}
        archived={chipArchived}
        sortable
      />,
    );
    const openCard = screen
      .getByText("Open topic")
      .closest('[aria-roledescription="sortable"]') as HTMLElement;
    expect(openCard).toBeTruthy();
    expect(openCard.getAttribute("aria-label")).toContain("Open topic");
    // D1：promoted 預設隱藏，開啟 header 開關才見衍生樹列；其列不參與拖排。
    fireEvent.click(screen.getByRole("button", { name: /顯示已轉出/ }));
    expect(
      screen.getByText("Fanout topic").closest('[aria-roledescription="sortable"]'),
    ).toBeNull();
  });

  it("未開 sortable 時卡片不掛 sortable（既有獨立渲染不受影響）", () => {
    render(<DiscussionColumn discussions={[openD]} changes={[]} archived={[]} />);
    expect(
      screen.getByText("Open topic").closest('[aria-roledescription="sortable"]'),
    ).toBeNull();
  });
});

describe("DiscussionColumn（兩級呈現）", () => {
  it("open 討論為全卡：topic＋「N 輪」文案，無任何動詞按鈕", () => {
    render(
      <DiscussionColumn discussions={[openD]} changes={[]} archived={[]} />,
    );
    const card = screen.getByText("Open topic").closest("[data-discussion]") as HTMLElement;
    expect(card).toBeTruthy();
    expect(within(card).getByText(/3 輪/)).toBeTruthy();
    expect(within(card).queryByText(/回合/)).toBeNull();
    expect(within(card).queryByRole("button")).toBeNull();
  });

  it("concluded 討論為全卡帶「封存」按鈕但無「轉為變更」；舊詞「促轉」「歸檔」不出現", () => {
    // D3：promote 已自 GUI 撤除——concluded 卡僅保留封存動詞，轉為變更改由 CLI／agent。
    const onArchiveDiscussion = vi.fn();
    render(
      <DiscussionColumn
        discussions={[concludedD]}
        changes={[]}
        archived={[]}
        onArchiveDiscussion={onArchiveDiscussion}
      />,
    );
    const card = screen.getByText("Settled topic").closest("[data-discussion]") as HTMLElement;
    fireEvent.click(within(card).getByRole("button", { name: /封存/ }));
    expect(onArchiveDiscussion).toHaveBeenCalledWith("settled");
    // 轉為變更（promote）已撤除。
    expect(within(card).queryByRole("button", { name: /轉為變更/ })).toBeNull();
    expect(within(card).queryByRole("button", { name: /促轉/ })).toBeNull();
    expect(within(card).queryByRole("button", { name: /歸檔/ })).toBeNull();
  });

  it("點全卡開啟討論（動詞按鈕除外）", () => {
    const onOpenDiscussion = vi.fn();
    render(
      <DiscussionColumn
        discussions={[openD]}
        changes={[]}
        archived={[]}
        onOpenDiscussion={onOpenDiscussion}
      />,
    );
    fireEvent.click(screen.getByText("Open topic"));
    expect(onOpenDiscussion).toHaveBeenCalledWith("open-topic");
  });

  it("promoted 討論收合為衍生樹細列：topic 首行、樹狀子項帶階段、slug 不出現、點列開啟", () => {
    const onOpenDiscussion = vi.fn();
    render(
      <DiscussionColumn
        discussions={[openD, promotedD]}
        changes={chipChanges}
        archived={chipArchived}
        onOpenDiscussion={onOpenDiscussion}
      />,
    );
    // D1：promoted 預設隱藏，先點 header「顯示已轉出」開關才切到已轉出檢視。
    fireEvent.click(screen.getByRole("button", { name: /顯示已轉出/ }));
    // 欄標題換為「已轉出討論」（不再於內容顯示群組標籤列）；細列首行為 topic。
    expect(screen.getByText("已轉出討論")).toBeTruthy();
    expect(screen.queryByText("已轉出變更的討論")).toBeNull();
    expect(screen.queryByText(/已促轉/)).toBeNull();
    expect(screen.queryByText("fanout")).toBeNull();
    const row = screen.getByText("Fanout topic").closest("[data-discussion]") as HTMLElement;
    // 樹狀前綴：末列 └、其餘 ├（三個子變更 → 2 個 ├、1 個 └）。
    expect(within(row).getAllByText("├")).toHaveLength(2);
    expect(within(row).getAllByText("└")).toHaveLength(1);
    expect(within(row).getByText("cut-a")).toBeTruthy();
    expect(within(row).getByText("cut-b")).toBeTruthy();
    expect(within(row).getByText("cut-gone")).toBeTruthy();
    // 子項帶階段標示：cut-a 提案中、cut-b 進行中、cut-gone 已刪除。
    expect(within(row).getByText("提案中")).toBeTruthy();
    expect(within(row).getByText("進行中")).toBeTruthy();
    expect(within(row).getByText("已刪除")).toBeTruthy();
    fireEvent.click(screen.getByText("Fanout topic"));
    expect(onOpenDiscussion).toHaveBeenCalledWith("fanout");
  });

  it("空清單顯示欄空狀態", () => {
    render(<DiscussionColumn discussions={[]} changes={[]} archived={[]} />);
    expect(screen.getByText("尚無討論")).toBeTruthy();
  });
});

describe("DiscussionColumn header 顯示已轉出開關（design D1）", () => {
  it("開關互斥切換：關閉只顯示討論中、開啟只顯示已轉出（衍生樹保留、各從欄頂排列）", () => {
    render(
      <DiscussionColumn
        discussions={[openD, promotedD]}
        changes={chipChanges}
        archived={chipArchived}
      />,
    );
    // header 開關存在且帶 promoted 計數（一筆 promoted → 1）。
    const toggle = screen.getByRole("button", { name: /顯示已轉出/ });
    expect(within(toggle).getByText("1")).toBeTruthy();
    // 預設（關閉）：只顯示討論中（open 全卡），已轉出隱藏、零佔位。
    expect(screen.getByText("Open topic")).toBeTruthy();
    expect(screen.queryByText("Fanout topic")).toBeNull();
    // 開啟：欄標題換為「已轉出討論」、只顯示已轉出衍生樹，討論中暫時隱藏。
    fireEvent.click(toggle);
    expect(screen.getByText("已轉出討論")).toBeTruthy();
    expect(screen.queryByText("已轉出變更的討論")).toBeNull();
    expect(screen.getByText("Fanout topic")).toBeTruthy();
    expect(screen.queryByText("Open topic")).toBeNull();
    // 再點 → 回到只顯示討論中。
    fireEvent.click(toggle);
    expect(screen.getByText("Open topic")).toBeTruthy();
    expect(screen.queryByText("Fanout topic")).toBeNull();
  });

  it("無 promoted 討論時 header 開關缺席", () => {
    render(<DiscussionColumn discussions={[openD]} changes={[]} archived={[]} />);
    expect(screen.queryByRole("button", { name: /顯示已轉出/ })).toBeNull();
  });
});

describe("DiscussionColumn 計數只算 active 與空狀態（design D3）", () => {
  it("欄計數徽章隨檢視：討論中檢視顯 active 數、已轉出檢視顯 promoted 數", () => {
    render(
      <DiscussionColumn
        discussions={[openD, concludedD, promotedD]}
        changes={chipChanges}
        archived={chipArchived}
      />,
    );
    // 討論中檢視（預設）：active 2（open＋concluded）。
    expect(screen.getByTestId("column-count").textContent).toBe("2");
    // 切到已轉出檢視：徽章改顯 promoted 數 1（與標題「已轉出討論」一致）。
    fireEvent.click(screen.getByRole("button", { name: /顯示已轉出/ }));
    expect(screen.getByTestId("column-count").textContent).toBe("1");
  });

  it("無 active 但有 promoted 時欄體不顯「尚無討論」，計數為 0", () => {
    render(
      <DiscussionColumn
        discussions={[promotedD]}
        changes={chipChanges}
        archived={chipArchived}
      />,
    );
    expect(screen.queryByText("尚無討論")).toBeNull();
    expect(screen.getByTestId("column-count").textContent).toBe("0");
  });
});

describe("DiscussionColumn promoted chip 階段配色（design D2）", () => {
  it("chip 沿看板 STAGE_STYLE 配色：提案中/進行中/已就緒 teal 濃度、已封存中性、已刪除 destructive 加刪除線", () => {
    // 五態各一子變更：提案中/進行中/已就緒（active 清單）、已封存、已刪除。
    const d: DiscussionItem = {
      ...promotedD,
      promotedTo: ["cut-a", "cut-b", "cut-ready", "cut-arch", "cut-gone"],
    };
    render(<DiscussionColumn discussions={[d]} changes={chipChanges} archived={chipArchived} />);
    fireEvent.click(screen.getByRole("button", { name: /顯示已轉出/ }));
    const chipCls = (label: string) => screen.getByText(label).className;
    // 提案中/進行中沿 STAGE_STYLE badge 的 teal 濃度階梯。
    expect(chipCls("提案中")).toContain("bg-primary/8");
    expect(chipCls("進行中")).toContain("bg-primary/12");
    // 已就緒為實心主色（以 text-primary-foreground 辨識，與濃度階梯區隔）。
    expect(chipCls("已就緒")).toContain("text-primary-foreground");
    // 已封存中性色。
    expect(chipCls("已封存")).toContain("bg-muted");
    // 已刪除 destructive 加刪除線。
    expect(chipCls("已刪除")).toContain("text-destructive");
    expect(chipCls("已刪除")).toContain("line-through");
  });
});

describe("discussionChipStage（spec Example「chip 階段派生矩陣」）", () => {
  // | promoted_to 子 change 的所在 | chip 標示 |
  const rows: Array<[string, string]> = [
    ["cut-a", "提案中"], // active 清單，無 started、0/24
    ["cut-b", "進行中"], // active 清單，有 started、13/24
    ["cut-ready", "已就緒"], // active 清單，24/24
    ["cut-arch", "已封存"], // 封存清單（dated name 尾碼命中）
    ["cut-gone", "已刪除"], // 兩清單皆無（討論維持已促轉）
  ];
  // 函式回傳 i18n key；斷言值仍為 spec Example 的 zh-TW 標示（經字典解析）。
  it.each(rows)("%s → %s", (name, label) => {
    const key = discussionChipStage(name, chipChanges, chipArchived);
    expect(MESSAGES["zh-TW"][key]).toBe(label);
  });
});

describe("KanbanBoard 四欄整合", () => {
  const lists: DiscussionLists = {
    active: [openD, concludedD],
    archived: [
      {
        slug: "done-topic",
        topic: "Done archived topic",
        status: "promoted",
        rounds: 2,
        created: "2026-06-01",
        promotedTo: ["old-cut"],
      },
    ],
  };
  const changes: ChangeItem[] = [
    { name: "working-y", status: "in-progress", totalTasks: 10, completedTasks: 4, startedAt: "2026-07-06" },
  ];

  it("討論欄為第 0 欄（最左），與既有三欄並列", () => {
    render(<KanbanBoard changes={changes} discussions={lists} />);
    const discussionsCol = column("discussions");
    expect(discussionsCol).toBeTruthy();
    expect(column("proposed")).toBeTruthy();
    // 第 0 欄在 DOM 序位於 proposed 之前。
    const cols = Array.from(document.querySelectorAll("[data-column]")).map((el) =>
      el.getAttribute("data-column"),
    );
    expect(cols.indexOf("discussions")).toBeLessThan(cols.indexOf("proposed"));
    expect(within(discussionsCol as HTMLElement).getByText("Open topic")).toBeTruthy();
  });

  it("封存討論不出現於討論欄", () => {
    render(<KanbanBoard changes={changes} discussions={lists} />);
    expect(screen.queryByText("Done archived topic")).toBeNull();
  });

  it("無討論的專案顯示欄空狀態", () => {
    render(
      <KanbanBoard changes={changes} discussions={{ active: [], archived: [] }} />,
    );
    expect(within(column("discussions") as HTMLElement).getByText("尚無討論")).toBeTruthy();
  });

  it("未傳 discussions 時維持三欄（向後相容）", () => {
    render(<KanbanBoard changes={changes} />);
    expect(column("discussions")).toBeNull();
  });
});

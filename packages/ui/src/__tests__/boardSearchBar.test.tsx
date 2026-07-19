// spec 需求「看板搜尋過濾卡片」的搜尋列（design D5）：搜尋圖示、輸入非空時的
// 清除鈕（清空後保持聚焦）與即時命中數、Cmd+F／Ctrl+F 聚焦快捷鍵。
import { describe, it, expect } from "vitest";
import { render as rtlRender, screen, fireEvent } from "@testing-library/react";
import { useState, type ReactElement, type ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { KanbanBoard } from "../components/KanbanBoard";
import type { ChangeItem, DiscussionLists } from "../adapter";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

const changes: ChangeItem[] = [
  {
    name: "engine-typed-core",
    status: "in-progress",
    totalTasks: 18,
    completedTasks: 0,
    summary: "typed 命令層",
    createdBy: "Momo Chen <momo@example.com>",
    fromDiscussions: ["collab"],
  },
  {
    name: "web-role-views",
    status: "in-progress",
    totalTasks: 5,
    completedTasks: 5,
    summary: "情境 1",
    createdBy: "Ann <ann@example.com>",
  },
];
const discussions: DiscussionLists = {
  active: [
    { slug: "board-search", topic: "看板搜尋", status: "open", rounds: 1, created: "2026-07-10", promotedTo: [] },
    {
      slug: "collab",
      topic: "多人協作",
      status: "promoted",
      rounds: 9,
      created: "2026-07-01",
      promotedTo: ["engine-typed-core"],
    },
  ],
  archived: [],
};

/** 受控 query 的宿主——模擬 App 的 store 接線。 */
function Host() {
  const [q, setQ] = useState("");
  return <KanbanBoard changes={changes} discussions={discussions} query={q} onQuery={setQ} />;
}

describe("BoardSearchBar（搜尋列，design D5）", () => {
  it("空 query 無清除鈕與命中數；輸入後顯示；點清除還原全量且輸入保持聚焦", () => {
    render(<Host />);
    expect(screen.queryByRole("button", { name: "清除搜尋" })).toBeNull();
    const input = screen.getByPlaceholderText("搜尋看板卡片…");
    fireEvent.change(input, { target: { value: "engine" } });
    // 命中數＝過濾後卡片總數（engine-typed-core 1 張）。
    expect(screen.getByText("1 張卡")).toBeTruthy();
    expect(screen.queryByText("web-role-views")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "清除搜尋" }));
    expect((input as HTMLInputElement).value).toBe("");
    expect(document.activeElement).toBe(input);
    expect(screen.getByText("web-role-views")).toBeTruthy();
  });

  it("Cmd+F（或 Ctrl+F）聚焦搜尋輸入", () => {
    render(<Host />);
    const input = screen.getByPlaceholderText("搜尋看板卡片…");
    expect(document.activeElement).not.toBe(input);
    fireEvent.keyDown(window, { key: "f", metaKey: true });
    expect(document.activeElement).toBe(input);
  });

  it("搜尋圖示呈現於輸入框內", () => {
    render(<Host />);
    expect(document.querySelector("svg.lucide-search")).toBeTruthy();
  });
});

// spec 需求「看板搜尋過濾卡片」的篩選面板（design D5）：收於漏斗開關的彈出面板、
// 三維度選單、與搜尋字串 AND 交集、可單獨清除與全部清除、關閉不清除過濾。
describe("篩選面板（design D5）", () => {
  it("面板預設關閉；點篩選鈕彈出三維度；關閉後過濾持續且鈕帶啟用計數", () => {
    render(<Host />);
    // 預設：單列只有搜尋框與篩選鈕，面板不佔版面。
    expect(screen.queryByLabelText("建立者")).toBeNull();
    const toggle = screen.getByRole("button", { name: "篩選" });
    expect(toggle.getAttribute("aria-expanded")).toBe("false");
    // 彈出：三個維度選單可見。
    fireEvent.click(toggle);
    expect(toggle.getAttribute("aria-expanded")).toBe("true");
    expect(screen.getByLabelText("建立者")).toBeTruthy();
    expect(screen.getByLabelText("建立時間")).toBeTruthy();
    expect(screen.getByLabelText("來源討論")).toBeTruthy();
    // 啟用一個維度後關閉面板：面板消失、過濾持續、鈕帶計數。
    fireEvent.change(screen.getByLabelText("建立時間"), { target: { value: "7d" } });
    fireEvent.click(toggle);
    expect(screen.queryByLabelText("建立時間")).toBeNull();
    expect(toggle.textContent).toContain("1");
    // 過濾持續生效：變更卡無 created → 被近 7 天窗過濾。
    expect(document.querySelector('[data-change="engine-typed-core"]')).toBeNull();
  });

  it("按 Esc 關閉面板", () => {
    render(<Host />);
    fireEvent.click(screen.getByRole("button", { name: "篩選" }));
    expect(screen.getByLabelText("建立者")).toBeTruthy();
    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByLabelText("建立者")).toBeNull();
  });

  it("建立者篩選過濾卡片；與搜尋字串 AND 交集；選回全部即單獨清除", () => {
    render(<Host />);
    fireEvent.click(screen.getByRole("button", { name: "篩選" }));
    fireEvent.change(screen.getByLabelText("建立者"), {
      target: { value: "Momo Chen <momo@example.com>" },
    });
    expect(screen.getByText("engine-typed-core")).toBeTruthy();
    expect(screen.queryByText("web-role-views")).toBeNull();
    // AND 交集：query 命中另一張 → 兩張皆不可見。
    const input = screen.getByPlaceholderText("搜尋看板卡片…");
    fireEvent.change(input, { target: { value: "web" } });
    expect(document.querySelector('[data-change="engine-typed-core"]')).toBeNull();
    expect(document.querySelector('[data-change="web-role-views"]')).toBeNull();
    // 單獨清除（選回全部）：回到僅以字串過濾（卡名經高亮 mark 拆分，改以 data-change 查詢）。
    fireEvent.change(screen.getByLabelText("建立者"), { target: { value: "" } });
    expect(document.querySelector('[data-change="web-role-views"]')).toBeTruthy();
  });

  it("面板內全部清除還原所有維度", () => {
    render(<Host />);
    const toggle = screen.getByRole("button", { name: "篩選" });
    fireEvent.click(toggle);
    fireEvent.change(screen.getByLabelText("建立者"), {
      target: { value: "Momo Chen <momo@example.com>" },
    });
    fireEvent.change(screen.getByLabelText("建立時間"), { target: { value: "7d" } });
    fireEvent.click(screen.getByRole("button", { name: "清除全部篩選" }));
    expect(document.querySelector('[data-change="web-role-views"]')).toBeTruthy();
    expect(document.querySelector('[data-change="engine-typed-core"]')).toBeTruthy();
    expect(toggle.textContent).not.toContain("2");
  });

  it("來源討論篩選：顯示該討論卡自身與其衍生變更卡", () => {
    render(<Host />);
    fireEvent.click(screen.getByRole("button", { name: "篩選" }));
    fireEvent.change(screen.getByLabelText("來源討論"), { target: { value: "collab" } });
    expect(screen.getByText("engine-typed-core")).toBeTruthy();
    expect(screen.queryByText("web-role-views")).toBeNull();
    // 非來源的 open 討論卡也被過濾。
    expect(screen.queryByText("board-search")).toBeNull();
    // 該討論自身（promoted）計入欄底收合列。
    expect(screen.getByRole("button", { name: /已轉出/ }).textContent).toContain("1");
  });

  it("建立時間篩選提供三窗選項", () => {
    render(<Host />);
    fireEvent.click(screen.getByRole("button", { name: "篩選" }));
    const sel = screen.getByLabelText("建立時間") as HTMLSelectElement;
    const labels = Array.from(sel.options).map((o) => o.textContent);
    expect(labels).toContain("近 7 天");
    expect(labels).toContain("近 30 天");
    expect(labels).toContain("更早");
  });
});

// spec 需求「看板搜尋過濾卡片」的全文與模糊比對層（design D6/D7）。
describe("全文命中與名稱層模糊比對併入可見集合", () => {
  it("欄位不命中但全文命中的卡片顯示；欄位與全文皆不命中者隱藏", () => {
    render(
      <KanbanBoard
        changes={changes}
        discussions={discussions}
        query="dispatch"
        onQuery={() => {}}
        fulltextHits={[
          { kind: "change", id: "web-role-views", artifact: "design.md", snippet: "…唯一 dispatch 相容層…" },
        ]}
      />,
    );
    expect(screen.getByText("web-role-views")).toBeTruthy();
    expect(screen.queryByText("engine-typed-core")).toBeNull();
  });

  it("名稱層 subsequence 模糊命中（etc → engine-typed-core）", () => {
    render(
      <KanbanBoard changes={changes} discussions={discussions} query="etc" onQuery={() => {}} />,
    );
    expect(screen.getByText("engine-typed-core")).toBeTruthy();
    expect(screen.queryByText("web-role-views")).toBeNull();
  });

  it("searchUnavailableReason 停用搜尋輸入並附說明（remote capability 缺口）", () => {
    render(
      <KanbanBoard
        changes={changes}
        discussions={discussions}
        query=""
        onQuery={() => {}}
        searchUnavailableReason="此 server 尚未提供全文搜尋——功能已停用"
      />,
    );
    const input = screen.getByPlaceholderText("搜尋看板卡片…") as HTMLInputElement;
    expect(input.disabled).toBe(true);
    expect(input.title).toBe("此 server 尚未提供全文搜尋——功能已停用");
  });

  it("未提供 searchUnavailableReason 時搜尋照常可用（本地不受影響）", () => {
    render(
      <KanbanBoard changes={changes} discussions={discussions} query="" onQuery={() => {}} />,
    );
    const input = screen.getByPlaceholderText("搜尋看板卡片…") as HTMLInputElement;
    expect(input.disabled).toBe(false);
  });
});

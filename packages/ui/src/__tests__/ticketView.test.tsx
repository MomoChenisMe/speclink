import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, fireEvent, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider, MESSAGES } from "../i18n";
import { TicketView, ticketSummary } from "../components/TicketView";
import type { StationTicket, TicketRound } from "../adapter";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

const text = (el: Element | null) => (el?.textContent ?? "").replace(/\s+/g, " ").trim();
const HASH = `sha256:${"a".repeat(64)}`;
const SCOPE = ["src/a.rs", "src/b.rs", "src/c.rs"];

// spec desktop-app Scenario「審查工單分頁出現並結構化呈現」的工單：Round 1 首輪
// 兩條 WARNING、Round 2 複驗一條 WARNING 結尾 (accepted) 與一條 SUGGESTION。
const TWO_ROUNDS: StationTicket = {
  rounds: [
    {
      index: 1,
      phase: "discovery",
      patchHash: HASH,
      scope: SCOPE,
      findings: [
        { severity: "WARNING", path: "src/a.rs", text: "Correctness: 未處理空清單" },
        { severity: "WARNING", path: "src/b.rs", text: "Standards: 命名不一致" },
      ],
    },
    {
      index: 2,
      phase: "validation",
      patchHash: HASH,
      scope: SCOPE,
      findings: [
        { severity: "WARNING", path: "src/b.rs", text: "Standards: 命名不一致 (accepted)" },
        { severity: "SUGGESTION", path: "src/c.rs", text: "Style: 可簡化" },
      ],
    },
  ],
};

const t = (key: string) => MESSAGES["zh-TW"][key] ?? key;
/** 閱讀卡標題區的推導（籤槽文字與數字磚），取代舊的段標題列。 */
const summaryOf = (station: "review" | "verify", ticket: StationTicket) => {
  const summary = ticketSummary(station, ticket, t);
  const badges = text(render(<>{summary.badges}</>).container);
  return { badges, tiles: summary.tiles.map((x) => `${x.label} ${x.value}`), callout: summary.callout };
};

const round = (container: HTMLElement, index: number) =>
  container.querySelector(`[data-ticket-round="${index}"]`) as HTMLElement;
const roundToggle = (el: HTMLElement) =>
  el.querySelector("[data-ticket-round-toggle]") as HTMLButtonElement;

describe("TicketView（design D4 呈現規則）", () => {
  it("籤槽為末輪輪數與階段詞，數字磚為輪數與末輪三級計數；內文不另畫段標題列", () => {
    const { container } = render(<TicketView station="review" ticket={TWO_ROUNDS} />);
    expect(container.querySelector("[data-ticket-header]")).toBeNull();
    const summary = summaryOf("review", TWO_ROUNDS);
    expect(summary.badges).toBe("第 2 輪複驗");
    expect(summary.tiles).toEqual(["輪 2", "CRITICAL（末輪） 0", "WARNING（末輪） 1", "SUGGESTION（末輪） 1"]);
    expect(summary.callout).toBeNull();
  });

  it("工單未載入或不存在時籤、磚與提示框皆空", () => {
    expect(ticketSummary("verify", undefined, t)).toEqual({ badges: null, tiles: [], callout: null });
    expect(ticketSummary("verify", null, t)).toEqual({ badges: null, tiles: [], callout: null });
  });

  it("末輪預設展開、舊輪收合，收合輪的標題含階段詞、範圍檔數與條數", () => {
    const { container } = render(<TicketView station="review" ticket={TWO_ROUNDS} />);
    const r1 = round(container, 1);
    const r2 = round(container, 2);
    expect(roundToggle(r2).getAttribute("aria-expanded")).toBe("true");
    expect(roundToggle(r1).getAttribute("aria-expanded")).toBe("false");
    expect(text(r1.querySelector("[data-ticket-round-header]"))).toContain("第 1 輪 · 首輪 · 範圍 3 檔 · 2 條");
    expect(text(r2.querySelector("[data-ticket-round-header]"))).toContain("第 2 輪 · 複驗 · 範圍 3 檔 · 2 條");
    // 展開輪列出兩條 finding；收合輪不列。
    expect(r2.querySelectorAll("[data-ticket-finding]")).toHaveLength(2);
    expect(r1.querySelectorAll("[data-ticket-finding]")).toHaveLength(0);
    // 點擊收合輪的標題即展開。
    fireEvent.click(roundToggle(r1));
    expect(roundToggle(r1).getAttribute("aria-expanded")).toBe("true");
    expect(r1.querySelectorAll("[data-ticket-finding]")).toHaveLength(2);
  });

  it("finding 列有嚴重度色章、等寬路徑與原文描述", () => {
    const { container } = render(<TicketView station="review" ticket={TWO_ROUNDS} />);
    const rows = round(container, 2).querySelectorAll("[data-ticket-finding]");
    expect(text(rows[1].querySelector("[data-ticket-severity]"))).toBe("SUGGESTION");
    expect(text(rows[1].querySelector("[data-ticket-path]"))).toBe("src/c.rs");
    expect(rows[1].querySelector("[data-ticket-path]")?.className).toMatch(/font-mono/);
    expect(text(rows[1].querySelector("[data-ticket-text]"))).toBe("Style: 可簡化");
  });

  it("展開的輪先列 scope 路徑（原樣、等寬）再列 findings；收合輪不列", () => {
    const { container } = render(<TicketView station="review" ticket={TWO_ROUNDS} />);
    const r2 = round(container, 2);
    expect(round(container, 1).querySelectorAll("[data-ticket-scope-path]")).toHaveLength(0);
    // 第 2 輪的 findings 只碰 b.rs／c.rs——a.rs 只會來自 scope 清單。
    const scopeItem = within(r2).getByText("src/a.rs");
    expect(scopeItem.className).toMatch(/font-mono/);
    expect(r2.querySelectorAll("[data-ticket-scope-path]")).toHaveLength(3);
  });

  it("零 findings 的輪顯示「本輪無發現」", () => {
    const empty: StationTicket = {
      rounds: [{ index: 3, phase: "validation", patchHash: HASH, scope: ["src/a.rs"], findings: [] }],
    };
    const { container } = render(<TicketView station="verify" ticket={empty} />);
    expect(text(round(container, 3))).toContain("本輪無發現");
    expect(text(round(container, 3))).not.toContain("findings");
    expect(summaryOf("verify", empty).badges).toBe("第 3 輪複驗");
  });

  // 引擎不檢查 `## Round N` 的 N 是否唯一：手改出同序號的工單，React key 與
  // 「末輪展開」都以陣列位置決定，不撞 key、不把兩輪一起展開。
  it("同序號的輪不撞 React key，只有位置上的末輪展開", () => {
    const dup: StationTicket = {
      rounds: [
        { index: 1, phase: "discovery", patchHash: HASH, scope: SCOPE, findings: [] },
        { index: 1, phase: "validation", patchHash: HASH, scope: SCOPE, findings: [] },
      ],
    };
    const errors = vi.spyOn(console, "error").mockImplementation(() => {});
    try {
      const { container } = render(<TicketView station="review" ticket={dup} />);
      const toggles = Array.from(
        container.querySelectorAll("[data-ticket-round-toggle]"),
      ) as HTMLButtonElement[];
      expect(toggles).toHaveLength(2);
      expect(toggles[0].getAttribute("aria-expanded")).toBe("false");
      expect(toggles[1].getAttribute("aria-expanded")).toBe("true");
      expect(errors.mock.calls.some((c) => String(c[0]).includes("same key"))).toBe(false);
    } finally {
      errors.mockRestore();
    }
  });

  // spec Example「階段詞與 accepted token」：一列一案。
  const EXAMPLE: Array<{
    phase: TicketRound["phase"];
    phaseWord: string | null;
    text: string;
    shown: string;
    axis: string;
    accepted: boolean;
  }> = [
    { phase: "discovery", phaseWord: "首輪", text: "Correctness: 未處理空清單", shown: "未處理空清單", axis: "Correctness", accepted: false },
    { phase: "validation", phaseWord: "複驗", text: "Standards: 命名不一致 (accepted)", shown: "命名不一致", axis: "Standards", accepted: true },
    { phase: null, phaseWord: null, text: "Standards: (accepted) 語意不清", shown: "(accepted) 語意不清", axis: "Standards", accepted: false },
  ];
  for (const row of EXAMPLE) {
    it(`phase=${String(row.phase)}：階段詞 ${row.phaseWord ?? "（不顯示）"}、描述「${row.shown}」、軸向籤 ${row.axis}、已接受籤 ${row.accepted ? "有" : "無"}`, () => {
      const ticket: StationTicket = {
        rounds: [
          {
            index: 1,
            phase: row.phase,
            patchHash: row.phase ? HASH : null,
            scope: ["src/a.rs"],
            findings: [{ severity: "WARNING", path: "src/a.rs", text: row.text }],
          },
        ],
      };
      const { container } = render(<TicketView station="review" ticket={ticket} />);
      const roundHeader = text(round(container, 1).querySelector("[data-ticket-round-header]"));
      const badges = summaryOf("review", ticket).badges;
      if (row.phaseWord) {
        expect(badges).toBe(`第 1 輪${row.phaseWord}`);
        expect(roundHeader).toContain(`第 1 輪 · ${row.phaseWord} · 範圍 1 檔 · 1 條`);
      } else {
        expect(badges).toBe("第 1 輪");
        expect(roundHeader).toContain("第 1 輪 · 範圍 1 檔 · 1 條");
        expect(roundHeader).not.toMatch(/首輪|複驗/);
      }
      const finding = round(container, 1).querySelector("[data-ticket-finding]") as HTMLElement;
      expect(text(finding.querySelector("[data-ticket-text]"))).toBe(row.shown);
      // 先切行尾 token、再取開頭前綴（design D8）：accepted 與軸向籤可同時出現。
      expect(text(finding.querySelector("[data-ticket-axis]"))).toBe(row.axis);
      const badge = finding.querySelector("[data-ticket-accepted]");
      if (row.accepted) {
        expect(text(badge)).toBe("已接受");
      } else {
        expect(badge).toBeNull();
      }
    });
  }
});

// spec「詳情頁的工單分頁」的軸向籤、可蓋章提示框與輪卡頭計數籤（desktop-change-doc-views design D8）。
describe("TicketView 軸向籤、可蓋章提示框與輪卡頭", () => {
  const single = (findings: StationTicket["rounds"][number]["findings"]): StationTicket => ({
    rounds: [{ index: 2, phase: "validation", patchHash: HASH, scope: ["src/a.rs"], findings }],
  });

  it("描述以軸向前綴開頭時出軸向籤並去前綴；(accepted) 在行中時原樣、無已接受籤", () => {
    const { container } = render(
      <TicketView
        station="review"
        ticket={single([
          { severity: "WARNING", path: "src/a.rs", text: "Correctness: 未處理空清單" },
          { severity: "WARNING", path: "src/a.rs", text: "Standards: (accepted) 語意不清" },
        ])}
      />,
    );
    const rows = container.querySelectorAll("[data-ticket-finding]");
    expect(text(rows[0].querySelector("[data-ticket-axis]"))).toBe("Correctness");
    expect(text(rows[0].querySelector("[data-ticket-text]"))).toBe("未處理空清單");
    expect(text(rows[1].querySelector("[data-ticket-axis]"))).toBe("Standards");
    expect(text(rows[1].querySelector("[data-ticket-text]"))).toBe("(accepted) 語意不清");
    expect(rows[1].querySelector("[data-ticket-accepted]")).toBeNull();
  });

  it("末輪只有一條 SUGGESTION 時 tip 提示框「可以蓋章」並點名該站 stamp 指令；末輪有 WARNING 時無提示框", () => {
    const ok = summaryOf("review", single([{ severity: "SUGGESTION", path: "src/a.rs", text: "Standards: possible Duplicated Code" }]));
    const box = render(<>{ok.callout}</>).container;
    expect(text(box.querySelector(".markdown-alert-title"))).toBe("可以蓋章");
    expect(text(box)).toContain("末輪只剩 1 條 SUGGESTION，不擋章。");
    expect(text(box)).toContain("speclink review stamp");
    expect(box.firstElementChild!.className).toContain("bg-status-success/10");
    const none = render(<>{summaryOf("verify", single([])).callout}</>).container;
    expect(text(none)).toContain("末輪無發現。");
    expect(text(none)).toContain("speclink verify stamp");
    expect(summaryOf("review", single([{ severity: "WARNING", path: "src/a.rs", text: "x" }])).callout).toBeNull();
  });

  it("輪卡頭為 16px 粗體「第 N 輪」並附各嚴重度計數籤（為零不出）", () => {
    const { container } = render(<TicketView station="review" ticket={TWO_ROUNDS} />);
    const r1 = round(container, 1);
    const heading = r1.querySelector("h3[data-doc-heading]") as HTMLElement;
    expect(text(heading)).toBe("第 1 輪");
    expect(heading.className).toContain("text-base");
    const counts = Array.from(r1.querySelectorAll("[data-ticket-count]")).map((c) => text(c));
    expect(counts).toEqual(["WARNING 2"]);
  });
});

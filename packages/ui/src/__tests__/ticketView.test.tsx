import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, fireEvent, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { TicketView } from "../components/TicketView";
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

const round = (container: HTMLElement, index: number) =>
  container.querySelector(`[data-ticket-round="${index}"]`) as HTMLElement;
const roundToggle = (el: HTMLElement) =>
  el.querySelector("[data-ticket-round-toggle]") as HTMLButtonElement;

describe("TicketView（design D4 呈現規則）", () => {
  it("段標題列站名、末輪輪數、末輪階段詞與三級計數", () => {
    const { container } = render(<TicketView station="review" ticket={TWO_ROUNDS} />);
    expect(text(container.querySelector("[data-ticket-header]"))).toBe(
      "審查 · 第 2 輪（複驗）CRITICAL 0 · WARNING 1 · SUGGESTION 1",
    );
  });

  it("驗證站的段標題以「驗證」開頭", () => {
    const { container } = render(<TicketView station="verify" ticket={TWO_ROUNDS} />);
    expect(text(container.querySelector("[data-ticket-header]"))).toMatch(/^驗證 · 第 2 輪/);
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

  it("「範圍 N 檔」點擊後列出 scope 路徑（原樣、等寬）", () => {
    const { container } = render(<TicketView station="review" ticket={TWO_ROUNDS} />);
    const r2 = round(container, 2);
    // 第 2 輪的 findings 只碰 b.rs／c.rs——a.rs 只會來自 scope 清單。
    expect(within(r2).queryByText("src/a.rs")).toBeNull();
    fireEvent.click(within(r2).getByRole("button", { name: "範圍 3 檔" }));
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
    expect(text(container.querySelector("[data-ticket-header]"))).toBe(
      "驗證 · 第 3 輪（複驗）CRITICAL 0 · WARNING 0 · SUGGESTION 0",
    );
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
    accepted: boolean;
  }> = [
    { phase: "discovery", phaseWord: "首輪", text: "Correctness: 未處理空清單", shown: "Correctness: 未處理空清單", accepted: false },
    { phase: "validation", phaseWord: "複驗", text: "Standards: 命名不一致 (accepted)", shown: "Standards: 命名不一致", accepted: true },
    { phase: null, phaseWord: null, text: "Standards: (accepted) 語意不清", shown: "Standards: (accepted) 語意不清", accepted: false },
  ];
  for (const row of EXAMPLE) {
    it(`phase=${String(row.phase)}：階段詞 ${row.phaseWord ?? "（不顯示）"}、描述「${row.shown}」、已接受籤 ${row.accepted ? "有" : "無"}`, () => {
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
      const header = text(container.querySelector("[data-ticket-header]"));
      const roundHeader = text(round(container, 1).querySelector("[data-ticket-round-header]"));
      if (row.phaseWord) {
        expect(header).toContain(`（${row.phaseWord}）`);
        expect(roundHeader).toContain(`第 1 輪 · ${row.phaseWord} · 範圍 1 檔 · 1 條`);
      } else {
        expect(header).toBe("審查 · 第 1 輪 · CRITICAL 0 · WARNING 1 · SUGGESTION 0");
        expect(roundHeader).toContain("第 1 輪 · 範圍 1 檔 · 1 條");
        expect(roundHeader).not.toMatch(/首輪|複驗/);
      }
      const finding = round(container, 1).querySelector("[data-ticket-finding]") as HTMLElement;
      expect(text(finding.querySelector("[data-ticket-text]"))).toBe(row.shown);
      const badge = finding.querySelector("[data-ticket-accepted]");
      if (row.accepted) {
        expect(text(badge)).toBe("已接受");
      } else {
        expect(badge).toBeNull();
      }
    });
  }
});

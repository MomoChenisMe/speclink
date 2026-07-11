// spec 需求「桌面 app 提供動詞操作面」的分析面板（design D1 兩層結構）：
// 結構驗證列＋繁中維度摘要卡（零＝無問題、非零＝N 個問題）＋發現卡
// （嚴重度、location、summary、recommendation 對應 --json 輸出）。
import { describe, it, expect, vi } from "vitest";
import { render as rtlRender, screen, within, fireEvent } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";

import { I18nProvider } from "../i18n";
import { AnalyzePanel } from "../components/AnalyzePanel";
import type { AnalyzeReport } from "../adapter";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">{children}</I18nProvider>
);
function render(ui: ReactElement) {
  return rtlRender(ui, { wrapper: zhWrapper });
}

const report: AnalyzeReport = {
  change_id: "x",
  dimensions: [
    { dimension: "Coverage", status: "1 issue(s) found", finding_count: 1 },
    { dimension: "Consistency", status: "Clean", finding_count: 0 },
    { dimension: "Ambiguity", status: "18 issue(s) found", finding_count: 18 },
    { dimension: "Gaps", status: "Clean", finding_count: 0 },
  ],
  findings: [
    {
      id: "COV-1",
      dimension: "Coverage",
      severity: "Critical",
      location: "tasks.md",
      summary: "Requirement 'bar' not covered by tasks",
      recommendation: "add a task",
    },
    {
      id: "AMB-1",
      dimension: "Ambiguity",
      severity: "Suggestion",
      location: "specs/desktop-app/spec.md",
      summary: "Scenario 'foo' has no concrete examples",
      recommendation: "add example",
    },
  ],
  artifacts_analyzed: ["proposal", "specs", "tasks"],
  artifacts_missing: [],
};

const ok = { valid: true, errors: [] };
const dim = (name: string) => document.querySelector(`[data-dimension="${name}"]`) as HTMLElement;

describe("AnalyzePanel（結構驗證列＋維度摘要卡＋發現卡）", () => {
  it("維度摘要卡以繁中名呈現；零發現呈「無問題」、非零呈「N 個問題」", () => {
    render(<AnalyzePanel report={report} validate={ok} />);
    expect(within(dim("Coverage")).getByText("覆蓋度")).toBeTruthy();
    expect(within(dim("Consistency")).getByText("一致性")).toBeTruthy();
    expect(within(dim("Ambiguity")).getByText("模糊度")).toBeTruthy();
    expect(within(dim("Gaps")).getByText("缺漏")).toBeTruthy();
    expect(within(dim("Ambiguity")).getByText("18 個問題")).toBeTruthy();
    expect(within(dim("Coverage")).getByText("1 個問題")).toBeTruthy();
    expect(within(dim("Consistency")).getByText("無問題")).toBeTruthy();
    expect(within(dim("Gaps")).getByText("無問題")).toBeTruthy();
  });

  it("發現卡呈現嚴重度徽章、來源檔、摘要與建議行", () => {
    render(<AnalyzePanel report={report} validate={ok} />);
    const cards = document.querySelectorAll("[data-finding]");
    expect(cards.length).toBe(2);
    const first = cards[0] as HTMLElement;
    expect(within(first).getByText("Critical")).toBeTruthy();
    expect(within(first).getByText("tasks.md")).toBeTruthy();
    expect(within(first).getByText(/not covered by tasks/)).toBeTruthy();
    expect(within(first).getByText(/add a task/)).toBeTruthy();
  });

  it("結構驗證通過呈單列通過標示", () => {
    render(<AnalyzePanel report={report} validate={ok} />);
    expect(screen.getByText("結構驗證通過")).toBeTruthy();
  });

  it("結構驗證失敗呈錯誤數並逐條列出錯誤", () => {
    render(
      <AnalyzePanel
        report={report}
        validate={{ valid: false, errors: ["proposal.md 缺 Why", "tasks.md 無任務"] }}
      />,
    );
    expect(screen.getByText(/結構驗證 2 個錯誤/)).toBeTruthy();
    expect(screen.getByText(/proposal\.md 缺 Why/)).toBeTruthy();
    expect(screen.getByText(/tasks\.md 無任務/)).toBeTruthy();
  });

  it("關閉鈕觸發 onClose", () => {
    const onClose = vi.fn();
    render(<AnalyzePanel report={report} validate={ok} onClose={onClose} />);
    fireEvent.click(screen.getByRole("button", { name: "關閉分析結果" }));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("IPC 邊界防禦：findings 與 dimensions 缺席時不崩潰、四卡仍呈現", () => {
    const malformed = { change_id: "x" } as unknown as AnalyzeReport;
    render(<AnalyzePanel report={malformed} validate={ok} />);
    for (const d of ["Coverage", "Consistency", "Ambiguity", "Gaps"]) {
      expect(dim(d)).toBeTruthy();
    }
    expect(within(dim("Coverage")).getByText("無問題")).toBeTruthy();
  });
});

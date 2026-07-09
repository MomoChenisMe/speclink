// spec 需求「桌面 app 提供動詞操作面」的 analyze 面：AnalyzeReport 依
// Coverage／Consistency／Ambiguity／Gaps 四維度呈各維度發現數與逐條發現項（D2）。
import { describe, it, expect } from "vitest";
import { render as rtlRender, screen, within } from "@testing-library/react";
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
    { dimension: "Ambiguity", status: "1 issue(s) found", finding_count: 1 },
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

const dim = (name: string) => document.querySelector(`[data-dimension="${name}"]`) as HTMLElement;

describe("AnalyzePanel（四維度分析面板）", () => {
  it("四維度皆呈現；有發現者列出嚴重度與訊息（對應 --json）", () => {
    render(<AnalyzePanel report={report} />);
    for (const d of ["Coverage", "Consistency", "Ambiguity", "Gaps"]) {
      expect(dim(d)).toBeTruthy();
    }
    const cov = dim("Coverage");
    expect(within(cov).getByText(/not covered by tasks/)).toBeTruthy();
    expect(within(cov).getByText("Critical")).toBeTruthy();
    const amb = dim("Ambiguity");
    expect(within(amb).getByText(/no concrete examples/)).toBeTruthy();
    expect(within(amb).getByText("Suggestion")).toBeTruthy();
  });

  it("無發現的維度呈現無發現態", () => {
    render(<AnalyzePanel report={report} />);
    expect(within(dim("Consistency")).getByText("無發現")).toBeTruthy();
    expect(within(dim("Gaps")).getByText("無發現")).toBeTruthy();
  });
});

import type { AnalyzeReport } from "../adapter";
import { useI18n } from "../i18n";

/** 分析維度（固定四維，順序對應 speclink analyze 的 --json 輸出）。 */
const DIMENSIONS = ["Coverage", "Consistency", "Ambiguity", "Gaps"] as const;

/** 嚴重度配色——Critical 破壞色、Warning 主色、Suggestion 中性。 */
const SEVERITY_CLS: Record<string, string> = {
  Critical: "bg-destructive/15 text-destructive",
  Warning: "bg-primary/15 text-primary",
  Suggestion: "bg-muted text-muted-foreground",
};

/**
 * 四維度分析面板（design D2）：沿用引擎回傳的 AnalyzeReport，依 Coverage／
 * Consistency／Ambiguity／Gaps 分維度呈各維度發現數與逐條發現項（嚴重度＋訊息）。
 * 純呈現、不新增 IPC。
 */
export function AnalyzePanel({ report }: { report: AnalyzeReport }) {
  const { t } = useI18n();
  // IPC 邊界防禦：findings 缺席（畸形 payload）時退回空陣列，不讓抽屜整個崩潰。
  const all = report.findings ?? [];
  return (
    <div data-analyze-panel className="flex flex-col gap-2">
      {DIMENSIONS.map((d) => {
        const findings = all.filter((f) => f.dimension === d);
        return (
          <div key={d} data-dimension={d} className="rounded-md border border-border/60 p-2">
            <div className="flex items-center gap-2">
              <span className="text-xs font-semibold">{d}</span>
              <span className="rounded-full bg-muted px-1.5 py-0.5 text-[10px] tabular-nums text-muted-foreground">
                {findings.length}
              </span>
            </div>
            {findings.length === 0 ? (
              <div className="mt-1 text-[11px] text-muted-foreground">{t("analyze.clean")}</div>
            ) : (
              <ul className="mt-1.5 flex flex-col gap-1.5">
                {findings.map((f) => (
                  <li key={f.id} className="flex items-start gap-1.5 text-xs">
                    <span
                      className={`shrink-0 rounded px-1 py-0.5 text-[10px] font-medium ${
                        SEVERITY_CLS[f.severity] ?? "bg-muted text-muted-foreground"
                      }`}
                    >
                      {f.severity}
                    </span>
                    <span className="min-w-0">{f.summary}</span>
                  </li>
                ))}
              </ul>
            )}
          </div>
        );
      })}
    </div>
  );
}

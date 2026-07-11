import { Check, X } from "lucide-react";

import type { AnalyzeReport } from "../adapter";
import { useI18n } from "../i18n";
import { Button } from "./ui/button";

/** 分析維度（固定四維，順序對應 speclink analyze 的 --json 輸出）。 */
const DIMENSIONS = ["Coverage", "Consistency", "Ambiguity", "Gaps"] as const;

/** 嚴重度配色——Critical 破壞色、Warning 主色、Suggestion 中性。 */
const SEVERITY_CLS: Record<string, string> = {
  Critical: "bg-destructive/15 text-destructive",
  Warning: "bg-primary/15 text-primary",
  Suggestion: "bg-muted text-muted-foreground",
};

/**
 * 分析面板（design D1 兩層結構）：頂部結構驗證列（validate 併入分析單一入口）、
 * 繁中維度摘要卡（零＝無問題成功語意、非零＝N 個問題警示語意、含 Critical 時
 * 破壞語意）、逐條發現卡（嚴重度徽章＋location＋summary＋recommendation）。
 * 純呈現、不新增 IPC；onClose 供宿主收合（design D2）。
 */
export function AnalyzePanel({
  report,
  validate,
  onClose,
}: {
  report?: AnalyzeReport | null;
  validate?: { valid: boolean; errors: string[] } | null;
  onClose?: () => void;
}) {
  const { t } = useI18n();
  // IPC 邊界防禦：findings／dimensions 缺席（畸形 payload）時退回空陣列，不讓抽屜崩潰。
  const findings = report?.findings ?? [];
  const dimensions = report?.dimensions ?? [];
  const count = (d: string) =>
    dimensions.find((x) => x.dimension === d)?.finding_count ??
    findings.filter((f) => f.dimension === d).length;
  const hasCritical = (d: string) =>
    findings.some((f) => f.dimension === d && f.severity === "Critical");
  return (
    <div data-analyze-panel className="rounded-md border border-border/60 bg-muted/20 p-2">
      <div className="flex items-start gap-2">
        <div className="min-w-0 flex-1 flex flex-col gap-2">
          {/* 結構驗證列：通過單列帶過、失敗呈錯誤數並逐條列出（與 speclink validate 一致）。 */}
          {validate &&
            (validate.valid ? (
              <div data-validate-row className="inline-flex items-center gap-1 text-xs font-medium text-primary">
                <Check className="h-3.5 w-3.5" /> {t("analyze.validatePass")}
              </div>
            ) : (
              <div data-validate-row className="flex flex-col gap-0.5 text-xs text-destructive">
                <span className="font-medium">
                  {t("analyze.validateFail").replace("{n}", String(validate.errors.length))}
                </span>
                {validate.errors.map((err, i) => (
                  <span key={i} className="min-w-0 break-all font-mono text-[11px]">
                    {err}
                  </span>
                ))}
              </div>
            ))}
          {/* 維度摘要卡：一排四張，一眼可見問題集中在哪個維度。 */}
          <div className="grid grid-cols-4 gap-1.5">
            {DIMENSIONS.map((d) => {
              const n = count(d);
              const tone =
                n === 0
                  ? "text-primary"
                  : hasCritical(d)
                    ? "text-destructive"
                    : "text-amber-600 dark:text-amber-500";
              return (
                <div
                  key={d}
                  data-dimension={d}
                  className="flex flex-col items-center gap-0.5 rounded-md border border-border/60 bg-background px-1 py-1.5 text-center"
                >
                  <span className="text-[11px] font-semibold text-muted-foreground">
                    {t(`analyze.dim.${d}`)}
                  </span>
                  <span className={`text-xs font-medium tabular-nums ${tone}`}>
                    {n === 0 ? t("analyze.noIssues") : t("analyze.issues").replace("{n}", String(n))}
                  </span>
                </div>
              );
            })}
          </div>
        </div>
        {onClose && (
          <Button
            type="button"
            variant="ghost"
            size="icon"
            aria-label={t("analyze.close")}
            className="h-5 w-5 shrink-0 text-muted-foreground hover:text-foreground"
            onClick={onClose}
          >
            <X className="h-3.5 w-3.5" />
          </Button>
        )}
      </div>
      {/* 發現卡：逐條含出處與建議；多時面板內捲動、不撐爆抽屜 header 區。 */}
      {findings.length > 0 && (
        <ul className="mt-2 flex max-h-[32vh] flex-col gap-1.5 overflow-y-auto pr-1">
          {findings.map((f) => (
            <li
              key={f.id}
              data-finding={f.id}
              className="rounded-md border border-border/60 bg-background p-2 text-xs"
            >
              <div className="flex items-center gap-1.5">
                <span
                  className={`shrink-0 rounded px-1 py-0.5 text-[10px] font-medium ${
                    SEVERITY_CLS[f.severity] ?? "bg-muted text-muted-foreground"
                  }`}
                >
                  {f.severity}
                </span>
                <span className="min-w-0 truncate font-mono text-[11px] text-muted-foreground">
                  {f.location}
                </span>
              </div>
              <div className="mt-1 min-w-0">{f.summary}</div>
              {f.recommendation && (
                <div className="mt-0.5 min-w-0 text-[11px] text-muted-foreground">
                  ↳ {f.recommendation}
                </div>
              )}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

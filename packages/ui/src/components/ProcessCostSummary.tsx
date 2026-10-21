import { useMemo, type ReactNode } from "react";

import type { ArchivedItem } from "../adapter";
import { useI18n } from "../i18n";
import { formatCost, roundCost, summarizeProcessCost, type CostStat } from "../processCost";
import { Card } from "./ui/card";

/** 一項（文件行數／任務數／建立到封存／品質關卡輪數）：標籤＋一到兩個數字。 */
function CostItem({ id, label, children }: { id: string; label: string; children: ReactNode }) {
  return (
    <div data-cost-item={id} className="min-w-0">
      <dt className="text-xs text-muted-foreground">{label}</dt>
      {children}
    </div>
  );
}

/**
 * 已封存頁的流程成本摘要（spec desktop-app「已封存頁的流程成本摘要」）：唯讀，最多四項，
 * 每個數字標出樣本數；沒有任何封存變更帶流程成本欄位時不渲染（遠端清單只帶任務數）。
 * `items` 是全部封存變更——頁內搜尋不影響摘要。
 */
export function ProcessCostSummary({ items }: { items: ArchivedItem[] }) {
  const { t, tn } = useI18n();
  const summary = useMemo(() => summarizeProcessCost(items), [items]);
  if (Object.keys(summary).length === 0) return null;
  const { docLines, tasks, leadDays, leadHours, reviewRounds, verifyRounds } = summary;

  const median = (stat: CostStat, unitKey?: string) =>
    `${t("archived.cost.median")} ${unitKey ? tn(unitKey, roundCost(stat.median)) : formatCost(stat.median)}`;
  /** 一個數字一行：`[前綴 ]部分 · 部分 · N 筆`。 */
  const value = (id: string, stat: CostStat, parts: string[], prefix?: string) => (
    <dd data-cost={id} className="text-sm tabular-nums">
      {(prefix ? `${prefix} ` : "") + [...parts, tn("archived.cost.samples", stat.samples)].join(" · ")}
    </dd>
  );

  return (
    <Card data-process-cost role="region" aria-label={t("archived.cost.title")} className="shrink-0 px-4 py-3">
      <h3 className="text-[13px] font-medium">{t("archived.cost.title")}</h3>
      <dl className="mt-2 grid grid-cols-2 gap-x-6 gap-y-2 sm:grid-cols-4">
        {docLines && (
          <CostItem id="docLines" label={t("archived.cost.docLines")}>
            {value("docLines", docLines, [median(docLines), `${t("archived.cost.p90")} ${formatCost(docLines.p90)}`])}
          </CostItem>
        )}
        {tasks && (
          <CostItem id="tasks" label={t("archived.cost.tasks")}>
            {value("tasks", tasks, [median(tasks)])}
          </CostItem>
        )}
        {(leadDays || leadHours) && (
          <CostItem id="lead" label={t("archived.cost.lead")}>
            {leadDays && value("leadDays", leadDays, [median(leadDays, "archived.cost.days")])}
            {leadHours && value("leadHours", leadHours, [median(leadHours, "archived.cost.hours")])}
          </CostItem>
        )}
        {(reviewRounds || verifyRounds) && (
          <CostItem id="rounds" label={t("archived.cost.rounds")}>
            {reviewRounds &&
              value("reviewRounds", reviewRounds, [median(reviewRounds)], t("archived.cost.review"))}
            {verifyRounds &&
              value("verifyRounds", verifyRounds, [median(verifyRounds)], t("archived.cost.verify"))}
          </CostItem>
        )}
      </dl>
    </Card>
  );
}

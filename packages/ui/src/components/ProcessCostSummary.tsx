import { useMemo } from "react";

import type { ArchivedItem } from "../adapter";
import { useI18n } from "../i18n";
import { formatCost, roundCost, summarizeProcessCost, type CostStat } from "../processCost";
import { DocTiles, type DocTile } from "./DetailShell";
import { Card } from "./ui/card";

/**
 * 已封存頁的流程成本摘要（spec desktop-app「已封存頁的流程成本摘要」）：唯讀，沿用數字磚
 * DocTiles，最多四格——大字是中位數（標題註明），小字是項目名與每個數字的樣本數（文件行數
 * 另附 P90）。DocTiles 的小字會截斷，所以小字保持精簡，樣本數才不會被截掉。
 * 沒有任何封存變更帶流程成本欄位時不渲染（遠端清單只帶任務數）。`items` 是全部封存
 * 變更——頁內搜尋不影響摘要。
 */
export function ProcessCostSummary({ items }: { items: ArchivedItem[] }) {
  const { t, tn } = useI18n();
  const summary = useMemo(() => summarizeProcessCost(items), [items]);
  if (Object.keys(summary).length === 0) return null;
  const { docLines, tasks, leadDays, leadHours, reviewRounds, verifyRounds } = summary;

  /** 小字：`<項目>（N 筆）`；一格兩個數字時為 `<項目>（N／M 筆）`，依大字裡數字的順序。 */
  const label = (itemKey: string, ...stats: (CostStat | undefined)[]) => {
    const counts = stats.filter((s): s is CostStat => s != null).map((s) => s.samples);
    const samples =
      counts.length === 1
        ? tn("archived.cost.samples", counts[0])
        : t("archived.cost.samplesList").replace("{list}", counts.join(t("archived.cost.sampleSep")));
    return t("archived.cost.itemSamples").replace("{label}", t(itemKey)).replace("{samples}", samples);
  };
  const join = (...parts: (string | false | undefined)[]) => parts.filter(Boolean).join(" · ");
  const withUnit = (stat: CostStat, unitKey: string) => tn(unitKey, roundCost(stat.median));
  const station = (key: string, stat: CostStat | undefined) =>
    stat && `${t(key)} ${formatCost(stat.median)}`;

  const tiles: DocTile[] = [];
  if (docLines) {
    tiles.push({
      value: formatCost(docLines.median),
      label: join(label("archived.cost.docLines", docLines), `${t("archived.cost.p90")} ${formatCost(docLines.p90)}`),
    });
  }
  if (tasks) tiles.push({ value: formatCost(tasks.median), label: label("archived.cost.tasks", tasks) });
  if (leadDays || leadHours) {
    tiles.push({
      value: join(
        leadDays && withUnit(leadDays, "archived.cost.days"),
        leadHours && withUnit(leadHours, "archived.cost.hours"),
      ),
      label: label("archived.cost.lead", leadDays, leadHours),
    });
  }
  if (reviewRounds || verifyRounds) {
    tiles.push({
      value: join(station("archived.cost.review", reviewRounds), station("archived.cost.verify", verifyRounds)),
      label: label("archived.cost.rounds", reviewRounds, verifyRounds),
    });
  }

  return (
    <Card data-process-cost role="region" aria-label={t("archived.cost.title")} className="shrink-0 px-4 py-3">
      <h3 className="mb-2 text-[13px] font-medium">{t("archived.cost.title")}</h3>
      <DocTiles tiles={tiles} />
    </Card>
  );
}

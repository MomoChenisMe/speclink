import type { ArchivedItem } from "./adapter";

/** 一個數字的中位數與樣本數（帶有該欄位的封存變更筆數）。 */
export interface CostStat {
  median: number;
  samples: number;
}

/**
 * 已封存頁流程成本摘要的彙總結果（spec desktop-app「已封存頁的流程成本摘要」）。
 * 樣本數為 0 的數字缺席；沒有任何封存變更帶流程成本欄位時為空物件——遠端清單
 * 只帶任務數，摘要因此整個不出現。
 */
export interface ProcessCostSummaryData {
  docLines?: CostStat & { p90: number };
  tasks?: CostStat;
  leadDays?: CostStat;
  leadHours?: CostStat;
  reviewRounds?: CostStat;
  verifyRounds?: CostStat;
}

const ascending = (values: number[]) => [...values].sort((a, b) => a - b);

/** 由小到大排序後的中間值；偶數筆取中間兩值的平均。 */
export function median(values: number[]): number {
  const sorted = ascending(values);
  const mid = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 1 ? sorted[mid] : (sorted[mid - 1] + sorted[mid]) / 2;
}

/** 由小到大第 ⌈0.9 × n⌉ 個值（整數運算，避開 0.9 × n 的浮點誤差）。 */
export function p90(values: number[]): number {
  return ascending(values)[Math.ceil((9 * values.length) / 10) - 1];
}

/** 四捨五入到一位小數——帶單位的計數文案（tn）直接吃這個數字。 */
export function roundCost(n: number): number {
  return Math.round(n * 10) / 10;
}

/** 最多一位小數，整數不帶小數點。 */
export function formatCost(n: number): string {
  return String(roundCost(n));
}

const COST_FIELDS = ["docLines", "leadDays", "leadHours", "reviewRounds", "verifyRounds"] as const;

const stat = (values: number[]): CostStat | undefined =>
  values.length > 0 ? { median: median(values), samples: values.length } : undefined;

/** 以全部已封存變更算出摘要的每個數字（呼叫端不得先套搜尋過濾）。 */
export function summarizeProcessCost(items: ArchivedItem[]): ProcessCostSummaryData {
  if (!items.some((item) => COST_FIELDS.some((field) => item[field] != null))) return {};
  const values = (pick: (item: ArchivedItem) => number | undefined) =>
    items.map(pick).filter((v): v is number => v != null);
  const docLines = values((item) => item.docLines);
  const summary: ProcessCostSummaryData = {
    docLines: docLines.length > 0 ? { ...stat(docLines)!, p90: p90(docLines) } : undefined,
    tasks: stat(values((item) => item.tasksTotal)),
    leadDays: stat(values((item) => item.leadDays)),
    leadHours: stat(values((item) => item.leadHours)),
    reviewRounds: stat(values((item) => item.reviewRounds)),
    verifyRounds: stat(values((item) => item.verifyRounds)),
  };
  // 缺席的數字不留 key，空摘要就是 `{}`。
  return Object.fromEntries(Object.entries(summary).filter(([, v]) => v)) as ProcessCostSummaryData;
}

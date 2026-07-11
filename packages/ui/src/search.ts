/**
 * 看板與已封存頁共用的搜尋比對規則（spec「與已封存頁的搜尋規則一致」的單一真相）：
 * query 去頭尾空白、不分大小寫、以子字串命中任一欄位；空（或僅空白）query 恆命中。
 */
export function matchesQuery(query: string, ...fields: (string | null | undefined)[]): boolean {
  const q = query.trim().toLowerCase();
  if (!q) return true;
  return fields.some((f) => (f ?? "").toLowerCase().includes(q));
}

/**
 * 名稱層模糊比對（design D7）：查詢字元依序出現於目標即命中（subsequence，
 * 不分大小寫、去頭尾空白）。僅套用於變更卡名稱與討論卡 slug——摘要與主題
 * 維持子字串（spec「名稱層模糊比對」）。空 query 恆命中。
 */
export function matchesFuzzy(query: string, target: string | null | undefined): boolean {
  const q = query.trim().toLowerCase();
  if (!q) return true;
  const t = (target ?? "").toLowerCase();
  let i = 0;
  for (const ch of t) {
    if (ch === q[i]) i++;
    if (i === q.length) return true;
  }
  return false;
}

/** 看板篩選狀態（design D5）：三維度、null＝未啟用；多 chip 與搜尋字串 AND 交集。 */
export interface BoardFilters {
  /** 建立者（"Name <email>" 全等比對）。 */
  createdBy: string | null;
  /** 建立時間窗：近 7 天／近 30 天／更早（超過 30 天）。 */
  createdWithin: "7d" | "30d" | "earlier" | null;
  /** 來源討論 slug：命中該討論卡自身與來源討論含該 slug 的變更卡。 */
  fromDiscussion: string | null;
}

export const EMPTY_FILTERS: BoardFilters = {
  createdBy: null,
  createdWithin: null,
  fromDiscussion: null,
};

/**
 * created 日期（YYYY-MM-DD）是否落在時間窗內（以 today 為基準）。
 * 未啟用（range null）恆命中；日期缺席或不可解析時不命中任何啟用中的窗。
 */
export function matchesCreatedRange(
  created: string | null | undefined,
  range: BoardFilters["createdWithin"],
  today: string,
): boolean {
  if (!range) return true;
  if (!created) return false;
  const days = (new Date(today).getTime() - new Date(created).getTime()) / 86_400_000;
  if (Number.isNaN(days)) return false;
  if (range === "7d") return days <= 7;
  if (range === "30d") return days <= 30;
  return days > 30;
}

/**
 * 篩選 chips 的 AND 交集比對（design D5）：卡片（變更或討論）須同時滿足所有
 * 啟用中的維度。變更卡以 fromDiscussions 命中來源討論；討論卡以自身 slug 命中
 * （spec「來源討論篩選」：該討論卡自身與其衍生變更卡）。
 */
export function matchesFilters(
  filters: BoardFilters,
  card: {
    createdBy?: string | null;
    created?: string | null;
    fromDiscussions?: string[];
    slug?: string;
  },
  today: string,
): boolean {
  if (filters.createdBy && card.createdBy !== filters.createdBy) return false;
  if (!matchesCreatedRange(card.created, filters.createdWithin, today)) return false;
  if (filters.fromDiscussion) {
    const hit =
      card.slug === filters.fromDiscussion ||
      (card.fromDiscussions ?? []).includes(filters.fromDiscussion);
    if (!hit) return false;
  }
  return true;
}

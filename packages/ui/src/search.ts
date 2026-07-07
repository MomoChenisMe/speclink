/**
 * 看板與已封存頁共用的搜尋比對規則（spec「與已封存頁的搜尋規則一致」的單一真相）：
 * query 去頭尾空白、不分大小寫、以子字串命中任一欄位；空（或僅空白）query 恆命中。
 */
export function matchesQuery(query: string, ...fields: (string | null | undefined)[]): boolean {
  const q = query.trim().toLowerCase();
  if (!q) return true;
  return fields.some((f) => (f ?? "").toLowerCase().includes(q));
}

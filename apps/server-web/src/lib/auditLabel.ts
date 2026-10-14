/** 稽核動作與來源的顯示名稱（稽核紀錄頁與總覽的最近活動共用）。兩者是伺服器的封閉集合；未知值
 *  原樣呈現而不猜——那代表伺服器新增了這裡還沒跟上的種類，顯示原始字串比顯示空白誠實。 */
export function auditLabel(t: (key: string) => string, namespace: "action" | "source", value: string): string {
  const key = `audit.${namespace}.${value}`;
  const text = t(key);
  return text === key ? value : text;
}

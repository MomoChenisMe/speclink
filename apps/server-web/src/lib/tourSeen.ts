// 首次導覽的「看過了」狀態（server-web-console「首次進入提供可略過的分步導覽」）。
//
// 這是瀏覽器端偏好而非伺服器真相——它不影響任何授權或資料，存伺服器要新增 identity
// 欄位、migration 與 API，為一個一次性提示不值得。代價是換瀏覽器會再看到一次。
//
// 隱私模式或停用儲存時 localStorage 的讀寫會丟例外，一律吞掉：讀失敗＝視為未看過
// （導覽照常出現），寫失敗＝不持久化（導覽照常結束）。

const KEY = "speclink.tourSeen";
const SEEN = "1";

/** 缺鍵、值不是 "1"、或儲存不可用，一律視為未看過。 */
export function readTourSeen(): boolean {
  try {
    return localStorage.getItem(KEY) === SEEN;
  } catch {
    return false;
  }
}

export function writeTourSeen(seen: boolean): void {
  try {
    if (seen) localStorage.setItem(KEY, SEEN);
    else localStorage.removeItem(KEY);
  } catch {
    // 不持久化即可，不影響本次導覽。
  }
}

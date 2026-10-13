// 面板可捲區的邊緣淡出（spec「面板樣式（macOS）」；design「專案 tab 條：「＋」固定右端、邊緣淡出、
// 捲到作用中」「中段上下緣淡出：與 tab 條共用邊緣淡出」）：tab 條（橫向）與可捲中段（縱向）共用
// 同一套判斷——軸向無關，start＝左／上、end＝右／下。純函式：jsdom 沒有版面，元件測試以注入
// 尺寸驗證接線，計算本身由純函式測試覆蓋。

/** 邊緣淡出的寬度（px）：遮罩漸層寬度，也是 tab 條捲到作用中 tab 時留給淡出區的餘裕。 */
export const EDGE_FADE_PX = 24;

/** 捲動位置可能是小數：差不到 1px 視為已到盡頭。 */
const TOLERANCE_PX = 1;

/** 淡出的側；以字串表示，元件存成狀態時值相同就不重繪。 */
export type EdgeFadeSide = "none" | "start" | "end" | "both";

/** 可捲區單軸的捲動量測；欄位具名，避免三個同型數字傳錯位置。 */
export interface ScrollMetrics {
  /** 目前捲動位置（scrollLeft／scrollTop）。 */
  position: number;
  /** 內容總長（scrollWidth／scrollHeight）。 */
  scrollSize: number;
  /** 可視長度（clientWidth／clientHeight）。 */
  viewportSize: number;
}

/** 淡出判斷：該側還有被捲出可視範圍的內容才淡出。 */
export function edgeFade({ position, scrollSize, viewportSize }: ScrollMetrics): EdgeFadeSide {
  const start = position > TOLERANCE_PX;
  const end = position + viewportSize < scrollSize - TOLERANCE_PX;
  return start && end ? "both" : start ? "start" : end ? "end" : "none";
}

/** 依淡出側組出遮罩漸層（掛在可捲區本身，不隨內容捲動）；不淡出時不掛遮罩。 */
export function edgeFadeMask(axis: "x" | "y", side: EdgeFadeSide): string | undefined {
  if (side === "none") return undefined;
  const start = side === "end" ? "black" : `transparent, black ${EDGE_FADE_PX}px`;
  const end = side === "start" ? "black" : `black calc(100% - ${EDGE_FADE_PX}px), transparent`;
  return `linear-gradient(${axis === "x" ? "to right" : "to bottom"}, ${start}, ${end})`;
}

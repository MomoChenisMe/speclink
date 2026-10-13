// 面板專案 tab 條的捲動計算（spec「面板樣式（macOS）」；design「專案 tab 條：「＋」固定
// 右端、邊緣淡出、捲到作用中」）：純函式——jsdom 沒有版面，元件測試以注入尺寸驗證接線，
// 計算本身由純函式測試覆蓋。參數一律具名，避免幾個同型數字傳錯位置。邊緣淡出的判斷與
// 中段共用，在 edgeFade.ts。
import { EDGE_FADE_PX } from "./edgeFade";

/**
 * 露出目標：讓作用中 tab 完整可見、且不落在淡出區內的 scrollLeft；本來就可見則回傳原值。
 * tabLeft 是 tab 在可捲內容中的左緣（可捲區為其 offsetParent 時即 offsetLeft）。超過最大
 * 捲動量的值交給瀏覽器夾回——捲到盡頭時該側本來就不淡出。
 */
export function tabRevealScrollLeft({
  tabLeft,
  tabWidth,
  scrollLeft,
  viewportWidth,
}: {
  tabLeft: number;
  tabWidth: number;
  scrollLeft: number;
  viewportWidth: number;
}): number {
  const minScroll = tabLeft + tabWidth + EDGE_FADE_PX - viewportWidth;
  const maxScroll = tabLeft - EDGE_FADE_PX;
  if (scrollLeft < minScroll) return minScroll;
  if (scrollLeft > maxScroll) return Math.max(0, maxScroll);
  return scrollLeft;
}

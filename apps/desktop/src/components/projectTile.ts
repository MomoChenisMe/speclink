// 專案首字母方塊的樣式常數（tray-panel-restyle design D3）：主視窗圖示列與系統匣
// 面板的專案 tab 取同一份，兩處永遠同形。零依賴——面板是獨立 bundle。

/** 圖示列方塊的尺寸樣式（專案方塊、「＋」、齒輪與更新鈕共用）。 */
export const RAIL_SQUARE =
  "relative flex h-8 w-8 shrink-0 items-center justify-center rounded-lg text-[13px] font-medium";

/** 作用中專案：實心主色＋反白字。 */
export const PROJECT_TILE_ACTIVE = "bg-primary text-primary-foreground";

/** 非作用中專案：灰底。 */
export const PROJECT_TILE_IDLE = "bg-muted text-muted-foreground";

/** 「加入專案」／「＋」的虛線方塊。 */
export const ADD_TILE = "border border-dashed border-border bg-transparent text-muted-foreground";

/** 方塊內的首字母：去頭尾空白後的第一個字（以 code point 計，不切斷代理對）轉大寫。 */
export function projectInitial(name: string): string {
  return Array.from(name.trim())[0]?.toUpperCase() ?? "";
}

/** 自訂色票未選取時混入 50% 白色，選取時保持深底白字。 */
const COLORS = {
  teal: ["bg-[color-mix(in_srgb,var(--color-teal-700)_50%,white)] text-teal-950", "bg-teal-700 text-white hover:bg-teal-800"],
  blue: ["bg-[color-mix(in_srgb,var(--color-blue-700)_50%,white)] text-blue-950", "bg-blue-700 text-white hover:bg-blue-800"],
  purple: ["bg-[color-mix(in_srgb,var(--color-purple-700)_50%,white)] text-purple-950", "bg-purple-700 text-white hover:bg-purple-800"],
  pink: ["bg-[color-mix(in_srgb,var(--color-pink-700)_50%,white)] text-pink-950", "bg-pink-700 text-white hover:bg-pink-800"],
  orange: ["bg-[color-mix(in_srgb,var(--color-orange-700)_50%,white)] text-orange-950", "bg-orange-700 text-white hover:bg-orange-800"],
  green: ["bg-[color-mix(in_srgb,var(--color-green-700)_50%,white)] text-green-950", "bg-green-700 text-white hover:bg-green-800"],
};
export function projectTileClass(color: import("../projectColors").ProjectColor | undefined, active: boolean): string {
  if (color && Object.prototype.hasOwnProperty.call(COLORS, color)) return COLORS[color][active ? 1 : 0];
  return active ? PROJECT_TILE_ACTIVE : `${PROJECT_TILE_IDLE} hover:bg-muted/70`;
}

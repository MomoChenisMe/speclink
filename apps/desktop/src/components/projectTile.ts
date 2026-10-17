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
  teal: ["bg-project-teal-muted text-project-teal-muted-foreground", "bg-project-teal text-project-foreground hover:bg-project-teal-hover"],
  blue: ["bg-project-blue-muted text-project-blue-muted-foreground", "bg-project-blue text-project-foreground hover:bg-project-blue-hover"],
  purple: ["bg-project-purple-muted text-project-purple-muted-foreground", "bg-project-purple text-project-foreground hover:bg-project-purple-hover"],
  pink: ["bg-project-pink-muted text-project-pink-muted-foreground", "bg-project-pink text-project-foreground hover:bg-project-pink-hover"],
  orange: ["bg-project-orange-muted text-project-orange-muted-foreground", "bg-project-orange text-project-foreground hover:bg-project-orange-hover"],
  green: ["bg-project-green-muted text-project-green-muted-foreground", "bg-project-green text-project-foreground hover:bg-project-green-hover"],
};
export function projectTileClass(color: import("../projectColors").ProjectColor | undefined, active: boolean): string {
  if (color && Object.prototype.hasOwnProperty.call(COLORS, color)) return COLORS[color][active ? 1 : 0];
  return active ? PROJECT_TILE_ACTIVE : `${PROJECT_TILE_IDLE} hover:bg-muted/70`;
}

// 面板討論卡底「已轉出」收合列的展開記憶（tray-panel-restyle design D4）：app 本機
// （localStorage）、值為 "true"／"false"。與看板的 speclink.board.promotedExpanded 刻意
// 分鍵——兩個視窗、兩個把手，面板的點按不改看板的收合狀態。

export const TRAY_PROMOTED_EXPANDED_STORAGE_KEY = "speclink.tray.promotedExpanded";

/** 鍵缺席、壞值或 localStorage 不可用一律視為收合（false），不拋錯。 */
export function readTrayPromotedExpanded(): boolean {
  try {
    return localStorage.getItem(TRAY_PROMOTED_EXPANDED_STORAGE_KEY) === "true";
  } catch {
    return false;
  }
}

/** 寫入失敗靜默——記不住只是下次開啟回到收合。 */
export function writeTrayPromotedExpanded(expanded: boolean): void {
  try {
    localStorage.setItem(TRAY_PROMOTED_EXPANDED_STORAGE_KEY, String(expanded));
  } catch {
    // localStorage 不可用：不記憶。
  }
}

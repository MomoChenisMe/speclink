// 面板內容卡的收合記憶（spec「面板內容卡收合」；design「收合狀態記在面板本機、所有專案
// 共用一份」）：app 本機（localStorage）單一鍵，值為收合中分區 id 的 JSON 陣列。收合是
// 個人閱讀習慣、與專案無關，故不分專案；形狀比照 promotedExpanded.ts。
import { STAGES, type Stage } from "@speclink/ui";

/** 可收合的四張內容卡：三個生命週期階段（即 Stage 值）＋討論。 */
export type TraySectionId = Stage | "discussions";

const SECTION_IDS: readonly TraySectionId[] = [...STAGES, "discussions"];

export const TRAY_COLLAPSED_SECTIONS_STORAGE_KEY = "speclink.tray.collapsedSections";

/** 鍵缺席、壞值或 localStorage 不可用一律視為全部展開（空集合），不拋錯；未知 id 忽略。 */
export function readTrayCollapsedSections(): Set<TraySectionId> {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(TRAY_COLLAPSED_SECTIONS_STORAGE_KEY) ?? "[]");
    if (!Array.isArray(parsed)) return new Set();
    return new Set(SECTION_IDS.filter((id) => parsed.includes(id)));
  } catch {
    return new Set();
  }
}

/** 以目前收合集合整份覆寫；寫入失敗靜默——記不住只是下次回到上次讀得到的狀態。 */
export function writeTrayCollapsedSections(sections: ReadonlySet<TraySectionId>): void {
  try {
    localStorage.setItem(
      TRAY_COLLAPSED_SECTIONS_STORAGE_KEY,
      JSON.stringify(SECTION_IDS.filter((id) => sections.has(id))),
    );
  } catch {
    // localStorage 不可用：不記憶。
  }
}

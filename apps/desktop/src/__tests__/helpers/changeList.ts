// 資料源 listChanges 的回應形狀（清單項＋頂層 planError）——測試替身共用。
import type { ChangeItem, ChangeListPayload, ChangeRequirementOverlap } from "@speclink/ui";

export function changeList(changes: ChangeItem[], planError: string | null = null): ChangeListPayload {
  return { changes, planError };
}

/** 清單項 requirementOverlap 的一列：與 `change` 同動 desktop-app 的「看板與任務」（雙方 MODIFIED、不衝突）。 */
export function sharedBoardRequirement(change: string): ChangeRequirementOverlap {
  return {
    change,
    capability: "desktop-app",
    requirement: "看板與任務",
    ownOperation: "MODIFIED",
    otherOperation: "MODIFIED",
    conflict: false,
  };
}

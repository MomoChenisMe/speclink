// 資料源 listChanges 的回應形狀（清單項＋頂層 planError）——測試替身共用。
import type { ChangeItem, ChangeListPayload } from "@speclink/ui";

export function changeList(changes: ChangeItem[], planError: string | null = null): ChangeListPayload {
  return { changes, planError };
}

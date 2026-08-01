import { useI18n } from "../i18n";
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogCancel,
} from "./ui/alert-dialog";
import { Button } from "./ui/button";
import { REVIEW_TONE } from "./reviewStyle";

export interface ReviewArchiveDialogProps {
  open: boolean;
  /** 要封存的 change 名稱（對話框文案帶出）。 */
  change: string | null;
  onOpenChange: (open: boolean) => void;
  /** 前往完成蓋章：不封存，導引使用者收尾審查（宿主通常開啟詳情抽屜）。 */
  onGoStamp: () => void;
  /** 放棄審查後封存（等同 `review discard` 再 archive）。 */
  onDiscardReview: () => void;
  /** 照樣帶走（等同 `archive --carry-review`）：永久顯示「曾審查未通過」。 */
  onCarryReview: () => void;
}

/** 封存入口的未結工單三選項對話框（spec「封存入口的未結工單三選項」；design
 * D6）：目標 change 審查中（工單未結）時取代一般封存確認；未選擇前不執行任何
 * 封存。 */
export function ReviewArchiveDialog({
  open,
  change,
  onOpenChange,
  onGoStamp,
  onDiscardReview,
  onCarryReview,
}: ReviewArchiveDialogProps) {
  const { t } = useI18n();
  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{t("review.archiveGateTitle")}</AlertDialogTitle>
          <AlertDialogDescription>
            {t("review.archiveGateDesc").replace("{name}", change ?? "")}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter className="flex-col gap-2 sm:flex-col sm:items-stretch sm:space-x-0">
          <Button onClick={onGoStamp}>{t("review.goStamp")}</Button>
          <Button variant="outline" onClick={onDiscardReview}>
            {t("review.discardReview")}
          </Button>
          <Button
            variant="outline"
            className={REVIEW_TONE.reviewedNotPassed}
            onClick={onCarryReview}
          >
            {t("review.carryReview")}
          </Button>
          <AlertDialogCancel>{t("review.cancel")}</AlertDialogCancel>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}

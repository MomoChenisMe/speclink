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
import { SEMANTIC_TONE } from "../tone";
import { Button } from "./ui/button";

export interface ReviewArchiveDialogProps {
  open: boolean;
  /** 要封存的 change 名稱（對話框文案帶出）。 */
  change: string | null;
  /** 哪一個品質站的工單未結；預設審查站。兩站工單並存時由宿主依序各彈一次
   * （spec「封存入口三選項擴及驗證工單」：分別處置後才封存）。 */
  station?: "review" | "verify";
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
  station = "review",
  onOpenChange,
  onGoStamp,
  onDiscardReview,
  onCarryReview,
}: ReviewArchiveDialogProps) {
  const { t } = useI18n();
  // 站別只換詞條前綴——版面、選項數與危險動作分工兩站一致。
  const k = (name: string) => `${station}.${name}`;
  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{t(k("archiveGateTitle"))}</AlertDialogTitle>
          <AlertDialogDescription>
            {t(k("archiveGateDesc")).replace("{name}", change ?? "")}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter className="flex-col gap-2 sm:flex-col sm:items-stretch sm:space-x-0">
          <Button onClick={onGoStamp}>{t(k("goStamp"))}</Button>
          <Button variant="outline" onClick={onDiscardReview}>
            {t(k("discardReview"))}
          </Button>
          {/* 危險動作走 destructive；rose 專屬「曾審查未通過」的永久標示（三紅分工）。 */}
          <Button variant="outline" className={SEMANTIC_TONE.danger} onClick={onCarryReview}>
            {t(k("carryReview"))}
          </Button>
          <AlertDialogCancel>{t(k("cancel"))}</AlertDialogCancel>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}

// 兩鍵式確認框（取消＋確認；spec desktop-app「共用元件唯一來源」、design D7）：
// 外觀只此一份，文案與狀態由呼叫端以屬性提供。三選項以上的對話框各自組 AlertDialog。
import type { ReactNode } from "react";

import { useI18n } from "../../i18n";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "./alert-dialog";
import { buttonVariants } from "./button";

export interface ConfirmDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  description?: ReactNode;
  confirmLabel: string;
  /** 預設 i18n `common.cancel`。 */
  cancelLabel?: string;
  /** 確認鈕用 destructive 變體（實心紅）。 */
  destructive?: boolean;
  /** 動作進行中：確認鈕與取消鈕都不可按。 */
  busy?: boolean;
  /** 只停用確認鈕（例如遠端能力不支援該動作）；取消照常可按。 */
  confirmDisabled?: boolean;
  /** 按下確認後對話框隨即關閉（onOpenChange(false)）；在此之前先同步呼叫。 */
  onConfirm: () => void | Promise<void>;
  /** 標題與說明之後、按鈕之前的額外內容（如工具多選）。 */
  children?: ReactNode;
}

export function ConfirmDialog({
  open,
  onOpenChange,
  title,
  description,
  confirmLabel,
  cancelLabel,
  destructive = false,
  busy = false,
  confirmDisabled = false,
  onConfirm,
  children,
}: ConfirmDialogProps) {
  const { t } = useI18n();
  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{title}</AlertDialogTitle>
          {description !== undefined && (
            <AlertDialogDescription>{description}</AlertDialogDescription>
          )}
        </AlertDialogHeader>
        {children}
        <AlertDialogFooter>
          <AlertDialogCancel disabled={busy}>{cancelLabel ?? t("common.cancel")}</AlertDialogCancel>
          <AlertDialogAction
            className={destructive ? buttonVariants({ variant: "destructive" }) : undefined}
            disabled={busy || confirmDisabled}
            onClick={() => void onConfirm()}
          >
            {confirmLabel}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}

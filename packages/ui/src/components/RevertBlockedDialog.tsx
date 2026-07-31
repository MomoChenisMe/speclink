import { useI18n } from "../i18n";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "./ui/alert-dialog";

/** 守門對話框的資料源：引擎回傳的工作痕跡證據＋被擋的 change 名。 */
export interface RevertBlockedInfo {
  change: string;
  checkedTasks: number;
  touchedFiles: string[];
}

export interface RevertBlockedDialogProps {
  /** null＝關閉。 */
  info: RevertBlockedInfo | null;
  onClose: () => void;
}

/**
 * 退回提案中的守門對話框（spec「進行中變更可自看板退回提案中」）：列出引擎
 * 證據（已勾任務數、touched 檔案清單）與出路說明——已勾任務可於任務分頁取消
 * 後重試；touched 需請 agent 判斷。不提供任何清理或強制退回的機械出路，
 * 唯一按鈕只負責關閉。
 */
export function RevertBlockedDialog({ info, onClose }: RevertBlockedDialogProps) {
  const { t } = useI18n();
  if (!info) return null;
  return (
    <AlertDialog open onOpenChange={(o) => !o && onClose()}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{t("revert.blockedTitle")}</AlertDialogTitle>
          <AlertDialogDescription>
            {t("revert.blockedDesc").replace("{name}", info.change)}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <div className="flex flex-col gap-3 text-sm">
          {info.checkedTasks > 0 && (
            <div data-blocked-tasks>
              <div className="font-medium">
                {t("revert.blockedChecked").replace("{n}", String(info.checkedTasks))}
              </div>
              <div className="text-muted-foreground">{t("revert.blockedCheckedHint")}</div>
            </div>
          )}
          {info.touchedFiles.length > 0 && (
            <div data-blocked-touched>
              <div className="font-medium">{t("revert.blockedTouched")}</div>
              <ul className="mt-1 max-h-40 overflow-y-auto rounded-md border border-border bg-muted/45 px-2.5 py-1.5 font-mono text-xs">
                {info.touchedFiles.map((file) => (
                  <li key={file} className="truncate leading-5">
                    {file}
                  </li>
                ))}
              </ul>
              <div className="mt-1 text-muted-foreground">{t("revert.blockedTouchedHint")}</div>
            </div>
          )}
        </div>
        <AlertDialogFooter>
          <AlertDialogAction onClick={onClose}>{t("revert.blockedClose")}</AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}

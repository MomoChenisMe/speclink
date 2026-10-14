import type { ReactNode } from "react";
import { Check } from "lucide-react";
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  Button,
  CopyButton,
  SEMANTIC_SURFACE,
  SEMANTIC_TONE,
  cn,
  useI18n,
} from "@speclink/ui";

// 建立類動作的置中對話框（server-web-detail-pages design D4，取代從側邊滑出的面板）：與 Desktop 新增專案對話框
// 同款——12px 圓角、15px 標題加灰字說明、頁尾細線上方右側為主要動作。AlertDialog 的原生行為：Esc 與
// 取消關閉、點遮罩不關閉（不會誤觸丟掉輸入）、關閉後焦點回到開啟它的按鈕；內容裡的下拉選單掛進
// 對話框自己。`done` 為已建立狀態：標題左側換成綠系勾號，內容改放 SecretDone。

export function FormDialog({
  open,
  onOpenChange,
  title,
  description,
  done = false,
  children,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  description?: string;
  done?: boolean;
  children: ReactNode;
}) {
  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent className="w-[520px]" {...(description ? {} : { "aria-describedby": undefined })}>
        <div className="flex items-start gap-3">
          {done && (
            <span
              aria-hidden="true"
              className={cn(
                "flex h-8 w-8 shrink-0 items-center justify-center rounded-full border",
                SEMANTIC_SURFACE.success,
                SEMANTIC_TONE.success,
              )}
            >
              <Check className="h-4 w-4" />
            </span>
          )}
          <AlertDialogHeader className="min-w-0 flex-1">
            <AlertDialogTitle>{title}</AlertDialogTitle>
            {description && <AlertDialogDescription>{description}</AlertDialogDescription>}
          </AlertDialogHeader>
        </div>
        {children}
      </AlertDialogContent>
    </AlertDialog>
  );
}

/** 頁尾：細線上方、右側排列（與專注流程卡的頁尾同一畫法）。 */
export function FormDialogFooter({ children }: { children: ReactNode }) {
  return <AlertDialogFooter className="border-t border-border pt-3.5">{children}</AlertDialogFooter>;
}

/** 已建立狀態的內容：只顯示一次的祕密值放在綠系成功框（以 aria-live 宣告）並附複製，頁尾只有「完成」。
 *  標題與說明由所在的 FormDialog（`done`）呈現。 */
export function SecretDone({
  value,
  copyLabel,
  note,
  onDone,
}: {
  value: string;
  copyLabel: string;
  note?: string;
  onDone: () => void;
}) {
  const { t } = useI18n();
  return (
    <>
      <div
        role="status"
        aria-live="polite"
        className={cn("flex items-start gap-3 rounded-xl border p-3", SEMANTIC_SURFACE.success)}
      >
        <code className="min-w-0 flex-1 break-all font-mono text-[13px]">{value}</code>
        <CopyButton size="sm" value={value} label={copyLabel} />
      </div>
      {note && <p className="m-0 text-xs text-muted-foreground">{note}</p>}
      <FormDialogFooter>
        <Button type="button" onClick={onDone}>
          {t("common.done")}
        </Button>
      </FormDialogFooter>
    </>
  );
}

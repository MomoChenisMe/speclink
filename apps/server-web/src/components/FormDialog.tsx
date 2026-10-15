import { useState, type ReactNode } from "react";
import { Check } from "lucide-react";
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogHeader,
  AlertDialogTitle,
  Button,
  CopyButton,
  SEMANTIC_SURFACE,
  SEMANTIC_TONE,
  cn,
  useI18n,
} from "@speclink/ui";
import { FocusFooter } from "../layouts/FocusLayout";
import { readFormError } from "../lib/formError";

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

/**
 * 對話框表單的送出狀態：送出中擋住重複送出；失敗時讀出可公開的訊息與欄位錯誤，輸入不動
 * （對話框保持開啟、保留輸入）。`run` 成功後由呼叫端決定關閉或換成已建立狀態。
 */
export function useDialogSubmit(fallback: string) {
  const [pending, setPending] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [fieldErrors, setFieldErrors] = useState<Record<string, string>>({});

  async function submit(run: () => Promise<void>) {
    if (pending) return;
    setPending(true);
    setMessage(null);
    setFieldErrors({});
    try {
      await run();
    } catch (error) {
      const read = readFormError(error, fallback);
      setMessage(read.message);
      setFieldErrors(read.fieldErrors);
    } finally {
      setPending(false);
    }
  }

  return { pending, message, fieldErrors, submit };
}

/** 表單底部的非欄位錯誤；有欄位錯誤時由欄位自己以 role=alert 呈現。 */
export function FormMessage({ message, fieldErrors }: { message: string | null; fieldErrors: Record<string, string> }) {
  if (!message || Object.keys(fieldErrors).length > 0) return null;
  return (
    <p role="alert" className="text-sm text-destructive">
      {message}
    </p>
  );
}

/** 表單頁尾：取消與送出，細線上方右側排列（與專注流程卡的頁尾同一個元件）。 */
export function FormDialogActions({
  submitLabel,
  pending,
  disabled = false,
  onCancel,
}: {
  submitLabel: string;
  pending: boolean;
  disabled?: boolean;
  onCancel: () => void;
}) {
  const { t } = useI18n();
  return (
    <FocusFooter>
      <Button type="button" variant="ghost" onClick={onCancel}>
        {t("common.cancel")}
      </Button>
      <Button type="submit" disabled={pending || disabled}>
        {submitLabel}
      </Button>
    </FocusFooter>
  );
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
      <FocusFooter>
        <Button type="button" onClick={onDone}>
          {t("common.done")}
        </Button>
      </FocusFooter>
    </>
  );
}

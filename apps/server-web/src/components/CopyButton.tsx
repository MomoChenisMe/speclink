import { useState } from "react";
import { Copy } from "lucide-react";
import { Button, useI18n } from "@speclink/ui";

// 一次性祕密（存取金鑰明文、邀請連結）旁的複製鈕：這些值只顯示這一次，要使用者自己
// 反白選取太容易漏字或多選到前後空白。複製結果以 aria-live 宣告，鍵盤與螢幕閱讀器
// 也拿得到回饋。
export function CopyButton({ value, label }: { value: string; label?: string }) {
  const { t } = useI18n();
  const [copied, setCopied] = useState(false);

  return (
    <>
      <Button
        type="button"
        variant="outline"
        size="sm"
        className="shrink-0 gap-1.5"
        aria-label={label}
        onClick={() => {
          void navigator.clipboard?.writeText(value);
          setCopied(true);
        }}
      >
        <Copy aria-hidden="true" className="h-3.5 w-3.5" />
        {copied ? t("common.copied") : t("common.copy")}
      </Button>
      <span role="status" aria-live="polite" className="sr-only">
        {copied ? t("common.copied") : ""}
      </span>
    </>
  );
}

// 指令檔提示橫幅（desktop-app spec「指令檔過期提示」，決策 7）：UpdateBanner 同構
// 的視覺語彙，掛在過期或缺失專案的分頁內容頂部——per 專案、非阻斷，不用 modal
// （阻斷開啟違反溫和定位）。主動作依探測態分文案（過期→更新、缺失→安裝），
// 兩者呼叫同一個再生入口。
import { AlertTriangle, FileWarning } from "lucide-react";
import { Button, useI18n } from "@speclink/ui";

import type { InstructionPromptState } from "../instructionPrompt";

export interface InstructionUpdatePromptProps {
  prompt: InstructionPromptState | null;
  /** 上次主動作的失敗訊息（呈現於原位、動作仍可重試）。 */
  error: string | null;
  /** 再生進行中：主動作停用。 */
  busy: boolean;
  onApply: () => void;
  onDismiss: () => void;
}

export function InstructionUpdatePrompt({
  prompt,
  error,
  busy,
  onApply,
  onDismiss,
}: InstructionUpdatePromptProps) {
  const { t } = useI18n();
  if (!prompt) return null;
  const missing = prompt.kind === "missing";
  const title = t(missing ? "instructions.missingTitle" : "instructions.staleTitle");
  const desc = t(missing ? "instructions.missingDesc" : "instructions.staleDesc").replace(
    "{count}",
    String(prompt.fileCount),
  );
  return (
    <div
      data-testid="instruction-prompt"
      role="status"
      className="mb-4 flex items-start gap-2.5 rounded-md border border-border bg-primary/8 px-3 py-2 text-sm"
    >
      {error ? (
        <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0 text-amber-600 dark:text-amber-400" />
      ) : (
        <FileWarning className="mt-0.5 h-4 w-4 shrink-0 text-primary" />
      )}
      <div className="min-w-0 flex-1">
        <div className="font-medium">{title}</div>
        <div className="text-muted-foreground">{desc}</div>
        {error && (
          <div className="mt-1 text-amber-700 dark:text-amber-400">
            {t("instructions.errorPrefix")}
            {error}
          </div>
        )}
      </div>
      <span className="flex shrink-0 gap-1.5">
        <Button type="button" size="sm" className="h-7" disabled={busy} onClick={onApply}>
          {t(missing ? "instructions.install" : "instructions.update")}
        </Button>
        <Button type="button" size="sm" variant="ghost" className="h-7" onClick={onDismiss}>
          {t("instructions.keep")}
        </Button>
      </span>
    </div>
  );
}

// 技能檔提示確認框（desktop-app spec「指令檔過期提示」；desktop-notice-relocation design D1）：
// 琥珀圖示＋標題＋說明、一張細框卡三列（專案技能檔版本、Speclink 版本、檔案數），按鈕依
// 狀態——過期／缺失三鍵「稍後」「保留現狀」「更新／安裝技能檔」、較新兩鍵「保留現狀」
// 「更新 Speclink」。ConfirmDialog 是兩鍵式，三選項以上各自組 AlertDialog（spec「共用元件
// 唯一來源」）。較新態不提供任何改寫動作：按「更新」把新版檔案改寫回舊內容，正是
// 2026-08-05 事故的形狀——改開設定頁軟體更新卡。「稍後」只關框、「保留現狀」寫入既有略過
// 記憶；更新／安裝不自行關框：成功由提示消失收合，失敗的錯誤留在內容卡下方可重試。
import { AlertCircle } from "lucide-react";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  Button,
  Card,
  SEMANTIC_TONE,
  cn,
  useI18n,
} from "@speclink/ui";

import type { AssetPromptState } from "../assetPrompt";

export interface AssetNoticeDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  prompt: AssetPromptState | null;
  /** 再生進行中：全部動作鍵停用。 */
  busy: boolean;
  /** 上次更新／安裝的失敗訊息（呈現於內容卡下方、可重試）。 */
  error: string | null;
  onApply: () => void;
  onDismiss: () => void;
  /** 較新態的主動作：切到設定頁並捲到軟體更新卡。 */
  onOpenSettingsUpdate: () => void;
}

export function AssetNoticeDialog({
  open,
  onOpenChange,
  prompt,
  busy,
  error,
  onApply,
  onDismiss,
  onOpenSettingsUpdate,
}: AssetNoticeDialogProps) {
  const { t } = useI18n();
  if (!prompt) return null;
  const newer = prompt.kind === "newer";
  // 前兩列同一套技能檔版號（專案的標記 vs 這版 Speclink 帶的），一眼比得出誰新誰舊。
  const rows: Array<[string, string]> = [
    [t("assets.dialog.projectVersion"), prompt.projectVersion ?? "—"],
    [t("assets.dialog.appVersion"), prompt.version],
    [t(newer ? "assets.dialog.revertCount" : "assets.dialog.fileCount"), String(prompt.fileCount)],
  ];
  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent data-testid="asset-notice-dialog">
        <AlertDialogHeader>
          <AlertDialogTitle className="flex items-center gap-2">
            <AlertCircle className={cn("h-5 w-5 shrink-0", SEMANTIC_TONE.warning)} />
            {t(`assets.dialog.${prompt.kind}Title`)}
          </AlertDialogTitle>
          <AlertDialogDescription>{t(`assets.dialog.${prompt.kind}Desc`)}</AlertDialogDescription>
        </AlertDialogHeader>
        <Card size="nested" data-testid="asset-notice-rows" className="divide-y divide-border text-[13px]">
          {rows.map(([label, value]) => (
            <div key={label} className="flex items-center justify-between gap-4 px-3 py-2">
              <span className="text-muted-foreground">{label}</span>
              <span data-testid="asset-notice-value" className="font-medium tabular-nums">
                {value}
              </span>
            </div>
          ))}
        </Card>
        {error && (
          <p role="alert" className={cn("m-0 text-[13px]", SEMANTIC_TONE.danger)}>
            {t("assets.errorPrefix")}
            {error}
          </p>
        )}
        <AlertDialogFooter data-testid="asset-notice-actions">
          {!newer && (
            <Button type="button" variant="ghost" disabled={busy} onClick={() => onOpenChange(false)}>
              {t("assets.dialog.later")}
            </Button>
          )}
          <AlertDialogCancel disabled={busy} onClick={onDismiss}>
            {t("assets.dialog.keep")}
          </AlertDialogCancel>
          {newer ? (
            <AlertDialogAction onClick={onOpenSettingsUpdate}>{t("assets.dialog.updateApp")}</AlertDialogAction>
          ) : (
            <Button type="button" disabled={busy} onClick={onApply}>
              {t(prompt.kind === "missing" ? "assets.dialog.install" : "assets.dialog.update")}
            </Button>
          )}
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}

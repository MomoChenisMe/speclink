// 技能檔提示卡（desktop-app spec「指令檔過期提示」；desktop-notice-relocation design D1）：
// 專案欄底部、專案設定項上方的常駐卡——琥珀淡底、一句標題、一句說明、›；整張可點，開
// 確認框。提示是「這個專案」的事，住在專案欄而非主區：主區不再為它扣高度或釘選。
import { AlertCircle, ChevronRight } from "lucide-react";
import { SEMANTIC_SURFACE, SEMANTIC_TONE, cn, useI18n } from "@speclink/ui";

import type { AssetPromptState } from "../assetPrompt";

export interface AssetNoticeCardProps {
  prompt: AssetPromptState | null;
  onOpen: () => void;
}

export function AssetNoticeCard({ prompt, onOpen }: AssetNoticeCardProps) {
  const { t } = useI18n();
  if (!prompt) return null;
  const title = t(`assets.card.${prompt.kind}Title`);
  const desc = t(`assets.card.${prompt.kind}Desc`).replace("{count}", String(prompt.fileCount));
  return (
    <button
      type="button"
      data-testid="asset-notice-card"
      aria-label={title}
      className={cn(
        "flex w-full items-center gap-2 rounded-xl border p-2.5 text-left transition-colors hover:bg-status-warning/15",
        SEMANTIC_SURFACE.warning,
      )}
      onClick={onOpen}
    >
      <AlertCircle className={cn("h-4 w-4 shrink-0", SEMANTIC_TONE.warning)} />
      <span className="min-w-0 flex-1">
        <span className="block text-xs font-medium leading-tight">{title}</span>
        <span className="mt-0.5 block text-[11px] leading-tight text-muted-foreground">{desc}</span>
      </span>
      <ChevronRight className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
    </button>
  );
}

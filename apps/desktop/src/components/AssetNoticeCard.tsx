// 技能檔提示卡（desktop-app spec「指令檔過期提示」；desktop-notice-relocation design D1）：
// 專案欄底部、專案設定項上方的常駐卡——琥珀淡底、一句標題、一句說明、›；整張可點，開
// 確認框。提示是「這個專案」的事，住在專案欄而非主區：主區不再為它扣高度或釘選。
import { AlertCircle, ChevronRight } from "lucide-react";
import { SEMANTIC_SURFACE, SEMANTIC_TONE, cn, useI18n, type Translator } from "@speclink/ui";

import type { AssetPromptState } from "../assetPrompt";

export interface AssetNoticeCardProps {
  prompt: AssetPromptState | null;
  onOpen: () => void;
}

export function AssetNoticeCard({ prompt, onOpen }: AssetNoticeCardProps) {
  const { t, tn } = useI18n();
  if (!prompt) return null;
  const { title, desc } = cardText(prompt, t, tn);
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

/** 卡片的標題與說明：技能檔三態代入檔案數；只剩規格資料夾待遷移時說明會改名什麼
 * （desktop-spec-dir-migration D2）。 */
function cardText(prompt: AssetPromptState, t: Translator["t"], tn: Translator["tn"]) {
  const { kind, migration } = prompt;
  if (kind !== "migrate") {
    return { title: t(`assets.card.${kind}Title`), desc: tn(`assets.card.${kind}Desc`, prompt.fileCount) };
  }
  const desc =
    migration.state === "pending" && migration.folder === "speclink"
      ? tn("assets.card.migrateFilesDesc", migration.metaFiles)
      : t("assets.card.migrateDesc");
  return { title: t("assets.card.migrateTitle"), desc };
}

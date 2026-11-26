import { useI18n } from "../i18n";
import { cn } from "../lib/utils";
import { relativeDays, useLocalDay } from "../time";
import { Tooltip, TooltipContent, TooltipTrigger } from "./ui/tooltip";

/**
 * 卡片 meta 列尾端的最後更新（spec「看板卡片顯示最後更新」；design D3）：天級相對時間，
 * 與規格頁同一個 relativeDays；提示為「最後更新」加使用者時區的日期時間。缺席或解析
 * 不出來時整段缺席——relativeDays 對解析失敗回原始字串，卡片不顯示它。
 */
export function LastUpdated({ at, className }: { at?: string | null; className?: string }) {
  const { t, tn, locale } = useI18n();
  // 看板開著跨過午夜時重算「今天／昨天」（design D6）；hook 必須在提早 return 之前呼叫。
  useLocalDay();
  if (!at || Number.isNaN(Date.parse(at))) return null;
  const label = t("card.lastUpdated").replace("{date}", new Date(at).toLocaleString(locale));
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <span data-last-updated aria-label={label} className={cn("shrink-0 whitespace-nowrap tabular-nums", className)}>
          {relativeDays(at, { t, tn })}
        </span>
      </TooltipTrigger>
      <TooltipContent>{label}</TooltipContent>
    </Tooltip>
  );
}

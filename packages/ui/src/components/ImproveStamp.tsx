import { Sparkles } from "lucide-react";

import { useI18n } from "../i18n";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "./ui/tooltip";

/** 改進討論的 i18n 詞條 key——卡片小章、已封存側與抽屜標示共用同一份文案。 */
export const IMPROVE_LABEL_KEY = "discussion.kindImprove";

/** kind 是否為改進討論（單一合法值，缺席即一般討論）。 */
export function isImproveKind(kind?: string | null): boolean {
  return kind === "improve";
}

/**
 * 改進討論的行內小章（spec desktop-app「看板討論卡片的改進標示」）：鏡射審查章
 * 樣式——僅圖示＋tooltip 狀態詞，不加文字列維持卡片極簡。色調取 indigo：審查／
 * 驗證章的紫是品質站蓋章專屬，此章標的是討論型別而非狀態，兩者不共色。
 */
export function ImproveStamp() {
  const { t } = useI18n();
  const label = t(IMPROVE_LABEL_KEY);
  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <span
            aria-label={label}
            className="shrink-0 text-indigo-600 dark:text-indigo-400"
          >
            <Sparkles className="h-3.5 w-3.5" />
          </span>
        </TooltipTrigger>
        <TooltipContent>{label}</TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}

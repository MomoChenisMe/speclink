import { useI18n } from "../i18n";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "./ui/tooltip";
import { IMPROVE_CHIP_TONE, IMPROVE_LABEL_KEY, IMPROVE_TONE, ImproveIcon } from "./improveStyle";

/**
 * 改進討論的行內小章（spec desktop-app「看板討論卡片的改進標示」）：鏡射審查章
 * 樣式——僅圖示＋tooltip 狀態詞，不加文字列維持卡片極簡。tone／icon／詞條的
 * 對照集中在 improveStyle.tsx。
 */
export function ImproveStamp() {
  const { t } = useI18n();
  const label = t(IMPROVE_LABEL_KEY);
  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <span aria-label={label} className={`shrink-0 ${IMPROVE_TONE}`}>
            <ImproveIcon className="h-3.5 w-3.5" />
          </span>
        </TooltipTrigger>
        <TooltipContent>{label}</TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}

/**
 * 改進討論的章籤（spec「討論抽屜的改進標示」）：抽屜有空間故以圖示＋狀態詞
 * 直出；活討論抽屜與封存抽屜共用同一份。
 */
export function ImproveChip() {
  const { t } = useI18n();
  return (
    <span
      data-discussion-kind="improve"
      className={`inline-flex items-center gap-1 rounded-full ${IMPROVE_CHIP_TONE} px-2 py-0.5 font-semibold ${IMPROVE_TONE}`}
    >
      <ImproveIcon className="h-3 w-3" />
      {t(IMPROVE_LABEL_KEY)}
    </span>
  );
}

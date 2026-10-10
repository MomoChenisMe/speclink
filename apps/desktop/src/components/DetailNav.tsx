// 麵包屑右端的 ‹ n／m ›（design D2）：在宿主清單的呈現順序內切換上一個／下一個，
// 到頭停用不循環；目前項目不在序列中（例如剛被外部刪除）時整組缺席。
import { ChevronLeft, ChevronRight } from "lucide-react";
import { Button, useI18n } from "@speclink/ui";

export function DetailNav({
  index,
  total,
  onStep,
}: {
  /** 目前項目在序列中的 0 起索引；-1＝不在序列中。 */
  index: number;
  total: number;
  onStep: (delta: -1 | 1) => void;
}) {
  const { t } = useI18n();
  if (index < 0 || total === 0) return null;
  return (
    <div className="flex shrink-0 items-center gap-0.5 pr-3">
      <Button
        variant="subtle"
        size="icon-xs"
        aria-label={t("detail.prev")}
        disabled={index === 0}
        onClick={() => onStep(-1)}
      >
        <ChevronLeft className="h-4 w-4" />
      </Button>
      <span
        aria-label={t("detail.position")
          .replace("{n}", String(index + 1))
          .replace("{m}", String(total))}
        className="min-w-[3.5em] text-center font-mono text-[11px] text-muted-foreground tabular-nums"
      >
        {index + 1} / {total}
      </span>
      <Button
        variant="subtle"
        size="icon-xs"
        aria-label={t("detail.next")}
        disabled={index >= total - 1}
        onClick={() => onStep(1)}
      >
        <ChevronRight className="h-4 w-4" />
      </Button>
    </div>
  );
}

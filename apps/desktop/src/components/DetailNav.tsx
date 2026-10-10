// 詳情頁的標題列導覽：麵包屑左側的返回鈕（desktop-readonly-detail-pages design D8），與右端的
// ‹ n／m ›（design D2）——在宿主清單的呈現順序內切換上一個／下一個，到頭停用不循環；
// 目前項目不在序列中（例如剛被外部刪除）時整組缺席。
import { ArrowLeft, ChevronLeft, ChevronRight } from "lucide-react";
import { Button, useI18n } from "@speclink/ui";

import type { Platform } from "../platform";

/** 返回快捷鍵（design D8）：macOS 為 ⌘[ 與 ⌘←、其他平台 Alt+←。以實體按鍵（e.code）判定——
 * 中文輸入法啟用時 `[` 鍵送進頁面的 e.key 不一定是「[」；e.key 留作沒有 code 的事件後備。 */
export function isBackShortcut(
  e: Pick<KeyboardEvent, "key" | "code" | "metaKey" | "altKey" | "ctrlKey" | "shiftKey">,
  platform: Platform,
): boolean {
  if (e.shiftKey || e.ctrlKey) return false;
  const leftArrow = e.code === "ArrowLeft" || e.key === "ArrowLeft";
  if (platform === "macos") {
    return e.metaKey && !e.altKey && (leftArrow || e.code === "BracketLeft" || e.key === "[");
  }
  return e.altKey && !e.metaKey && leftArrow;
}

/** 返回鈕：回宿主清單頁（與 Esc、麵包屑中段同一個動作），提示標示 Esc 與返回快捷鍵。 */
export function DetailBack({
  page,
  keys,
  onBack,
}: {
  /** 宿主頁名（規格、已封存、變更、手冊）。 */
  page: string;
  /** 依平台的返回快捷鍵（⌘[ 或 Alt+←）。 */
  keys: string;
  onBack: () => void;
}) {
  const { t } = useI18n();
  const label = t("detail.back").replace("{page}", page);
  return (
    <Button
      variant="subtle"
      size="icon-xs"
      aria-label={label}
      title={t("detail.backHint").replace("{label}", label).replace("{keys}", keys)}
      className="-ml-1.5 mr-1.5 shrink-0"
      onClick={onBack}
    >
      <ArrowLeft className="h-4 w-4" />
    </Button>
  );
}

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

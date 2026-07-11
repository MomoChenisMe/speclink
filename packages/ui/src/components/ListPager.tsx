import { ChevronLeft, ChevronRight } from "lucide-react";

import { useI18n } from "../i18n";
import { Button } from "./ui/button";

/** 每頁筆數（spec「清單最新在前與換頁瀏覽」）——清單元件與測試共用。 */
export const PAGE_SIZE = 20;

export interface ListPagerProps {
  /** 目前頁碼（1 起算）。 */
  page: number;
  /** 總頁數。 */
  pageCount: number;
  /** 換頁回呼——受控形態，頁碼狀態由呼叫端持有。 */
  onPage: (next: number) => void;
}

/** 換頁控制列（design D2：自建受控元件，不採 shadcn Pagination）：上一頁鈕＋
 * 「第 N／M 頁」字樣＋下一頁鈕；pageCount ≤ 1（單頁或空清單）不渲染任何內容。 */
export function ListPager({ page, pageCount, onPage }: ListPagerProps) {
  const { t } = useI18n();
  if (pageCount <= 1) return null;
  return (
    <div className="flex items-center justify-center gap-2 py-1">
      <Button
        type="button"
        variant="outline"
        size="sm"
        className="h-7 gap-1"
        disabled={page <= 1}
        onClick={() => onPage(page - 1)}
      >
        <ChevronLeft className="h-3.5 w-3.5" /> {t("pager.prev")}
      </Button>
      <span className="text-xs text-muted-foreground tabular-nums">
        {t("pager.page").replace("{n}", String(page)).replace("{m}", String(pageCount))}
      </span>
      <Button
        type="button"
        variant="outline"
        size="sm"
        className="h-7 gap-1"
        disabled={page >= pageCount}
        onClick={() => onPage(page + 1)}
      >
        {t("pager.next")} <ChevronRight className="h-3.5 w-3.5" />
      </Button>
    </div>
  );
}

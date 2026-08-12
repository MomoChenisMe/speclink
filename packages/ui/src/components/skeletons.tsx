// 載入中佔位組件（design D4）：以 Skeleton 基元組合出三種形狀，模仿其所替代的
// 真實內容輪廓——首訪空窗期看得出「有東西正在載」，與真空態文案可區分。
// 全數 aria-busy，且不含任何文字（載入中不冒充空態文案）。
// 同檔另收「載入失敗」的終態列：它替代的是同一塊卡片區，差別在載入已經結束。
import { CloudOff } from "lucide-react";

import { useI18n } from "../i18n";
import { Card } from "./ui/card";
import { Skeleton } from "./ui/skeleton";

/** 看板佔位卡：模仿 ChangeCard 的名稱列＋進度條輪廓。外框直接用 Card 而非自刻。 */
export function CardSkeleton() {
  return (
    <Card aria-busy="true" className="p-3 flex flex-col gap-2">
      <Skeleton className="h-3.5 w-2/3" />
      <Skeleton className="h-2 w-full" />
    </Card>
  );
}

/** 看板欄內的佔位卡組：兩張足以讀出「這欄有內容正在載」，不假裝知道實際筆數。 */
export function ColumnSkeleton() {
  return (
    <>
      <CardSkeleton />
      <CardSkeleton />
    </>
  );
}

/** 看板欄內的載入失敗終態列（spec「首訪載入失敗終態呈現」）：讀不到 ≠ 確認是
    空的，故取代空態文案；載入已結束，故不是 skeleton（無 aria-busy）。 */
export function ColumnLoadFailed() {
  const { t } = useI18n();
  return (
    <div
      data-testid="column-load-failed"
      role="status"
      className="flex items-start gap-1.5 px-1.5 pt-2 text-xs text-muted-foreground"
    >
      <CloudOff className="h-3.5 w-3.5 shrink-0" />
      <span>{t("kanban.loadFailed")}</span>
    </div>
  );
}

/** 面板佔位列：模仿 tray 面板分區內單行變更／討論列。 */
export function RowSkeleton() {
  return (
    <div aria-busy="true" className="flex items-center gap-2 px-2 py-1.5">
      <Skeleton className="h-2.5 w-2.5 shrink-0 rounded-full" />
      <Skeleton className="h-2.5 w-2/5" />
    </div>
  );
}

/** 文件佔位：標題條＋數行內文條，模仿 markdown 文件的段落輪廓。 */
export function DocSkeleton() {
  return (
    <div aria-busy="true" className="flex flex-col gap-2 py-2">
      <Skeleton className="h-4 w-32" />
      <Skeleton className="h-3 w-full" />
      <Skeleton className="h-3 w-11/12" />
      <Skeleton className="h-3 w-4/5" />
    </div>
  );
}

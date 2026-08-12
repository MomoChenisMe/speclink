// 載入中佔位組件（design D4）：以 Skeleton 基元組合出三種形狀，模仿其所替代的
// 真實內容輪廓——首訪空窗期看得出「有東西正在載」，與真空態文案可區分。
// 全數 aria-busy，且不含任何文字（載入中不冒充空態文案）。
import { Skeleton } from "./ui/skeleton";

/** 看板佔位卡：模仿 ChangeCard 的名稱列＋進度條輪廓。 */
export function CardSkeleton() {
  return (
    <div aria-busy="true" className="rounded-xl border bg-card p-3 flex flex-col gap-2">
      <Skeleton className="h-3.5 w-2/3" />
      <Skeleton className="h-2 w-full" />
    </div>
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

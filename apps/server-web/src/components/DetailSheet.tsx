import type { ReactNode } from "react";
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from "@speclink/ui";

// 管理面所有建立與編輯的共用抽屜結構（design：建立與編輯一律以 Sheet 抽屜承載）。
// 標題列、可選副標、可選動作列與內容捲動區；關閉鈕與 focus trap 由 Sheet 原語提供，
// 關閉後 focus 自動歸還觸發元素。窄螢幕改為全寬（sm 以下 w-full）。
export function DetailSheet({
  open,
  onOpenChange,
  title,
  description,
  actions,
  children,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  description?: ReactNode;
  /** 標題下方的動作列（破壞性動作仍各自以 AlertDialog 確認）。 */
  actions?: ReactNode;
  children: ReactNode;
}) {
  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent className="w-full sm:w-[440px]">
        <SheetHeader className="pr-8">
          <SheetTitle>{title}</SheetTitle>
          {description && <SheetDescription asChild={typeof description !== "string"}>{description}</SheetDescription>}
        </SheetHeader>
        {actions && <div className="flex flex-wrap gap-2">{actions}</div>}
        <div className="min-w-0 flex-1">{children}</div>
      </SheetContent>
    </Sheet>
  );
}

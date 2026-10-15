import * as React from "react";

import { cn } from "../../lib/utils";

// shadcn Table（Tailwind v4）：語意 <table> 結構＋themed 邊框。外層 div 提供窄容器
// 橫向捲動，避免寬表格撐破版面。列邊框明確用 border-border——裸 border 於 Tailwind v4
// 會落到 currentColor（≈黑框）。`wrapped={false}` 不包外層：呼叫端自己就是捲動區（列表卡）時，
// 外層的 overflow-x-auto 會自成捲動容器，sticky 表頭就黏在那層、隨捲動區捲走。
export const Table = React.forwardRef<
  HTMLTableElement,
  React.HTMLAttributes<HTMLTableElement> & { wrapped?: boolean }
>(({ className, wrapped = true, ...props }, ref) => {
  const table = <table ref={ref} className={cn("w-full caption-bottom text-sm", className)} {...props} />;
  return wrapped ? <div className="relative w-full overflow-x-auto">{table}</div> : table;
});
Table.displayName = "Table";

/** `sticky`：表頭固定在捲動區頂端並帶卡片底色蓋住底下的列（列表卡內的表格；搭配 `Table wrapped={false}`）。 */
export const TableHeader = React.forwardRef<
  HTMLTableSectionElement,
  React.HTMLAttributes<HTMLTableSectionElement> & { sticky?: boolean }
>(({ className, sticky = false, ...props }, ref) => (
  <thead ref={ref} className={cn(sticky && "sticky top-0 z-10 bg-card", className)} {...props} />
));
TableHeader.displayName = "TableHeader";

export const TableBody = React.forwardRef<
  HTMLTableSectionElement,
  React.HTMLAttributes<HTMLTableSectionElement>
>(({ className, ...props }, ref) => (
  <tbody ref={ref} className={cn("[&_tr:last-child]:border-0", className)} {...props} />
));
TableBody.displayName = "TableBody";

export const TableRow = React.forwardRef<
  HTMLTableRowElement,
  React.HTMLAttributes<HTMLTableRowElement>
>(({ className, ...props }, ref) => (
  <tr
    ref={ref}
    className={cn("border-b border-border transition-colors hover:bg-muted/50", className)}
    {...props}
  />
));
TableRow.displayName = "TableRow";

export const TableHead = React.forwardRef<
  HTMLTableCellElement,
  React.ThHTMLAttributes<HTMLTableCellElement>
>(({ className, ...props }, ref) => (
  <th
    ref={ref}
    className={cn(
      "h-10 px-3 text-left align-middle text-xs font-medium text-muted-foreground",
      className,
    )}
    {...props}
  />
));
TableHead.displayName = "TableHead";

export const TableCell = React.forwardRef<
  HTMLTableCellElement,
  React.TdHTMLAttributes<HTMLTableCellElement>
>(({ className, ...props }, ref) => (
  <td ref={ref} className={cn("px-3 py-2.5 align-middle", className)} {...props} />
));
TableCell.displayName = "TableCell";

export const TableCaption = React.forwardRef<
  HTMLTableCaptionElement,
  React.HTMLAttributes<HTMLTableCaptionElement>
>(({ className, ...props }, ref) => (
  <caption ref={ref} className={cn("mt-3 text-sm text-muted-foreground", className)} {...props} />
));
TableCaption.displayName = "TableCaption";

import type { ReactNode } from "react";
import {
  Card,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  useI18n,
} from "@speclink/ui";
import { useMediaQuery } from "../lib/useMediaQuery";

// 管理列表的資料呈現（server-web-console「共用設計系統維持高密度可存取體驗」）：
// ≥1024px 為表格，更窄改為卡片列。窄螢幕不能只讓表格自己橫捲或被裁切——欄數一多，
// 前者讓整頁跟著橫捲，後者直接讓內容消失。同一份欄位描述同時餵給兩種版面，欄位加減
// 只需改一處，兩種寬度不會漂移。

export type Column<T> = {
  /** 表頭文字，也是卡片模式每列的標籤。 */
  header: string;
  cell: (item: T) => ReactNode;
  /** 卡片模式的標題列（不加標籤、字級較大）。每份描述恰有一欄設為 true。 */
  primary?: boolean;
};

export function DataList<T>({
  items,
  columns,
  keyOf,
  onSelect,
  action,
}: {
  items: T[];
  columns: Column<T>[];
  keyOf: (item: T) => string;
  /** 有值時整列／整張卡片可點（滑鼠便利；鍵盤走 action 提供的按鈕）。 */
  onSelect?: (item: T) => void;
  /** 列尾動作（詳細資料入口或撤銷）。 */
  action?: (item: T) => ReactNode;
}) {
  const { t } = useI18n();
  const narrow = useMediaQuery("(max-width: 1023px)");

  if (narrow) {
    return (
      <ul className="space-y-3">
        {items.map((item) => (
          <li key={keyOf(item)}>
            <Card
              className={`p-4 ${onSelect ? "cursor-pointer" : ""}`}
              onClick={onSelect ? () => onSelect(item) : undefined}
            >
              <div className="flex items-start gap-3">
                <dl className="min-w-0 flex-1 space-y-1">
                  {columns.map((column) =>
                    column.primary ? (
                      <div key={column.header} className="min-w-0 font-medium">
                        <dt className="sr-only">{column.header}</dt>
                        <dd className="min-w-0 break-words">{column.cell(item)}</dd>
                      </div>
                    ) : (
                      <div key={column.header} className="flex min-w-0 gap-2 text-sm">
                        <dt className="shrink-0 text-muted-foreground">{column.header}</dt>
                        <dd className="min-w-0 break-words">{column.cell(item)}</dd>
                      </div>
                    ),
                  )}
                </dl>
                {action && <div className="shrink-0">{action(item)}</div>}
              </div>
            </Card>
          </li>
        ))}
      </ul>
    );
  }

  return (
    <Card className="overflow-hidden">
      <Table>
        <TableHeader>
          <TableRow>
            {columns.map((column) => (
              <TableHead key={column.header}>{column.header}</TableHead>
            ))}
            {action && (
              <TableHead>
                <span className="sr-only">{t("common.actions")}</span>
              </TableHead>
            )}
          </TableRow>
        </TableHeader>
        <TableBody>
          {items.map((item) => (
            <TableRow
              key={keyOf(item)}
              className={onSelect ? "cursor-pointer" : undefined}
              onClick={onSelect ? () => onSelect(item) : undefined}
            >
              {columns.map((column) => (
                <TableCell key={column.header}>{column.cell(item)}</TableCell>
              ))}
              {action && <TableCell>{action(item)}</TableCell>}
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </Card>
  );
}

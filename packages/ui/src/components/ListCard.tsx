import type { ReactNode, Ref } from "react";

import { cn } from "../lib/utils";
import { Card } from "./ui/card";

export interface ListCardProps {
  /** 卡頂槽（已封存頁的卡片標頭式分頁列）；有它時內容卡以 rounded-t-none border-t-0 接在其下。 */
  header?: ReactNode;
  /** 卡底固定槽（換頁工具列）；槽本身不畫分隔線——分隔線屬工具列，工具列不渲染時卡底不留空線。 */
  footer?: ReactNode;
  children: ReactNode;
  /** 內容捲動容器（data-list-scroll）——換頁後捲回頂部、聚焦列捲入視窗的目標。 */
  scrollRef?: Ref<HTMLDivElement>;
}

/**
 * 列表卡（spec desktop-app「清單最新在前與換頁瀏覽」；design D1）：白底細框容器填滿
 * 主區剩餘高度，列在卡內縱向捲動，header／footer 固定不隨列捲動。header 為
 * Tabs card variant 的分頁列時，內容卡依該原語的既定接法去掉上圓角與上框線。
 */
export function ListCard({ header, footer, children, scrollRef }: ListCardProps) {
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      {header}
      <Card
        data-list-card
        className={cn("flex min-h-0 flex-1 flex-col overflow-hidden", header && "rounded-t-none border-t-0")}
      >
        <div ref={scrollRef} data-list-scroll className="min-h-0 flex-1 overflow-y-auto">
          {children}
        </div>
        {footer && (
          <div data-list-footer className="shrink-0">
            {footer}
          </div>
        )}
      </Card>
    </div>
  );
}

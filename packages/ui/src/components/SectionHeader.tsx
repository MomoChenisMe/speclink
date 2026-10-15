import type { ReactNode } from "react";

import { cn } from "../lib/utils";
import { Badge } from "./ui/badge";

export interface SectionHeaderProps {
  icon?: ReactNode;
  label: string;
  /** 項目計數；未給（例如載入中、計數未知）則不出徽章，不謊報 0。 */
  count?: number;
  /** 計數徽章配色（例如生命週期 STAGE_BADGE）；未給為中性（Badge 的 secondary 變體）。 */
  countClassName?: string;
  /** 列尾動作。 */
  action?: ReactNode;
  className?: string;
}

/** 區段標題（spec desktop-app「共用元件唯一來源」；design D8）：xs 粗體灰字一列，計數為徽章。 */
export function SectionHeader({ icon, label, count, countClassName, action, className }: SectionHeaderProps) {
  return (
    <div className={cn("flex items-center gap-1.5 text-xs font-semibold text-muted-foreground", className)}>
      {icon}
      {label}
      {count !== undefined && (
        <Badge
          data-testid="section-count"
          className={cn("ml-auto h-5 min-w-5 justify-center px-1.5 font-semibold tabular-nums", countClassName)}
        >
          {count}
        </Badge>
      )}
      {action && <div className={cn("flex items-center", count === undefined && "ml-auto")}>{action}</div>}
    </div>
  );
}

import type { HTMLAttributes } from "react";

import { cn } from "../lib/utils";
import { SEMANTIC_SURFACE, SEMANTIC_TONE } from "../tone";

/** 嚴重度色：CRITICAL 破壞色、WARNING 琥珀、SUGGESTION 中性；三紅分工（tone.ts 表頭）之外
 * 不另立一紅。以小寫比對——工單寫 CRITICAL、分析報告寫 Critical，同一級同一色。 */
const SEVERITY_TONE: Record<string, string> = {
  critical: "bg-destructive/15 text-destructive",
  warning: `${SEMANTIC_SURFACE.warning} ${SEMANTIC_TONE.warning}`,
  suggestion: "bg-muted text-muted-foreground",
};

/** 嚴重度色章：審查／驗證工單、分析面板與 desktop 審查規則頁共用同一個元件與尺寸。
 * 預設顯示嚴重度名稱，`children` 可換成「WARNING 3」之類的內容；未知嚴重度走中性色。 */
export function SeverityTag({
  severity,
  className,
  children,
  ...rest
}: { severity: string } & HTMLAttributes<HTMLSpanElement>) {
  return (
    <span
      {...rest}
      className={cn(
        "shrink-0 rounded px-1 py-0.5 text-[10px] font-medium",
        SEVERITY_TONE[severity.toLowerCase()] ?? SEVERITY_TONE.suggestion,
        className,
      )}
    >
      {children ?? severity}
    </span>
  );
}

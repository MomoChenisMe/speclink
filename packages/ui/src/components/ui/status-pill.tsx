import * as React from "react";

import { cn } from "../../lib/utils";

/** 狀態籤的語意色鍵：每個 tone 對應一個 theme token，neutral 走中性灰。 */
export type StatusPillTone =
  | "neutral"
  | "primary"
  | "progress"
  | "success"
  | "warning"
  | "destructive"
  | "stamp"
  | "improve"
  | "manual";

/** 以 i18n 鍵描述的狀態籤內容：狀態對照函式回傳它，呈現端以 t(key) 填進 StatusPill。 */
export interface StatusPillLabel {
  key: string;
  tone: StatusPillTone;
}

/* tone → token class：12% 淡底＋本色字。色值只在 theme.css，這裡只對照 token。 */
const TONE_CLASS: Record<StatusPillTone, string> = {
  neutral: "bg-muted text-muted-foreground",
  primary: "bg-primary/12 text-primary",
  progress: "bg-status-progress/12 text-status-progress",
  success: "bg-status-success/12 text-status-success",
  warning: "bg-status-warning/12 text-status-warning",
  destructive: "bg-destructive/12 text-destructive",
  stamp: "bg-stamp/12 text-stamp",
  improve: "bg-improve/12 text-improve",
  manual: "bg-status-manual/12 text-status-manual",
};

export interface StatusPillProps extends React.HTMLAttributes<HTMLSpanElement> {
  tone: StatusPillTone;
  icon?: React.ReactNode;
}

/** 狀態籤（design D5）：圓角全圓、語意色淡底深字；詳情頁的章籤、討論狀態籤與
   衍生變更階段籤共用。`icon` 置於文字之前。轉發 ref 與其餘屬性，可直接當提示的
   trigger（asChild）。 */
export const StatusPill = React.forwardRef<HTMLSpanElement, StatusPillProps>(
  ({ tone, icon, children, className, ...props }, ref) => (
    <span
      ref={ref}
      className={cn(
        "inline-flex h-5 shrink-0 items-center gap-1 rounded-full px-2 text-[11px] font-medium whitespace-nowrap",
        TONE_CLASS[tone],
        className,
      )}
      {...props}
    >
      {icon}
      {children}
    </span>
  ),
);
StatusPill.displayName = "StatusPill";

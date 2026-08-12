import * as React from "react";

import { cn } from "../../lib/utils";

/** 載入中佔位灰塊（shadcn 標準基元）。動畫逐處加 motion-reduce: 是本 repo 慣例。 */
export function Skeleton({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn("animate-pulse motion-reduce:animate-none rounded-md bg-muted", className)}
      {...props}
    />
  );
}

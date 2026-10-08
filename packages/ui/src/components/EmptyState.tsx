import type { ComponentType, ReactNode } from "react";

import { cn } from "../lib/utils";

export interface EmptyStateProps {
  icon: ComponentType<{ className?: string; "aria-hidden"?: boolean | "true" | "false" }>;
  title: string;
  description?: string;
  /** 該頁的 primary action（通常為 default 按鈕）。 */
  action?: ReactNode;
  /** 只放版面（例如 h-full 填滿主區）；外觀由元件決定。 */
  className?: string;
}

/** 空狀態（spec desktop-app「共用元件唯一來源」；design D8）：置中直欄的圖示、標題、
 *  一句說明與動作。桌面零分頁引導頁與 server web 管理列表共用。 */
export function EmptyState({ icon: Icon, title, description, action, className }: EmptyStateProps) {
  return (
    <div className={cn("flex flex-col items-center justify-center gap-3 px-6 py-10 text-center", className)}>
      <Icon aria-hidden="true" className="h-10 w-10 text-muted-foreground/40" />
      <h2 className="text-2xl font-normal">{title}</h2>
      {description && <p className="max-w-md text-[13px] text-muted-foreground">{description}</p>}
      {action && <div className="mt-1">{action}</div>}
    </div>
  );
}

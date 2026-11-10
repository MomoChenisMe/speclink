import type { ReactNode } from "react";
import { Slot, Slottable } from "@radix-ui/react-slot";

import { cn } from "../lib/utils";

export interface NavItemProps {
  /** 子項變體不傳。 */
  icon?: ReactNode;
  label: string;
  /** `sub`：上層分類底下的子項——無圖示、28px 高、文字左緣與預設變體的文字對齊
   *  （左內距 10px＋圖示 16px＋間距 8px＝34px）。 */
  variant?: "default" | "sub";
  active?: boolean;
  /** 右端計數徽章（如已封存數）；未給即不渲染。 */
  count?: number;
  /** 標籤之後的標記槽（如設定分類項的解析失敗警示點）；不併入無障礙名稱時由呼叫端設 aria-hidden。 */
  badge?: ReactNode;
  /** 設定時無障礙名稱不被徽章數字污染。 */
  ariaLabel?: string;
  onClick?: () => void;
  /** 渲染成唯一子元素（如路由連結），圖示與標籤放進子元素內。 */
  asChild?: boolean;
  children?: ReactNode;
  /** 只放版面（如 mt-auto 沉底）。 */
  className?: string;
}

/** 導覽項（design D4）：desktop 專案欄與 server-web 管理面導覽共用。 */
export function NavItem({
  icon,
  label,
  variant = "default",
  active,
  count,
  badge,
  ariaLabel,
  onClick,
  asChild,
  children,
  className,
}: NavItemProps) {
  const Comp = asChild ? Slot : "button";
  return (
    <Comp
      type={asChild ? undefined : "button"}
      aria-label={ariaLabel}
      aria-current={active ? "page" : undefined}
      onClick={onClick}
      className={cn(
        "flex w-full items-center gap-2 rounded-lg text-[13px] transition-colors [&_svg]:h-4 [&_svg]:w-4 [&_svg]:shrink-0",
        variant === "sub" ? "h-7 pl-[34px] pr-2.5" : "h-8 px-2.5",
        active ? "bg-primary/12 font-medium text-primary" : "text-foreground hover:bg-foreground/5",
        className,
      )}
    >
      {variant === "default" && icon}
      {asChild && <Slottable>{children}</Slottable>}
      <span className="truncate">{label}</span>
      {badge}
      {count !== undefined && (
        <span className="ml-auto inline-flex h-4 min-w-4 items-center justify-center rounded-full bg-muted px-1 text-[11px] tabular-nums text-muted-foreground">
          {count}
        </span>
      )}
    </Comp>
  );
}

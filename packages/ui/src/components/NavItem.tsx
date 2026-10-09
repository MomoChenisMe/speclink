import type { ReactNode } from "react";
import { Slot, Slottable } from "@radix-ui/react-slot";

import { cn } from "../lib/utils";

export interface NavItemProps {
  icon: ReactNode;
  label: string;
  active?: boolean;
  /** 右端計數徽章（如已封存數）；未給即不渲染。 */
  count?: number;
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
  active,
  count,
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
        "flex h-8 w-full items-center gap-2 rounded-lg px-2.5 text-[13px] transition-colors [&_svg]:h-4 [&_svg]:w-4 [&_svg]:shrink-0",
        active ? "bg-primary/12 font-medium text-primary" : "text-foreground hover:bg-foreground/5",
        className,
      )}
    >
      {icon}
      {asChild && <Slottable>{children}</Slottable>}
      <span className="truncate">{label}</span>
      {count !== undefined && (
        <span className="ml-auto inline-flex h-4 min-w-4 items-center justify-center rounded-full bg-muted px-1 text-[11px] tabular-nums text-muted-foreground">
          {count}
        </span>
      )}
    </Comp>
  );
}

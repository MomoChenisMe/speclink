// shadcn/ui tooltip 原語（Radix）——teal 設計系統的一部分。
import * as React from "react";
import * as TooltipPrimitive from "@radix-ui/react-tooltip";

import { cn } from "../../lib/utils";

// spec「主題化提示統一延遲」：停留延遲的單一預設在此，個別介面不再自訂。
// skipDelay 沿用 Radix 既有行為——連續於多個觸發點間移動時第二個起立即顯示。
const TooltipProvider = ({
  delayDuration = 300,
  ...props
}: React.ComponentPropsWithoutRef<typeof TooltipPrimitive.Provider>) => (
  <TooltipPrimitive.Provider delayDuration={delayDuration} {...props} />
);

const Tooltip = TooltipPrimitive.Root;

const TooltipTrigger = TooltipPrimitive.Trigger;

const TooltipContent = React.forwardRef<
  React.ElementRef<typeof TooltipPrimitive.Content>,
  React.ComponentPropsWithoutRef<typeof TooltipPrimitive.Content>
>(({ className, sideOffset = 4, ...props }, ref) => (
  <TooltipPrimitive.Portal>
    <TooltipPrimitive.Content
      ref={ref}
      sideOffset={sideOffset}
      className={cn(
        // 反色中性氣泡（shadcn 傳統深底）：主色實心氣泡會與「已就緒」實心徽章撞色。
        "z-50 overflow-hidden rounded-md bg-foreground px-3 py-1.5 text-xs text-background",
        // Radix Tooltip 開啟態的 data-state 是 delayed-open／instant-open（沒有 open）：
        // 進場變體要掛這兩個值，掛 data-[state=open] 永遠不命中、進場動畫直接消失。
        "data-[state=delayed-open]:animate-in data-[state=instant-open]:animate-in data-[state=delayed-open]:fade-in-0 data-[state=instant-open]:fade-in-0 data-[state=delayed-open]:zoom-in-95 data-[state=instant-open]:zoom-in-95",
        "data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 motion-reduce:animate-none",
        className,
      )}
      {...props}
    />
  </TooltipPrimitive.Portal>
));
TooltipContent.displayName = TooltipPrimitive.Content.displayName;

export { Tooltip, TooltipTrigger, TooltipContent, TooltipProvider };

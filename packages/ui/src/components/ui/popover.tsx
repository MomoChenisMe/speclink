// shadcn/ui popover 原語（Radix）——teal 設計系統的一部分。
import * as React from "react";
import * as PopoverPrimitive from "@radix-ui/react-popover";

import { cn } from "../../lib/utils";
import { usePortalContainer } from "./portal-container";

const Popover = PopoverPrimitive.Root;

const PopoverTrigger = PopoverPrimitive.Trigger;

const PopoverContent = React.forwardRef<
  React.ElementRef<typeof PopoverPrimitive.Content>,
  React.ComponentPropsWithoutRef<typeof PopoverPrimitive.Content>
>(({ className, align = "start", sideOffset = 4, ...props }, ref) => {
  // 在抽屜／對話框內時 portal 進該容器，否則維持 body（見 portal-container.tsx）。
  const container = usePortalContainer();
  return (
    <PopoverPrimitive.Portal container={container ?? undefined}>
      <PopoverPrimitive.Content
        ref={ref}
        align={align}
        sideOffset={sideOffset}
        className={cn(
          // 浮層底色用 bg-card：這份 theme 沒有 --popover，上游 shadcn 的 bg-popover
          // 會產出解析不到值的宣告，浮層直接變透明。Select 與 Sheet 也是 bg-card。
          "z-50 rounded-md border border-border bg-card p-1 text-card-foreground shadow-md outline-none",
          "animate-in fade-in-0 zoom-in-95",
          className,
        )}
        {...props}
      />
    </PopoverPrimitive.Portal>
  );
});
PopoverContent.displayName = PopoverPrimitive.Content.displayName;

export { Popover, PopoverTrigger, PopoverContent };

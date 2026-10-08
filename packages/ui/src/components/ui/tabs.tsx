import * as React from "react";
import * as TabsPrimitive from "@radix-ui/react-tabs";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "../../lib/utils";

export const Tabs = TabsPrimitive.Root;

const tabsListVariants = cva("flex w-full items-center", {
  variants: {
    variant: {
      /** 整列底線：預設。各頁版面尚未把內容放進卡片時用這款，不畫外框。 */
      underline: "border-b border-border",
      /** 卡片標頭：作為內容卡的頂部，內容卡以 rounded-t-none border-t-0 接在下方。 */
      card: "px-2 bg-card border border-border rounded-t-2xl",
    },
  },
  defaultVariants: { variant: "underline" },
});

/**
 * 分頁列。`actions` 放頁級動作，渲染在同一列右端、tablist 之外（tablist 只容納分頁）。
 */
export const TabsList = React.forwardRef<
  React.ElementRef<typeof TabsPrimitive.List>,
  React.ComponentPropsWithoutRef<typeof TabsPrimitive.List> &
    VariantProps<typeof tabsListVariants> & { actions?: React.ReactNode }
>(({ className, actions, variant, ...props }, ref) => (
  <div className={cn(tabsListVariants({ variant }), className)}>
    <TabsPrimitive.List ref={ref} className="flex items-center" {...props} />
    {actions && <div className="ml-auto flex items-center gap-1">{actions}</div>}
  </div>
));
TabsList.displayName = "TabsList";

export const TabsTrigger = React.forwardRef<
  React.ElementRef<typeof TabsPrimitive.Trigger>,
  React.ComponentPropsWithoutRef<typeof TabsPrimitive.Trigger>
>(({ className, ...props }, ref) => (
  <TabsPrimitive.Trigger
    ref={ref}
    className={cn(
      "inline-flex h-10 items-center gap-1.5 px-3.5 text-sm font-medium text-muted-foreground border-b-2 border-transparent -mb-px transition-colors hover:text-foreground data-[state=active]:text-primary data-[state=active]:border-primary",
      className,
    )}
    {...props}
  />
));
TabsTrigger.displayName = "TabsTrigger";

export const TabsContent = React.forwardRef<
  React.ElementRef<typeof TabsPrimitive.Content>,
  React.ComponentPropsWithoutRef<typeof TabsPrimitive.Content>
>(({ className, ...props }, ref) => (
  <TabsPrimitive.Content ref={ref} className={cn("focus-visible:outline-none", className)} {...props} />
));
TabsContent.displayName = "TabsContent";

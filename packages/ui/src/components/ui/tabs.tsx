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
      /** 膠囊切換：頁標題右側的小型兩段切換（專案說明的「編輯／預覽」）。 */
      segmented: "w-auto h-7 rounded-lg bg-muted p-0.5",
      /** 文件樹：詳情頁左欄的直向整寬節點清單（design D2）；Tabs 根要給 orientation="vertical"。 */
      tree: "flex-col items-stretch",
    },
  },
  defaultVariants: { variant: "underline" },
});

type TabsListVariant = NonNullable<VariantProps<typeof tabsListVariants>["variant"]>;

/** 分頁的樣式跟著所在 TabsList 的變體走，呼叫端不必逐個分頁指定。 */
const TabsVariantContext = React.createContext<TabsListVariant>("underline");

const UNDERLINE_TRIGGER =
  "inline-flex h-10 items-center gap-1.5 px-3.5 text-sm font-medium text-muted-foreground border-b-2 border-transparent -mb-px transition-colors hover:text-foreground data-[state=active]:text-primary data-[state=active]:border-primary";
const SEGMENTED_TRIGGER =
  "inline-flex h-6 items-center gap-1.5 rounded-md px-3 text-xs font-medium text-muted-foreground transition-colors hover:text-foreground data-[state=active]:bg-card data-[state=active]:text-foreground data-[state=active]:shadow-sm";
/** 文件樹節點：與專案欄導覽項同款（主色淡底＝選取），徽章以 ml-auto 靠右。 */
const TREE_TRIGGER =
  "flex h-[30px] w-full items-center gap-2 rounded-lg px-2.5 text-left text-[13px] text-foreground transition-colors hover:bg-foreground/5 data-[state=active]:bg-primary/12 data-[state=active]:font-medium data-[state=active]:text-primary [&_svg]:shrink-0";

const TRIGGER_CLS: Record<TabsListVariant, string> = {
  underline: UNDERLINE_TRIGGER,
  card: UNDERLINE_TRIGGER,
  segmented: SEGMENTED_TRIGGER,
  tree: TREE_TRIGGER,
};

/**
 * 分頁列。`actions` 放頁級動作，渲染在同一列右端、tablist 之外（tablist 只容納分頁）。
 */
export const TabsList = React.forwardRef<
  React.ElementRef<typeof TabsPrimitive.List>,
  React.ComponentPropsWithoutRef<typeof TabsPrimitive.List> &
    VariantProps<typeof tabsListVariants> & { actions?: React.ReactNode }
>(({ className, actions, variant, ...props }, ref) => (
  <TabsVariantContext.Provider value={variant ?? "underline"}>
    <div className={cn(tabsListVariants({ variant }), className)}>
      <TabsPrimitive.List
        ref={ref}
        className={variant === "tree" ? "flex w-full flex-col items-stretch gap-0.5" : "flex items-center"}
        {...props}
      />
      {actions && <div className="ml-auto flex items-center gap-1">{actions}</div>}
    </div>
  </TabsVariantContext.Provider>
));
TabsList.displayName = "TabsList";

export const TabsTrigger = React.forwardRef<
  React.ElementRef<typeof TabsPrimitive.Trigger>,
  React.ComponentPropsWithoutRef<typeof TabsPrimitive.Trigger>
>(({ className, ...props }, ref) => (
  <TabsPrimitive.Trigger
    ref={ref}
    className={cn(TRIGGER_CLS[React.useContext(TabsVariantContext)], className)}
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

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
/** 文件樹節點：與專案欄導覽項同款（主色淡底＝選取），徽章以 ml-auto 靠右。手冊目錄樹的頁列共用
 * 同一串（以 data-state 標作用中），樣式只改這裡。 */
export const TREE_NODE_CLS =
  "flex h-[30px] w-full items-center gap-2 rounded-lg px-2.5 text-left text-[13px] text-foreground transition-colors hover:bg-foreground/5 data-[state=active]:bg-primary/12 data-[state=active]:font-medium data-[state=active]:text-primary [&_svg]:shrink-0";

const TRIGGER_CLS: Record<TabsListVariant, string> = {
  underline: UNDERLINE_TRIGGER,
  card: UNDERLINE_TRIGGER,
  segmented: SEGMENTED_TRIGGER,
  tree: TREE_NODE_CLS,
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

/** 分頁標籤後的計數籤（已封存頁、server 使用者頁與憑證頁）。 */
const COUNT_PILL_CLS =
  "inline-flex items-center justify-center min-w-5 h-5 px-1.5 rounded-full bg-muted text-muted-foreground text-xs font-medium tabular-nums";

/** 分頁。`count` 給了就在標籤後渲染計數籤（0 也渲染）。 */
export const TabsTrigger = React.forwardRef<
  React.ElementRef<typeof TabsPrimitive.Trigger>,
  React.ComponentPropsWithoutRef<typeof TabsPrimitive.Trigger> & { count?: number }
>(({ className, count, children, ...props }, ref) => (
  <TabsPrimitive.Trigger
    ref={ref}
    className={cn(TRIGGER_CLS[React.useContext(TabsVariantContext)], className)}
    {...props}
  >
    {count === undefined ? (
      // 沒有計數籤時子節點原樣交出：asChild 的 Slot 只接受單一子元素。
      children
    ) : (
      <>
        {children}
        <span data-tab-count className={COUNT_PILL_CLS}>
          {count}
        </span>
      </>
    )}
  </TabsPrimitive.Trigger>
));
TabsTrigger.displayName = "TabsTrigger";

export const TabsContent = React.forwardRef<
  React.ElementRef<typeof TabsPrimitive.Content>,
  React.ComponentPropsWithoutRef<typeof TabsPrimitive.Content>
>(({ className, ...props }, ref) => (
  <TabsPrimitive.Content ref={ref} className={cn("focus-visible:outline-none", className)} {...props} />
));
TabsContent.displayName = "TabsContent";

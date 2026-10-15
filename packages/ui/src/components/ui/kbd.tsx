import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "../../lib/utils";

const kbdVariants = cva("font-mono text-[11px] tracking-wide", {
  variants: {
    variant: {
      default: "text-muted-foreground",
      /** 跟隨所在元素的字色：放在按鈕內（如「儲存 ⌘S」）。 */
      inherit: "opacity-80",
    },
  },
  defaultVariants: { variant: "default" },
});

/** 鍵盤快捷鍵（design D5）：等寬 11px；預設灰字，給選單項右端的快捷鍵槽用；`inherit` 跟隨
 *  所在元素的字色，給按鈕內的快捷鍵提示用。 */
function Kbd({
  className,
  variant,
  ...props
}: React.ComponentProps<"kbd"> & VariantProps<typeof kbdVariants>) {
  return <kbd className={cn(kbdVariants({ variant }), className)} {...props} />;
}

/** 選單項右端的快捷鍵槽（下拉選單與右鍵選單共用）。 */
function MenuShortcut({ className, ...props }: React.ComponentProps<"kbd">) {
  return <Kbd className={cn("ml-auto", className)} {...props} />;
}

export { Kbd, MenuShortcut };

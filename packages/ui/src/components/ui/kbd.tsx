import * as React from "react";

import { cn } from "../../lib/utils";

/** 鍵盤快捷鍵（design D5）：等寬 11px 灰字，選單項右端的快捷鍵槽用。 */
function Kbd({ className, ...props }: React.ComponentProps<"kbd">) {
  return (
    <kbd
      className={cn("font-mono text-[11px] tracking-wide text-muted-foreground", className)}
      {...props}
    />
  );
}

/** 選單項右端的快捷鍵槽（下拉選單與右鍵選單共用）。 */
function MenuShortcut({ className, ...props }: React.ComponentProps<"kbd">) {
  return <Kbd className={cn("ml-auto", className)} {...props} />;
}

export { Kbd, MenuShortcut };

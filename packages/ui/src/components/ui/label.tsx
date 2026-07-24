import * as React from "react";

import { cn } from "../../lib/utils";

/** 表單標籤。原生 `<label>`（`htmlFor` 已提供點擊聚焦與可存取關聯，毋需 Radix）。 */
export const Label = React.forwardRef<
  HTMLLabelElement,
  React.LabelHTMLAttributes<HTMLLabelElement>
>(({ className, ...props }, ref) => (
  <label
    ref={ref}
    className={cn(
      "text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70",
      className,
    )}
    {...props}
  />
));
Label.displayName = "Label";

import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "../../lib/utils";

/** 輸入框：8px 圓角；搜尋列的全圓膠囊用 shape="pill"。呼叫端不以 className 覆蓋圓角與顏色。 */
const inputVariants = cva(
  "flex h-8 w-full border border-input bg-background px-3 py-1 text-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring/30 disabled:cursor-not-allowed disabled:opacity-50",
  {
    variants: { shape: { default: "rounded-lg", pill: "rounded-full" } },
    defaultVariants: { shape: "default" },
  },
);

export interface InputProps
  extends React.InputHTMLAttributes<HTMLInputElement>,
    VariantProps<typeof inputVariants> {}

export const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, type, shape, ...props }, ref) => (
    <input type={type} ref={ref} className={cn(inputVariants({ shape }), className)} {...props} />
  ),
);
Input.displayName = "Input";

import * as React from "react";
import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "../../lib/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50",
  {
    variants: {
      variant: {
        default: "bg-primary text-primary-foreground hover:bg-primary/90",
        secondary: "bg-primary/10 text-primary hover:bg-primary/15",
        outline:
          "border border-input bg-background hover:bg-accent hover:text-accent-foreground",
        ghost: "hover:bg-accent hover:text-accent-foreground",
        /** 圖示鈕：靜態無底，hover 才浮出淡底。 */
        subtle: "text-muted-foreground hover:bg-foreground/5 hover:text-foreground",
        /** 開關圖示鈕（看板篩選）：8px 圓角、靜態無底；展開態（aria-expanded）淺灰底深字。 */
        toggle:
          "rounded-lg text-muted-foreground hover:bg-foreground/5 hover:text-foreground aria-expanded:bg-muted aria-expanded:text-foreground",
        link: "text-primary underline-offset-4 hover:underline",
        destructive: "bg-destructive text-white hover:bg-destructive/90",
        danger: "bg-destructive/10 text-destructive hover:bg-destructive/15",
      },
      size: {
        default: "h-9 px-4 py-2",
        sm: "h-7 px-2.5 text-xs",
        toolbar: "h-8 px-3",
        icon: "h-8 w-8",
        "icon-sm": "h-7 w-7",
        "icon-xs": "h-6 w-6",
      },
    },
    defaultVariants: { variant: "default", size: "default" },
  },
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean;
}

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, asChild = false, ...props }, ref) => {
    const Comp = asChild ? Slot : "button";
    return (
      <Comp className={cn(buttonVariants({ variant, size, className }))} ref={ref} {...props} />
    );
  },
);
Button.displayName = "Button";

export { buttonVariants };

import * as React from "react";
import * as SheetPrimitive from "@radix-ui/react-dialog";
import { X } from "lucide-react";

import { cn } from "../../lib/utils";
import { PortalContainerProvider } from "./portal-container";

export const Sheet = SheetPrimitive.Root;
export const SheetTrigger = SheetPrimitive.Trigger;
export const SheetClose = SheetPrimitive.Close;
export const SheetPortal = SheetPrimitive.Portal;

export const SheetOverlay = React.forwardRef<
  React.ElementRef<typeof SheetPrimitive.Overlay>,
  React.ComponentPropsWithoutRef<typeof SheetPrimitive.Overlay>
>(({ className, ...props }, ref) => (
  <SheetPrimitive.Overlay
    ref={ref}
    className={cn(
      "fixed inset-0 z-50 bg-black/40 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0",
      className,
    )}
    {...props}
  />
));
SheetOverlay.displayName = "SheetOverlay";

export const SheetContent = React.forwardRef<
  React.ElementRef<typeof SheetPrimitive.Content>,
  React.ComponentPropsWithoutRef<typeof SheetPrimitive.Content>
>(({ className, children, ...props }, ref) => {
  // 抽屜內的浮層要 portal 進抽屜自己（見 portal-container.tsx）：portal 到 body 會落在
  // focus trap 外面，與 Dialog 的 FocusScope 互推焦點。
  const [content, setContent] = React.useState<HTMLElement | null>(null);
  return (
    <SheetPortal>
      <SheetOverlay />
      <SheetPrimitive.Content
        ref={(node) => {
          setContent(node);
          if (typeof ref === "function") ref(node);
          else if (ref) ref.current = node;
        }}
        className={cn(
          // overflow-x-hidden 為必要而非贅述：CSS 規定 overflow-x: visible 搭配
          // overflow-y: auto 時，x 軸會被計算為 auto——單一過長子項即讓面板長出水平捲軸。
          "fixed z-50 inset-y-0 right-0 h-full w-[440px] max-w-[92vw] bg-card border-l border-border shadow-lg p-5 overflow-y-auto overflow-x-hidden flex flex-col gap-4 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:slide-out-to-right data-[state=open]:slide-in-from-right",
          className,
        )}
        {...props}
      >
        <PortalContainerProvider value={content}>{children}</PortalContainerProvider>
        {/* shadcn ghost icon 同款樣式：帶 hover 底色回饋（desktop-ux-polish 統一按鈕 hover）。 */}
        <SheetPrimitive.Close className="absolute right-3 top-3 inline-flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground">
          <X className="h-4 w-4" />
          <span className="sr-only">Close</span>
        </SheetPrimitive.Close>
      </SheetPrimitive.Content>
    </SheetPortal>
  );
});
SheetContent.displayName = "SheetContent";

export function SheetHeader({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn("flex flex-col gap-1", className)} {...props} />;
}

export function SheetTitle({ className, ...props }: React.ComponentPropsWithoutRef<typeof SheetPrimitive.Title>) {
  return <SheetPrimitive.Title className={cn("text-base font-semibold", className)} {...props} />;
}

export function SheetDescription({ className, ...props }: React.ComponentPropsWithoutRef<typeof SheetPrimitive.Description>) {
  return <SheetPrimitive.Description className={cn("text-sm text-muted-foreground", className)} {...props} />;
}

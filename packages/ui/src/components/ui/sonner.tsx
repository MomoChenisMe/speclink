import type { CSSProperties } from "react";
import { Toaster as Sonner, type ToasterProps } from "sonner";

import { useI18n } from "../../i18n";
import { cn } from "../../lib/utils";

const TOKEN_STYLE = {
  "--normal-bg": "var(--card)",
  "--normal-text": "var(--card-foreground)",
  "--normal-border": "var(--border)",
  "--border-radius": "var(--radius)",
  fontFamily: "inherit",
} as CSSProperties;

const TOKEN_CLASS_NAMES = {
  toast: "!shadow-lg",
  error: "!border-destructive/40 [&_[data-icon]]:text-destructive",
  closeButton:
    "!border-border !bg-card !text-muted-foreground hover:!bg-accent hover:!text-accent-foreground",
};

function Toaster({ toastOptions, style, ...props }: ToasterProps) {
  const { t } = useI18n();
  const classNames = toastOptions?.classNames;

  return (
    <Sonner
      {...props}
      style={{ ...TOKEN_STYLE, ...style }}
      theme="system"
      duration={6000}
      closeButton
      visibleToasts={1}
      toastOptions={{
        ...toastOptions,
        classNames: {
          ...classNames,
          toast: cn(TOKEN_CLASS_NAMES.toast, classNames?.toast),
          error: cn(TOKEN_CLASS_NAMES.error, classNames?.error),
          closeButton: cn(TOKEN_CLASS_NAMES.closeButton, classNames?.closeButton),
        },
        closeButtonAriaLabel: t("toast.close"),
      }}
    />
  );
}

export { Toaster };

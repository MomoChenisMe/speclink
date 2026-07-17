import { Toaster as Sonner, type ToasterProps } from "sonner";

import { useI18n } from "../../i18n";

function Toaster({ toastOptions, ...props }: ToasterProps) {
  const { t } = useI18n();

  return (
    <Sonner
      {...props}
      theme="system"
      duration={6000}
      closeButton
      visibleToasts={1}
      toastOptions={{
        ...toastOptions,
        closeButtonAriaLabel: t("toast.close"),
      }}
    />
  );
}

export { Toaster };

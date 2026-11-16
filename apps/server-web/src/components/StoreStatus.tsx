import { SettingsRow, StatusPill, useI18n } from "@speclink/ui";

// 儲存後端的種類與可用性——系統頁與初次設定頁共用同一列，兩處看到的是同一種呈現。

/** 列右側的等寬值。 */
export function Mono({ children }: { children: string | number }) {
  return <span className="break-all font-mono text-[13px]">{children}</span>;
}

/** 「儲存後端」列：種類、是否可用；不可用時在說明列出原因。 */
export function StoreStatusRow({
  driver,
  healthy,
  healthError,
}: {
  driver: string;
  healthy: boolean;
  healthError?: string | null;
}) {
  const { t } = useI18n();
  return (
    <SettingsRow
      label={t("system.storeDriver")}
      description={healthy ? undefined : (healthError ?? undefined)}
      control={
        <span className="flex items-center gap-2">
          <Mono>{driver}</Mono>
          <StatusPill tone={healthy ? "success" : "destructive"}>
            {healthy ? t("common.normal") : t("common.abnormal")}
          </StatusPill>
        </span>
      }
    />
  );
}

// 伺服器頁（desktop-settings-mode design D7；spec「伺服器管理最小面」）：頁標題右側「新增連線」
// 切換頁頂新增卡；連線卡（ServersPanel）；預設卡（新增遠端專案時預設使用的連線）；登入方式卡。
import { useState } from "react";
import { Plus } from "lucide-react";
import {
  Button,
  SettingsCard,
  SettingsChoice,
  SettingsPageHeader,
  SettingsRow,
  useI18n,
} from "@speclink/ui";

import { defaultConnectionOf } from "../../adapter/connections";
import { ServersPanel, type ServersPanelProps } from "../../components/ServersPanel";

/** 「不預選」在偏好裡是 null，Radix Select 的值不接受空值：以哨兵值往返。 */
const NO_DEFAULT = "__none__";

export interface ServersPageProps extends Omit<ServersPanelProps, "addOpen" | "onAddOpenChange"> {
  /** 新增遠端專案時預設使用的連線（null＝不預選）。 */
  defaultConnectionId: string | null;
  onDefaultConnectionChange: (connectionId: string | null) => void;
}

export function ServersPage({ defaultConnectionId, onDefaultConnectionChange, ...panel }: ServersPageProps) {
  const { t } = useI18n();
  const [addOpen, setAddOpen] = useState(false);
  const loggedIn = panel.connections.filter((c) => c.loggedIn);
  // 指向的連線已登出或被移除：視同不預選（新增專案對話框同樣忽略它）。
  const defaultValue = defaultConnectionOf(panel.connections, defaultConnectionId)?.id ?? NO_DEFAULT;
  const defaultLabel = t("servers.page.defaultForNew");
  return (
    <>
      <SettingsPageHeader
        title={t("settings.section.servers")}
        description={t("servers.help")}
        actions={
          <Button type="button" variant="secondary" size="sm" onClick={() => setAddOpen((open) => !open)}>
            <Plus className="h-3.5 w-3.5" />
            {t("servers.page.addConnection")}
          </Button>
        }
      />
      <ServersPanel {...panel} addOpen={addOpen} onAddOpenChange={setAddOpen} />
      <SettingsCard title={t("servers.page.defaultsCard")}>
        <SettingsRow
          label={defaultLabel}
          control={
            <SettingsChoice
              ariaLabel={defaultLabel}
              value={defaultValue}
              options={[
                { value: NO_DEFAULT, label: t("servers.page.noDefault") },
                ...loggedIn.map((c) => ({ value: c.id, label: c.name })),
              ]}
              onChange={(v) => onDefaultConnectionChange(v === NO_DEFAULT ? null : v)}
            />
          }
        />
      </SettingsCard>
      <SettingsCard title={t("servers.page.loginCard")}>
        <SettingsRow
          label={t("servers.page.deviceLogin")}
          description={t("servers.page.deviceLoginHint")}
          control={<span className="text-xs text-muted-foreground">{t("servers.page.isDefault")}</span>}
        />
        <SettingsRow label={t("servers.page.pat")} description={t("servers.page.patHint")} control={null} />
      </SettingsCard>
    </>
  );
}

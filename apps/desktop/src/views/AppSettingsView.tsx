import { AlertTriangle } from "lucide-react";
import {
  Button,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  cn,
  useI18n,
} from "@speclink/ui";

import { ServersPanel, type ServersPanelProps } from "../components/ServersPanel";
import type { LocalePreference } from "../i18n/locale";

export interface AppSettingsViewProps {
  /** UI 語言偏好現值（null＝跟隨系統）。 */
  localePref: LocalePreference;
  /** 切換即時生效並持久化（App 層負責寫 localStorage）。 */
  onLocalePrefChange: (pref: LocalePreference) => void;
  /** 面板建立失敗時於本機設定簽浮出的單行錯誤。 */
  trayPanelError?: string | null;
  /** app 全域伺服器管理面；測試或不支援連線的殼層可不注入。 */
  servers?: ServersPanelProps;
  /** needs-reauth 導向時預選伺服器簽並聚焦該 connection。 */
  focusConnectionId?: string | null;
}

function FieldHelp({ children }: { children: React.ReactNode }) {
  return <p className="text-xs text-muted-foreground m-0">{children}</p>;
}

function TrayPanelError({ message }: { message: string }) {
  const { t } = useI18n();
  return (
    <p role="alert" className="flex items-start gap-1.5 text-xs text-amber-600 dark:text-amber-400 m-0">
      <AlertTriangle className="h-3.5 w-3.5 shrink-0 mt-0.5" />
      <span>
        {t("settings.parseErrorHint")} {message}
      </span>
    </p>
  );
}

/** 與任何 workspace 分頁無關的應用程式設定：本機偏好與伺服器連線。 */
export function AppSettingsView({
  localePref,
  onLocalePrefChange,
  trayPanelError = null,
  servers,
  focusConnectionId = null,
}: AppSettingsViewProps) {
  const { t } = useI18n();
  const uiLocaleOptions: Array<{ value: LocalePreference; label: string }> = [
    { value: null, label: t("settings.followSystem") },
    { value: "zh-TW", label: "繁體中文" },
    { value: "en", label: "English" },
  ];

  return (
    <div className="max-w-2xl mx-auto w-full">
      <Tabs
        key={focusConnectionId ?? "settings"}
        defaultValue={focusConnectionId ? "servers" : "local"}
      >
        <TabsList>
          <TabsTrigger value="local">{t("settings.localTabLabel")}</TabsTrigger>
          {servers && <TabsTrigger value="servers">{t("settings.serversTabLabel")}</TabsTrigger>}
        </TabsList>

        <TabsContent value="local" className="pt-3 flex flex-col gap-4">
          <span data-testid="local-note" className="text-xs text-muted-foreground">
            {t("settings.localTabNote")}
          </span>
          <Card data-testid="ui-locale-card">
            <CardHeader>
              <CardTitle className="text-base">{t("settings.uiLocaleLabel")}</CardTitle>
            </CardHeader>
            <CardContent className="gap-2">
              <div className="flex gap-1.5" data-testid="ui-locale">
                {uiLocaleOptions.map((option) => (
                  <Button
                    key={String(option.value)}
                    type="button"
                    variant="outline"
                    size="sm"
                    className={cn(
                      "text-sm font-normal",
                      localePref === option.value
                        ? "border-primary bg-primary/8 font-medium text-primary hover:bg-primary/8 hover:text-primary"
                        : "text-muted-foreground hover:text-foreground",
                    )}
                    onClick={() => onLocalePrefChange(option.value)}
                  >
                    {option.label}
                  </Button>
                ))}
              </div>
              <FieldHelp>{t("settings.uiLocaleHelp")}</FieldHelp>
            </CardContent>
          </Card>
          {trayPanelError && <TrayPanelError message={trayPanelError} />}
        </TabsContent>

        {servers && (
          <TabsContent value="servers" className="pt-3 flex flex-col gap-4">
            <ServersPanel {...servers} />
          </TabsContent>
        )}
      </Tabs>
    </div>
  );
}

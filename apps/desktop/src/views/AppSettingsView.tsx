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
  SEMANTIC_TONE,
  cn,
  useI18n,
} from "@speclink/ui";

import { ServersPanel, type ServersPanelProps } from "../components/ServersPanel";
import type { LocalePreference } from "../i18n/locale";
import type { UpdaterState } from "../core/updater";
import type { CliInstallView } from "../store";

/** 軟體更新卡的注入面：狀態機現值＋手動檢查入口＋常駐現版號（null＝尚未取得）。 */
export interface AppSettingsUpdaterProps {
  state: UpdaterState;
  onCheck: () => void;
  currentVersion?: string | null;
}

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
  /** 軟體更新面（desktop-app「桌面自動更新」）；測試或不支援更新的殼層可不注入。 */
  updater?: AppSettingsUpdaterProps;
  /** 安裝 CLI 指令面（desktop-app「安裝 CLI 指令到 PATH」）；探測前或無殼層支援時不注入。 */
  cliInstall?: { view: CliInstallView; onInstall: () => void };
}

function FieldHelp({ children }: { children: React.ReactNode }) {
  return <p className="text-xs text-muted-foreground m-0">{children}</p>;
}

/** 軟體更新卡的行內狀態（手動檢查結果與進行中狀態；閒置不顯示）。 */
function UpdaterInlineStatus({ state }: { state: UpdaterState }) {
  const { t } = useI18n();
  switch (state.phase) {
    case "checking":
      return <span className="text-xs text-muted-foreground">{t("updater.checking")}</span>;
    case "upToDate":
      return <span className="text-xs text-muted-foreground">{t("updater.upToDate")}</span>;
    case "checkFailed":
      return (
        <span className={`text-xs ${SEMANTIC_TONE.danger}`}>
          {t("updater.checkFailed")}
        </span>
      );
    case "available":
      return (
        <span className={`text-xs ${SEMANTIC_TONE.inProgress}`}>
          {t("updater.available")} {state.version}
        </span>
      );
    case "downloading":
      return (
        <span className="text-xs text-muted-foreground">
          {t("updater.downloading")} {state.version}…
        </span>
      );
    case "restartPending":
      return <span className="text-xs text-primary">{t("updater.restartPending")}</span>;
    case "error":
      return (
        <span className={`text-xs ${SEMANTIC_TONE.danger}`}>
          {t("updater.errorPrefix")}
          {state.message}
        </span>
      );
    case "idle":
      return null;
  }
}

/** CLI 卡狀態行：三態＋版本；佈署方式歸屬（安裝器／套件管理器）另有說明列。 */
function CliInstallStatusLine({ view }: { view: CliInstallView }) {
  const { t } = useI18n();
  switch (view.status.kind) {
    case "not-installed":
      return <span className="text-xs text-muted-foreground">{t("cliInstall.notInstalled")}</span>;
    case "installed":
      return (
        <span className="text-xs text-muted-foreground">
          {t("cliInstall.installed")} {view.status.version}
        </span>
      );
    case "version-mismatch":
      return (
        <span className={`text-xs ${SEMANTIC_TONE.warning}`}>
          {t("cliInstall.mismatch")}（{view.status.version}）
        </span>
      );
  }
}

function TrayPanelError({ message }: { message: string }) {
  const { t } = useI18n();
  return (
    <p role="alert" className={`flex items-start gap-1.5 text-xs m-0 ${SEMANTIC_TONE.danger}`}>
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
  updater,
  cliInstall,
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
          {updater && (
            <Card data-testid="updater-card">
              <CardHeader>
                <CardTitle className="text-base">{t("updater.cardTitle")}</CardTitle>
              </CardHeader>
              <CardContent className="gap-2">
                <div className="flex items-center gap-2.5">
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="text-sm font-normal"
                    disabled={updater.state.phase === "checking"}
                    onClick={updater.onCheck}
                  >
                    {t("updater.check")}
                  </Button>
                  {updater.currentVersion && (
                    <span className="text-xs text-muted-foreground">
                      {t("updater.currentVersion")} {updater.currentVersion}
                    </span>
                  )}
                  <UpdaterInlineStatus state={updater.state} />
                </div>
                <FieldHelp>{t("updater.help")}</FieldHelp>
              </CardContent>
            </Card>
          )}
          {cliInstall && (
            <Card data-testid="cli-install-card">
              <CardHeader>
                <CardTitle className="text-base">{t("cliInstall.cardTitle")}</CardTitle>
              </CardHeader>
              <CardContent className="gap-2">
                <div className="flex items-center gap-2.5">
                  {cliInstall.view.canDeploy &&
                    cliInstall.view.status.kind !== "installed" && (
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        className="text-sm font-normal"
                        disabled={cliInstall.view.busy}
                        onClick={cliInstall.onInstall}
                      >
                        {cliInstall.view.busy
                          ? t("cliInstall.installing")
                          : cliInstall.view.status.kind === "version-mismatch"
                            ? t("cliInstall.reinstall")
                            : t("cliInstall.install")}
                      </Button>
                    )}
                  <CliInstallStatusLine view={cliInstall.view} />
                </div>
                {cliInstall.view.platform === "windows" && (
                  <FieldHelp>{t("cliInstall.installerManaged")}</FieldHelp>
                )}
                {cliInstall.view.platform === "linux-deb" && (
                  <FieldHelp>{t("cliInstall.packageManaged")}</FieldHelp>
                )}
                {cliInstall.view.pathHint && cliInstall.view.deployDir && (
                  <p
                    data-testid="cli-path-hint"
                    className={`flex items-start gap-1.5 text-xs m-0 ${SEMANTIC_TONE.warning}`}
                  >
                    <AlertTriangle className="h-3.5 w-3.5 shrink-0 mt-0.5" />
                    <span>
                      {cliInstall.view.deployDir} {t("cliInstall.pathHintSuffix")}{" "}
                      <code>export PATH=&quot;$PATH:{cliInstall.view.deployDir}&quot;</code>
                    </span>
                  </p>
                )}
                {cliInstall.view.error && (
                  <p role="alert" className="text-xs text-destructive m-0">
                    {cliInstall.view.error}
                  </p>
                )}
                {cliInstall.view.canDeploy && <FieldHelp>{t("cliInstall.help")}</FieldHelp>}
              </CardContent>
            </Card>
          )}
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

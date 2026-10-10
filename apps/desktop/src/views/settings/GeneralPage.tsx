// 一般頁（desktop-settings-mode design D5）：介面語言、軟體更新、CLI 指令、鍵盤快捷鍵四卡；
// 軟體更新卡只有目前版本、檢查更新（含行內結果）與更新日誌三列，下載與安裝在更新頁。
import { AlertTriangle } from "lucide-react";
import {
  Button,
  Kbd,
  SEMANTIC_TONE,
  SettingsCard,
  SettingsPageHeader,
  SettingsRow,
  StatusPill,
  useI18n,
} from "@speclink/ui";

import { buildShortcutList } from "../../appMenu";
import type { Platform } from "../../platform";
import type { UpdaterState } from "../../core/updater";
import type { AppSettingsUpdaterProps } from "../AppSettingsView";
import { LocaleRow, ThemeRow, type InterfacePrefsProps } from "./AppearancePage";
import { CliInstallRow, type CliInstallProps } from "./CliPage";
import { VersionText, latestOf } from "./UpdatesPage";

/** 檢查更新列的行內結果：與更新頁「最新版本」列同一組狀態籤（發現新版帶等寬版號）；閒置與
 * 安裝失敗不顯示——安裝失敗在圖示列更新鈕與更新頁處置。 */
function UpdaterInlineStatus({ state }: { state: UpdaterState }) {
  const { t } = useI18n();
  if (state.phase === "idle" || state.phase === "error") return null;
  const latest = latestOf(state, null);
  return (
    <>
      {latest.version && <VersionText>{latest.version}</VersionText>}
      <StatusPill tone={latest.pill.tone}>{t(latest.pill.key)}</StatusPill>
    </>
  );
}

function TrayPanelError({ message }: { message: string }) {
  const { t } = useI18n();
  return (
    <p role="alert" className={`m-0 flex items-start gap-1.5 text-xs ${SEMANTIC_TONE.danger}`}>
      <AlertTriangle className="mt-0.5 h-3.5 w-3.5 shrink-0" />
      <span>
        {t("settings.parseErrorHint")} {message}
      </span>
    </p>
  );
}

export interface GeneralPageProps extends InterfacePrefsProps {
  platform: Platform;
  updater?: AppSettingsUpdaterProps;
  cliInstall?: CliInstallProps;
  trayPanelError?: string | null;
}

export function GeneralPage({
  platform,
  localePref,
  onLocalePrefChange,
  themePref,
  onThemePrefChange,
  updater,
  cliInstall,
  trayPanelError,
}: GeneralPageProps) {
  const { t } = useI18n();
  return (
    <>
      <SettingsPageHeader title={t("settings.section.general")} note={t("settings.pageNote")} />
      <SettingsCard title={t("settings.uiLocaleLabel")}>
        <LocaleRow localePref={localePref} onLocalePrefChange={onLocalePrefChange} />
        <ThemeRow themePref={themePref} onThemePrefChange={onThemePrefChange} />
      </SettingsCard>
      {updater && (
        <SettingsCard title={t("updater.cardTitle")}>
          <SettingsRow
            label={t("updater.currentVersion")}
            control={<VersionText>{updater.currentVersion ?? "—"}</VersionText>}
          />
          <SettingsRow
            label={t("updater.check")}
            description={t("updater.help")}
            control={
              <>
                <UpdaterInlineStatus state={updater.state} />
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  disabled={updater.state.phase === "checking"}
                  onClick={updater.onCheck}
                >
                  {t("updater.check")}
                </Button>
              </>
            }
          />
          <SettingsRow
            label={t("releaseNotes.open")}
            control={
              <Button type="button" variant="outline" size="sm" onClick={updater.onShowReleaseNotes}>
                {t("updater.page.viewNotes")}
              </Button>
            }
          />
        </SettingsCard>
      )}
      {cliInstall && (
        <SettingsCard title={t("cliInstall.cardTitle")}>
          <CliInstallRow {...cliInstall} />
        </SettingsCard>
      )}
      <SettingsCard title={t("settings.shortcutsTitle")}>
        {buildShortcutList({ t, platform }).map((row) => (
          <SettingsRow key={row.id} label={row.text} control={<Kbd>{row.keys}</Kbd>} />
        ))}
      </SettingsCard>
      {trayPanelError && <TrayPanelError message={trayPanelError} />}
    </>
  );
}

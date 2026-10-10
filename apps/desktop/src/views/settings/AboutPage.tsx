// 關於頁（desktop-about-page design D2；spec「關於頁」）：品牌卡（鎖版、版本列、產品描述）、
// 版本卡（檢查更新、更新日誌、複製版本資訊）、連結卡（原始碼、說明文件、回報問題）、
// 授權卡（授權條款、第三方授權）。
import { useEffect, useState } from "react";
import { toast } from "sonner";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  Button,
  Card,
  CopyButton,
  SettingsCard,
  SettingsPageHeader,
  SettingsRow,
  StatusPill,
  Wordmark,
  useI18n,
} from "@speclink/ui";

import type { AppInfo } from "../../adapter/appInfo";
import {
  BUNDLED_THIRD_PARTY_LICENSES,
  LicenseDialog,
  type LicenseDialogContent,
  type ThirdPartyLicenses,
} from "../../components/LicenseDialog";
import type { AppSettingsUpdaterProps } from "../AppSettingsView";
import { latestOf } from "./UpdatesPage";

/** 倉庫網址（原生選單的 GitHub／回報問題也用它；能力檔的 opener 權限只放行這個前綴）。 */
export const REPO_URL = "https://github.com/MomoChenisMe/speclink";
const DOCS_URL = `${REPO_URL}/tree/main/docs`;

/** 「複製版本資訊」與回報問題 issue 內文共用的四行。 */
export function versionLines(info: AppInfo): string {
  return [
    `Speclink ${info.appVersion}`,
    `Engine ${info.engineVersion}`,
    `Platform ${info.platform} ${info.arch}`,
    `Skills ${info.assetsVersion}`,
  ].join("\n");
}

/** 上次檢查的相對時間：一分鐘內「剛剛」，再依分鐘、小時、天。 */
function sinceText(at: number, t: (key: string) => string): string {
  const minutes = Math.floor((Date.now() - at) / 60_000);
  if (minutes < 1) return t("about.justNow");
  if (minutes < 60) return t("about.minutesAgo").replace("{n}", String(minutes));
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return t("about.hoursAgo").replace("{n}", String(hours));
  return t("time.daysAgo").replace("{n}", String(Math.floor(hours / 24)));
}

export interface AboutPageProps {
  /** null＝尚未取得（版本列顯示「—」、無複製鈕）。 */
  appInfo: AppInfo | null;
  /** 進頁時要求載入版本資訊（store 快取，重複呼叫不重取）。 */
  onLoadAppInfo: () => void;
  /** 上次檢查更新完成的時刻；null＝從未檢查。 */
  lastCheckedAt: number | null;
  /** 與更新頁共用的更新狀態；未注入時不顯示檢查更新與更新日誌兩列。 */
  updater?: AppSettingsUpdaterProps;
  thirdPartyLicenses?: ThirdPartyLicenses | null;
}

export function AboutPage({
  appInfo,
  onLoadAppInfo,
  lastCheckedAt,
  updater,
  thirdPartyLicenses = BUNDLED_THIRD_PARTY_LICENSES,
}: AboutPageProps) {
  const { t } = useI18n();
  const [dialog, setDialog] = useState<LicenseDialogContent | null>(null);
  useEffect(() => {
    onLoadAppInfo();
  }, [onLoadAppInfo]);

  const lines = appInfo ? versionLines(appInfo) : null;
  const pill = updater ? latestOf(updater.state, null).pill : null;
  const open = (url: string) => {
    openUrl(url).catch((e) => toast.error(String(e)));
  };
  const viewButton = (content: LicenseDialogContent) => (
    <Button type="button" variant="outline" size="sm" onClick={() => setDialog(content)}>
      {t("about.view")}
    </Button>
  );

  return (
    <>
      <SettingsPageHeader title={t("about.title")} description={t("about.description")} />
      <Card data-testid="about-brand" className="flex flex-col gap-3 p-5">
        <Wordmark className="h-10 self-start" />
        <div className="flex flex-wrap items-center gap-2 text-[13px] text-muted-foreground">
          <span>
            {appInfo
              ? `${appInfo.appVersion} · ${t("about.engine")} ${appInfo.engineVersion} · ${appInfo.platform} ${appInfo.arch}`
              : "—"}
          </span>
          {pill && (
            <>
              <span aria-hidden>·</span>
              <StatusPill tone={pill.tone}>{t(pill.key)}</StatusPill>
            </>
          )}
        </div>
        <p className="m-0 text-[15px]">{t("about.tagline")}</p>
      </Card>

      <SettingsCard title={t("about.versionCard")}>
        {updater && (
          <>
            <SettingsRow
              label={t("updater.check")}
              description={
                lastCheckedAt === null
                  ? t("about.neverChecked")
                  : t("about.lastChecked").replace("{time}", sinceText(lastCheckedAt, t))
              }
              control={
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  disabled={updater.state.phase === "checking"}
                  onClick={updater.onCheck}
                >
                  {t("updater.check")}
                </Button>
              }
            />
            <SettingsRow
              label={t("releaseNotes.open")}
              control={
                <Button type="button" variant="outline" size="sm" onClick={updater.onShowReleaseNotes}>
                  {t("about.view")}
                </Button>
              }
            />
          </>
        )}
        <SettingsRow
          label={t("about.copyInfo")}
          description={t("about.copyInfoHint")}
          control={lines && <CopyButton size="sm" value={lines} label={t("about.copyInfo")} />}
        />
      </SettingsCard>

      <SettingsCard title={t("about.linksCard")}>
        <SettingsRow
          label={t("about.source")}
          description={<span className="font-mono">{REPO_URL.replace(/^https:\/\//, "")}</span>}
          onClick={() => open(REPO_URL)}
        />
        <SettingsRow label={t("about.docs")} description={t("about.docsHint")} onClick={() => open(DOCS_URL)} />
        <SettingsRow
          label={t("about.report")}
          description={t("about.reportHint")}
          onClick={() => open(`${REPO_URL}/issues/new${lines ? `?body=${encodeURIComponent(lines)}` : ""}`)}
        />
      </SettingsCard>

      <SettingsCard title={t("about.licenseCard")}>
        <SettingsRow label={t("about.license")} description={t("about.licenseHint")} control={viewButton("license")} />
        <SettingsRow
          label={t("about.thirdParty")}
          description={t("about.thirdPartyHint")}
          control={viewButton("thirdParty")}
        />
      </SettingsCard>

      <LicenseDialog content={dialog} thirdPartyLicenses={thirdPartyLicenses} onClose={() => setDialog(null)} />
    </>
  );
}
